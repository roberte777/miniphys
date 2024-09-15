use std::{f64::consts::SQRT_2, time::Duration};

#[derive(Clone, Copy, Debug)]
pub struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
    pub fn new(x: f64, y: f64) -> Self {
        Vec2 { x, y }
    }
    pub fn normalize(self) -> Self {
        let len = self.length();
        if len == 0.0 {
            return Vec2::zero();
        }
        self.mul(len.recip())
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn zero() -> Self {
        Vec2 { x: 0.0, y: 0.0 }
    }

    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn sub(&self, other: &Vec2) -> Vec2 {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    pub fn add(&self, other: &Vec2) -> Vec2 {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    pub fn mul(&self, scalar: f64) -> Vec2 {
        Vec2 {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }

    pub fn div(&self, scalar: f64) -> Vec2 {
        Vec2 {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }
}

pub struct Particle {
    position: Vec2,
    previous_position: Vec2,
    acceleration: Vec2,
    mass: f64,
    pinned: bool,
}

impl Particle {
    fn new(position: Vec2, pinned: bool) -> Self {
        Particle {
            position,
            previous_position: position,
            acceleration: Vec2::zero(),
            mass: 1.0,
            pinned,
        }
    }

    fn apply_force(&mut self, force: Vec2) {
        self.acceleration = self.acceleration.add(&force.div(self.mass));
    }

    /// Update based on number of seconds since last update
    fn update(&mut self, delta_time: f64) {
        if self.pinned {
            return;
        }

        let temp = self.position;

        // Verlet integration
        let velocity = self.position.sub(&self.previous_position);
        self.position = self
            .position
            .add(&velocity)
            .add(&self.acceleration.mul(delta_time * delta_time));

        self.previous_position = temp;
    }

    pub fn position(&self) -> &Vec2 {
        &self.position
    }
    fn set_position(&mut self, position: Vec2) {
        self.position = position;
        self.previous_position = position; // Prevent sudden velocity changes
        self.acceleration = Vec2::zero();
    }
    pub fn pinned(&self) -> bool {
        self.pinned
    }

    /// Calculate veloctiy based on change in position and change in time (seconds)
    fn velocity(&self, delta_time: f64) -> Vec2 {
        self.position.sub(&self.previous_position).div(delta_time)
    }

    /// Calculate damping force based on seconds since last update
    fn damping_force(&self, delta_time: f64) -> Vec2 {
        let velocity = self.velocity(delta_time);
        velocity.mul(-DAMPING_CONSTANT)
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
    selection_offsets: Vec<Vec2>, // Stores offsets from mouse position
}

impl Cloth {
    pub fn new(width: usize, height: usize, spacing: f64) -> Self {
        let mut particles = Vec::new();
        let mut constraints = Vec::new();

        // Create particles
        for y in 0..height {
            for x in 0..width {
                let position = Vec2::new(x as f64 * spacing, y as f64 * spacing);
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
        // Apply gravity and external forces
        for particle in self.particles.iter_mut() {
            if !particle.pinned {
                particle.apply_force(gravity());
                let damping = particle.damping_force(delta_time);
                particle.apply_force(damping);
            }
        }

        // Apply spring forces
        for constraint in self.constraints.iter() {
            let (index1, index2) = constraint.particles();
            let p1 = self.particles[index1].position();
            let p2 = self.particles[index2].position();
            let force = hookes_law(p1, p2, constraint.rest_length);

            if !self.particles[index1].pinned {
                self.particles[index1].apply_force(force);
            }
            if !self.particles[index2].pinned {
                self.particles[index2].apply_force(force.mul(-1.0));
            }
        }

        // Update particle positions
        for particle in self.particles.iter_mut() {
            particle.update(delta_time);
        }

        // Reset accelerations
        for particle in self.particles.iter_mut() {
            particle.acceleration = Vec2::zero();
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

    pub fn cut_at_mouse(&mut self, mouse_position: Vec2) {
        // Find the nearest particle to the mouse position
        let (nearest_particle_index, distance) = self
            .particles()
            .iter()
            .enumerate()
            .map(|(i, particle)| (i, particle.position().sub(&mouse_position).length()))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap();

        // If the particle is close enough, cut its constraints
        let cut_threshold = 10.0; // Adjust based on your coordinate system
        if distance < cut_threshold {
            self.cut_constraints_at_particle(nearest_particle_index);
        }
    }

    pub fn select_particles(&mut self, mouse_pos: Vec2, radius: f64) {
        self.selected_particles.clear();
        self.selection_offsets.clear();
        for (i, particle) in self.particles.iter_mut().enumerate() {
            let distance = particle.position().sub(&mouse_pos).length();
            if distance <= radius {
                self.selected_particles.push(i);
                let offset = particle.position().sub(&mouse_pos);
                self.selection_offsets.push(offset);
                particle.pinned = true; // Pin the particle
            }
        }
    }
    pub fn move_selected_particles(&mut self, mouse_pos: Vec2) {
        for (idx, &particle_index) in self.selected_particles.iter().enumerate() {
            let offset = self.selection_offsets[idx];
            let new_position = mouse_pos.add(&offset);
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
const SPRING_CONSTANT: f64 = 500.;
const DAMPING_CONSTANT: f64 = 5.0;
//functions to return forces
pub fn gravity() -> Vec2 {
    Vec2::new(0.0, GRAVITY)
}

pub fn hookes_law(p1: &Vec2, p2: &Vec2, rest_length: f64) -> Vec2 {
    // spring_force = -k * displacement
    let displacement = p2.sub(p1);
    let displacement = displacement
        .normalize()
        .mul(rest_length - displacement.length());

    displacement.mul(-SPRING_CONSTANT)
}

pub fn damping_force(velocity: &Vec2) -> Vec2 {
    // damping_force = -c * relative_velocity
    let relative_velocity = velocity;

    relative_velocity.mul(-DAMPING_CONSTANT)
}
