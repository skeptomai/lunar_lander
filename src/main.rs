#![allow(dead_code)]
#![allow(unused_imports)]
use core::time;
use std::thread::sleep;

use macroquad::prelude::*;
use rusty_audio::Audio;

mod physics;
mod surface;
mod rendering;
mod entity;
mod collision;
mod input;
mod audio;
mod utils;
mod assets;

use physics::{Physics, RocketEngine};
use entity::{Entity, add_lander_entity};
use collision::{CollisionType, check_collision};
use input::{handle_input, stop_lander};
use audio::{load_audio, shutdown_audio};
use rendering::{render, configure_camera};

const MILLIS_DELAY: u64 = 40;
// acceleration due to gravity on earth
//const ACCEL_GRAV_Y: f32 = 9.8;
// acceleration due to gravity on the moon
const ACCEL_GRAV_Y: f32 = 1.625;

// Define systems
fn update_physics(entities: &mut Vec<Entity>) {
    let dt = get_frame_time();

    for entity in entities {
        if entity.dead {
            continue;
        }

        if let Some(physics) = &mut entity.physics {
            // Reset forces for this frame
            physics.reset_forces();

            // Apply gravity force
            let gravity_force = Vec2::new(0.0, -ACCEL_GRAV_Y * physics.mass as f32);
            physics.add_force(gravity_force);

            // Generate thrust force if rocket engine present
            if let Some(rocket) = &mut entity.rocket_physics {
                // Update physics mass based on current rocket mass
                physics.mass = rocket.total_mass();
                
                let thrust_force = rocket.generate_thrust(dt);
                physics.add_force(thrust_force);
            }

            // Integrate forces into motion
            physics.integrate(dt);
            entity.transform.position += physics.velocity * dt;

            // Wrap around screen (maintain lunar lander behavior)
            entity.transform.position.x = entity.transform.position.x.rem_euclid(screen_width());
            entity.transform.position.y = entity.transform.position.y.rem_euclid(screen_height());

            // Update elapsed time with proper precision
            entity.time_elapsed += dt;
        }
    }
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

            // Check for empty fuel using rocket engine
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