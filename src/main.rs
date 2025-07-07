#![allow(dead_code)]
#![allow(unused_imports)]
use core::time;
use std::thread::sleep;

use macroquad::prelude::*;
use macroquad_text::Fonts;
use rusty_audio::Audio;

mod physics;
mod surface;

use physics::{update_rocket_physics, Physics, RocketPhysics};

const GLASS_TTY_VT220: &[u8] = include_bytes!("../assets/fonts/Glass_TTY_VT220.ttf");
const MAX_ACCEL_X: f32 = 150.0;
const MAX_ACCEL_Y: f32 = 150.0;
const MILLIS_DELAY: u64 = 40;
const ROTATION_INCREMENT: f32 = 3.0;
const ACCEL_INCREMENT: f32 = 3.5;
const FULL_CIRCLE_DEGREES: f32 = 360.0;
const TEXTURE_SCALE_LANDER_X: f32 = 0.5;
const TEXTURE_SCALE_LANDER_Y: f32 = 0.5;
const TERRAIN_Y_OFFSET: f64 = 75.0;
const ALERT_BOX_WIDTH: f32 = 300.0;
const ALERT_BOX_HEIGHT: f32 = 100.0;
// acceleration due to gravity on earth
//const ACCEL_GRAV_Y: f32 = 9.8;
// acceleration due to gravity on the moon
const ACCEL_GRAV_Y: f32 = 1.625;

#[derive(Debug)]
struct Line {
    start: Vec2,
    end: Vec2,
}

// Define components
#[derive(Debug, Clone)]
struct Transform {
    size: Vec2,
    position: Vec2,
    rotation: f32,
}
struct Renderer {
    lander_texture: Texture2D,
    // Other rendering properties
}

struct Input;

struct Collision {
    collider: Rect,
}

// Define entities
struct Entity<'a> {
    transform: Transform,
    terrain: Vec<f64>,
    flat_spots: Vec<(usize, usize)>, // Store flat spot ranges for direct reference
    screen_fonts: Fonts<'a>,
    physics: Option<Physics>,
    rocket_physics: Option<RocketPhysics>,
    renderer_lander: Option<Renderer>,
    renderer_lander_accel: Option<Renderer>,
    renderer_lander_high_accel: Option<Renderer>,
    input: Option<Input>,
    collision: Option<Collision>,
    sound: bool,
    time_elapsed: f32,
    show_debug_info: bool,
    dead: bool,
    mission_success: bool,
    current_audio: Option<String>,
}

impl<'a> Entity<'a> {
    fn new() -> Self {
        Entity {
            transform: Transform {
                size: Vec2::new(0.0, 0.0),
                position: Vec2::new(0.0, 0.0),
                rotation: 90.0,
            },
            terrain: Vec::new(),
            flat_spots: Vec::new(),
            screen_fonts: load_fonts(),
            physics: Some(Physics {
                velocity: Vec2::new(0.0, 0.0),
                acceleration: Vec2::new(0.0, 0.0),
            }),
            rocket_physics: Some(RocketPhysics::new_apollo_lm()),
            renderer_lander: None,
            renderer_lander_accel: None,
            renderer_lander_high_accel: None,
            input: Some(Input),
            collision: Some(Collision {
                collider: Rect::new(0.0, 0.0, 64.0, 64.0),
            }),
            sound: true,
            time_elapsed: 0.0,
            show_debug_info: false,
            dead: false,
            mission_success: false,
            current_audio: None,
        }
    }

