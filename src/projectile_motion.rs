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

// air resistance
// fn update(&mut self, delta_time: f64) {
//     // Air resistance parameters
//     let drag_coefficient = 0.05; // Adjust this value as needed
//     let speed = (self.velocity[0].powi(2) + self.velocity[1].powi(2)).sqrt();
//
//     // Calculate drag force components
//     let drag_force_x = -drag_coefficient * self.velocity[0] * speed;
//     let drag_force_y = -drag_coefficient * self.velocity[1] * speed;
//
//     // Update accelerations
//     self.acceleration[0] = drag_force_x;
//     self.acceleration[1] = -9.81 + drag_force_y; // Gravity plus drag
//
//     // Update velocities
//     self.velocity[0] += self.acceleration[0] * delta_time;
//     self.velocity[1] += self.acceleration[1] * delta_time;
//
//     // Update positions
//     self.position[0] += self.velocity[0] * delta_time;
//     self.position[1] += self.velocity[1] * delta_time;
// }
