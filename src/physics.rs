use macroquad::prelude::*;

/// Improved rocket physics component with realistic parameters
#[derive(Debug, Clone)]
pub struct RocketPhysics {
    pub dry_mass: f64,           // Mass without fuel (kg)
    pub fuel_mass: f64,          // Current fuel mass (kg)
    pub max_fuel_mass: f64,      // Maximum fuel capacity (kg)
    pub exhaust_velocity: f64,   // Effective exhaust velocity (m/s)
    pub max_thrust: f64,         // Maximum thrust force (N)
    pub thrust_vector: Vec2,     // Current thrust as 2D vector (N)
    pub is_thrusting: bool,      // Whether engine is firing
}

impl RocketPhysics {
    /// Create a new rocket with Apollo Lunar Module specifications (enhanced for gameplay)
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

    /// Get total mass of the rocket
    pub fn total_mass(&self) -> f64 {
        self.dry_mass + self.fuel_mass
    }

    /// Get fuel percentage remaining
    pub fn fuel_percentage(&self) -> f32 {
        (self.fuel_mass / self.max_fuel_mass * 100.0) as f32
    }

    /// Check if rocket has fuel remaining
    pub fn has_fuel(&self) -> bool {
        self.fuel_mass > 0.0
    }

    /// Reset fuel to maximum
    pub fn refuel(&mut self) {
        self.fuel_mass = self.max_fuel_mass;
    }

    /// Stop all thrust
    pub fn stop_thrust(&mut self) {
        self.thrust_vector = Vec2::ZERO;
        self.is_thrusting = false;
    }
}

#[derive(Debug, Clone)]
pub struct Physics {
    pub velocity: Vec2,
    pub acceleration: Vec2,
}

impl Physics {
    pub fn new() -> Self {
        Self {
            velocity: Vec2::ZERO,
            acceleration: Vec2::ZERO,
        }
    }

}

/// Advanced rocket physics using proper Tsiolkovsky equation implementation
///
/// This implementation properly handles:
/// - Variable mass flow based on actual thrust
/// - 2D thrust vectors
/// - Proper force-based acceleration
/// - Separate gravity handling
pub fn update_rocket_physics(rocket: &mut RocketPhysics, physics: &mut Physics, dt: f32) {
    if !rocket.is_thrusting || rocket.fuel_mass <= 0.0 {
        rocket.thrust_vector = Vec2::ZERO;
        return;
    }

    let current_total_mass = rocket.total_mass();

    // Calculate thrust magnitude (clamped to available fuel and max thrust)
    let thrust_magnitude = rocket.thrust_vector.length().min(rocket.max_thrust as f32);

    if thrust_magnitude > 0.0 {
        // Calculate mass flow rate from thrust and exhaust velocity
        // F = dm/dt * v_e, so dm/dt = F / v_e
        let mass_flow_rate = (thrust_magnitude as f64) / rocket.exhaust_velocity;

        // Update fuel mass
        let fuel_consumed = mass_flow_rate * (dt as f64);
        rocket.fuel_mass = (rocket.fuel_mass - fuel_consumed).max(0.0);

        // Apply thrust acceleration: F = ma, so a = F/m
        let thrust_acceleration = rocket.thrust_vector / (current_total_mass as f32);
        physics.acceleration += thrust_acceleration;
    }
}

/// Calculate delta-V capability of rocket with current fuel
/// This is useful for mission planning
pub fn calculate_delta_v(rocket: &RocketPhysics) -> f64 {
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
        let rocket = RocketPhysics::new_apollo_lm();
        assert_eq!(rocket.dry_mass, 15000.0);
        assert_eq!(rocket.fuel_mass, 8200.0);
        assert_eq!(rocket.total_mass(), 23200.0);
        assert_eq!(rocket.fuel_percentage(), 100.0);
    }

    #[test]
    fn test_delta_v_calculation() {
        let rocket = RocketPhysics::new_apollo_lm();
        let delta_v = calculate_delta_v(&rocket);

        // With our current values: dry_mass=15000, fuel_mass=8200, exhaust_velocity=3050
        // Δv = 3050 * ln(23200/15000) = 3050 * ln(1.547) = 3050 * 0.436 = 1330 m/s
        // This is reasonable for a lunar lander with limited fuel capacity
        assert!((delta_v - 1330.0).abs() < 50.0, "Delta-V was {}, expected ~1330", delta_v);
    }

    #[test]
    fn test_fuel_consumption() {
        let mut rocket = RocketPhysics::new_apollo_lm();
        let mut physics = Physics::new();

        rocket.is_thrusting = true;
        rocket.thrust_vector = Vec2::new(0.0, rocket.max_thrust as f32);

        let initial_fuel = rocket.fuel_mass;
        update_rocket_physics(&mut rocket, &mut physics, 1.0); // 1 second

        assert!(rocket.fuel_mass < initial_fuel, "Fuel should be consumed during thrust");
        assert!(physics.acceleration.length() > 0.0, "Should have acceleration from thrust");
    }
}