use macroquad::prelude::*;

use crate::entity::Entity;
use crate::surface::{LandingZone, LandingZoneDifficulty};

const COLLISION_MARGIN: f32 = 3.0;
const LEG_HEIGHT_RATIO: f32 = 0.25; // Bottom 25% is legs
const LEG_WIDTH_RATIO: f32 = 0.3; // Each leg takes 30% of width (20% gap in middle)
const MAX_LANDING_VELOCITY: f32 = 10.0; // Maximum safe landing speed
const MAX_LANDING_ANGLE_DEGREES: f32 = 15.0; // Maximum angle from vertical for safe landing

#[derive(Debug, PartialEq)]
pub enum CollisionType {
    None,
    LegCollision,
    BodyCollision,
    LandingSuccess,
}

pub fn get_landing_zone_info(terrain_indices: &[usize], landing_zones: &[LandingZone], _lander_width_terrain_points: usize) -> Option<(LandingZoneDifficulty, f32, f32)> {
    // Check if the lander is properly positioned within any landing zone
    // Returns (difficulty, distance_from_left_edge, distance_from_right_edge) if successful
    if terrain_indices.is_empty() {
        return None;
    }
    
    // Find the span of terrain indices the lander is touching
    let min_idx = *terrain_indices.iter().min().unwrap();
    let max_idx = *terrain_indices.iter().max().unwrap();
    
    for zone in landing_zones {
        // For a successful landing, the entire lander span must be within the zone
        // No tolerance - require exact positioning within the zone boundaries
        if min_idx >= zone.start && max_idx <= zone.end {
            // Calculate distances from zone edges (in terrain points)
            let distance_from_left = (min_idx - zone.start) as f32;
            let distance_from_right = (zone.end - max_idx) as f32;
            
            return Some((zone.difficulty, distance_from_left, distance_from_right));
        }
    }
    
    None
}

// Legacy function for backward compatibility
pub fn is_on_landing_zone(terrain_indices: &[usize], landing_zones: &[LandingZone], lander_width_terrain_points: usize) -> Option<LandingZoneDifficulty> {
    get_landing_zone_info(terrain_indices, landing_zones, lander_width_terrain_points)
        .map(|(difficulty, _, _)| difficulty)
}

// Legacy function for backward compatibility
pub fn is_on_flat_spot(terrain_indices: &[usize], flat_spot_range: (usize, usize), _lander_width_terrain_points: usize) -> bool {
    // Check if the entire lander is within the flat spot range
    // Updated to match new strict positioning requirements
    if terrain_indices.is_empty() {
        return false;
    }

    let (flat_start, flat_end) = flat_spot_range;
    
    // Find the span of terrain indices the lander is touching
    let min_idx = *terrain_indices.iter().min().unwrap();
    let max_idx = *terrain_indices.iter().max().unwrap();
    
    // Require entire lander to be within the flat spot
    min_idx >= flat_start && max_idx <= flat_end
}

