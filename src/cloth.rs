use nalgebra::base::Vector2;
use std::{f64::consts::SQRT_2, time::Duration};

pub struct Particle {
    position: Vector2<f64>,
    previous_position: Vector2<f64>,
    acceleration: Vector2<f64>,
    mass: f64,
    pinned: bool,
}

impl Particle {
    fn new(position: Vector2<f64>, pinned: bool) -> Self {
        Particle {
            position,
            previous_position: position,
            acceleration: Vector2::zeros(),
            mass: 1.0,
            pinned,
        }
    }

    fn apply_force(&mut self, force: Vector2<f64>) {
        self.acceleration += force / self.mass;
    }

    /// Update based on number of seconds since last update
    fn update(&mut self, delta_time: f64) {
        if self.pinned {
            return;
        }

        // Verlet integration
        let new_pos = self.position
            + (self.position - self.previous_position) * 0.99
            + self.acceleration * 0.99 * delta_time * delta_time;

        self.previous_position = self.position;
        self.position = new_pos;
    }

    pub fn position(&self) -> Vector2<f64> {
        self.position
    }
    fn set_position(&mut self, position: Vector2<f64>) {
        self.position = position;
        self.acceleration = Vector2::zeros();
    }
    pub fn pinned(&self) -> bool {
        self.pinned
    }
}

pub struct Constraint {
    particle_a: usize,
    particle_b: usize,
    rest_length: f64,
}

impl Constraint {
    pub fn new(particle_a: usize, particle_b: usize, rest_length: f64) -> Self {
        Constraint {
            particle_a,
            particle_b,
            rest_length,
        }
    }

    pub fn particles(&self) -> (usize, usize) {
        (self.particle_a, self.particle_b)
    }

    pub fn rest_length(&self) -> f64 {
        self.rest_length
    }
}

pub struct Cloth {
    particles: Vec<Particle>,
    constraints: Vec<Constraint>,
    width: usize,
    height: usize,
    selected_particles: Vec<usize>,
    selection_offsets: Vec<Vector2<f64>>, // Stores offsets from mouse position
}

impl Cloth {
    pub fn new(width: usize, height: usize, spacing: f64) -> Self {
        let mut particles = Vec::new();
        let mut constraints = Vec::new();

        // Create particles
        for y in 0..height {
            for x in 0..width {
                let position = Vector2::new(x as f64 * spacing, y as f64 * spacing);
                // Pin the top row of particles to simulate hanging cloth
                let pinned = y == 0;
                particles.push(Particle::new(position, pinned));
            }
        }

        // Create constraints
        for y in 0..height {
            for x in 0..width {
                let index = y * width + x;

                // Structural constraints (right and below)
                if x < width - 1 {
                    let right = index + 1;
                    constraints.push(Constraint::new(index, right, spacing));
                }
                if y < height - 1 {
                    let below = index + width;
                    constraints.push(Constraint::new(index, below, spacing));
                }

                // Shear constraints (diagonals)
                if x < width - 1 && y < height - 1 {
                    let diag_right = index + width + 1;
                    constraints.push(Constraint::new(index, diag_right, spacing * SQRT_2));
                }
                if x > 0 && y < height - 1 {
                    let diag_left = index + width - 1;
                    constraints.push(Constraint::new(index, diag_left, spacing * SQRT_2));
                }

                // Bend constraints (skip one particle)
                if x < width - 2 {
                    let right = index + 2;
                    constraints.push(Constraint::new(index, right, spacing * 2.0));
                }
                if y < height - 2 {
                    let below = index + width * 2;
                    constraints.push(Constraint::new(index, below, spacing * 2.0));
                }
            }
        }

        Cloth {
            particles,
            constraints,
            width,
            height,
            selected_particles: Vec::new(),
            selection_offsets: Vec::new(),
        }
    }

