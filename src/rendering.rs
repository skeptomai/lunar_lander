//! Rendering system for the lunar lander game.
//!
//! This module handles all visual aspects of the game including:
//! - Lander sprite rendering with thrust-based texture selection
//! - Procedural terrain rendering with color-coded landing zones
//! - UI elements (fuel, velocity, mission timer, zone information)
//! - Debug visualization (collision boxes, coordinate markers)
//! - Camera system with proper coordinate transformations

use macroquad::prelude::*;
use macroquad_text::Fonts;

use crate::entity::Entity;
use crate::physics::Physics;
use crate::session::{GameSession, AttemptResult};
use crate::surface::LandingZoneDifficulty;

/// Main rendering function that draws all game entities and UI elements.
///
/// This function orchestrates the complete rendering pipeline:
/// - Entity rendering (lander, terrain, debug info)
/// - UI rendering (HUD, alerts, debug overlays)
/// - Camera management for proper coordinate transformations
///
/// # Arguments
///
/// * `entities` - Vector of all game entities to render
/// * `camera` - Camera configuration for coordinate transformations
/// * `session` - Game session state for status display
pub fn render(entities: &Vec<Entity>, camera: &Camera2D, session: &GameSession) {
    for entity in entities {
        if let Some(phys) = &entity.physics {
            render_debug_info(entity, phys, camera);

            render_lander(entity, camera);

            render_terrain(entity, camera);

            if entity.show_debug_info {
                debug_render(entity);
            }

            if entity.dead {
                set_default_camera();
                draw_alert_box(entity, session);
            } else {
                draw_text(&entity);
            }
            
            // Always render session status
            render_session_status(entity, session);
        }
    }
}

/// Renders debug information and collision visualization.
///
/// When debug mode is enabled, this function displays:
/// - Physics data (position, velocity, forces)
/// - Rocket engine status (fuel, mass, thrust)
/// - Collision bounding boxes and detection zones
///
/// # Arguments
///
/// * `entity` - The entity to render debug info for
/// * `phys` - Physics component containing motion data
/// * `camera` - Camera for coordinate transformations
pub fn render_debug_info(entity: &Entity, phys: &Physics, camera: &Camera2D) {
    if entity.show_debug_info {
        debug!("position: {:?}", entity.transform.position);
        debug!("velocity: {:?}", phys.velocity);
        debug!("forces: {:?}", phys.forces);
        if let Some(rocket) = &entity.rocket_physics {
            debug!("fuel_mass: {:.1} kg", rocket.fuel_mass);
            debug!("total_mass: {:.1} kg", rocket.total_mass());
            debug!("thrust_vector: {:?} N", rocket.thrust_vector);
            debug!("is_thrusting: {}", rocket.is_thrusting);
            debug!("fuel_percentage: {:.1}%", rocket.fuel_percentage());
        }
        draw_collision_bounding_box(entity, camera);
    }
}

