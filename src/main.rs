#![allow(dead_code)]
#![allow(unused_imports)]
use core::time;
use std::thread::sleep;

use macroquad::prelude::*;
use rusty_audio::Audio;
use macroquad_text::Fonts;

mod surface;

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
struct Transform {
    size: Vec2,
    position: Vec2,
    rotation: f32,
}

struct Physics {
    velocity: Vec2,
    acceleration: Vec2,
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
    renderer_lander: Option<Renderer>,
    renderer_lander_accel: Option<Renderer>,
    renderer_lander_high_accel: Option<Renderer>,
    input: Option<Input>,
    collision: Option<Collision>,
    sound: bool,
    time_elapsed: i32,
    show_debug_info: bool,
    mass_of_craft: f64,
    mass_of_fuel: f64,
    mass_flow_rate: f64,
    exhaust_velocity: f64,
    dead: bool,
}

impl<'a> Entity<'a> {
    fn new() -> Self {
        Self {
            transform: Transform {
                size: vec2(0.0,0.0),
                position: vec2(0.0, 0.0),
                rotation: 0.0,
            },
            terrain: Vec::new(),
            screen_fonts: Fonts::<'a>::default(),
            physics: None,
            renderer_lander: None,
            renderer_lander_accel: None,
            renderer_lander_high_accel: None,
            input: None,
            collision: None,
            sound: true,
            time_elapsed: 0,
            show_debug_info: false,
            mass_of_craft: 50000.0,
            mass_of_fuel: 20000.0,
            mass_flow_rate: 50.0,
            exhaust_velocity: 300.0,
            dead: false,
        }
    }
}

// Define systems
fn update_physics(entities: &mut Vec<Entity>) {
    for entity in entities {
        if entity.dead {
            continue;
        }
        if let Some(physics) = &mut entity.physics {
            let frame_time = 0.1;
            physics.velocity.x = physics.velocity.x + physics.acceleration.x * frame_time;
            physics.velocity.y = physics.velocity.y + (physics.acceleration.y - ACCEL_GRAV_Y) * frame_time;
            entity.transform.position += physics.velocity * get_frame_time();
            entity.transform.position.x = entity.transform.position.x.rem_euclid(screen_width());
            entity.transform.position.y = entity.transform.position.y.rem_euclid(screen_height());
            entity.time_elapsed += frame_time as i32;
            entity.mass_of_fuel -= entity.mass_flow_rate;
        }
    }
}

/// Calculates the new mass and velocity of the rocket.
///
/// # Arguments
///
/// * `current_mass` - The current mass of the rocket (kg).
/// * `mass_flow_rate` - The rate at which mass is lost (kg/s).
/// * `current_velocity` - The current velocity of the rocket (m/s).
/// * `time_step` - The time step over which the mass is reduced (s).
/// * `exhaust_velocity` - The effective exhaust velocity (m/s).
///
/// # Returns
///
/// * `(new_mass, new_velocity)` - The new mass and velocity of the rocket.
pub fn update_mass_and_velocity(
    current_mass: f64,
    mass_flow_rate: f64,
    current_velocity: f64,
    time_step: f64,
    exhaust_velocity: f64,
) -> (f64, f64) {
    // Calculate the new mass after the time step
    let new_mass = current_mass - mass_flow_rate * time_step;

    // Ensure new_mass is not negative
    let new_mass = if new_mass < 0.0 { 0.0 } else { new_mass };

    // Calculate the change in velocity using Tsiolkovsky's equation
    let delta_v = exhaust_velocity * (current_mass / new_mass).ln();

    // Calculate the new velocity
    let new_velocity = current_velocity + delta_v + (1.625 * time_step);

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
                draw_collision_bounding_box(entity);
            }

            // If there's acceleration, use the appropriate image (lander_accel or lander_high_accel)
            let accel = phys.acceleration;
            let o_renderer = if accel.length() > 0.0 {
                if accel.length() > 40.0 {
                    &entity.renderer_lander_high_accel
                } else {
                    &entity.renderer_lander_accel
                }
            } else {
                &entity.renderer_lander
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

            // plot surface
            for i in 0..(entity.terrain.len() - 1) {
                draw_line(
                    i as f32,
                    entity.terrain[i] as f32,
                    (i + 1) as f32,
                    entity.terrain[i + 1] as f32,
                    2.0,
                    DARKGREEN,
                );
            }

            set_default_camera();
            if !entity.dead {
                draw_text(&entity);
            }
        }
    }
}

