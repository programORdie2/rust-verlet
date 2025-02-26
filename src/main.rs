use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::video::Window;
use glam::Vec2;
use std::time::{Duration, Instant};

const WIDTH: u32 = 600;
const HEIGHT: u32 = 600;
const GRAVITY: Vec2 = Vec2::new(0.0, 750.0);
const PARTICLE_RADIUS: f32 = 10.0;
const CENTER: Vec2 = Vec2::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0);
const FRAMES_BETWEEN_NEW_PARTICLES: u32 = 15;

#[derive(Debug, Clone)]
struct Particle {
    pos: Vec2,
    old_pos: Vec2,
    acceleration: Vec2,
}

impl Particle {
    fn new(x: f32, y: f32, vx: f32, vy: f32) -> Self {
        Particle {
            pos: Vec2::new(x, y),
            old_pos: Vec2::new(vx, vy),
            acceleration: Vec2::ZERO,
        }
    }

    fn update(&mut self, dt: f32) {
        let vel = self.pos - self.old_pos;
        self.old_pos = self.pos;
        self.pos += vel + self.acceleration * dt * dt;
        self.acceleration = Vec2::ZERO;
    }

    fn accelerate(&mut self, acc: Vec2) {
        self.acceleration += acc;
    }
}

struct VerletSimulation {
    particles: Vec<Particle>
}

impl VerletSimulation {
    fn new() -> Self {
        // Initialize empty particles vector
        let particles = Vec::new();
        
        VerletSimulation {
            particles
        }
    }

    fn spawn_particle(&mut self, x: f32, y: f32, dir: f32) {
        let speed = 0.1;
        let vx = speed * dir.cos();
        let vy = speed * dir.sin();
        self.particles.push(Particle::new(
            x, y, x + vx, y + vy
        ));
    }

    fn update(&mut self, dt: f32, frame: u32) {
        if frame % FRAMES_BETWEEN_NEW_PARTICLES == 0 && self.particles.len() < 200 {
            self.spawn_particle(CENTER.x, 100.0, 0.0);
        }

        let sub_runs = 8;
        let sub_dt = dt / sub_runs as f32;

        for _ in 0..sub_runs {
            // Apply forces
            for particle in &mut self.particles {
                particle.accelerate(GRAVITY);
            }

            self.apply_constraints();
            self.solve_collisions();

            // Update positions
            for particle in &mut self.particles {
                particle.update(sub_dt);
            }
        }
    }

    fn apply_constraints(&mut self) {
        let radius = 250.0 - PARTICLE_RADIUS;

        for particle in &mut self.particles {
            let to_obj = particle.pos - CENTER;
            let dist = to_obj.length();
            if dist > radius - PARTICLE_RADIUS {
                let n: Vec2 = to_obj / dist; // Normalized direction vector
                let penetration = dist - (radius - PARTICLE_RADIUS);
                
                // Move the particle outside the boundary
                particle.pos -= n * penetration;

                // Reflect the velocity (correctly modify old_pos)
                let vel = particle.pos - particle.old_pos;
                particle.old_pos = particle.pos - vel.reflect(n) * 0.999; // Apply damping
            }
        }
    }

    fn solve_collisions(&mut self) {
        for i in 0..self.particles.len() {
            for j in i + 1..self.particles.len() {
                let (p1, p2) = self.particles.split_at_mut(j);
                let p1 = &mut p1[i];
                let p2 = &mut p2[0];

                let axis = p1.pos - p2.pos;
                let dist = axis.length();
                if dist < PARTICLE_RADIUS * 2.0 {
                    let n: Vec2 = axis / dist; // Normalized direction vector
                    let delta = PARTICLE_RADIUS * 2.0 - dist;

                    // Separate the particles
                    p1.pos += 0.5 * n * delta;
                    p2.pos -= 0.5 * n * delta;

                    // Velocity correction for realistic bounces
                    let v1 = p1.pos - p1.old_pos;
                    let v2 = p2.pos - p2.old_pos;
                    let relative_velocity = v1 - v2;

                    let normal_vel = relative_velocity.dot(n);
                    if normal_vel < 0.0 {
                        let restitution = 0.7;
                        let impulse = -normal_vel * restitution * n;

                        p1.old_pos -= impulse;
                        p2.old_pos += impulse;
                    }
                }
            }
        }
    }


    fn draw_circle(&self, canvas: &mut Canvas<Window>, center_x: i32, center_y: i32, r: i32) -> Result<(), String> {
        let mut x = r;
        let mut y = 0;
        let mut dx = 1 - 2 * r;
        let mut dy = 1;
        let mut err = 0;
        
        while x >= y {
            canvas.draw_point(Point::new(center_x + x, center_y + y))?;
            canvas.draw_point(Point::new(center_x + y, center_y + x))?;
            canvas.draw_point(Point::new(center_x - x, center_y + y))?;
            canvas.draw_point(Point::new(center_x - y, center_y + x))?;
            canvas.draw_point(Point::new(center_x - x, center_y - y))?;
            canvas.draw_point(Point::new(center_x - y, center_y - x))?;
            canvas.draw_point(Point::new(center_x + x, center_y - y))?;
            canvas.draw_point(Point::new(center_x + y, center_y - x))?;
            
            y += 1;
            err += dy;
            dy += 2;
            if 2 * err + dx > 0 {
                x -= 1;
                err += dx;
                dx += 2;
            }
        }
        Ok(())
    }

    fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        // Clear the screen
        canvas.set_draw_color(Color::RGB(50, 50, 50));
        canvas.clear();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        // Draw the center
        let center = Vec2::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0);
        let radius = 250;
        self.draw_circle(canvas, center.x as i32, center.y as i32, radius - PARTICLE_RADIUS as i32)?;

        // Draw particles
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        for particle in &self.particles {
            let x = particle.pos.x as i32;
            let y = particle.pos.y as i32;
            let r = PARTICLE_RADIUS as i32;

            // Draw a filled circle
            self.draw_circle(canvas, x, y, r)?;
        }

        canvas.present();
        Ok(())
    }
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    
    let window = video_subsystem
        .window("Verlet Physics Simulation", WIDTH, HEIGHT)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    
    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    
    let mut event_pump = sdl_context.event_pump()?;
    let mut simulation = VerletSimulation::new();

    let dt: f32 = 1.0 / 60.0;
    let mut frame: u32 = 0;
    let mut update_time: Duration = Duration::new(0, 0);
    let mut render_time: Duration = Duration::new(0, 0);
    
    'running: loop {
        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                },
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    println!("Update time: {:?} Render time: {:?}", update_time, render_time);
                },
                _ => {}
            }
        }
        
        frame += 1;

        let mut start = Instant::now();
        // Update the simulation with a fixed timestep
        simulation.update(dt, frame);

        update_time = start.elapsed();
        start = Instant::now();
        
        // Render
        simulation.render(&mut canvas)?;

        render_time = start.elapsed();
        
        // Cap at 60 FPS
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    
    Ok(())
}