/// Renders the lunar lander and thrust flames as separate components.
///
/// The rendering consists of:
/// - Lander texture: Always rendered at the entity position
/// - Thrust texture: Rendered beneath lander when thrusting, scaled and rotated with lander
///
/// # Arguments
///
/// * `entity` - The lander entity to render
/// * `camera` - Camera for coordinate transformations
pub fn render_lander(entity: &Entity, camera: &Camera2D) {
    set_camera(camera);
    
    // Thrust flame scale factor relative to lander (easy to adjust)
    const THRUST_SCALE_FACTOR: f32 = 0.75;
    
    // Always render the main lander texture
    if let Some(lander_renderer) = &entity.renderer_lander {
        draw_texture_ex(
            &lander_renderer.lander_texture,
            entity.transform.position.x,
            entity.transform.position.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(entity.transform.size),
                rotation: entity.transform.rotation.to_radians(),
                flip_x: false,
                flip_y: true,
                ..Default::default()
            },
        );
    }
    
    // Render thrust flames when thrusting
    let should_render_thrust = if let Some(rocket) = &entity.rocket_physics {
        rocket.is_thrusting && rocket.has_fuel()
    } else if let Some(phys) = &entity.physics {
        // Fallback to force-based detection
        phys.forces.length() > 0.0
    } else {
        false
    };
    
    if should_render_thrust {
        if let Some(thrust_renderer) = &entity.renderer_thrust {
            // Calculate thrust flame size
            let thrust_size = entity.transform.size * THRUST_SCALE_FACTOR;
            
            // Calculate position beneath lander (touching lander bottom)
            let angle = entity.transform.rotation.to_radians();
            let lander_center_x = entity.transform.position.x + entity.transform.size.x * 0.5;
            let lander_center_y = entity.transform.position.y + entity.transform.size.y * 0.5;
            
            // Offset distance: half lander height + half thrust height (so they touch)
            let offset_distance = (entity.transform.size.y + thrust_size.y) * 0.5;
            
            // Calculate thrust position (beneath lander in the direction opposite to lander's "up")
            // Since 0° is "up", thrust should be in the opposite direction (180° rotated)
            let thrust_center_x = lander_center_x + (angle.sin() * offset_distance);
            let thrust_center_y = lander_center_y - (angle.cos() * offset_distance);
            
            // Position thrust texture (top-left corner for rendering)
            let thrust_x = thrust_center_x - thrust_size.x * 0.5;
            let thrust_y = thrust_center_y - thrust_size.y * 0.5;
            
            draw_texture_ex(
                &thrust_renderer.lander_texture,
                thrust_x,
                thrust_y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(thrust_size),
                    rotation: entity.transform.rotation.to_radians(),
                    flip_x: false,
                    flip_y: true,
                    ..Default::default()
                },
            );
        }
    }
}

/// Renders the procedurally generated terrain with color-coded landing zones and scores.
///
/// Terrain is rendered as connected line segments with different colors:
/// - Green: Normal rough terrain
/// - Red: Hard landing zones (1.0x lander width) - Score: 2.0
/// - Orange: Medium landing zones (1.25x lander width) - Score: 1.6
/// - Yellow: Easy landing zones (1.5x lander width) - Score: 1.3
///
/// Scores are displayed above each landing zone.
///
/// # Arguments
///
/// * `entity` - Entity containing terrain data and landing zones
/// * `_camera` - Camera (unused, terrain uses screen coordinates)
pub fn render_terrain(entity: &Entity, _camera: &Camera2D) {
    // Draw terrain with 1:1 pixel correspondence - much simpler coordinate system
    for i in 0..entity.terrain.len() - 1 {
        let start_x = i as f32;
        let start_y = entity.terrain[i] as f32;
        let end_x = (i + 1) as f32;
        let end_y = entity.terrain[i + 1] as f32;

        // Check if BOTH endpoints of this segment are within any landing zone
        let mut in_landing_zone = None;
        for zone in &entity.landing_zones {
            if i >= zone.start && (i + 1) <= zone.end {
                in_landing_zone = Some(zone.difficulty);
                break;
            }
        }

        // Determine color and width based on landing zone difficulty
        let (line_color, line_width) = if let Some(difficulty) = in_landing_zone {
            let color = match difficulty {
                LandingZoneDifficulty::Hard => RED,    // Hardest zones are red (1.0x width)
                LandingZoneDifficulty::Medium => ORANGE, // Medium zones are orange (1.25x width)
                LandingZoneDifficulty::Easy => YELLOW,   // Easiest zones are yellow (1.5x width)
            };
            (color, 4.0) // Thicker lines for all landing zones
        } else {
            (GREEN, 2.0) // Regular terrain in green with normal width
        };

        draw_line(start_x, start_y, end_x, end_y, line_width, line_color);
    }

    // Render scores above landing zones using the same coordinate system as terrain
    // (Don't change camera - keep using the same coordinates as terrain rendering)

    for zone in &entity.landing_zones {
        // Calculate the center position of the zone
        let zone_center_x = (zone.start + zone.end) as f32 / 2.0;

        // Find the terrain height at the center of the zone
        let center_index = (zone_center_x as usize).min(entity.terrain.len() - 1);
        let terrain_height = entity.terrain[center_index] as f32;

        // Position score text above the zone (offset upward from terrain)
        let score_y = terrain_height - 25.0; // 25 pixels above terrain

        // Calculate and format the score
        let score = zone.difficulty.score();
        let score_text = format!("{:.1}", score);

        // Choose color to match zone difficulty
        let text_color = match zone.difficulty {
            LandingZoneDifficulty::Hard => RED,
            LandingZoneDifficulty::Medium => ORANGE,
            LandingZoneDifficulty::Easy => YELLOW,
        };

        // Draw score text centered above the zone
        let text_size = 18.0;
        let text_width = measure_text(&score_text, None, text_size as u16, 1.0).width;
        let text_x = zone_center_x - text_width / 2.0;

        // Convert to screen coordinates for proper text rendering
        // Score_y is in terrain coordinates (Y increases downward)
        // Convert to screen coordinates where Y=0 is at the top
        let screen_y = screen_height() - score_y;

        // Use default camera for text rendering to avoid coordinate system issues
        set_default_camera();
        macroquad::text::draw_text(&score_text, text_x, screen_y, text_size, text_color);
    }
}

