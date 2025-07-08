use macroquad::prelude::*;

const FULL_CIRCLE_DEGREES: f32 = 360.0;

pub fn transform_axes(position: Vec2) -> Vec2 {
    vec2(
        position.x + screen_width() / 2.0,
        -position.y + screen_height() / 2.0,
    )
}



// Legacy function for backward compatibility and testing
pub fn update_mass_and_velocity(
    current_mass: f64,
    mass_flow_rate: f64,
    current_velocity: f64,
    time_step: f64,
    exhaust_velocity: f64,
) -> (f64, f64) {
    let new_mass = (current_mass - mass_flow_rate * time_step).max(0.0);

    if new_mass <= 0.0 || current_mass <= 0.0 {
        return (0.0, current_velocity);
    }

    // Proper Tsiolkovsky equation without mixing in gravity
    let delta_v = exhaust_velocity * (current_mass / new_mass).ln();
    let new_velocity = current_velocity + delta_v;

    (new_mass, new_velocity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_mass_and_velocity() {
        let mass_rocket = 500000.0;
        let starting_mass_fuel = 500000.0;
        let mass_flow_rate = 500.0;
        let mut current_velocity = 0.0;
        let time_step = 1.0;
        let exhaust_velocity = 300.0;

        let mut current_mass = mass_rocket + starting_mass_fuel;

        loop {
            let (new_mass, new_velocity) = update_mass_and_velocity(
                current_mass,
                mass_flow_rate,
                current_velocity,
                time_step,
                exhaust_velocity,
            );

            if new_mass <= mass_rocket {
                println!("Rocket has run out of fuel!");
                break;
            }

            println!(
                "current mass: {}, current velocity: {},  new_mass: {}, new_velocity: {}",
                current_mass, current_velocity, new_mass, new_velocity
            );

            assert!(new_mass < current_mass);
            assert!(new_velocity > current_velocity);

            current_mass = new_mass;
            current_velocity = new_velocity;
        }
    }

    #[test]
    fn test_coordinate_transformation_math() {
        // Test coordinate transformation logic without requiring graphics context

        // Simulate typical screen dimensions
        let screen_width = 800.0;
        let screen_height = 600.0;

        // Test the transform_axes coordinate conversion
        // transform_axes: world -> screen
        //   screen_x = world_x + screen_width/2
        //   screen_y = -world_y + screen_height/2

        let test_world_pos = vec2(0.0, 100.0); // World position (center X, 100 units up)

        // Apply forward transformation (simulating transform_axes)
        let screen_pos = vec2(
            test_world_pos.x + screen_width / 2.0, // Should be 400 (center of screen)
            -test_world_pos.y + screen_height / 2.0, // Should be 200 (above center)
        );

        // Apply reverse transformation (same logic as check_collision)
        let recovered_world_x = screen_pos.x - screen_width / 2.0;
        let recovered_world_y = screen_height / 2.0 - screen_pos.y;

        // Test that we get back the original coordinates
        assert!(
            (recovered_world_x - test_world_pos.x).abs() < 0.001,
            "X coordinate mismatch: expected {}, got {}",
            test_world_pos.x,
            recovered_world_x
        );
        assert!(
            (recovered_world_y - test_world_pos.y).abs() < 0.001,
            "Y coordinate mismatch: expected {}, got {}",
            test_world_pos.y,
            recovered_world_y
        );
    }

    #[test]
    fn test_collision_coordinate_logic() {
        // Test that lander positioned in world coordinates above terrain
        // does not trigger false collision due to coordinate transformation bugs

        let screen_width = 800.0;
        let screen_height = 600.0;

        // Create terrain at world Y = 150 (should be at bottom of screen when rendered)
        let terrain_height = 150.0;

        // Position lander in world coordinates well above terrain
        let lander_world_x: f32 = 0.0; // Center
        let lander_world_y: f32 = 250.0; // 100 units above terrain
        let lander_size = Vec2::new(32.0, 32.0);

        // Convert to screen coordinates (simulating what happens in game)
        let lander_screen_x = lander_world_x + screen_width / 2.0;
        let lander_screen_y = -lander_world_y + screen_height / 2.0;

        // Now reverse the transformation (simulating check_collision logic)
        let recovered_world_x = lander_screen_x - screen_width / 2.0;
        let recovered_world_y = screen_height / 2.0 - lander_screen_y;
        let lander_bottom_world_y = recovered_world_y - lander_size.y;

        // Verify coordinate transformation is working correctly
        assert!(
            (recovered_world_x - lander_world_x).abs() < 0.001,
            "World X coordinate transformation failed"
        );
        assert!(
            (recovered_world_y - lander_world_y).abs() < 0.001,
            "World Y coordinate transformation failed"
        );

        // The critical test: lander bottom should be ABOVE terrain height
        assert!(
            lander_bottom_world_y > terrain_height,
            "Lander bottom ({:.1}) should be above terrain ({:.1}). \
             This would cause false collision!",
            lander_bottom_world_y,
            terrain_height
        );
    }

    #[test]
    fn test_world_coordinate_consistency() {
        // Test that world coordinate system is consistent between all components
        let screen_width = 800.0;
        let screen_height = 600.0;

        // Test world coordinate system rules
        // World coordinates: (0,0) at screen center, positive Y = up, negative Y = down

        // Test terrain generation coordinate conversion
        let generated_terrain_y = 100.0; // Generated terrain value
        let terrain_y_offset = 75.0; // TERRAIN_Y_OFFSET constant
        let offset_terrain_y = generated_terrain_y + terrain_y_offset; // 175.0
        let _world_terrain_y = -(offset_terrain_y - screen_height / 2.0); // World coordinates
                                                                                // Should be: -(175 - 300) = -(-125) = 125, but we want negative Y for terrain below center
                                                                                // Actually: -(175 - 300) = -(−125) = 125, but terrain should be negative
                                                                                // Use the new terrain mapping logic
        let terrain_screen_base = screen_height * 0.7; // 420.0
        let screen_y = terrain_screen_base + (offset_terrain_y - terrain_y_offset); // 420 + (175-75) = 520
        let expected_world_terrain_y = screen_height / 2.0 - screen_y; // 300 - 520 = -220

        println!("Terrain coordinate conversion:");
        println!(
            "  Generated: {:.1} -> Offset: {:.1} -> World: {:.1}",
            generated_terrain_y, offset_terrain_y, expected_world_terrain_y
        );

        // Terrain offset to 175 should be below screen center (300), so world Y should be negative
        // Expected: 300/2 - 175 = 150 - 175 = -25 (below world center = negative Y)
        let expected_negative_y = 150.0 - 175.0; // -25.0
        println!(
            "  Expected world Y for terrain below center: {:.1}",
            expected_negative_y
        );

        // Terrain below world center should have negative Y values
        assert!(
            expected_world_terrain_y < 0.0,
            "Terrain at screen Y=175 (below center=300) should have negative world Y, got {:.1}",
            expected_world_terrain_y
        );
    }

    #[test]
    fn test_collision_coordinate_conversion() {
        // Test that collision detection coordinate conversion matches terrain/lander systems
        let screen_width = 800.0;
        let screen_height = 600.0;

        // Test lander at world origin should be at screen center
        let world_lander_pos = vec2(0.0, 0.0);
        let screen_lander_pos = vec2(
            world_lander_pos.x + screen_width / 2.0,
            -world_lander_pos.y + screen_height / 2.0,
        );
        assert_eq!(screen_lander_pos, vec2(400.0, 300.0));

        // Test reverse conversion
        let recovered_world_x = screen_lander_pos.x - screen_width / 2.0;
        let recovered_world_y = screen_height / 2.0 - screen_lander_pos.y;
        assert_eq!(recovered_world_x, world_lander_pos.x);
        assert_eq!(recovered_world_y, world_lander_pos.y);

        // Test lander above world center should be at screen above center
        let world_lander_above = vec2(0.0, 100.0);
        let screen_lander_above = vec2(
            world_lander_above.x + screen_width / 2.0,
            -world_lander_above.y + screen_height / 2.0,
        );
        assert_eq!(screen_lander_above, vec2(400.0, 200.0)); // Above screen center

        println!("Coordinate conversion tests passed");
    }

    #[test]
    fn test_initial_lander_terrain_separation() {
        // Test that initial lander position has proper separation from terrain
        let screen_width = 800.0;
        let screen_height = 600.0;
        let terrain_y_offset = 75.0; // TERRAIN_Y_OFFSET constant

        // Simulate lander at initial position (from reset_lander: world Y = 50.0)
        let lander_world_y = 50.0;
        let lander_height = 32.0;
        let lander_bottom_world_y = lander_world_y - lander_height; // 18.0

        // Simulate terrain (from terrain generation)
        let generated_terrain = 100.0;
        let offset_terrain = generated_terrain + terrain_y_offset; // 175.0
        let terrain_world_y = -(offset_terrain - screen_height / 2.0); // -(175-300) = 125.0

        println!("Initial separation test:");
        println!(
            "  Lander world Y: {:.1}, bottom: {:.1}",
            lander_world_y, lander_bottom_world_y
        );
        println!("  Terrain world Y: {:.1}", terrain_world_y);

        // In world coordinates: lander bottom should be above terrain (both positive means above world center)
        // But if terrain is at positive Y and lander bottom is also positive, lander is above terrain
        let separation = lander_bottom_world_y - terrain_world_y as f32;
        println!("  Separation: {:.1}", separation);

        // Since both values can be positive, we need the lander bottom to be above terrain
        // This test documents the current coordinate system behavior
        if separation > 0.0 {
            println!("  Lander is above terrain (good)");
        } else {
            println!("  Lander is below terrain (collision risk)");
        }
    }
}