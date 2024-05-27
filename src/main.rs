#![allow(dead_code)]
#![allow(unused_imports)]
use std::thread::sleep;

use macroquad::prelude::*;
use rusty_audio::Audio;
use macroquad_text::Fonts;

const GLASS_TTY_VT220: &[u8] = include_bytes!("../assets/fonts/Glass_TTY_VT220.ttf");
const MAX_ACCEL_X: f32 = 150.0;
const MAX_ACCEL_Y: f32 = 150.0;
const MILLIS_DELAY: u64 = 40;
const ROTATION_INCREMENT: f32 = 2.0;
const ACCEL_INCREMENT: f32 = 6.5;
const FULL_CIRCLE_DEGREES: f32 = 360.0;
const TEXTURE_SCALE_X: f32 = 0.5;
const TEXTURE_SCALE_Y: f32 = 0.5;
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
    screen_fonts: Fonts<'a>,
    surface: Vec<Line>,
    physics: Option<Physics>,
    renderer_lander: Option<Renderer>,
    renderer_lander_accel: Option<Renderer>,
    renderer_lander_high_accel: Option<Renderer>,    
    input: Option<Input>,
    collision: Option<Collision>,
}

impl<'a> Entity<'a> {
    fn new() -> Self {
        Self {
            transform: Transform {
                size: vec2(0.0,0.0),
                position: vec2(0.0, 0.0),
                rotation: 0.0,
            },
            screen_fonts: Fonts::<'a>::default(),
            surface: Vec::<Line>::new(),
            physics: None,
            renderer_lander: None,
            renderer_lander_accel: None,
            renderer_lander_high_accel: None,
            input: None,
            collision: None,
        }
    }
}

// Define systems
fn update_physics(entities: &mut Vec<Entity>) {
    for entity in entities {
        if let Some(physics) = &mut entity.physics {
            //BUGBUG: This is not correct physics
            physics.velocity.x = physics.velocity.x + physics.acceleration.x * get_frame_time();
            physics.velocity.y = physics.velocity.y + physics.acceleration.y * get_frame_time();            
            entity.transform.position += physics.velocity * get_frame_time();
            entity.transform.position.x = entity.transform.position.x.rem_euclid(screen_width());
            entity.transform.position.y = entity.transform.position.y.rem_euclid(screen_height());
        }
    }
}