/// Draws the main game UI including mission status, fuel, velocity, and landing zone info.
///
/// The UI displays:
/// - Mission timer and status
/// - Fuel percentage and spacecraft mass
/// - Velocity components and total speed
/// - Landing zone count and difficulty breakdown
/// - Thrust status indicator
///
/// # Arguments
///
/// * `entity` - Entity containing all game state and UI data
pub fn draw_text(entity: &Entity) {
    set_default_camera();
    let fonts = &entity.screen_fonts;
    let phys = entity.physics.as_ref().unwrap();

    let time_elapsed_text = format!("TIME {:.1}", entity.time_elapsed);
    fonts.draw_text("MISSION", 20.0, 0.0, 15.0, Color::from([1.0; 4]));
    fonts.draw_text(&time_elapsed_text, 20.0, 20.0, 15.0, Color::from([1.0; 4]));

    // Display fuel information from rocket physics if available
    let fuel_text = if let Some(rocket) = &entity.rocket_physics {
        format!("FUEL: {:.1}%", rocket.fuel_percentage())
    } else {
        "FUEL: N/A".to_string()
    };
    fonts.draw_text(&fuel_text, 20.0, 40.0, 15.0, Color::from([1.0; 4]));

    // Add mass information for realism
    if let Some(rocket) = &entity.rocket_physics {
        let mass_text = format!("MASS: {:.0}kg", rocket.total_mass());
        fonts.draw_text(&mass_text, 20.0, 60.0, 15.0, Color::from([1.0; 4]));
    }
    
    // Display landing zones information
    if !entity.landing_zones.is_empty() {
        let zones_text = format!("ZONES: {}", entity.landing_zones.len());
        fonts.draw_text(&zones_text, 20.0, 80.0, 15.0, Color::from([1.0; 4]));
        
        // Show difficulty breakdown
        let mut hard_count = 0;
        let mut medium_count = 0;
        let mut easy_count = 0;
        for zone in &entity.landing_zones {
            match zone.difficulty {
                LandingZoneDifficulty::Hard => hard_count += 1,
                LandingZoneDifficulty::Medium => medium_count += 1,
                LandingZoneDifficulty::Easy => easy_count += 1,
            }
        }
        
        let mut y_offset = 100.0;
        if hard_count > 0 {
            let hard_text = format!("RED: {} hard", hard_count);
            fonts.draw_text(&hard_text, 20.0, y_offset, 12.0, RED);
            y_offset += 15.0;
        }
        if medium_count > 0 {
            let medium_text = format!("ORANGE: {} med", medium_count);
            fonts.draw_text(&medium_text, 20.0, y_offset, 12.0, ORANGE);
            y_offset += 15.0;
        }
        if easy_count > 0 {
            let easy_text = format!("YELLOW: {} easy", easy_count);
            fonts.draw_text(&easy_text, 20.0, y_offset, 12.0, YELLOW);
        }
    }

    let w = macroquad::window::screen_width();
    let right_text_start = w - 195.0;
    let altitude_text = format!("ALTITUDE: {:.1}", entity.transform.position.y);
    let horizontal_speed_text = format!("H-SPEED: {:.1} m/s", phys.velocity.x);
    let vertical_speed_text = format!("V-SPEED: {:.1} m/s", phys.velocity.y);
    fonts.draw_text(
        &altitude_text,
        right_text_start,
        0.0,
        15.0,
        Color::from([1.0; 4]),
    );
    fonts.draw_text(
        &horizontal_speed_text,
        right_text_start,
        20.0,
        15.0,
        Color::from([1.0; 4]),
    );
    fonts.draw_text(
        &vertical_speed_text,
        right_text_start,
        40.0,
        15.0,
        Color::from([1.0; 4]),
    );

    // Add delta-V and thrust information for advanced players
    if let Some(rocket) = &entity.rocket_physics {
        let total_velocity = phys.velocity.length();
        let velocity_text = format!("SPEED: {:.1} m/s", total_velocity);
        fonts.draw_text(
            &velocity_text,
            right_text_start,
            60.0,
            15.0,
            Color::from([1.0; 4]),
        );

        // Show thrust status
        if rocket.is_thrusting {
            let thrust_percent =
                (rocket.thrust_vector.length() / rocket.max_thrust as f32 * 100.0) as i32;
            let thrust_text = format!("THRUST: {}%", thrust_percent);
            fonts.draw_text(
                &thrust_text,
                right_text_start,
                80.0,
                15.0,
                Color::from([1.0, 1.0, 0.0, 1.0]),
            ); // Yellow for thrust
        } else {
            fonts.draw_text(
                "THRUST: 0%",
                right_text_start,
                80.0,
                15.0,
                Color::from([0.5, 0.5, 0.5, 1.0]),
            ); // Gray when off
        }
    }
}

