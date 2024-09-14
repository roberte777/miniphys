/******************************************************************************

  Copyright (c) 2008-2012 Ryan Juckett
  http://www.ryanjuckett.com/

  This software is provided 'as-is', without any express or implied
  warranty. In no event will the authors be held liable for any damages
  arising from the use of this software.

  Permission is granted to anyone to use this software for any purpose,
  including commercial applications, and to alter it and redistribute it
  freely, subject to the following restrictions:

  1. The origin of this software must not be misrepresented; you must not
     claim that you wrote the original software. If you use this software
     in a product, an acknowledgment in the product documentation would be
     appreciated but is not required.

  2. Altered source versions must be plainly marked as such, and must not be
     misrepresented as being the original software.

  3. This notice may not be removed or altered from any source
     distribution.

*******************************************************************************

  Ported to Rust by Ethan Wilkes in 2024.

******************************************************************************/
use std::f64;

/// An object representing a simplified damped harmonic oscillator, as written
/// by [Ryan Juckett](http://www.ryanjuckett.com/). I have not tried to update
/// the API for this code
///
/// A [`Spring`] object represents a cached set of coefficients that can be used
/// to update any position and velocities values as if they were attached via
/// spring to a certain equillibrium point. This point can change (for example,
/// if you wanted a circle to follow your mouse).
///
/// The position and velocity values are one dimensional. If you would like to
/// use this code to create 2D or 3D spring motion, try something like this:
///
/// ```rust
/// use miniphys::spring::{fps, Spring};
///
/// let delta_time = fps(60);
/// let spring_cache = Spring::new(delta_time, 6.0, 0.5);
/// let mut x_pos = 0.;
/// let mut y_pos = 0.;
/// let mut x_vel = 0.;
/// let mut y_vel = 0.;
///
/// (x_pos, x_vel) = spring_cache.update(x_pos, x_vel, 0.0);
/// (y_pos, y_vel) = spring_cache.update(y_pos, y_vel, 0.0);
/// ```
pub struct Spring {
    pos_pos_coef: f64,
    pos_vel_coef: f64,
    vel_pos_coef: f64,
    vel_vel_coef: f64,
}

impl Spring {
    /// Creates a new `Spring` instance, computing the parameters needed to
    /// simulate a damped spring over a given period of time.
    ///
    /// - `delta_time`: The time step to advance (essentially the frame duration).
    /// - `angular_frequency`: The angular frequency of motion, affecting the speed.
    /// - `damping_ratio`: The damping ratio of motion, determining the oscillation.
    ///
    /// Damping ratio categories:
    /// - > 1: Over-damped (no oscillation, slower to equilibrium).
    /// - = 1: Critically damped (fastest to equilibrium without oscillation).
    /// - < 1: Under-damped (fastest to equilibrium with oscillation).
    pub fn new(delta_time: f64, angular_frequency: f64, damping_ratio: f64) -> Self {
        // let epsilon = f64::EPSILON;
        let epsilon = 0.0001;

        // Ensure angular frequency and damping ratio are non-negative.
        let angular_frequency = angular_frequency.max(0.0);
        let damping_ratio = damping_ratio.max(0.0);

        // If angular frequency is negligible, return identity coefficients.
        if angular_frequency < epsilon {
            return Spring {
                pos_pos_coef: 1.0,
                pos_vel_coef: 0.0,
                vel_pos_coef: 0.0,
                vel_vel_coef: 1.0,
            };
        }

        let pos_pos_coef;
        let pos_vel_coef;
        let vel_pos_coef;
        let vel_vel_coef;

        if damping_ratio > 1.0 + epsilon {
            // Over-damped.
            let za = -angular_frequency * damping_ratio;
            let zb = angular_frequency * (damping_ratio * damping_ratio - 1.0).sqrt();
            let z1 = za - zb;
            let z2 = za + zb;

            let e1 = (z1 * delta_time).exp();
            let e2 = (z2 * delta_time).exp();

            let inv_two_zb = 1.0 / (2.0 * zb);

            let e1_over_two_zb = e1 * inv_two_zb;
            let e2_over_two_zb = e2 * inv_two_zb;

            let z1e1_over_two_zb = z1 * e1_over_two_zb;
            let z2e2_over_two_zb = z2 * e2_over_two_zb;

            pos_pos_coef = e1_over_two_zb * z2 - z2e2_over_two_zb + e2;
            pos_vel_coef = -e1_over_two_zb + e2_over_two_zb;

            vel_pos_coef = (z1e1_over_two_zb - z2e2_over_two_zb + e2) * z2;
            vel_vel_coef = -z1e1_over_two_zb + z2e2_over_two_zb;
        } else if damping_ratio < 1.0 - epsilon {
            // Under-damped.
            let omega_zeta = angular_frequency * damping_ratio;
            let alpha = angular_frequency * (1.0 - damping_ratio * damping_ratio).sqrt();

            let exp_term = (-omega_zeta * delta_time).exp();
            let cos_term = (alpha * delta_time).cos();
            let sin_term = (alpha * delta_time).sin();

            let inv_alpha = 1.0 / alpha;

            let exp_sin = exp_term * sin_term;
            let exp_cos = exp_term * cos_term;
            let exp_omega_zeta_sin_over_alpha = exp_term * omega_zeta * sin_term * inv_alpha;

            pos_pos_coef = exp_cos + exp_omega_zeta_sin_over_alpha;
            pos_vel_coef = exp_sin * inv_alpha;

            vel_pos_coef = -exp_sin * alpha - omega_zeta * exp_omega_zeta_sin_over_alpha;
            vel_vel_coef = -exp_omega_zeta_sin_over_alpha + exp_cos;
        } else {
            // Critically damped.
            let exp_term = (-angular_frequency * delta_time).exp();
            let time_exp = delta_time * exp_term;
            let time_exp_freq = time_exp * angular_frequency;

            pos_pos_coef = time_exp_freq + exp_term;
            pos_vel_coef = time_exp;

            vel_pos_coef = -angular_frequency * time_exp_freq;
            vel_vel_coef = -time_exp_freq + exp_term;
        }

        Spring {
            pos_pos_coef,
            pos_vel_coef,
            vel_pos_coef,
            vel_vel_coef,
        }
    }

    /// Updates the position and velocity values towards the equilibrium position.
    ///
    /// - `pos`: The current position.
    /// - `vel`: The current velocity.
    /// - `equilibrium_pos`: The target equilibrium position.
    ///
    /// Returns the new position and velocity as a tuple.
    pub fn update(&self, pos: f64, vel: f64, equilibrium_pos: f64) -> (f64, f64) {
        let old_pos = pos - equilibrium_pos;
        let old_vel = vel;

        let new_pos = old_pos * self.pos_pos_coef + old_vel * self.pos_vel_coef + equilibrium_pos;
        let new_vel = old_pos * self.vel_pos_coef + old_vel * self.vel_vel_coef;

        (new_pos, new_vel)
    }
}

/// Calculates the time delta for a given number of frames per second.
/// This value can be used as the time delta when initializing a `Spring`.
///
/// # Example
///
/// ```rust
/// use miniphys::spring::fps;
///
/// let delta_time = fps(60);
/// ```
pub fn fps(n: u32) -> f64 {
    1.0 / n as f64
}
