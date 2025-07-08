use macroquad::prelude::*;

use crate::entity::Entity;

const COLLISION_MARGIN: f32 = 3.0;
const LEG_HEIGHT_RATIO: f32 = 0.25; // Bottom 25% is legs
const LEG_WIDTH_RATIO: f32 = 0.3; // Each leg takes 30% of width (20% gap in middle)
const MAX_LANDING_VELOCITY: f32 = 10.0; // Maximum safe landing speed

#[derive(Debug, PartialEq)]
pub enum CollisionType {
    None,
    LegCollision,
    BodyCollision,
    LandingSuccess,
}

pub fn is_on_flat_spot(terrain_indices: &[usize], flat_spot_range: (usize, usize), lander_width_terrain_points: usize) -> bool {
    // Check if any of the collision terrain indices are within the known flat spot range
    // Use 1.3x lander width tolerance as requested
    if terrain_indices.is_empty() {
        return false;
    }

    let (flat_start, flat_end) = flat_spot_range;
    
    // Calculate tolerance: 1.3x lander width means 0.3x extra width total
    // Split evenly on both sides: 0.15x lander width on each side
    let tolerance_points = ((lander_width_terrain_points as f32 * 0.15) as usize).max(1);
    
    let tolerance_start = flat_start.saturating_sub(tolerance_points);
    let tolerance_end = flat_end + tolerance_points;
    
    for &idx in terrain_indices {
        if idx >= tolerance_start && idx <= tolerance_end {
            return true; // At least part of the lander is on or near the flat spot
        }
    }

    false
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

    // Determine collision type based on flat terrain, velocity, and collision zones
    // CRITICAL: Only flat spots are safe landing zones!
    if leg_collision {
        // Check if landing on a flat spot (mandatory for success)
        // We know exactly where the flat spot is now, so check against the known range
        let flat_spot_range = entity.flat_spots[0]; // We have exactly one flat spot
        
        // Calculate lander width in terrain points for tolerance
        let screen_width = macroquad::window::screen_width();
        let terrain_points_per_pixel = 1000.0 / (screen_width * 2.0);
        let lander_width_terrain_points = (entity.transform.size.x * terrain_points_per_pixel) as usize;
        
        let on_flat_spot = is_on_flat_spot(&collision_terrain_indices, flat_spot_range, lander_width_terrain_points);

        if !on_flat_spot {
            info!("ROUGH TERRAIN LANDING: Not on flat spot - Mission Failed!");
            return CollisionType::LegCollision;
        }

        // On flat spot - now check velocity for success vs crash
        if let Some(physics) = &entity.physics {
            let landing_velocity = physics.velocity.length();
            if landing_velocity <= MAX_LANDING_VELOCITY {
                info!(
                    "SUCCESSFUL LANDING: velocity={:.1} on flat spot",
                    landing_velocity
                );
                CollisionType::LandingSuccess
            } else {
                info!(
                    "HARD LANDING: velocity={:.1} > {:.1} on flat spot",
                    landing_velocity, MAX_LANDING_VELOCITY
                );
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
        
        // Test lander partially on flat spot (within tolerance)
        let terrain_indices = vec![98, 102]; // Just outside but within tolerance
        assert!(is_on_flat_spot(&terrain_indices, flat_spot_range, lander_width_terrain_points));
        
        // Test lander completely off flat spot
        let terrain_indices = vec![50, 55, 60];
        assert!(!is_on_flat_spot(&terrain_indices, flat_spot_range, lander_width_terrain_points));
        
        // Test empty terrain indices
        let terrain_indices = vec![];
        assert!(!is_on_flat_spot(&terrain_indices, flat_spot_range, lander_width_terrain_points));
    }
}