/// Draws mission result alert box for success or failure scenarios.
///
/// The alert box appears when the mission ends, showing:
/// - Success: "Mission Success!" with green text
/// - Failure: "Mission Failed!" with red text
/// - Restart instructions
///
/// # Arguments
///
/// * `entity` - Entity containing mission status
/// * `session` - Game session for attempt context
pub fn draw_alert_box(entity: &Entity, session: &GameSession) {
    let fonts = &entity.screen_fonts;

    let screen_width = screen_width();
    let screen_height = screen_height();
    
    const ALERT_BOX_WIDTH: f32 = 320.0;
    const ALERT_BOX_HEIGHT: f32 = 130.0;
    
    let box_x = (screen_width - ALERT_BOX_WIDTH) / 2.0;
    let box_y = (screen_height - ALERT_BOX_HEIGHT) / 2.5;

    draw_rectangle(box_x, box_y, ALERT_BOX_WIDTH, ALERT_BOX_HEIGHT, LIGHTGRAY);

    // Main result text - centered
    let mut current_y = box_y + 25.0;
    
    if entity.mission_success {
        let success_text = "Attempt Success!";
        let success_text_width = measure_text(success_text, None, 28, 1.0).width;
        let success_text_x = box_x + (ALERT_BOX_WIDTH - success_text_width) / 2.0;
        fonts.draw_text(success_text, success_text_x, current_y, 28.0, GREEN);
        current_y += 30.0;
        
        if let Some(attempt) = session.attempts.get(session.current_attempt.saturating_sub(1)) {
            let score_text = format!("Score: {:.0}", attempt.score);
            let score_text_width = measure_text(&score_text, None, 18, 1.0).width;
            let score_text_x = box_x + (ALERT_BOX_WIDTH - score_text_width) / 2.0;
            fonts.draw_text(&score_text, score_text_x, current_y, 18.0, WHITE);
            current_y += 25.0;
        }
    } else {
        let failed_text = "Attempt Failed!";
        let failed_text_width = measure_text(failed_text, None, 28, 1.0).width;
        let failed_text_x = box_x + (ALERT_BOX_WIDTH - failed_text_width) / 2.0;
        fonts.draw_text(failed_text, failed_text_x, current_y, 28.0, RED);
        current_y += 35.0;
    }
    
    // Show session progress context - centered
    let progress_text = if session.session_complete {
        format!("Session Complete! Total: {}", session.total_score as i32)
    } else {
        format!("Attempt {} of {}", session.current_attempt, session.max_attempts)
    };
    let progress_text_width = measure_text(&progress_text, None, 16, 1.0).width;
    let progress_text_x = box_x + (ALERT_BOX_WIDTH - progress_text_width) / 2.0;
    fonts.draw_text(&progress_text, progress_text_x, current_y, 16.0, WHITE);
    current_y += 25.0;

    // Show appropriate restart message based on session state - centered
    let restart_text = if session.session_complete {
        // All 3 attempts completed - offer new session
        "Press R for New Session"
    } else {
        // Still have attempts remaining - offer next attempt
        "Press R for Next Attempt"
    };
    
    let restart_text_width = measure_text(restart_text, None, 16, 1.0).width;
    let restart_text_x = box_x + (ALERT_BOX_WIDTH - restart_text_width) / 2.0;
    fonts.draw_text(
        restart_text,
        restart_text_x,
        current_y,
        16.0,
        WHITE,
    );
}