fn draw_text(entity: &Entity) {
    let fonts = &entity.screen_fonts;
    let phys = entity.physics.as_ref().unwrap();

    let time_elapsed_text = format!("TIME {}", entity.time_elapsed);
    fonts.draw_text("SCORE", 20.0, 0.0, 15.0, Color::from([1.0; 4]));
    fonts.draw_text(&time_elapsed_text, 20.0, 20.0, 15.0, Color::from([1.0; 4]));
    let fuel_text = format!("FUEL: {:.2}", entity.mass_of_fuel);
    fonts.draw_text(&fuel_text, 20.0, 40.0, 15.0, Color::from([1.0; 4]));

    let w = macroquad::window::screen_width();
    let right_text_start = w - 195.0;
    let altitude_text = format!("ALTITUDE: {:.2}", entity.transform.position.y);
    let horizontal_speed_text = format!("HORIZONTAL SPEED: {:.2}", phys.velocity.x);
    let vertical_speed_text = format!("VERTICAL SPEED: {:.2}", phys.velocity.y);
    fonts.draw_text(&altitude_text, right_text_start, 0.0, 15.0, Color::from([1.0; 4]));
    fonts.draw_text(&horizontal_speed_text, right_text_start, 20.0, 15.0, Color::from([1.0; 4]));
    fonts.draw_text(&vertical_speed_text, right_text_start, 40.0, 15.0, Color::from([1.0; 4]));
}

fn draw_alert_box(entity: &Entity) {
    let fonts = &entity.screen_fonts;

    let screen_width = screen_width();
    let screen_height = screen_height();
    let box_x = (screen_width - ALERT_BOX_WIDTH) / 2.0;
    let box_y = (screen_height - ALERT_BOX_HEIGHT) / 2.0;

    draw_rectangle(box_x, box_y, ALERT_BOX_WIDTH, ALERT_BOX_HEIGHT, GRAY);
    fonts.draw_text("Mission Failed!", box_x + 20.0, box_y + 40.0, 30.0, RED);
    fonts.draw_text("Press R to Restart", box_x + 20.0, box_y + 70.0, 20.0, WHITE);
}

fn handle_input(lander: &mut Entity, audio: &mut Audio) {
    // Handle input
    if is_key_down(KeyCode::R) {
        reset_lander(lander);        
        update_audio(audio);
    }
    if is_key_down(KeyCode::Escape) {
        // Exit game
        shutdown_audio(audio);
        std::process::exit(0);
    }
    if is_key_released(KeyCode::S) {
        lander.sound = !lander.sound;
    }
    if is_key_down(KeyCode::Right) {
        // rotate lander right
        lander.transform.rotation = (lander.transform.rotation - ROTATION_INCREMENT).rem_euclid(FULL_CIRCLE_DEGREES) as f32;
    }
    if is_key_down(KeyCode::Left) {
        // rotate lander left
        lander.transform.rotation = (lander.transform.rotation + ROTATION_INCREMENT).rem_euclid(FULL_CIRCLE_DEGREES) as f32;
    }
    if is_key_down(KeyCode::Up){

        // accelerate lander
        // Get the angle of the lander
        if let Some(phys) = lander.physics.as_mut() {
            let angle = lander.transform.rotation.to_radians();
            update_increment_acceleration(angle, phys);
            let current_mass = lander.mass_of_craft + lander.mass_of_fuel;
            let mass_flow_rate = lander.mass_flow_rate;
            let current_velocity = phys.velocity.y;
            let time_step = get_frame_time();
            let exhaust_velocity = lander.exhaust_velocity;
            let (_current_mass, _current_velocity) = update_mass_and_velocity(current_mass, mass_flow_rate, current_velocity.into(), 
                time_step.into(), exhaust_velocity);
            audio.play("acceleration");
        }
    }
    if is_key_released(KeyCode::Up){
        // stop acceleration
        if let Some(phys) = lander.physics.as_mut() {
            phys.acceleration = vec2(0.0, 0.0);
            shutdown_audio(audio);
        }
    }
    if is_key_released(KeyCode::D) {
        lander.show_debug_info = !lander.show_debug_info;
    }
    if lander.sound {
        update_audio(audio);
    } else {
        if audio.is_playing() {
            shutdown_audio(audio);
        }
    }

    // Check for collision
    if check_collision(lander) {
        debug!("Collision Detected!");
        draw_alert_box(lander);
        stop_lander(lander);
        shutdown_audio(audio);
        lander.sound = false;
        lander.dead = true;
    }

    // Check for empty fuel
    if lander.mass_of_fuel <= 0.0 {
        debug!("Out of fuel!");
        draw_alert_box(lander);
        stop_lander(lander);
        shutdown_audio(audio);
        lander.sound = false;
        lander.dead = true;
    }
}

