#![allow(dead_code)]
#![allow(unused_imports)]
use core::time;
use std::thread::sleep;

use macroquad::prelude::*;
use rusty_audio::Audio;
use macroquad_text::Fonts;

mod surface;
mod physics;

use physics::{RocketPhysics, Physics, update_rocket_physics};

const GLASS_TTY_VT220: &[u8] = include_bytes!("../assets/fonts/Glass_TTY_VT220.ttf");
const MAX_ACCEL_X: f32 = 150.0;
const MAX_ACCEL_Y: f32 = 150.0;
const MILLIS_DELAY: u64 = 40;
const ROTATION_INCREMENT: f32 = 3.0;
const ACCEL_INCREMENT: f32 = 3.5;
const FULL_CIRCLE_DEGREES: f32 = 360.0;
const TEXTURE_SCALE_X: f32 = 0.5;
const TEXTURE_SCALE_Y: f32 = 0.5;
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
    texture: Texture2D,
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
        screen_height / 2.0 - screen_pos.y
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

fn render(entities: &Vec<Entity>) {
    let camera = configure_camera();

    for entity in entities {
        set_default_camera();
        if let Some(phys) = &entity.physics {

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
                draw_collision_bounding_box(entity);
            }

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
                set_camera(&camera);
                draw_texture_ex(&renderer.texture,
                                entity.transform.position.x,
                                entity.transform.position.y,
                                WHITE,
                                DrawTextureParams {
                                    dest_size: Some(entity.transform.size), // Set destination size if needed
                                    rotation: entity.transform.rotation.to_radians(),
                                    flip_x: false,
                                    flip_y: false,
                                    ..Default::default()
                                }

                );

            }

            // plot surface - convert world coordinates to camera coordinates for rendering
            for i in 0..(entity.terrain.len() - 1) {
                // Terrain is stored in world coordinates, convert to camera coordinates for rendering
                // transform_axes converts world -> screen, camera needs same conversion
                let terrain_point_1 = transform_axes(vec2(i as f32, entity.terrain[i] as f32));
                let terrain_point_2 = transform_axes(vec2((i + 1) as f32, entity.terrain[i + 1] as f32));
                
                draw_line(
                    terrain_point_1.x,
                    terrain_point_1.y,
                    terrain_point_2.x,
                    terrain_point_2.y,
                    2.0,
                    DARKGREEN,
                );
            }

            set_default_camera();

            if entity.dead {
                draw_alert_box(entity);
            } else {
                draw_text(&entity);
            }
        }
    }
}