/// Draws detailed collision detection visualization for debugging.
///
/// This function renders:
/// - Lander bounding box with corner markers
/// - Leg collision zones (bottom 25%, left/right 30%)
/// - Body collision zone (center 40%)
/// - Critical collision edges and margins
///
/// # Arguments
///
/// * `entity` - Entity to visualize collision detection for
/// * `camera` - Camera for coordinate transformations
pub fn draw_collision_bounding_box(entity: &Entity, camera: &Camera2D) -> () {
    // Draw in camera coordinates (same as where the lander is actually rendered)
    set_camera(camera);

    let lander_x = entity.transform.position.x;
    let lander_y = entity.transform.position.y;
    let lander_width = entity.transform.size.x;
    let lander_height = entity.transform.size.y;

    // Draw debug box at EXACTLY the same position as the rocket texture
    draw_rectangle_lines(lander_x, lander_y, lander_width, lander_height, 3.0, RED);

    // Mark the four corners - camera coordinates: Y increases UPWARD (-2.0/screen_height zoom)
    // lander_y is BOTTOM, lander_y + lander_height is TOP
    draw_circle(lander_x, lander_y, 4.0, BLUE); // Bottom-left of rocket
    draw_circle(lander_x + lander_width, lander_y, 4.0, GREEN); // Bottom-right of rocket
    draw_circle(lander_x, lander_y + lander_height, 4.0, YELLOW); // Top-left of rocket
    draw_circle(
        lander_x + lander_width,
        lander_y + lander_height,
        4.0,
        ORANGE,
    ); // Top-right of rocket

    // Collision zones - corrected for inverted Y coordinates
    const LEG_HEIGHT_RATIO: f32 = 0.25;
    const LEG_WIDTH_RATIO: f32 = 0.3;
    let leg_height = lander_height * LEG_HEIGHT_RATIO;
    let leg_width = lander_width * LEG_WIDTH_RATIO;

    // Bottom 25% of rocket for legs (lander_y is the bottom)
    let leg_zone_bottom = lander_y;
    // let leg_zone_top = lander_y + leg_height;

    // Left leg zone (green rectangles) - bottom 25% left edge
    draw_rectangle_lines(lander_x, leg_zone_bottom, leg_width, leg_height, 2.0, GREEN);

    // Right leg zone (green rectangles) - bottom 25% right edge
    draw_rectangle_lines(
        lander_x + lander_width - leg_width,
        leg_zone_bottom,
        leg_width,
        leg_height,
        2.0,
        GREEN,
    );

    // Body collision zone (red center area) - full height, center area
    let body_left = lander_x + leg_width;
    let body_width = lander_width - (2.0 * leg_width);
    draw_rectangle_lines(body_left, lander_y, body_width, lander_height, 2.0, RED);

    // Bottom edge line - this is where collision actually happens (at lander_y)
    draw_line(
        lander_x,
        lander_y,
        lander_x + lander_width,
        lander_y,
        6.0,
        YELLOW,
    );

    // Top edge line - for visual completeness
    draw_line(
        lander_x,
        lander_y + lander_height,
        lander_x + lander_width,
        lander_y + lander_height,
        6.0,
        MAGENTA,
    );

    // Now draw terrain collision points in camera coordinates
    set_camera(camera);

    // Use same terrain index calculation as collision detection
    let screen_width = macroquad::window::screen_width();
    let lander_left_x = lander_x;
    let lander_right_x = lander_x + lander_width;

    // Convert to terrain array indices (same as collision detection)
    let _terrain_start_idx = (((lander_left_x + screen_width) / (screen_width * 2.0) * 800.0) as i32).max(0) as usize;
    let _terrain_end_idx = (((lander_right_x + screen_width) / (screen_width * 2.0) * 800.0) as i32)
        .min(799) as usize;

    // Debug collision points removed for cleaner display

    set_default_camera()
}

