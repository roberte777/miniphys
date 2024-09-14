pub struct Projectile {
    position: [f64; 2],
    velocity: [f64; 2],
    acceleration: [f64; 2],
}

impl Projectile {
    /// Creates a new `Projectile` instance with initial position, velocity, and acceleration.
    pub fn new(position: [f64; 2], velocity: [f64; 2], acceleration: [f64; 2]) -> Self {
        Projectile {
            position,
            velocity,
            acceleration,
        }
    }

    /// Updates the projectile's position and velocity over time.
    pub fn update(&mut self, delta_time: f64) {
        // Update velocity
        self.velocity[0] += self.acceleration[0] * delta_time;
        self.velocity[1] += self.acceleration[1] * delta_time;

        // Update position
        self.position[0] += self.velocity[0] * delta_time;
        self.position[1] += self.velocity[1] * delta_time;
    }

    pub fn position(&self) -> (f64, f64) {
        self.position.into()
    }
}
