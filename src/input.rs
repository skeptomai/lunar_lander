use macroquad::prelude::*;
use rusty_audio::Audio;

use crate::audio::{update_audio, shutdown_audio};
use crate::entity::{Entity, Collision};

const ROTATION_INCREMENT: f32 = 3.0;
const FULL_CIRCLE_DEGREES: f32 = 360.0;

pub fn handle_input(lander: &mut Entity, audio: &mut Audio) {
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
            // 0° = up, so add 90° to convert to standard math coordinates
            let angle = (lander.transform.rotation + 90.0).to_radians();
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

pub fn stop_lander(lander: &mut Entity) {
    if let Some(phys) = lander.physics.as_mut() {
        phys.velocity = vec2(0.0, 0.0);
        phys.forces = vec2(0.0, 0.0);
    }
    if let Some(rocket) = &mut lander.rocket_physics {
        rocket.stop_thrust();
    }
    lander.collision = Some(Collision {
        collider: Rect::new(0.0, 0.0, 0.0, 0.0),
    });
}

pub fn reset_lander(lander: &mut Entity) {
    // Reset lander using common initialization method
    let lander_texture_size = lander.transform.size; // Preserve existing size
    lander.initialize_with_terrain_and_position(lander_texture_size);
}