/// Renders debug overlay markers for screen coordinate validation.
///
/// This function draws reference markers:
/// - Screen center crosshair
/// - Edge markers at maximum terrain height
/// - Coordinate system validation points
///
/// # Arguments
///
/// * `entity` - Entity containing terrain data for marker positioning
pub fn debug_render(entity: &Entity) {
    // Debug: Draw center reticle to show screen center
    let screen_center_x = screen_width() / 2.0;
    let screen_center_y = screen_height() / 2.0;
    // Horizontal line (50 pixels each direction)
    draw_line(
        screen_center_x - 50.0,
        screen_center_y,
        screen_center_x + 50.0,
        screen_center_y,
        2.0,
        RED,
    );
    // Vertical line (50 pixels each direction)
    draw_line(
        screen_center_x,
        screen_center_y - 50.0,
        screen_center_x,
        screen_center_y + 50.0,
        2.0,
        RED,
    );

    // Find maximum terrain height for blue marker positioning
    let max_terrain_height = entity.terrain.iter().cloned().fold(f64::NEG_INFINITY, f64::max) as f32;
    
    // Debug: Draw edge markers at maximum terrain height
    // Left edge marker (10 pixels from left edge)
    draw_line(
        0.0,
        max_terrain_height,
        10.0,
        max_terrain_height,
        2.0,
        BLUE,
    );
    // Right edge marker (10 pixels from right edge toward center)
    draw_line(
        screen_width() - 10.0,
        max_terrain_height,
        screen_width(),
        max_terrain_height,
        2.0,
        BLUE,
    );
    // Center marker (10 pixels straddling center X axis)
    draw_line(
        screen_center_x - 5.0,
        max_terrain_height,
        screen_center_x + 5.0,
        max_terrain_height,
        2.0,
        BLUE,
    );
}

/// Configures the 2D camera with proper coordinate system transformations.
///
/// The camera setup:
/// - Inverts Y-axis for standard mathematical coordinates
/// - Centers on screen with appropriate zoom levels
/// - Handles coordinate transformations between screen and world space
///
/// # Returns
///
/// Configured `Camera2D` instance ready for rendering
pub fn configure_camera() -> Camera2D {
    let screen_width = screen_width();
    let screen_height = screen_height();

    // Create a Camera2D with the standard x, y axes orientation
    Camera2D {
        zoom: vec2(2.0 / screen_width, -2.0 / screen_height), // Invert y-axis
        target: vec2(screen_width / 2.0, screen_height / 2.0),
        ..Default::default()
    }
}