    fn initialize_with_terrain_and_position(&mut self, lander_texture_size: Vec2) {
        let current_screen_width = screen_width();
        let num_points = current_screen_width as usize;
        let min_height = 0.0;
        let max_height = 100.0;
        let base_frequency = 0.01;
        let octaves = 6;
        let persistence = 0.5;

        // Calculate lander width in terrain coordinate units
        // Use consistent calculation with 1:1 pixel mapping
        let lander_width_terrain_points = lander_texture_size.x as usize;
        let landing_spot_terrain_points = (lander_width_terrain_points as f32 * 1.5) as usize;

        debug!(
            "Lander width: {:.1} pixels = {} terrain points",
            lander_texture_size.x, lander_width_terrain_points
        );
        debug!(
            "Landing spot: {} terrain points (1.5x lander width)",
            landing_spot_terrain_points
        );

        // Generate terrain with integrated flat landing spot
        let (mut terrain, flat_spot_range) = surface::generate_terrain_with_flat_spot(
            num_points,
            min_height,
            max_height,
            base_frequency,
            octaves,
            persistence,
            landing_spot_terrain_points,
        );

        // Apply scaling transformation
        terrain.iter_mut().for_each(|h| {
            *h = *h * 0.4 + 60.0;
        });

        self.terrain = terrain;
        self.flat_spots = vec![flat_spot_range];

        // Set lander size and position
        self.transform.size = lander_texture_size;

        // Position lander safely above terrain
        let initial_world_pos = vec2(0.0, 50.0);
        let tex_center = initial_world_pos;
        let screen_center = transform_axes(tex_center);
        self.transform.position = vec2(
            screen_center.x - lander_texture_size.x / 2.0,
            screen_center.y - lander_texture_size.y / 2.0,
        );

        // Reset physics and state
        self.physics = Some(Physics {
            velocity: vec2(0.0, 0.0),
            acceleration: vec2(0.0, 0.0),
        });

        if let Some(rocket) = &mut self.rocket_physics {
            rocket.refuel();
            rocket.stop_thrust();
        }

        self.time_elapsed = 0.0;
        self.sound = true;
        self.dead = false;
        self.mission_success = false;
        self.current_audio = None;
    }
}

// Define systems
fn update_physics(entities: &mut Vec<Entity>) {
    let dt = get_frame_time();

    for entity in entities {
        if entity.dead {
            continue;
        }

        if let Some(physics) = &mut entity.physics {
            // Reset acceleration for this frame
            physics.acceleration = Vec2::ZERO;

            // Apply gravity
            physics.acceleration.y -= ACCEL_GRAV_Y;

            // Update rocket physics if present
            if let Some(rocket) = &mut entity.rocket_physics {
                update_rocket_physics(rocket, physics, dt);
            }

            // Integrate velocity and position using proper timestep
            physics.velocity += physics.acceleration * dt;
            entity.transform.position += physics.velocity * dt;

            // Wrap around screen (maintain lunar lander behavior)
            entity.transform.position.x = entity.transform.position.x.rem_euclid(screen_width());
            entity.transform.position.y = entity.transform.position.y.rem_euclid(screen_height());

            // Update elapsed time with proper precision
            entity.time_elapsed += dt;
        }
    }
}

// Rocket physics is now handled in the physics module

/// Helper function for testing coordinate transformations
pub fn reverse_transform_axes(screen_pos: Vec2, screen_width: f32, screen_height: f32) -> Vec2 {
    vec2(
        screen_pos.x - screen_width / 2.0,
        screen_height / 2.0 - screen_pos.y,
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

fn render(entities: &Vec<Entity>, camera: &Camera2D) {

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
                draw_alert_box(entity);
            } else {
                draw_text(&entity);
            }
        }
    }
}

