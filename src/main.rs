#![allow(dead_code)]
use macroquad::prelude::*;
use rusty_audio::Audio;

use std::{thread, time};

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
struct Entity {
    transform: Transform,
    physics: Option<Physics>,
    renderer_lander: Option<Renderer>,
    renderer_lander_accel: Option<Renderer>,    
    input: Option<Input>,
    collision: Option<Collision>,
}

impl Entity {
    fn new() -> Self {
        Self {
            transform: Transform {
                size: vec2(0.0,0.0),
                position: vec2(0.0, 0.0),
                rotation: 0.0,
            },
            physics: None,
            renderer_lander: None,
            renderer_lander_accel: None,            
            input: None,
            collision: None,
        }
    }
}

// Define systems
fn update_physics(entities: &mut Vec<Entity>) {
    for entity in entities {
        if let Some(physics) = &mut entity.physics {
            physics.velocity += physics.acceleration * get_frame_time();
            entity.transform.position += physics.velocity * get_frame_time();
        }
    }
}

fn render(entities: &Vec<Entity>) {
    for entity in entities {
        let o_renderer = if entity.physics.as_ref().unwrap().acceleration.y > 0.0 {
            &entity.renderer_lander_accel
        } else {
            &entity.renderer_lander
        };
        
        if let Some(renderer) = o_renderer {
            draw_texture_ex(&renderer.texture,
                            entity.transform.position.x,
                            entity.transform.position.y,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(entity.transform.size), // Set destination size if needed
                                rotation: entity.transform.rotation.to_radians(), // Rotate by 45 degrees (converted to radians)
                                ..Default::default() // Other parameters set to default
                            }

            );
        }
    }
}

// Main game loop
#[macroquad::main("Lunar Lander")]
async fn main() {
    let mut audio = Audio::new();
    audio.add("ambient", "218883-jet_whine_v2_mid_loop.wav"); 
    audio.add("acceleration", "218837-jet_turbine_main_blast.wav"); 

    let mut entities = Vec::new();
    // Load a texture (replace "texture.png" with the path to your texture)
    let lander = load_texture("lander.png").await.expect("Failed to load texture");
    let lander_accel = load_texture("lander-accel.png").await.expect("Failed to load texture");

    // Get the size of the texture
    let lander_tex_size = lander.size();

    // Create entities
    let lander = Entity {
        transform: Transform {
            size: lander_tex_size,
            position: vec2(screen_width() / 2.0 - 0.5*lander_tex_size.x, screen_height() / 2.0 - 0.5*lander_tex_size.y),
            rotation: 0.0,
        },
        physics: Some(Physics {
            velocity: vec2(0.0, 0.0),
            acceleration: vec2(0.0, 0.0),
        }),
        renderer_lander: Some(Renderer {
            texture: lander,
        }),
        renderer_lander_accel: Some(Renderer {
            texture: lander_accel,
        }),        
        input: Some(Input),
        collision: Some(Collision {
            collider: Rect::new(0.0, 0.0, 64.0, 64.0), // Adjust collider size as needed
        }),
    };
    entities.push(lander);

    loop {
        clear_background(BLACK);

        // Update systems
        update_physics(&mut entities);

        let lander: &mut Entity = entities.first_mut().unwrap();

        // Handle input
        if is_key_down(KeyCode::Right) {
            // rotate lander right
            lander.transform.rotation = (lander.transform.rotation + 15.).rem_euclid(360.0) as f32;    
        }
        if is_key_down(KeyCode::Left) {
            // rotate lander left
            lander.transform.rotation = (lander.transform.rotation - 15.).rem_euclid(360.0) as f32;    
        }
        if is_key_down(KeyCode::Up){
            // accelerate lander
            let angle = lander.transform.rotation.to_radians();
            let acceleration = vec2(0.0, 0.1);
            let acceleration = vec2(acceleration.x * angle.cos() - acceleration.y * angle.sin(),
                                    acceleration.x * angle.sin() + acceleration.y * angle.cos());
            lander.physics.as_mut().unwrap().acceleration = acceleration;
            audio.play("acceleration");

        }
        if is_key_released(KeyCode::Up){
            // stop acceleration
            lander.physics.as_mut().unwrap().acceleration = vec2(0.0, 0.0);
            audio.stop();
        }

        // Check if the "Space" key is released
        //if is_key_released(KeyCode::A) {
            // stop rotation
        //}

        // Render systems
        render(&entities);

        let hundred_millis = time::Duration::from_millis(100);
        //let now = time::Instant::now();

        if !audio.is_playing() {
            audio.play("ambient"); // Execution continues while playback occurs in another thread.            
        }        

        thread::sleep(hundred_millis);

        next_frame().await
    }
}