pub fn check_collision(entity: &Entity) -> CollisionType {
    // CAMERA COORDINATE COLLISION DETECTION
    // Both lander and terrain are already in camera coordinates:
    // - Lander position: stored in camera coordinates (entity.transform.position)
    // - Terrain Y values: stored directly as camera Y coordinates
    // - Terrain X mapping: array indices 0-1000 map to camera X range

    let _screen_width = macroquad::window::screen_width();

    // Lander position in camera coordinates (already correct)
    let lander_x = entity.transform.position.x;
    let lander_y = entity.transform.position.y;
    let lander_width = entity.transform.size.x;
    let lander_height = entity.transform.size.y;

    // Calculate lander bottom in camera coordinates
    // In camera coordinates: Y increases UPWARD (due to -2.0/screen_height zoom), so bottom = Y position
    let lander_bottom_y = lander_y;

    // Convert lander camera X position to terrain array indices
    // Simple 1:1 mapping: camera_x = i, so i = camera_x
    let lander_left_x = lander_x;
    let lander_right_x = lander_x + lander_width;

    // Convert to terrain array indices (simple 1:1 mapping)
    let terrain_start_idx = (lander_left_x as i32).max(0) as usize;
    let terrain_end_idx = (lander_right_x as i32).min((entity.terrain.len() - 1) as i32) as usize;

    // Safety bounds check
    if terrain_start_idx >= entity.terrain.len() || terrain_end_idx >= entity.terrain.len() {
        return CollisionType::None;
    }

    // Collision zones - divide lander into legs and body
    // Legs: bottom 25% of lander, only at the edges (left and right 30% of width)
    // Body: upper 75% of lander, or center 40% of width at bottom

    // Define collision zones - corrected for camera coordinates (Y increases upward)
    let leg_zone_bottom = lander_bottom_y; // Bottom of lander (lower Y value)
    let leg_zone_top = lander_bottom_y + (lander_height * LEG_HEIGHT_RATIO); // 25% up from bottom
    let body_zone_bottom = leg_zone_top; // Body starts where legs end

    // Leg collision areas (left and right edges)
    let leg_width = lander_width * LEG_WIDTH_RATIO;
    let left_leg_start = lander_left_x;
    let left_leg_end = lander_left_x + leg_width;
    let right_leg_start = lander_right_x - leg_width;
    let right_leg_end = lander_right_x;

    // Body collision area (center section)
    let body_left = left_leg_end;
    let body_right = right_leg_start;

    // Check for collisions in different zones and collect terrain indices under lander
    let mut leg_collision = false;
    let mut body_collision = false;
    let mut collision_terrain_indices = Vec::new();

    for i in terrain_start_idx..=terrain_end_idx {
        let terrain_y = entity.terrain[i] as f32;
        let terrain_x = i as f32; // Simple 1:1 mapping

        // Check leg collisions (only at lander bottom, in leg zones)
        if leg_zone_bottom <= terrain_y + COLLISION_MARGIN {
            if (terrain_x >= left_leg_start && terrain_x <= left_leg_end)
                || (terrain_x >= right_leg_start && terrain_x <= right_leg_end)
            {
                leg_collision = true;
                collision_terrain_indices.push(i);
                info!(
                    "LEG COLLISION: terrain_idx={}, leg_bottom={:.1}, terrain_y={:.1}",
                    i, leg_zone_bottom, terrain_y
                );
            }
        }

        // Check body collision (center section or higher up)
        if body_zone_bottom <= terrain_y + COLLISION_MARGIN {
            if terrain_x >= body_left && terrain_x <= body_right {
                body_collision = true;
                collision_terrain_indices.push(i);
                info!(
                    "BODY COLLISION: terrain_idx={}, body_bottom={:.1}, terrain_y={:.1}",
                    i, body_zone_bottom, terrain_y
                );
            }
        }
    }

    // Determine collision type based on landing zones, velocity, and collision zones
    // CRITICAL: Only landing zones are safe landing spots!
    if leg_collision {
        // Check if landing on any landing zone (mandatory for success)
        let screen_width = macroquad::window::screen_width();
        let terrain_points_per_pixel = 1000.0 / (screen_width * 2.0);
        let lander_width_terrain_points = (entity.transform.size.x * terrain_points_per_pixel) as usize;
        
        let landing_zone_info = get_landing_zone_info(&collision_terrain_indices, &entity.landing_zones, lander_width_terrain_points);

        if let Some((difficulty, dist_left, dist_right)) = landing_zone_info {
            info!("LANDING ON {} ZONE: {} difficulty, distances: {:.1} from left edge, {:.1} from right edge", 
                  difficulty.name().to_uppercase(), difficulty.name(), dist_left, dist_right);
        } else {
            info!("ROUGH TERRAIN LANDING: Not on any landing zone - Mission Failed!");
            return CollisionType::LegCollision;
        }

        // On landing zone - now check velocity and angle for success vs crash
        if let Some(physics) = &entity.physics {
            let landing_velocity = physics.velocity.length();
            
            // Check lander angle relative to vertical (90 degrees is straight up)
            // Convert rotation from degrees to a normalized angle from vertical
            let lander_angle = entity.transform.rotation;
            // Normalize to 0-360 range
            let normalized_angle = lander_angle.rem_euclid(360.0);
            // Calculate deviation from vertical (90 degrees)
            let angle_from_vertical = (normalized_angle - 90.0).abs();
            // Handle wraparound case (e.g., 270 degrees = 90 degrees from vertical on other side)
            let angle_deviation = if angle_from_vertical > 180.0 {
                360.0 - angle_from_vertical
            } else {
                angle_from_vertical
            };
            
            // Check both velocity and angle requirements
            let velocity_ok = landing_velocity <= MAX_LANDING_VELOCITY;
            let angle_ok = angle_deviation <= MAX_LANDING_ANGLE_DEGREES;
            
            if velocity_ok && angle_ok {
                if let Some((difficulty, dist_left, dist_right)) = landing_zone_info {
                    info!(
                        "SUCCESSFUL LANDING: velocity={:.1}, angle={:.1}° from vertical on {} zone (edges: {:.1}L, {:.1}R)",
                        landing_velocity, angle_deviation, difficulty.name().to_lowercase(), dist_left, dist_right
                    );
                } else {
                    info!(
                        "SUCCESSFUL LANDING: velocity={:.1}, angle={:.1}° from vertical",
                        landing_velocity, angle_deviation
                    );
                }
                CollisionType::LandingSuccess
            } else {
                let (zone_name, edge_info) = if let Some((difficulty, dist_left, dist_right)) = landing_zone_info {
                    (difficulty.name().to_lowercase(), format!(" (edges: {:.1}L, {:.1}R)", dist_left, dist_right))
                } else {
                    ("landing".to_string(), String::new())
                };
                
                if !velocity_ok && !angle_ok {
                    info!(
                        "HARD LANDING: velocity={:.1} > {:.1} AND angle={:.1}° > {:.1}° on {} zone{}",
                        landing_velocity, MAX_LANDING_VELOCITY, angle_deviation, MAX_LANDING_ANGLE_DEGREES, zone_name, edge_info
                    );
                } else if !velocity_ok {
                    info!(
                        "HARD LANDING: velocity={:.1} > {:.1} on {} zone (angle ok: {:.1}°){}",
                        landing_velocity, MAX_LANDING_VELOCITY, zone_name, angle_deviation, edge_info
                    );
                } else {
                    info!(
                        "TILTED LANDING: angle={:.1}° > {:.1}° on {} zone (velocity ok: {:.1}){}",
                        angle_deviation, MAX_LANDING_ANGLE_DEGREES, zone_name, landing_velocity, edge_info
                    );
                }
                CollisionType::LegCollision
            }
        } else {
            CollisionType::LegCollision
        }
    } else if body_collision {
        CollisionType::BodyCollision
    } else {
        CollisionType::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{Entity, Transform, Collision};
    use crate::physics::Physics;

    #[test]
    fn test_is_on_flat_spot() {
        // Test that lander on flat spot is detected correctly
        let flat_spot_range = (100, 130); // Flat spot from index 100 to 130
        let lander_width_terrain_points = 20;
        
        // Test lander fully on flat spot
        let terrain_indices = vec![110, 115, 120];
        assert!(is_on_flat_spot(&terrain_indices, flat_spot_range, lander_width_terrain_points));
        
        // Test lander partially on flat spot (should now FAIL with strict requirements)
        let terrain_indices = vec![98, 102]; // Partially outside zone (98 < 100)
        assert!(!is_on_flat_spot(&terrain_indices, flat_spot_range, lander_width_terrain_points), "Partial overlap should fail");
        
        // Test lander exactly on flat spot boundaries (should succeed)
        let terrain_indices = vec![100, 105, 130]; // Exactly within zone boundaries
        assert!(is_on_flat_spot(&terrain_indices, flat_spot_range, lander_width_terrain_points), "Exact boundaries should succeed");
        
        // Test lander completely off flat spot
        let terrain_indices = vec![50, 55, 60];
        assert!(!is_on_flat_spot(&terrain_indices, flat_spot_range, lander_width_terrain_points));
        
        // Test empty terrain indices
        let terrain_indices = vec![];
        assert!(!is_on_flat_spot(&terrain_indices, flat_spot_range, lander_width_terrain_points));
    }

    #[test]
    fn test_landing_angle_requirements() {
        // Test the angle calculation logic used in collision detection
        // This tests the same logic as used in check_collision without requiring macroquad context
        
        // Test 1: Vertical landing (90 degrees) - should succeed
        let rotation: f32 = 90.0;
        let normalized_angle = rotation.rem_euclid(360.0);
        let angle_from_vertical = (normalized_angle - 90.0).abs();
        let angle_deviation = if angle_from_vertical > 180.0 {
            360.0 - angle_from_vertical
        } else {
            angle_from_vertical
        };
        assert!(angle_deviation <= MAX_LANDING_ANGLE_DEGREES, 
                "Vertical lander ({}°) should be within angle limit, deviation: {}°", 
                rotation, angle_deviation);
        
        // Test 2: Slightly tilted (10 degrees from vertical) - should succeed  
        let rotation: f32 = 80.0; // 10 degrees left of vertical
        let normalized_angle = rotation.rem_euclid(360.0);
        let angle_from_vertical = (normalized_angle - 90.0).abs();
        let angle_deviation = if angle_from_vertical > 180.0 {
            360.0 - angle_from_vertical
        } else {
            angle_from_vertical
        };
        assert!(angle_deviation <= MAX_LANDING_ANGLE_DEGREES, 
                "Slightly tilted lander ({}°) should be within angle limit, deviation: {}°", 
                rotation, angle_deviation);
        
        // Test 3: Too tilted (20 degrees from vertical) - should fail
        let rotation: f32 = 70.0; // 20 degrees left of vertical
        let normalized_angle = rotation.rem_euclid(360.0);
        let angle_from_vertical = (normalized_angle - 90.0).abs();
        let angle_deviation = if angle_from_vertical > 180.0 {
            360.0 - angle_from_vertical
        } else {
            angle_from_vertical
        };
        assert!(angle_deviation > MAX_LANDING_ANGLE_DEGREES, 
                "Heavily tilted lander ({}°) should exceed angle limit, deviation: {}°", 
                rotation, angle_deviation);
        
        // Test 4: Upside down (270 degrees = 180 degrees from vertical) - should fail
        let rotation: f32 = 270.0;
        let normalized_angle = rotation.rem_euclid(360.0);
        let angle_from_vertical = (normalized_angle - 90.0).abs();
        let angle_deviation = if angle_from_vertical > 180.0 {
            360.0 - angle_from_vertical
        } else {
            angle_from_vertical
        };
        assert!(angle_deviation > MAX_LANDING_ANGLE_DEGREES, 
                "Upside down lander ({}°) should exceed angle limit, deviation: {}°", 
                rotation, angle_deviation);
        
        // Test 5: Right-side tilted (100 degrees = 10 degrees right of vertical) - should succeed
        let rotation: f32 = 100.0; // 10 degrees right of vertical
        let normalized_angle = rotation.rem_euclid(360.0);
        let angle_from_vertical = (normalized_angle - 90.0).abs();
        let angle_deviation = if angle_from_vertical > 180.0 {
            360.0 - angle_from_vertical
        } else {
            angle_from_vertical
        };
        assert!(angle_deviation <= MAX_LANDING_ANGLE_DEGREES, 
                "Right-tilted lander ({}°) should be within angle limit, deviation: {}°", 
                rotation, angle_deviation);
        
        // Test 6: Edge case - exactly at the limit (15 degrees from vertical) - should succeed
        let rotation: f32 = 75.0; // Exactly 15 degrees left of vertical
        let normalized_angle = rotation.rem_euclid(360.0);
        let angle_from_vertical = (normalized_angle - 90.0).abs();
        let angle_deviation = if angle_from_vertical > 180.0 {
            360.0 - angle_from_vertical
        } else {
            angle_from_vertical
        };
        assert!(angle_deviation <= MAX_LANDING_ANGLE_DEGREES, 
                "Lander at exact limit ({}°) should be within angle limit, deviation: {}°", 
                rotation, angle_deviation);
    }
    
    #[test]
    fn test_multiple_landing_zones() {
        use crate::surface::{LandingZone, LandingZoneDifficulty};
        
        // Create test landing zones with different difficulties
        let landing_zones = vec![
            LandingZone {
                start: 100,
                end: 120,
                difficulty: LandingZoneDifficulty::Hard,
                width_points: 20,
            },
            LandingZone {
                start: 200,
                end: 225,
                difficulty: LandingZoneDifficulty::Medium,
                width_points: 25,
            },
            LandingZone {
                start: 300,
                end: 330,
                difficulty: LandingZoneDifficulty::Easy,
                width_points: 30,
            },
        ];
        
        let lander_width_terrain_points = 20;
        
        // Test landing on hard zone
        let terrain_indices = vec![110, 115]; // Within hard zone (100-120)
        let result = get_landing_zone_info(&terrain_indices, &landing_zones, lander_width_terrain_points);
        assert_eq!(result, Some((LandingZoneDifficulty::Hard, 10.0, 5.0))); // 10 from left (110-100), 5 from right (120-115)
        
        // Test landing on medium zone
        let terrain_indices = vec![210, 215]; // Within medium zone (200-225)
        let result = get_landing_zone_info(&terrain_indices, &landing_zones, lander_width_terrain_points);
        assert_eq!(result, Some((LandingZoneDifficulty::Medium, 10.0, 10.0))); // 10 from left (210-200), 10 from right (225-215)
        
        // Test landing on easy zone
        let terrain_indices = vec![315, 320]; // Within easy zone (300-330)
        let result = get_landing_zone_info(&terrain_indices, &landing_zones, lander_width_terrain_points);
        assert_eq!(result, Some((LandingZoneDifficulty::Easy, 15.0, 10.0))); // 15 from left (315-300), 10 from right (330-320)
        
        // Test landing on rough terrain (not in any zone)
        let terrain_indices = vec![50, 55]; // Outside all zones
        let result = get_landing_zone_info(&terrain_indices, &landing_zones, lander_width_terrain_points);
        assert_eq!(result, None);
        
        // Test strict positioning - landing partially outside zone should fail
        let terrain_indices = vec![97, 102]; // Spans outside zone start (97 < 100) - should fail
        let result = get_landing_zone_info(&terrain_indices, &landing_zones, lander_width_terrain_points);
        assert_eq!(result, None, "Should NOT detect landing when partially outside zone");
        
        // Test edge case - landing exactly at zone boundaries should succeed
        let terrain_indices = vec![100, 120]; // Exactly at zone boundaries (100-120)
        let result = get_landing_zone_info(&terrain_indices, &landing_zones, lander_width_terrain_points);
        assert_eq!(result, Some((LandingZoneDifficulty::Hard, 0.0, 0.0)), "Should detect landing exactly at zone boundaries with zero tolerance");
        
        // Test legacy function still works
        let terrain_indices = vec![110, 115];
        let legacy_result = is_on_landing_zone(&terrain_indices, &landing_zones, lander_width_terrain_points);
        assert_eq!(legacy_result, Some(LandingZoneDifficulty::Hard));
    }
}