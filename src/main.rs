use macroquad::prelude::*;
use std::time::{Duration, Instant};

const WIDTH: u32 = 600;
const HEIGHT: u32 = 600;
const GRAVITY: Vec2 = Vec2::new(0.0, 750.0);
const PARTICLE_RADIUS: f32 = 4.0;
const CENTER: Vec2 = Vec2::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0);
const FRAMES_BETWEEN_NEW_PARTICLES: u32 = 1;
const MAX_PARTICLES: usize = 1000;

fn reflect_vec2(vec: Vec2, normal: Vec2) -> Vec2 {
    vec - 2.0 * vec.dot(normal) * normal
}

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
        let speed = 4.0;
        let vx = speed * dir.cos();
        let vy = speed * dir.sin();
        self.particles.push(Particle::new(
            x, y, x + vx, y + vy
        ));
    }

    fn update(&mut self, dt: f32, frame: u32) -> String {
        if frame % FRAMES_BETWEEN_NEW_PARTICLES == 0 && self.particles.len() < MAX_PARTICLES {
            let len = self.particles.len();
            let dir = len % 40;
            if dir > 20 {
                // Spray to the right
                self.spawn_particle(CENTER.x, 100.0, (80.0 - dir as f32) * 0.1);
            } else {
                // Spray to the left
                self.spawn_particle(CENTER.x, 100.0, (dir as f32 + 40.0) * 0.1);
            }
        }

        let sub_runs = 6;
        let sub_dt = dt / sub_runs as f32;

        let mut gravity_time = Duration::new(0, 0);
        let mut collision_time = Duration::new(0, 0);
        let mut constraint_time = Duration::new(0, 0);
        let mut update_time = Duration::new(0, 0);
        let mut start: Instant;

        for _ in 0..sub_runs {
            start = Instant::now();
            // Apply forces
            for particle in &mut self.particles {
                particle.accelerate(GRAVITY);
            }

            gravity_time += start.elapsed();
            start = Instant::now();

            self.apply_constraints();

            constraint_time += start.elapsed();
            start = Instant::now();

            self.solve_collisions();

            collision_time += start.elapsed();
            start = Instant::now();

            // Update positions
            for particle in &mut self.particles {
                particle.update(sub_dt);
            }

            update_time += start.elapsed();
        }

        format!("Gravity: {:.2}ms\nCollisions: {:.2}ms\nConstraints: {:.2}ms\nUpdate: {:.2}ms\n",
            gravity_time.as_millis(),
            collision_time.as_millis(),
            constraint_time.as_millis(),
            update_time.as_millis())
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
                particle.old_pos = particle.pos - reflect_vec2(vel, n) * 0.999; // Apply damping
            }
        }
    }


    fn solve_collisions(&mut self) {
        let len = self.particles.len();
        let min_dist_sq = (PARTICLE_RADIUS * 2.0) * (PARTICLE_RADIUS * 2.0);

        let particles_ptr = self.particles.as_mut_ptr(); // Get raw pointer for fast access

        for i in 0..len {
            unsafe {
                let p1 = &mut *particles_ptr.add(i);
                let (x1, y1) = (p1.pos.x, p1.pos.y);

                for j in i + 1..len {
                    let p2 = &mut *particles_ptr.add(j);
                    let (x2, y2) = (p2.pos.x, p2.pos.y);

                    let dx = x1 - x2;
                    let dy = y1 - y2;
                    let dist_sq = dx * dx + dy * dy;

                    if dist_sq < min_dist_sq {
                        // Normalize vector only when needed
                        let inv_dist = (1.0 / dist_sq.sqrt()) * 0.5;
                        let n_x = dx * inv_dist;
                        let n_y = dy * inv_dist;
                        let delta = PARTICLE_RADIUS * 2.0 - dist_sq.sqrt();

                        // Move particles
                        p1.pos.x += n_x * delta;
                        p1.pos.y += n_y * delta;
                        p2.pos.x -= n_x * delta;
                        p2.pos.y -= n_y * delta;
                    }
                }
            }
        }
    }

    fn render(&self, text: &str) -> Result<(), String> {
        // Clear the screen
        clear_background(Color::from_rgba(0, 0, 0, 255));

        // Draw the center
        let center = Vec2::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0);
        let radius = 250.0;
        draw_circle(center.x, center.y, radius - PARTICLE_RADIUS, Color::from_rgba(255, 255, 255, 100));

        // Draw particles
        for particle in &self.particles {
            let x = particle.pos.x;
            let y = particle.pos.y;
            let r = PARTICLE_RADIUS;

            // Draw a filled circle
            draw_circle(x, y, r, Color::from_rgba(255, 255, 255, 255));
        }

        // Draw debug info
        draw_text(
            text,
            10.0, 10.0, 20.0, Color::from_rgba(255, 255, 255, 255)
        );

        Ok(())
    }
}

#[macroquad::main("BasicShapes")]
async fn main() {
    let mut simulation = VerletSimulation::new();
    
    let dt: f32 = 1.0 / 60.0;
    let mut frame: u32 = 0;
    let mut update_time: Duration;
    let mut render_time: Duration = Duration::new(0, 0);

    loop {
        frame += 1;

        let mut start = Instant::now();
        // Update the simulation with a fixed timestep
        let timings = simulation.update(dt, frame);

        update_time = start.elapsed();
        start = Instant::now();
        
        // Render
        simulation.render(&format!("Update: {:.2}ms\n Render: {:.2}ms\n Particles: {}\n{}", update_time.as_millis(), render_time.as_millis(), simulation.particles.len(), timings)).unwrap();

        render_time = start.elapsed();
            
        // Cap at 60 FPS
        next_frame().await
    }
}