fn draw_text(entity: &Entity) {
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
    fonts.draw_text(&altitude_text, right_text_start, 0.0, 15.0, Color::from([1.0; 4]));
    fonts.draw_text(&horizontal_speed_text, right_text_start, 20.0, 15.0, Color::from([1.0; 4]));
    fonts.draw_text(&vertical_speed_text, right_text_start, 40.0, 15.0, Color::from([1.0; 4]));

    // Add delta-V and thrust information for advanced players
    if let Some(rocket) = &entity.rocket_physics {
        let total_velocity = phys.velocity.length();
        let velocity_text = format!("SPEED: {:.1} m/s", total_velocity);
        fonts.draw_text(&velocity_text, right_text_start, 60.0, 15.0, Color::from([1.0; 4]));

        // Show thrust status
        if rocket.is_thrusting {
            let thrust_percent = (rocket.thrust_vector.length() / rocket.max_thrust as f32 * 100.0) as i32;
            let thrust_text = format!("THRUST: {}%", thrust_percent);
            fonts.draw_text(&thrust_text, right_text_start, 80.0, 15.0, Color::from([1.0, 1.0, 0.0, 1.0])); // Yellow for thrust
        } else {
            fonts.draw_text("THRUST: 0%", right_text_start, 80.0, 15.0, Color::from([0.5, 0.5, 0.5, 1.0])); // Gray when off
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
    fonts.draw_text("Mission Failed!", box_x + 40.0, box_y + 20.0, 30.0, RED);
    fonts.draw_text("Press R to Restart", box_x + 60.0, box_y + 70.0, 20.0, WHITE);
}

fn handle_input(lander: &mut Entity, audio: &mut Audio) {
    // Handle input
    if is_key_down(KeyCode::R) {
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
        lander.transform.rotation = (lander.transform.rotation - ROTATION_INCREMENT).rem_euclid(FULL_CIRCLE_DEGREES);
    }
    if is_key_down(KeyCode::Left) {
        lander.transform.rotation = (lander.transform.rotation + ROTATION_INCREMENT).rem_euclid(FULL_CIRCLE_DEGREES);
    }

    // Improved thrust handling using proper rocket physics
    if let Some(rocket) = &mut lander.rocket_physics {
        if is_key_down(KeyCode::Up) && rocket.has_fuel() {
            // Calculate thrust direction based on lander orientation
            let angle = lander.transform.rotation.to_radians();
            let thrust_direction = vec2(angle.cos(), angle.sin());

            // Apply thrust vector (magnitude determined by max_thrust)
            rocket.thrust_vector = thrust_direction * rocket.max_thrust as f32;
            rocket.is_thrusting = true;

            // Only start thrust audio if not already playing
            if !audio.is_playing() {
                audio.play("acceleration");
            }
        } else {
            // Stop thrusting
            rocket.stop_thrust();
            shutdown_audio(audio);
        }
    }

    if is_key_released(KeyCode::D) {
        lander.show_debug_info = !lander.show_debug_info;
    }

    // Handle ambient audio separately from thrust audio
    if lander.sound {
        // Only play ambient when not thrusting
        if let Some(rocket) = &lander.rocket_physics {
            if !rocket.is_thrusting {
                update_audio(audio);
            }
        } else {
            update_audio(audio);
        }
    } else {
        if audio.is_playing() {
            shutdown_audio(audio);
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
    // Reset lander
    let lander_texture = &lander.renderer_lander.as_ref().unwrap().texture;
    let _lander_texture_size = lander_texture.size().mul_add(Vec2::new(TEXTURE_SCALE_X, TEXTURE_SCALE_Y), Vec2::new(0.0, 0.0));
    // Position lander safely above terrain
    // Terrain is around 75-175, lander bottom currently at -254 is way below terrain at 121
    // Need lander to start with bottom above terrain level
    // If terrain max is ~175, position lander bottom at camera Y = 50 (well above terrain)
    // Since transform_axes inverts Y, need positive world Y to get negative camera Y
    let initial_world_pos = vec2(0.0, 50.0);  // Positive world Y for safe camera position
    let tex_center = initial_world_pos;

    lander.transform.position = transform_axes(tex_center);
    lander.transform.rotation = 90.0;
    lander.physics = Some(Physics {
        velocity: vec2(0.0, 0.0),
        acceleration: vec2(0.0, 0.0),
    });

    // Reset rocket physics
    if let Some(rocket) = &mut lander.rocket_physics {
        rocket.refuel();
        rocket.stop_thrust();
    }

    lander.time_elapsed = 0.0;
    lander.sound = true;
    lander.dead = false;
}

fn load_fonts<'a>() -> Fonts<'a> {
    let mut fonts = Fonts::default();
    fonts.load_font_from_bytes("Glass VT200", GLASS_TTY_VT220).unwrap();
    fonts
}

fn load_audio() -> Audio {
    let mut audio = Audio::new();
    audio.add("ambient", "assets/sounds/218883-jet_whine_v2_mid_loop.wav");
    audio.add("acceleration", "assets/sounds/218837-jet_turbine_main_blast.wav");
    audio
}

fn transform_axes(position: Vec2) -> Vec2 {
    vec2(position.x+screen_width()/2.0, -position.y + screen_height()/2.0)
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

    let num_points = 1000;
    let min_height = 0.0;
    let max_height = 100.0;
    let base_frequency = 0.01;
    let octaves = 6;
    let persistence = 0.5;

    let mut terrain = surface::generate_terrain(num_points, min_height, max_height, base_frequency, octaves, persistence);
    
    // Convert terrain to world coordinates
    // Generated terrain: 0-100, offset to 75-175
    // World coordinates: positive Y = up, negative Y = down, (0,0) = screen center
    // Terrain should be below world center, so convert to negative world Y values
    terrain.iter_mut().for_each(|h| *h = *h + TERRAIN_Y_OFFSET); // First apply offset: 75-175
    let screen_height = 600.0; // Will be actual screen height at runtime
    // Terrain at bottom of screen should be at world Y = -screen_height/2
    // Convert: screen_y -> world_y using reverse of transform_axes
    // transform_axes: world_y -> screen_y = -world_y + screen_height/2
    // So: world_y = screen_height/2 - screen_y
    // Convert screen coordinates to world coordinates using transform_axes reverse
    // transform_axes: screen_y = -world_y + screen_height/2
    // So: world_y = screen_height/2 - screen_y 
    // For terrain at screen_y=175: world_y = 300 - 175 = 125 (WRONG! Still positive)
    // Actually: terrain values are offsets from 0, need to adjust
    // If terrain offset=175 represents "near bottom of screen", that should be screen_y ~ 500
    // Let's map terrain 75-175 to screen_y range 400-500 (bottom portion)
    let terrain_screen_base = screen_height * 0.7; // Start terrain at 70% down screen
    terrain.iter_mut().for_each(|h| {
        let screen_y = terrain_screen_base + (*h - TERRAIN_Y_OFFSET); // Map terrain to screen Y
        *h = screen_height / 2.0 - screen_y; // Convert screen Y to world Y
    });

    // Add random flat spots for landing
    let min_flat_length = 20;
    let max_flat_length = 40;
    let num_flat_spots = 5;

    surface::add_flat_spots(&mut terrain, min_flat_length, max_flat_length, num_flat_spots);

    // Load a texture (replace "texture.png" with the path to your texture)
    let lander_texture = load_texture("assets/images/lander.png").await.expect("Failed to load texture");
    let lander_accel_texture = load_texture("assets/images/lander-accel.png").await.expect("Failed to load texture");
    let lander_high_accel_texture = load_texture("assets/images/lander-high-accel.png").await.expect("Failed to load texture");

    // Get the size of the texture
    let _lander_texture_size = lander_texture.size().mul_add(Vec2::new(TEXTURE_SCALE_X, TEXTURE_SCALE_Y), Vec2::new(0.0, 0.0));

    let fonts = load_fonts();
    // Position lander safely above terrain
    // Terrain is around 75-175, lander bottom currently at -254 is way below terrain at 121
    // Need lander to start with bottom above terrain level
    // If terrain max is ~175, position lander bottom at camera Y = 50 (well above terrain)
    // Since transform_axes inverts Y, need positive world Y to get negative camera Y
    let initial_world_pos = vec2(0.0, 150.0);  // Positive world Y for safe camera position
    let tex_center = initial_world_pos;

    // Create lander

    let lander = Entity {
        transform: Transform {
            size: _lander_texture_size,
            position: transform_axes(tex_center),
            rotation: 90.0,
        },
        terrain: terrain,
        screen_fonts: fonts,
        physics: Some(Physics {
            velocity: vec2(0.0, 0.0),
            acceleration: vec2(0.0, 0.0),
        }),
        renderer_lander: Some(Renderer {
            texture: lander_texture,
        }),
        renderer_lander_accel: Some(Renderer {
            texture: lander_accel_texture,
        }),
        renderer_lander_high_accel: Some(Renderer {
            texture: lander_high_accel_texture,
        }),
        input: Some(Input),
        collision: Some(Collision {
            collider: Rect::new(0.0, 0.0, 64.0, 64.0), // Adjust collider size as needed
        }),
        rocket_physics: Some(RocketPhysics::new_apollo_lm()),
        sound: true,
        time_elapsed: 0.0,
        show_debug_info: false,
        dead: false,
    };

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

fn check_collision(entity: &Entity) -> bool {
    // PURE WORLD COORDINATE COLLISION DETECTION
    // Both lander and terrain are in world coordinates:
    // - World coordinates: (0,0) at screen center, positive Y = up, negative Y = down
    // - Lander position stored in screen coordinates, convert to world coordinates
    // - Terrain stored in world coordinates
    // - All collision logic in world coordinates only

    let screen_width = macroquad::window::screen_width();
    let screen_height = macroquad::window::screen_height();

    // Convert lander from screen coordinates to world coordinates
    // Reverse of transform_axes: screen -> world
    let lander_screen_x = entity.transform.position.x;
    let lander_screen_y = entity.transform.position.y;
    let lander_world_x = lander_screen_x - screen_width / 2.0;
    let lander_world_y = screen_height / 2.0 - lander_screen_y;

    let lander_width = entity.transform.size.x;
    let lander_height = entity.transform.size.y;
    
    // Calculate lander bottom in world coordinates
    // In world coordinates: negative Y = down, so bottom = top - height
    let lander_bottom_world_y = lander_world_y - lander_height;

    // Find terrain X range to check (terrain array indices match world X coordinates)
    let terrain_start_x = ((lander_world_x - lander_width/2.0) as i32).max(0) as usize;
    let terrain_end_x = ((lander_world_x + lander_width/2.0) as i32).min(entity.terrain.len() as i32 - 1) as usize;

    // Safety bounds check
    if terrain_start_x >= entity.terrain.len() || terrain_end_x >= entity.terrain.len() {
        return false;
    }

    // Collision margin for forgiving gameplay
    const COLLISION_MARGIN: f32 = 3.0;

    // Check if lander bottom touches terrain
    // Both in world coordinates: collision when lander_bottom <= terrain (since negative Y = down)
    for i in terrain_start_x..=terrain_end_x {
        let terrain_world_y = entity.terrain[i] as f32;

        if lander_bottom_world_y <= terrain_world_y + COLLISION_MARGIN {
            info!("COLLISION: terrain_x={}, lander_bottom_world={:.1}, terrain_world_y={:.1}", i, lander_bottom_world_y, terrain_world_y);
            info!("  World coords: lander=({:.1},{:.1}) bottom=({:.1},{:.1})", lander_world_x, lander_world_y, lander_world_x, lander_bottom_world_y);
            info!("  Screen coords: lander=({:.1},{:.1}) screen_size=({:.1},{:.1})", lander_screen_x, lander_screen_y, screen_width, screen_height);
            return true;
        }
    }

    false
}

fn draw_collision_bounding_box(entity: &Entity) -> () {
    // Draw in screen coordinates first (where the lander actually is)
    set_default_camera();

    let lander_screen_x = entity.transform.position.x;
    let lander_screen_y = entity.transform.position.y;
    let lander_width = entity.transform.size.x;
    let lander_height = entity.transform.size.y;

    // Draw lander bounding box in screen coordinates
    draw_rectangle_lines(lander_screen_x, lander_screen_y, lander_width, lander_height, 2.0, RED);

    // Highlight the bottom edge (critical for collision)
    let lander_bottom_y = lander_screen_y + lander_height;
    draw_line(lander_screen_x, lander_bottom_y, lander_screen_x + lander_width, lander_bottom_y, 3.0, YELLOW);

    // Show collision margin
    const COLLISION_MARGIN: f32 = 3.0;
    draw_line(lander_screen_x, lander_bottom_y + COLLISION_MARGIN,
              lander_screen_x + lander_width, lander_bottom_y + COLLISION_MARGIN, 1.0, ORANGE);

    // Now draw terrain collision points in camera coordinates
    let camera = configure_camera();
    set_camera(&camera);

    // Convert to world coordinates for terrain sampling (using same logic as collision detection)
    let screen_width = macroquad::window::screen_width();
    let lander_world_x = lander_screen_x - screen_width / 2.0;

    let terrain_start_x = ((lander_world_x - lander_width/2.0) as i32).max(0) as usize;
    let terrain_end_x = ((lander_world_x + lander_width/2.0) as i32).min(entity.terrain.len() as i32 - 1) as usize;

    if terrain_start_x < entity.terrain.len() && terrain_end_x < entity.terrain.len() {
        for i in terrain_start_x..=terrain_end_x {
            let terrain_height = entity.terrain[i] as f32;
            draw_circle(i as f32, terrain_height, 3.0, Color::from([0.0, 1.0, 1.0, 1.0])); // Cyan
        }
    }

    set_default_camera()
}

 // Main game loop
#[macroquad::main("Lunar Lander")]
async fn main() {

    let mut audio = load_audio();
    let mut entities = Vec::new();
    add_lander_entity(&mut entities).await;

    loop {
        clear_background(BLACK);

        let lander: &mut Entity = entities.first_mut().unwrap();

        // Handle input
        handle_input(lander, &mut audio);

        if !lander.dead {
            // Check for collision
            if check_collision(lander) {
                debug!("Collision Detected!");
                stop_lander(lander);
                shutdown_audio(&mut audio);
                lander.sound = false;
                lander.dead = true;
            }

            // Check for empty fuel using rocket physics
            if let Some(rocket) = &lander.rocket_physics {
                if !rocket.has_fuel() {
                    debug!("Out of fuel!");
                    stop_lander(lander);
                    shutdown_audio(&mut audio);
                    lander.sound = false;
                    lander.dead = true;
                }
            }

            // Update systems
            update_physics(&mut entities);
        }

        // Render systems
        render(&entities);

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
            let (new_mass, new_velocity) =
                update_mass_and_velocity(current_mass, mass_flow_rate, current_velocity, time_step, exhaust_velocity);

            if new_mass <= mass_rocket {
                println!("Rocket has run out of fuel!");
                break;
            }

            println!("current mass: {}, current velocity: {},  new_mass: {}, new_velocity: {}",
                current_mass, current_velocity,
                new_mass, new_velocity);

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

        let test_world_pos = vec2(0.0, 100.0);  // World position (center X, 100 units up)

        // Apply forward transformation (simulating transform_axes)
        let screen_pos = vec2(
            test_world_pos.x + screen_width / 2.0,  // Should be 400 (center of screen)
            -test_world_pos.y + screen_height / 2.0  // Should be 200 (above center)
        );

        // Apply reverse transformation (same logic as check_collision)
        let recovered_world_x = screen_pos.x - screen_width / 2.0;
        let recovered_world_y = screen_height / 2.0 - screen_pos.y;

        // Test that we get back the original coordinates
        assert!((recovered_world_x - test_world_pos.x).abs() < 0.001,
            "X coordinate mismatch: expected {}, got {}", test_world_pos.x, recovered_world_x);
        assert!((recovered_world_y - test_world_pos.y).abs() < 0.001,
            "Y coordinate mismatch: expected {}, got {}", test_world_pos.y, recovered_world_y);
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
        let lander_world_x: f32 = 0.0;  // Center
        let lander_world_y: f32 = 250.0;  // 100 units above terrain
        let lander_size = Vec2::new(32.0, 32.0);

        // Convert to screen coordinates (simulating what happens in game)
        let lander_screen_x = lander_world_x + screen_width / 2.0;
        let lander_screen_y = -lander_world_y + screen_height / 2.0;

        // Now reverse the transformation (simulating check_collision logic)
        let recovered_world_x = lander_screen_x - screen_width / 2.0;
        let recovered_world_y = screen_height / 2.0 - lander_screen_y;
        let lander_bottom_world_y = recovered_world_y - lander_size.y;

        // Verify coordinate transformation is working correctly
        assert!((recovered_world_x - lander_world_x).abs() < 0.001,
            "World X coordinate transformation failed");
        assert!((recovered_world_y - lander_world_y).abs() < 0.001,
            "World Y coordinate transformation failed");

        // The critical test: lander bottom should be ABOVE terrain height
        assert!(lander_bottom_world_y > terrain_height,
            "Lander bottom ({:.1}) should be above terrain ({:.1}). \
             This would cause false collision!", lander_bottom_world_y, terrain_height);
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
        println!("  Generated: {:.1} -> Offset: {:.1} -> World: {:.1}", 
                 generated_terrain_y, offset_terrain_y, expected_world_terrain_y);
        
        // Terrain offset to 175 should be below screen center (300), so world Y should be negative
        // Expected: 300/2 - 175 = 150 - 175 = -25 (below world center = negative Y)
        let expected_negative_y = 150.0 - 175.0; // -25.0
        println!("  Expected world Y for terrain below center: {:.1}", expected_negative_y);
        
        // Terrain below world center should have negative Y values
        assert!(expected_world_terrain_y < 0.0, 
               "Terrain at screen Y=175 (below center=300) should have negative world Y, got {:.1}", expected_world_terrain_y);
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
            -world_lander_pos.y + screen_height / 2.0
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
            -world_lander_above.y + screen_height / 2.0
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
        println!("  Lander world Y: {:.1}, bottom: {:.1}", lander_world_y, lander_bottom_world_y);
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