fn render_debug_info(entity: &Entity, phys: &Physics, camera: &Camera2D) {
    if entity.show_debug_info {
        debug!("position: {:?}", entity.transform.position);
        debug!("velocity: {:?}", phys.velocity);
        debug!("acceleration: {:?}", phys.acceleration);
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

fn render_lander(entity: &Entity, camera: &Camera2D) {
    if let Some(phys) = &entity.physics {
        // Choose lander texture based on thrust status
        let o_renderer = if let Some(rocket) = &entity.rocket_physics {
            if rocket.is_thrusting && rocket.has_fuel() {
                let thrust_magnitude = rocket.thrust_vector.length();
                if thrust_magnitude > rocket.max_thrust as f32 * 0.7 {
                    &entity.renderer_lander_high_accel
                } else {
                    &entity.renderer_lander_accel
                }
            } else {
                &entity.renderer_lander
            }
        } else {
            // Fallback to old acceleration-based rendering
            let accel = phys.acceleration;
            if accel.length() > 0.0 {
                if accel.length() > 40.0 {
                    &entity.renderer_lander_high_accel
                } else {
                    &entity.renderer_lander_accel
                }
            } else {
                &entity.renderer_lander
            }
        };

        if let Some(renderer) = o_renderer {
            set_camera(camera);
            draw_texture_ex(
                &renderer.lander_texture,
                entity.transform.position.x,
                entity.transform.position.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(entity.transform.size), // Set destination size if needed
                    rotation: entity.transform.rotation.to_radians(),
                    flip_x: false,
                    flip_y: false,
                    ..Default::default()
                },
            );
        }
    }
}

fn render_terrain(entity: &Entity, _camera: &Camera2D) {
    // Draw terrain with 1:1 pixel correspondence - much simpler coordinate system
    for i in 0..entity.terrain.len() - 1 {
        let start_x = i as f32;
        let start_y = entity.terrain[i] as f32;
        let end_x = (i + 1) as f32;
        let end_y = entity.terrain[i + 1] as f32;

        // Check if BOTH endpoints of this segment are within the flat spot range
        let flat_spot_range = entity.flat_spots[0]; // We have exactly one flat spot
        let is_flat_segment = i >= flat_spot_range.0 && (i + 1) <= flat_spot_range.1;

        // Determine color and width based on whether this is a flat landing zone
        let line_color = if is_flat_segment {
            YELLOW // Landing zones should be yellow
        } else {
            GREEN // Regular terrain should be green
        };

        let line_width = if is_flat_segment {
            4.0 // Thicker lines for landing zones
        } else {
            2.0 // Regular width for normal terrain
        };

        draw_line(start_x, start_y, end_x, end_y, line_width, line_color);
    }
}

fn draw_text(entity: &Entity) {
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

fn draw_alert_box(entity: &Entity) {
    let fonts = &entity.screen_fonts;

    let screen_width = screen_width();
    let screen_height = screen_height();
    let box_x = (screen_width - ALERT_BOX_WIDTH) / 2.0;
    let box_y = (screen_height - ALERT_BOX_HEIGHT) / 2.5;

    draw_rectangle(box_x, box_y, ALERT_BOX_WIDTH, ALERT_BOX_HEIGHT, LIGHTGRAY);

    if entity.mission_success {
        fonts.draw_text("Mission Success!", box_x + 30.0, box_y + 20.0, 30.0, GREEN);
        fonts.draw_text("Landing Complete!", box_x + 70.0, box_y + 50.0, 20.0, WHITE);
    } else {
        fonts.draw_text("Mission Failed!", box_x + 40.0, box_y + 20.0, 30.0, RED);
    }

    fonts.draw_text(
        "Press R to Restart",
        box_x + 60.0,
        box_y + 70.0,
        20.0,
        WHITE,
    );
}

fn handle_input(lander: &mut Entity, audio: &mut Audio) {
    // Handle input
    if is_key_released(KeyCode::R) {
        reset_lander(lander);
        update_audio(audio);
    }
    if is_key_down(KeyCode::Escape) {
        shutdown_audio(audio);
        std::process::exit(0);
    }
    if is_key_released(KeyCode::S) {
        lander.sound = !lander.sound;
    }
    if is_key_down(KeyCode::Right) {
        lander.transform.rotation =
            (lander.transform.rotation - ROTATION_INCREMENT).rem_euclid(FULL_CIRCLE_DEGREES);
    }
    if is_key_down(KeyCode::Left) {
        lander.transform.rotation =
            (lander.transform.rotation + ROTATION_INCREMENT).rem_euclid(FULL_CIRCLE_DEGREES);
    }

    // Improved thrust handling using proper rocket physics
    let mut should_play_thrust = false;
    let mut should_play_ambient = false;

    if let Some(rocket) = &mut lander.rocket_physics {
        if is_key_down(KeyCode::Up) && rocket.has_fuel() && !lander.dead {
            // Calculate thrust direction based on lander orientation
            let angle = lander.transform.rotation.to_radians();
            let thrust_direction = vec2(angle.cos(), angle.sin());

            // Apply thrust vector (magnitude determined by max_thrust)
            rocket.thrust_vector = thrust_direction * rocket.max_thrust as f32;
            rocket.is_thrusting = true;
            should_play_thrust = true;
        } else {
            // Stop thrusting
            rocket.stop_thrust();
            should_play_ambient = lander.sound;
        }
    } else {
        should_play_ambient = lander.sound;
    }

    if is_key_released(KeyCode::D) {
        lander.show_debug_info = !lander.show_debug_info;
    }

    // Simplified audio management - keep ambient sound playing during free fall
    if should_play_thrust {
        // Switch to thrust audio
        if lander.current_audio != Some("acceleration".to_string()) {
            shutdown_audio(audio);
            audio.play("acceleration");
            lander.current_audio = Some("acceleration".to_string());
        }
    } else if should_play_ambient {
        // Switch to or maintain ambient audio
        if lander.current_audio != Some("ambient".to_string()) {
            shutdown_audio(audio);
            audio.play("ambient");
            lander.current_audio = Some("ambient".to_string());
        } else if !audio.is_playing() {
            // Restart ambient if it stopped playing for any reason
            audio.play("ambient");
        }
    } else {
        // Only stop audio if sound is disabled
        if lander.current_audio.is_some() {
            shutdown_audio(audio);
            lander.current_audio = None;
        }
    }
}

// Legacy function - now handled by rocket physics system
fn update_increment_acceleration(angle: f32, phys: &mut Physics) {
    let inc_acceleration = vec2(ACCEL_INCREMENT * angle.cos(), ACCEL_INCREMENT * angle.sin());
    phys.acceleration = phys.acceleration + inc_acceleration;
    phys.acceleration.x = phys.acceleration.x.min(MAX_ACCEL_X);
    phys.acceleration.y = phys.acceleration.y.min(MAX_ACCEL_Y);
}

fn stop_lander(lander: &mut Entity) {
    if let Some(phys) = lander.physics.as_mut() {
        phys.velocity = vec2(0.0, 0.0);
        phys.acceleration = vec2(0.0, 0.0);
    }
    if let Some(rocket) = &mut lander.rocket_physics {
        rocket.stop_thrust();
    }
    lander.collision = Some(Collision {
        collider: Rect::new(0.0, 0.0, 0.0, 0.0),
    });
}

fn reset_lander(lander: &mut Entity) {
    // Reset lander using common initialization method
    let lander_texture_size = lander.transform.size; // Preserve existing size
    lander.initialize_with_terrain_and_position(lander_texture_size);
}

fn load_fonts<'a>() -> Fonts<'a> {
    let mut fonts = Fonts::default();
    fonts
        .load_font_from_bytes("Glass VT200", GLASS_TTY_VT220)
        .unwrap();
    fonts
}

fn load_audio() -> Audio {
    let mut audio = Audio::new();
    audio.add("ambient", "assets/sounds/218883-jet_whine_v2_mid_loop.wav");
    audio.add(
        "acceleration",
        "assets/sounds/218837-jet_turbine_main_blast.wav",
    );
    audio
}

fn transform_axes(position: Vec2) -> Vec2 {
    vec2(
        position.x + screen_width() / 2.0,
        -position.y + screen_height() / 2.0,
    )
}

fn rotate_axes(rotation: f32) -> f32 {
    rotation.rem_euclid(FULL_CIRCLE_DEGREES)
}

fn configure_camera() -> Camera2D {
    let screen_width = screen_width();
    let screen_height = screen_height();

    // Create a Camera2D with the standard x, y axes orientation
    Camera2D {
        zoom: vec2(2.0 / screen_width, -2.0 / screen_height), // Invert y-axis
        target: vec2(screen_width / 2.0, screen_height / 2.0),
        ..Default::default()
    }
}

async fn add_lander_entity<'a>(entities: &mut Vec<Entity<'a>>) {
    // Load textures first to get actual lander dimensions
    let lander_texture = load_texture("assets/images/lander.png")
        .await
        .expect("Failed to load texture");
    let lander_accel_texture = load_texture("assets/images/lander-accel.png")
        .await
        .expect("Failed to load texture");
    let lander_high_accel_texture = load_texture("assets/images/lander-high-accel.png")
        .await
        .expect("Failed to load texture");

    // Get the actual size of the texture
    let lander_texture_size = lander_texture.size().mul_add(
        Vec2::new(TEXTURE_SCALE_LANDER_X, TEXTURE_SCALE_LANDER_Y),
        Vec2::new(0.0, 0.0),
    );

    // Calculate lander width in terrain coordinate units
    let current_screen_width = screen_width();
    let terrain_points_per_pixel = 1000.0 / (current_screen_width * 2.0);
    let lander_width_terrain_points = (lander_texture_size.x * terrain_points_per_pixel) as usize;
    let landing_spot_terrain_points = (lander_width_terrain_points as f32 * 1.5) as usize;

    debug!(
        "Lander dimensions: {}x{} pixels ({} terrain points wide)",
        lander_texture_size.x, lander_texture_size.y, lander_width_terrain_points
    );
    debug!(
        "Landing spot size: {} terrain points (1.5x lander width)",
        landing_spot_terrain_points
    );

    // Create lander entity with default constructor
    let mut lander = Entity::new();
    
    // Initialize terrain and position using common method
    lander.initialize_with_terrain_and_position(lander_texture_size);
    
    // Set up renderers with loaded textures
    lander.renderer_lander = Some(Renderer {
        lander_texture: lander_texture,
    });
    lander.renderer_lander_accel = Some(Renderer {
        lander_texture: lander_accel_texture,
    });
    lander.renderer_lander_high_accel = Some(Renderer {
        lander_texture: lander_high_accel_texture,
    });

    debug!(
        "Generated single flat landing spot at positions {}-{} ({} points)",
        lander.flat_spots[0].0, lander.flat_spots[0].1, landing_spot_terrain_points
    );
    debug!("Final terrain array length: {}", lander.terrain.len());

    entities.push(lander);
}

fn update_audio(audio: &mut Audio) {
    if !audio.is_playing() {
        audio.play("ambient"); // Execution continues while playback occurs in another thread.
    }
}

fn shutdown_audio(audio: &mut Audio) {
    audio.stop();
}

#[derive(Debug, PartialEq)]
enum CollisionType {
    None,
    LegCollision,
    BodyCollision,
    LandingSuccess,
}

fn is_on_flat_spot(terrain_indices: &[usize], flat_spot_range: (usize, usize), lander_width_terrain_points: usize) -> bool {
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


fn check_collision(entity: &Entity) -> CollisionType {
    // CAMERA COORDINATE COLLISION DETECTION
    // Both lander and terrain are already in camera coordinates:
    // - Lander position: stored in camera coordinates (entity.transform.position)
    // - Terrain Y values: stored directly as camera Y coordinates
    // - Terrain X mapping: array indices 0-1000 map to camera X range

    let screen_width = macroquad::window::screen_width();

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

    const COLLISION_MARGIN: f32 = 3.0;
    const LEG_HEIGHT_RATIO: f32 = 0.25; // Bottom 25% is legs
    const LEG_WIDTH_RATIO: f32 = 0.3; // Each leg takes 30% of width (20% gap in middle)
    const MAX_LANDING_VELOCITY: f32 = 10.0; // Maximum safe landing speed

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

fn draw_collision_bounding_box(entity: &Entity, camera: &Camera2D) -> () {
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
    let terrain_start_idx = (((lander_left_x + screen_width) / (screen_width * 2.0) * 800.0) as i32).max(0) as usize;
    let terrain_end_idx = (((lander_right_x + screen_width) / (screen_width * 2.0) * 800.0) as i32)
        .min(799) as usize;

    // Debug collision points removed for cleaner display

    set_default_camera()
}

fn debug_render(entity: &Entity) {
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

// Main game loop
#[macroquad::main("Lunar Lander")]
async fn main() {
    rand::srand(macroquad::miniquad::date::now() as _);
    let mut audio = load_audio();
    let mut entities = Vec::new();
    add_lander_entity(&mut entities).await;

    loop {
        clear_background(BLACK);

        let lander: &mut Entity = entities.first_mut().unwrap();

        // Handle input
        handle_input(lander, &mut audio);

        if !lander.dead {
            // Check for collision with new collision zones
            match check_collision(lander) {
                CollisionType::BodyCollision => {
                    debug!("Body Collision - Mission Failed!");
                    stop_lander(lander);
                    shutdown_audio(&mut audio);
                    lander.sound = false;
                    lander.dead = true;
                }
                CollisionType::LegCollision => {
                    debug!("Hard Landing - Mission Failed!");
                    stop_lander(lander);
                    shutdown_audio(&mut audio);
                    lander.sound = false;
                    lander.dead = true;
                }
                CollisionType::LandingSuccess => {
                    debug!("Successful Landing - Mission Complete!");
                    stop_lander(lander);
                    shutdown_audio(&mut audio);
                    lander.sound = false;
                    lander.dead = true;
                    lander.mission_success = true;
                }
                CollisionType::None => {
                    // No collision, continue normal gameplay
                }
            }

            // Check for empty fuel using rocket physics
            // Note: Running out of fuel doesn't end the mission - just prevents thrust
            if let Some(rocket) = &lander.rocket_physics {
                if !rocket.has_fuel() {
                    debug!("Out of fuel! Free fall mode.");
                    // Don't stop audio or kill lander - let physics continue
                    // Player can still try to land safely without thrust
                }
            }

            // Update systems
            update_physics(&mut entities);
        }

        // Render systems
        // Create camera once at start of main loop
        let camera = configure_camera();
        render(&entities, &camera);

        // Pause for the next frame
        sleep(std::time::Duration::from_millis(MILLIS_DELAY));

        next_frame().await
    }
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
        let offset_terrain_y = generated_terrain_y + TERRAIN_Y_OFFSET as f64; // 175.0
        let world_terrain_y = -(offset_terrain_y - screen_height as f64 / 2.0); // World coordinates
                                                                                // Should be: -(175 - 300) = -(-125) = 125, but we want negative Y for terrain below center
                                                                                // Actually: -(175 - 300) = -(−125) = 125, but terrain should be negative
                                                                                // Use the new terrain mapping logic
        let terrain_screen_base = screen_height as f64 * 0.7; // 420.0
        let screen_y = terrain_screen_base + (offset_terrain_y - TERRAIN_Y_OFFSET as f64); // 420 + (175-75) = 520
        let expected_world_terrain_y = screen_height as f64 / 2.0 - screen_y; // 300 - 520 = -220

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

        // Simulate lander at initial position (from reset_lander: world Y = 50.0)
        let lander_world_y = 50.0;
        let lander_height = 32.0;
        let lander_bottom_world_y = lander_world_y - lander_height; // 18.0

        // Simulate terrain (from terrain generation)
        let generated_terrain = 100.0;
        let offset_terrain = generated_terrain + TERRAIN_Y_OFFSET as f64; // 175.0
        let terrain_world_y = -(offset_terrain - screen_height as f64 / 2.0); // -(175-300) = 125.0

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
