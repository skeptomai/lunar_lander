//! Advanced rocket physics system with realistic Apollo Lunar Module specifications.
//!
//! This module implements proper rocket physics using:
//! - Tsiolkovsky rocket equation for delta-V calculations
//! - Force-based physics integration with F = ma
//! - Realistic mass flow rates and fuel consumption
//! - Apollo LM-based specifications for authenticity
//!
//! The physics system separates concerns between:
//! - `RocketEngine`: Thrust generation and fuel management
//! - `Physics`: Motion integration and force accumulation

use macroquad::prelude::*;

/// Rocket engine component with realistic propulsion parameters
#[derive(Debug, Clone)]
pub struct RocketEngine {
    pub dry_mass: f64,           // Mass without fuel (kg)
    pub fuel_mass: f64,          // Current fuel mass (kg)
    pub max_fuel_mass: f64,      // Maximum fuel capacity (kg)
    pub exhaust_velocity: f64,   // Effective exhaust velocity (m/s)
    pub max_thrust: f64,         // Maximum thrust force (N)
    pub thrust_vector: Vec2,     // Current thrust as 2D vector (N)
    pub is_thrusting: bool,      // Whether engine is firing
}

impl RocketEngine {
    /// Creates a new rocket engine with Apollo Lunar Module specifications.
    ///
    /// # Specifications
    ///
    /// - **Dry mass**: 15,000 kg (unfueled spacecraft)
    /// - **Fuel capacity**: 8,200 kg
    /// - **Exhaust velocity**: 3,050 m/s
    /// - **Maximum thrust**: 150,000 N (4x realistic for better gameplay)
    /// - **Thrust-to-weight ratio**: 4.0+ (excellent controllability)
    ///
    /// These values are based on the Apollo Lunar Module but enhanced for gameplay.
    ///
    /// # Returns
    ///
    /// A new `RocketEngine` instance with Apollo LM specifications
    pub fn new_apollo_lm() -> Self {
        Self {
            dry_mass: 15000.0,        // Apollo LM dry mass (~15,000 kg)
            fuel_mass: 8200.0,        // Apollo LM fuel mass (~8,200 kg)
            max_fuel_mass: 8200.0,    // Maximum fuel capacity
            exhaust_velocity: 3050.0, // Apollo LM engine exhaust velocity
            max_thrust: 150000.0,     // Enhanced thrust for better gameplay (4x realistic)
            thrust_vector: Vec2::ZERO,
            is_thrusting: false,
        }
    }

    /// Returns the total mass of the spacecraft (dry mass + fuel).
    ///
    /// # Returns
    ///
    /// Total mass in kilograms
    pub fn total_mass(&self) -> f64 {
        self.dry_mass + self.fuel_mass
    }

    /// Returns the current fuel level as a percentage.
    ///
    /// # Returns
    ///
    /// Fuel percentage from 0.0 to 100.0
    pub fn fuel_percentage(&self) -> f32 {
        (self.fuel_mass / self.max_fuel_mass * 100.0) as f32
    }

    /// Checks if the rocket has fuel remaining.
    ///
    /// # Returns
    ///
    /// `true` if fuel_mass > 0, `false` otherwise
    pub fn has_fuel(&self) -> bool {
        self.fuel_mass > 0.0
    }

    /// Refuels the rocket to full capacity.
    ///
    /// Resets fuel_mass to max_fuel_mass for mission restart scenarios.
    pub fn refuel(&mut self) {
        self.fuel_mass = self.max_fuel_mass;
    }

    /// Stops thrust generation and resets thrust vector to zero.
    ///
    /// This is a convenience method for input handling and emergency stops.
    pub fn stop_thrust(&mut self) {
        self.thrust_vector = Vec2::ZERO;
        self.is_thrusting = false;
    }

    /// Generates thrust force and consumes fuel based on current thrust settings.
    ///
    /// This function implements realistic rocket physics:
    /// - Thrust force is applied in the direction of `thrust_vector`
    /// - Fuel consumption follows: dm/dt = F / v_e
    /// - Only consumes fuel when actively thrusting
    ///
    /// # Arguments
    ///
    /// * `dt` - Time step in seconds
    ///
    /// # Returns
    ///
    /// Thrust force vector in Newtons, or zero vector if no fuel/not thrusting
    pub fn generate_thrust(&mut self, dt: f32) -> Vec2 {
        if !self.is_thrusting || self.fuel_mass <= 0.0 {
            self.thrust_vector = Vec2::ZERO;
            return Vec2::ZERO;
        }

        // Calculate thrust magnitude (clamped to available fuel and max thrust)
        let thrust_magnitude = self.thrust_vector.length().min(self.max_thrust as f32);

        if thrust_magnitude > 0.0 {
            // Calculate mass flow rate from thrust and exhaust velocity
            // F = dm/dt * v_e, so dm/dt = F / v_e
            let mass_flow_rate = (thrust_magnitude as f64) / self.exhaust_velocity;

            // Update fuel mass
            let fuel_consumed = mass_flow_rate * (dt as f64);
            self.fuel_mass = (self.fuel_mass - fuel_consumed).max(0.0);

            // Return thrust force vector
            self.thrust_vector
        } else {
            Vec2::ZERO
        }
    }
}