fn render(entities: &Vec<Entity>) {
    let camera = configure_camera();
    
    for entity in entities {
        set_default_camera();
        if let Some(phys) = &entity.physics {

            let x = entity.transform.position.x;
            let y = entity.transform.position.y;
            let angle_degrees = entity.transform.rotation;
            let accel = phys.acceleration;                        
            let accel_x = phys.acceleration.x;
            let accel_y = phys.acceleration.y;
            let vel_x = phys.velocity.x;
            let vel_y = phys.velocity.y;

             
            let mut text = format!("x: {:.2}, y: {:.2}, angle: {:.2}", x, y, angle_degrees);
            entity.screen_fonts.draw_text(&text, 0.0, 480.0, 10.0, Color::from([1.0; 4]));            
            text = format!("accel_x: {:.2}, accel_y: {:.2}, accel_length: {:.2}, vel_x: {:.2}, vel_y: {:.2}", accel_x, accel_y, accel.length(), vel_x, vel_y);
            entity.screen_fonts.draw_text(&text, 0.0, 510.0, 10.0, Color::from([1.0; 4]));
         
        // If there's acceleration, use the appropriate image (lander_accel or lander_high_accel)
        let accel = phys.acceleration;
        let o_renderer = if accel.length() > 0.0 {
            if accel.length() > 1.0 {
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
            //draw_surface(entity);
            set_default_camera();
            draw_text(&entity.screen_fonts);
        }
    }
    }
}

fn draw_surface(entity: &Entity) {
    for line in &entity.surface {
        draw_line(line.start.x, line.start.y, line.end.x, line.end.y, 1.0, WHITE);
    }
}

fn define_surface(screen_height: f32, _screen_width: f32) -> Vec<Line> {
    let mut lines = Vec::new();
    // Maximum length of each line segment (1/20th of the screen height)
    let max_line_length = screen_height / 20.0;

    // Generate random-length connected line segments and store them in the vector
    let num_lines = 20; // Number of lines
    let mut last_end = vec2(0.0, rand::gen_range(0.0, screen_height)); // Starting point for the first line
    for _ in 0..num_lines {
        let line_length = rand::gen_range(0.0, max_line_length); // Random line length
        let end_x = last_end.x + line_length; // End x-coordinate of the line
        let end_y = rand::gen_range(0.0, screen_height); // Random y-coordinate of the line
        let end = vec2(end_x, end_y); // End point of the line
        lines.push(Line { start: last_end, end }); // Add line to the vector
        last_end = end; // Update last end point for the next line
    }    
    lines
}

fn draw_text(fonts: &Fonts) {
    fonts.draw_text("SCORE", 20.0, 0.0, 15.0, Color::from([1.0; 4]));
    fonts.draw_text("TIME", 20.0, 20.0, 15.0, Color::from([1.0; 4]));
    fonts.draw_text("FUEL", 20.0, 40.0, 15.0, Color::from([1.0; 4]));

    let w = macroquad::window::screen_width();
    let right_text_start = w - 175.0;
    fonts.draw_text("ALTITUDE", right_text_start, 0.0, 15.0, Color::from([1.0; 4]));
    fonts.draw_text("HORIZONTAL SPEED", right_text_start, 20.0, 15.0, Color::from([1.0; 4]));
    fonts.draw_text("VERTICAL SPEED", right_text_start, 40.0, 15.0, Color::from([1.0; 4]));
    //fonts.draw_text("0,0", 0.0, 0.0, 15.0, Color::from([1.0; 4]));
    //fonts.draw_text("X_MAX,0", screen_width()-60.0, 0.0, 15.0, Color::from([1.0; 4]));
    //fonts.draw_text("0,Y_MAX", 0.0, screen_height()-20.0, 15.0, Color::from([1.0; 4]));
    //fonts.draw_text("X_MAX,Y_MAX", screen_width()-90.0, screen_height()-20.0, 15.0, Color::from([1.0; 4]));
}

fn handle_input(lander: &mut Entity, audio: &mut Audio) {
        // Handle input
        if is_key_down(KeyCode::Right) {
            println!("Right key down before: {:.2}", lander.transform.rotation);
            // rotate lander right
            lander.transform.rotation = (lander.transform.rotation - ROTATION_INCREMENT).rem_euclid(FULL_CIRCLE_DEGREES) as f32;
            println!("Right key down after: {:.2}", lander.transform.rotation);
        }
        if is_key_down(KeyCode::Left) {
            println!("Left key down before: {:.2}", lander.transform.rotation);
            // rotate lander left
            lander.transform.rotation = (lander.transform.rotation + ROTATION_INCREMENT).rem_euclid(FULL_CIRCLE_DEGREES) as f32;
            println!("Left key down after: {:.2}", lander.transform.rotation);
        }
        if is_key_down(KeyCode::Up){
            // accelerate lander
            if let Some(phys) = lander.physics.as_mut() {
                let angle = lander.transform.rotation.to_radians();
                // Incremental acceleration is in direction of the lander
                //let inc_acceleration = vec2(ACCEL_INCREMENT * angle.cos() - ACCEL_INCREMENT * angle.sin(),
                //                        ACCEL_INCREMENT * angle.sin() + ACCEL_INCREMENT * angle.cos());

                let inc_acceleration = vec2(ACCEL_INCREMENT * angle.cos(), ACCEL_INCREMENT * angle.sin());
                println!("angle: {:?}, current acceleration: {:?}", angle.to_degrees(), inc_acceleration);

                phys.acceleration = phys.acceleration + inc_acceleration;
                phys.acceleration.x = phys.acceleration.x.min(MAX_ACCEL_X);
                phys.acceleration.y = phys.acceleration.y.min(MAX_ACCEL_Y);
                println!("acceleration: {:?}", phys.acceleration);
                audio.play("acceleration"); 
            }
        }
        if is_key_released(KeyCode::Up){
            // stop acceleration
            if let Some(phys) = lander.physics.as_mut() {
                phys.acceleration = vec2(0.0, 0.0);
                audio.stop();
            }
        }
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
    // Load a texture (replace "texture.png" with the path to your texture)
    let lander_texture = load_texture("assets/images/lander.png").await.expect("Failed to load texture");
    let lander_accel_texture = load_texture("assets/images/lander-accel.png").await.expect("Failed to load texture");
    let lander_high_accel_texture = load_texture("assets/images/lander-high-accel.png").await.expect("Failed to load texture");    

    // Get the size of the texture
    let lander_texture_size = lander_texture.size().mul_add(Vec2::new(TEXTURE_SCALE_X, TEXTURE_SCALE_Y), Vec2::new(0.0, 0.0));

    let screen_width = macroquad::window::screen_width();
    let screen_height = macroquad::window::screen_height();
    let lines = define_surface(screen_height, screen_width);

    let fonts = load_fonts();
    let tex_center = vec2(-lander_texture_size.x / 2.0, lander_texture_size.y / 2.0);

    // Create entities
    let lander = Entity {
        transform: Transform {
            size: lander_texture_size,
            position: transform_axes(tex_center),
            rotation: 0.0,
        },
        screen_fonts: fonts,
        surface: lines,
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
    };

    entities.push(lander); 

}

fn update_audio(audio: &mut Audio) {
    if !audio.is_playing() {
        audio.play("ambient"); // Execution continues while playback occurs in another thread.            
    }        
}

/* #[macroquad::main("Standard Axes Orientation")]
async fn main() {
    let camera = configure_camera();
    let fonts = load_fonts();
    loop {
        // Set the camera
        set_camera(&camera);

        clear_background(WHITE);

        // Draw a coordinate system for demonstration
        draw_line(0.0, screen_height() / 2.0, screen_width(), screen_height() / 2.0, 2.0, BLACK); // x-axis
        draw_line(screen_width() / 2.0, 0.0, screen_width() / 2.0, screen_height(), 2.0, BLACK); // y-axis

        // Draw some shapes
        draw_circle(screen_width() / 2.0, screen_height() / 2.0, 30.0, RED); // center circle
        fonts.draw_text("Center", screen_width() / 2.0 + 40.0, screen_height() / 2.0, 30, DARKGRAY);

        draw_circle(screen_width() / 2.0, screen_height() / 4.0, 20.0, BLUE); // top circle
        fonts.draw_text("Top", screen_width() / 2.0 + 30.0, screen_height() / 4.0, 30, DARKGRAY);

        draw_circle(screen_width() / 4.0, screen_height() / 2.0, 20.0, GREEN); // left circle
        fonts.draw_text("Left", screen_width() / 4.0, screen_height() / 2.0 + 30.0, 30, DARKGRAY);

        // Reset to the default camera before ending the frame
        set_default_camera();

        next_frame().await;
    }
}
 */

 // Main game loop
#[macroquad::main("Lunar Lander")]
async fn main() {

    let mut audio = load_audio();
    let mut entities = Vec::new();
    add_lander_entity(&mut entities).await;

    loop {
        clear_background(BLACK);

        // Update systems
        update_physics(&mut entities);

        let lander: &mut Entity = entities.first_mut().unwrap();

        // Handle input
        handle_input(lander, &mut audio);

        // Render systems
        render(&entities);

        update_audio(&mut audio);

        // Pause for the next frame
        sleep(std::time::Duration::from_millis(MILLIS_DELAY));

        next_frame().await
    }
}