fn update_increment_acceleration(angle: f32, phys: &mut Physics) {
    // Increment acceleration in the direction of the lander
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
    lander.collision = Some(Collision {
        collider: Rect::new(0.0, 0.0, 0.0, 0.0),
    });
}

fn reset_lander(lander: &mut Entity) {
    // Reset lander
    // Get the size of the texture
    let lander_texture = &lander.renderer_lander.as_ref().unwrap().texture;
    let lander_texture_size = lander_texture.size().mul_add(Vec2::new(TEXTURE_SCALE_X, TEXTURE_SCALE_Y), Vec2::new(0.0, 0.0));
    let tex_center = vec2(-lander_texture_size.x / 2.0, lander_texture_size.y / 2.0);

    lander.transform.position = transform_axes(tex_center);
    lander.transform.rotation = 90.0;
    lander.physics = Some(Physics {
        velocity: vec2(0.0, 0.0),
        acceleration: vec2(0.0, 0.0),
    });
    lander.time_elapsed = 0;
    lander.mass_of_fuel = 20000.0;
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
    terrain.iter_mut().for_each(|h| *h = *h + TERRAIN_Y_OFFSET); // offset terrain to the bottom of the screen

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
    let lander_texture_size = lander_texture.size().mul_add(Vec2::new(TEXTURE_SCALE_X, TEXTURE_SCALE_Y), Vec2::new(0.0, 0.0));

    let fonts = load_fonts();
    let tex_center = vec2(-lander_texture_size.x / 2.0, lander_texture_size.y / 2.0);

    // Create entities
    let lander = Entity {
        transform: Transform {
            size: lander_texture_size,
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
        sound: true,
        time_elapsed: 0,
        show_debug_info: false,
        exhaust_velocity: 300.0,
        mass_of_craft: 50000.0,
        mass_of_fuel: 20000.0,
        mass_flow_rate: 50.0,
        dead: false,
    };

    entities.push(lander);

}

fn update_audio(audio: &mut Audio) {
    if !audio.is_playing() {
        debug!("Audio is playing. Updating audio.");
        audio.play("ambient"); // Execution continues while playback occurs in another thread.
    }
}

fn shutdown_audio(audio: &mut Audio) {
    audio.stop();
}

fn check_collision(entity: &Entity) -> bool {

    let x0 = entity.transform.position.x;
    let y0 = entity.transform.position.y;
    let x1 = entity.transform.position.x + entity.transform.size.x;
    let _y1 = entity.transform.position.y + entity.transform.size.y;    

    if x0 >= entity.terrain.len() as f32 {
        return false;
    }

    for i in x0 as usize..x1 as usize {
        if y0 < entity.terrain[i] as f32 {
            info!("Collision detected at x: {}, y: {}", entity.transform.position.x, entity.transform.position.y);
            return true;
        }
    }

    false
}

fn draw_collision_bounding_box(entity: &Entity) -> () {
    let camera = configure_camera();
    set_camera(&camera);
    let x0 = entity.transform.position.x;
    let y0 = entity.transform.position.y;
    let x1 = entity.transform.position.x + entity.transform.size.x;
    let _y1 = entity.transform.position.y + entity.transform.size.y;

    draw_rectangle_lines(x0, y0, entity.transform.size.x , entity.transform.size.y+2.0 , 2.0, RED);
    draw_line(x0, y0, x1, y0, 3.0, BLUE);
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

        // Update systems
        update_physics(&mut entities);

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
}