#[derive(Debug, Clone)]
pub struct Physics {
    pub velocity: Vec2,
    pub mass: f64,
    pub forces: Vec2,  // Accumulated forces for this frame
}

impl Physics {
    /// Creates a new physics component with the specified mass.
    ///
    /// # Arguments
    ///
    /// * `mass` - Initial mass in kilograms
    ///
    /// # Returns
    ///
    /// A new `Physics` instance with zero velocity and forces
    pub fn new(mass: f64) -> Self {
        Self {
            velocity: Vec2::ZERO,
            mass,
            forces: Vec2::ZERO,
        }
    }

    /// Clears accumulated forces for the next physics step.
    ///
    /// This should be called at the start of each frame to prevent
    /// force accumulation across multiple frames.
    pub fn reset_forces(&mut self) {
        self.forces = Vec2::ZERO;
    }

    /// Adds a force to the accumulated forces for this physics step.
    ///
    /// Forces are accumulated and then applied during `integrate`.
    ///
    /// # Arguments
    ///
    /// * `force` - Force vector in Newtons
    pub fn add_force(&mut self, force: Vec2) {
        self.forces += force;
    }

    /// Integrates motion using accumulated forces.
    ///
    /// This function performs numerical integration using:
    /// - F = ma (force equals mass times acceleration)
    /// - v = v₀ + at (velocity integration)
    ///
    /// # Arguments
    ///
    /// * `dt` - Time step in seconds
    pub fn integrate(&mut self, dt: f32) {
        if self.mass > 0.0 {
            let acceleration = self.forces / self.mass as f32;
            self.velocity += acceleration * dt;
        }
    }
}


/// Calculates the remaining delta-V capability using the Tsiolkovsky rocket equation.
///
/// The rocket equation: Δv = v_e × ln(m_initial / m_final)
/// where:
/// - v_e = exhaust velocity
/// - m_initial = current total mass
/// - m_final = dry mass (after all fuel is burned)
///
/// This is useful for mission planning and determining if the lander has
/// enough fuel to complete landing maneuvers.
///
/// # Arguments
///
/// * `rocket` - The rocket engine to calculate delta-V for
///
/// # Returns
///
/// Remaining delta-V in m/s, or 0.0 if no fuel remains
pub fn calculate_delta_v(rocket: &RocketEngine) -> f64 {
    if rocket.fuel_mass <= 0.0 {
        return 0.0;
    }

    let initial_mass = rocket.total_mass();
    let final_mass = rocket.dry_mass;

    // Tsiolkovsky rocket equation: Δv = v_e * ln(m_initial / m_final)
    rocket.exhaust_velocity * (initial_mass / final_mass).ln()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apollo_lm_specs() {
        let rocket = RocketEngine::new_apollo_lm();
        assert_eq!(rocket.dry_mass, 15000.0);
        assert_eq!(rocket.fuel_mass, 8200.0);
        assert_eq!(rocket.total_mass(), 23200.0);
        assert_eq!(rocket.fuel_percentage(), 100.0);
    }

    #[test]
    fn test_delta_v_calculation() {
        let rocket = RocketEngine::new_apollo_lm();
        let delta_v = calculate_delta_v(&rocket);

        // With our current values: dry_mass=15000, fuel_mass=8200, exhaust_velocity=3050
        // Δv = 3050 * ln(23200/15000) = 3050 * ln(1.547) = 3050 * 0.436 = 1330 m/s
        // This is reasonable for a lunar lander with limited fuel capacity
        assert!((delta_v - 1330.0).abs() < 50.0, "Delta-V was {}, expected ~1330", delta_v);
    }

    #[test]
    fn test_fuel_consumption() {
        let mut rocket = RocketEngine::new_apollo_lm();
        let mut physics = Physics::new(rocket.total_mass());

        rocket.is_thrusting = true;
        rocket.thrust_vector = Vec2::new(0.0, rocket.max_thrust as f32);

        let initial_fuel = rocket.fuel_mass;
        let thrust_force = rocket.generate_thrust(1.0); // 1 second

        assert!(rocket.fuel_mass < initial_fuel, "Fuel should be consumed during thrust");
        assert!(thrust_force.length() > 0.0, "Should generate thrust force");
        
        // Test force integration
        physics.add_force(thrust_force);
        physics.integrate(1.0);
        assert!(physics.velocity.length() > 0.0, "Should have velocity from thrust");
    }
}