    pub fn simulate(&mut self, delta_time: Duration) {
        let delta_time = delta_time.as_secs_f64();
        // update points
        for particle in self.particles.iter_mut() {
            particle.apply_force(gravity());
            particle.update(delta_time)
        }

        for _ in 0..2 {
            for constraint in self.constraints.iter() {
                let (p0_index, p1_index) = constraint.particles();
                let p0_pos = self.particles[p0_index].position;
                let p1_pos = self.particles[p1_index].position;

                let diff = p0_pos - p1_pos;
                let dist = diff.norm();
                let diff_factor = (constraint.rest_length() - dist) / dist;
                let offset = diff * diff_factor * 0.5;

                if !self.particles[p0_index].pinned() {
                    self.particles[p0_index].set_position(p0_pos + offset);
                }
                if !self.particles[p1_index].pinned() {
                    self.particles[p1_index].set_position(p1_pos - offset);
                }
            }
        }
        // Reset accelerations
        for particle in self.particles.iter_mut() {
            particle.acceleration = Vector2::zeros();
        }
    }

    /// Returns a reference to the particles.
    pub fn particles(&self) -> &Vec<Particle> {
        &self.particles
    }

    /// Returns a reference to the constraints.
    pub fn constraints(&self) -> &Vec<Constraint> {
        &self.constraints
    }

    /// Removes a constraint at the specified index.
    pub fn remove_constraint(&mut self, index: usize) {
        if index < self.constraints.len() {
            self.constraints.swap_remove(index);
        }
    }

    /// Removes constraints based on a custom condition.
    pub fn remove_constraints<F>(&mut self, mut condition: F)
    where
        F: FnMut(&Constraint) -> bool,
    {
        self.constraints.retain(|constraint| !condition(constraint));
    }

    /// Returns the width of the cloth.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns the height of the cloth.
    pub fn height(&self) -> usize {
        self.height
    }
    /// Removes all constraints connected to a given particle.
    pub fn cut_constraints_at_particle(&mut self, particle_index: usize) {
        self.constraints.retain(|constraint| {
            constraint.particle_a != particle_index && constraint.particle_b != particle_index
        });
    }

    pub fn cut_at_mouse(&mut self, mouse_position: Vector2<f64>) {
        // Find the nearest particle to the mouse position
        let (nearest_particle_index, distance) = self
            .particles()
            .iter()
            .enumerate()
            .map(|(i, particle)| (i, (particle.position() - mouse_position).magnitude()))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap();

        // If the particle is close enough, cut its constraints
        let cut_threshold = 10.0; // Adjust based on your coordinate system
        if distance < cut_threshold {
            self.cut_constraints_at_particle(nearest_particle_index);
        }
    }

    pub fn select_particles(&mut self, mouse_pos: Vector2<f64>, radius: f64) {
        self.selected_particles.clear();
        self.selection_offsets.clear();
        for (i, particle) in self.particles.iter_mut().enumerate() {
            let distance = (particle.position() - mouse_pos).magnitude();
            if distance <= radius {
                self.selected_particles.push(i);
                let offset = particle.position() - mouse_pos;
                self.selection_offsets.push(offset);
                particle.pinned = true; // Pin the particle
            }
        }
    }
    pub fn move_selected_particles(&mut self, mouse_pos: Vector2<f64>) {
        for (idx, &particle_index) in self.selected_particles.iter().enumerate() {
            let offset = self.selection_offsets[idx];
            let new_position = mouse_pos + offset;
            self.particles[particle_index].set_position(new_position);
        }
    }

    pub fn clear_selection(&mut self) {
        for &particle_index in &self.selected_particles {
            self.particles[particle_index].pinned = false; // Unpin the particle
        }
        self.selected_particles.clear();
        self.selection_offsets.clear();
    }

    pub fn selected_particles(&self) -> &Vec<usize> {
        &self.selected_particles
    }
}
const GRAVITY: f64 = 987.;
//functions to return forces
pub fn gravity() -> Vector2<f64> {
    Vector2::new(0.0, GRAVITY)
}
