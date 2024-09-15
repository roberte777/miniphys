use std::time::Duration;

pub struct Pendulum {
    angle: f64, // Current angle from the vertical (radians)
    angular_velocity: f64,
    angular_acceleration: f64,
    length: f64,  // Length of the pendulum (meters)
    gravity: f64, // Acceleration due to gravity (m/s^2)
    damping: f64, // Damping coefficient
}

impl Pendulum {
    pub fn new(length: f64, initial_angle_deg: f64, damping: f64) -> Self {
        Pendulum {
            angle: initial_angle_deg.to_radians(),
            angular_velocity: 0.0,
            angular_acceleration: 0.0,
            length,
            gravity: 9.81,
            damping,
        }
    }

    pub fn update(&mut self, delta_time: Duration) {
        let delta_time = delta_time.as_secs_f64();
        // Equation of motion for a pendulum
        self.angular_acceleration = -self.gravity / self.length * self.angle.sin();

        // Apply damping
        self.angular_acceleration -= self.damping * self.angular_velocity;

        // Update angular velocity and angle
        self.angular_velocity += self.angular_acceleration * delta_time;
        self.angle += self.angular_velocity * delta_time;
    }

    pub fn position(&self) -> (f64, f64) {
        // Calculate the x and y position based on the angle
        let x = self.length * self.angle.sin();
        let y = -self.length * self.angle.cos();
        (x, y)
    }
}
