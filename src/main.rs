#![allow(dead_code)]
use macroquad::prelude::*;
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
    renderer: Option<Renderer>,
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
            renderer: None,
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
        if let Some(renderer) = &entity.renderer {
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
            // Render other properties such as rotation
        }
    }
}

// Main game loop
#[macroquad::main("Lunar Lander")]
async fn main() {
    let mut entities = Vec::new();
    // Load a texture (replace "texture.png" with the path to your texture)
    let texture = load_texture("lander.png").await.expect("Failed to load texture");

    // Get the size of the texture
    let tex_size = texture.size();

    // Create entities
    let lander = Entity {
        transform: Transform {
            size: tex_size,
            position: vec2(screen_width() / 2.0 - 0.5*tex_size.x, screen_height() / 2.0 - 0.5*tex_size.y),
            rotation: 25.0,
        },
        physics: Some(Physics {
            velocity: vec2(0.0, 0.0),
            acceleration: vec2(0.0, 0.0),
        }),
        renderer: Some(Renderer {
            texture: texture,
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

        // Handle input

        // Render systems
        render(&entities);

        let hundred_millis = time::Duration::from_millis(100);
        //let now = time::Instant::now();

        thread::sleep(hundred_millis);

        let lander: &mut Entity = entities.first_mut().unwrap();

        lander.transform.rotation = (lander.transform.rotation + 15.).rem_euclid(360.0) as f32;

        next_frame().await
    }
}