/// Renders the game session status bar showing attempt indicators and scoring information.
///
/// The status bar displays:
/// - Visual indicators for each of the 3 attempts (gray = future, yellow = current, green = success, red = failure)
/// - Current attempt number and total attempts
/// - Total session score
/// - Performance summary when session is complete
///
/// # Arguments
///
/// * `entity` - Entity containing fonts for text rendering
/// * `session` - Game session state to display
pub fn render_session_status(entity: &Entity, session: &GameSession) {
    set_default_camera();
    let fonts = &entity.screen_fonts;
    
    // Session status bar position - centered horizontally
    let screen_width = screen_width();
    let icon_size = 16.0 * 0.7; // Reduce size by 30%
    let icon_spacing = 40.0;
    let total_width = (session.max_attempts as f32 - 1.0) * icon_spacing;
    let start_x = (screen_width - total_width) / 2.0;
    let start_y = 20.0; // Move up to avoid overlap with mission stats
    
    // Draw attempt indicators using circles with lander symbols
    for (i, attempt) in session.attempts.iter().enumerate() {
        let x = start_x + (i as f32 * icon_spacing);
        let y = start_y;
        
        // Determine color and appearance based on attempt status
        let (circle_color, text_color, is_current) = match attempt.result {
            AttemptResult::Success => (GREEN, GREEN, false),
            AttemptResult::Failure => (RED, RED, false),
            AttemptResult::InProgress => {
                if i == session.current_attempt {
                    (YELLOW, YELLOW, true) // Current attempt
                } else {
                    (GRAY, GRAY, false)   // Future attempt
                }
            }
        };
        
        // Draw background circle
        draw_circle(x, y, icon_size, circle_color);
        
        // No text needed - the colored circles are clear enough indicators
        
        // Highlight current attempt with a ring
        if is_current {
            draw_circle_lines(x, y, icon_size + 2.0, 2.0, WHITE);
        }
        
        // Show score below successful attempts - positioned better to avoid overlap
        if attempt.result == AttemptResult::Success && attempt.score > 0.0 {
            let score_text = format!("{:.0}", attempt.score);
            // Center the score text under the icon
            let score_text_width = measure_text(&score_text, None, 12, 1.0).width;
            let score_text_x = x - score_text_width / 2.0;
            fonts.draw_text(&score_text, score_text_x, y + 35.0, 12.0, WHITE);
        }
    }
    
    // Draw session information - centered below the icons with more space
    let info_y = start_y + 60.0; // Position below the icons and scores
    
    // Current attempt indicator - centered
    let attempt_text = format!("ATTEMPT: {}/{}", 
        (session.current_attempt + 1).min(session.max_attempts), 
        session.max_attempts);
    let attempt_text_width = measure_text(&attempt_text, None, 16, 1.0).width;
    let attempt_text_x = (screen_width - attempt_text_width) / 2.0;
    fonts.draw_text(&attempt_text, attempt_text_x, info_y, 16.0, WHITE);
    
    // Total score - centered below attempt indicator
    let score_text = format!("TOTAL SCORE: {:.0}", session.total_score);
    let score_text_width = measure_text(&score_text, None, 16, 1.0).width;
    let score_text_x = (screen_width - score_text_width) / 2.0;
    fonts.draw_text(&score_text, score_text_x, info_y + 20.0, 16.0, WHITE);
    
    // Session statistics - centered
    if session.session_complete {
        // Performance rating - centered
        let rating_text = format!("RATING: {}", session.performance_rating());
        let rating_text_width = measure_text(&rating_text, None, 14, 1.0).width;
        let rating_text_x = (screen_width - rating_text_width) / 2.0;
        fonts.draw_text(&rating_text, rating_text_x, info_y + 40.0, 14.0, GOLD);
        
        // Success/failure count - centered
        let results_text = format!("SUCCESSES: {} / FAILURES: {}", 
            session.success_count(), 
            session.failure_count());
        let results_text_width = measure_text(&results_text, None, 12, 1.0).width;
        let results_text_x = (screen_width - results_text_width) / 2.0;
        fonts.draw_text(&results_text, results_text_x, info_y + 60.0, 12.0, WHITE);
        
        // Average fuel efficiency for successful attempts - centered
        if session.success_count() > 0 {
            let fuel_text = format!("AVG FUEL: {:.1}%", session.average_fuel_efficiency());
            let fuel_text_width = measure_text(&fuel_text, None, 12, 1.0).width;
            let fuel_text_x = (screen_width - fuel_text_width) / 2.0;
            fonts.draw_text(&fuel_text, fuel_text_x, info_y + 80.0, 12.0, WHITE);
        }
    } else {
        // Show current attempt fuel during active play - centered
        if !entity.dead {
            if let Some(rocket) = &entity.rocket_physics {
                let fuel_text = format!("FUEL: {:.1}%", rocket.fuel_percentage());
                let fuel_text_width = measure_text(&fuel_text, None, 14, 1.0).width;
                let fuel_text_x = (screen_width - fuel_text_width) / 2.0;
                fonts.draw_text(&fuel_text, fuel_text_x, info_y + 40.0, 14.0, 
                    if rocket.fuel_percentage() < 25.0 { RED } else { WHITE });
            }
        }
    }
}
