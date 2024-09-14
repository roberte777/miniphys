pub mod cached;
use std::{f64, time::Duration};

pub enum Damping {
    OverDamped(f64),
    UnderDamped(f64),
    CriticallyDamped(f64),
}

pub struct Spring {
    initial_position: f64,
    initial_velocity: f64,
    position: f64,
    velocity: f64,
    angular_frequency: f64,
    damping: Damping,
    time: f64,
}

impl Spring {
    pub fn new(
        initial_pos: f64,
        initial_vel: f64,
        angular_frequency: f64,
        damping_ratio: f64,
    ) -> Self {
        let damping_ratio = damping_ratio.max(0.0);
        let angular_frequency = angular_frequency.max(0.);

        let damping = if damping_ratio > 1. {
            Damping::OverDamped(damping_ratio)
        } else if damping_ratio == 1. {
            Damping::CriticallyDamped(1.)
        } else {
            Damping::UnderDamped(damping_ratio)
        };

        Spring {
            position: initial_pos,
            velocity: initial_vel,
            initial_position: initial_pos,
            initial_velocity: initial_vel,
            angular_frequency,
            damping,
            time: 0.,
        }
    }

    /// Updates the position and velocity values towards the equilibrium position.
    ///
    /// - `pos`: The current position.
    /// - `vel`: The current velocity.
    /// - `equilibrium_pos`: The target equilibrium position.
    ///
    /// Returns the new position and velocity as a tuple.
    pub fn update(&mut self, delta_time: Duration, equilibrium_pos: f64) -> (f64, f64) {
        self.time += delta_time.as_millis() as f64;
        let x_initial = self.position - equilibrium_pos;
        match self.damping {
            // omega == angular frequency
            // zi == dr
            Damping::OverDamped(dr) => {
                let z1 =
                    -self.angular_frequency * dr - self.angular_frequency * (dr * dr - 1.).sqrt();
                let z2 =
                    -self.angular_frequency * dr + self.angular_frequency * (dr * dr - 1.).sqrt();
                let xt1 = ((self.initial_velocity - (x_initial * z2)) / (z1 - z2))
                    * f64::exp(z1 * self.time);

                let xt2 = (x_initial - ((self.initial_velocity - x_initial * z2) / (z1 - z2)))
                    * f64::exp(z2 * self.time);
                let xt_final = xt1 + xt2;
                println!("{}", xt_final);
                self.position = xt_final;
            }
            Damping::UnderDamped(dr) => {
                todo!()
            }
            Damping::CriticallyDamped(dr) => {
                todo!()
            }
        }
        println!("{}", self.position);
        (self.position, self.velocity)
    }
}
