use macroquad::prelude::*;
use macroquad_text::Fonts;

use crate::assets::load_fonts;
use crate::physics::{Physics, RocketEngine};
use crate::rendering::load_lander_textures;
use crate::surface;
use crate::utils::transform_axes;

const TERRAIN_Y_OFFSET: f64 = 75.0;
const TEXTURE_SCALE_LANDER_X: f32 = 0.5;
const TEXTURE_SCALE_LANDER_Y: f32 = 0.5;

#[derive(Debug)]
pub struct Line {
    pub start: Vec2,
    pub end: Vec2,
}

// Define components
#[derive(Debug, Clone)]
pub struct Transform {
    pub size: Vec2,
    pub position: Vec2,
    pub rotation: f32,
}

pub struct Renderer {
    pub lander_texture: Texture2D,
    // Other rendering properties
}

pub struct Input;

pub struct Collision {
    pub collider: Rect,
}

// Define entities
pub struct Entity<'a> {
    pub transform: Transform,
    pub terrain: Vec<f64>,
    pub flat_spots: Vec<(usize, usize)>, // Store flat spot ranges for direct reference
    pub screen_fonts: Fonts<'a>,
    pub physics: Option<Physics>,
    pub rocket_physics: Option<RocketEngine>,
    pub renderer_lander: Option<Renderer>,
    pub renderer_lander_accel: Option<Renderer>,
    pub renderer_lander_high_accel: Option<Renderer>,
    pub input: Option<Input>,
    pub collision: Option<Collision>,
    pub sound: bool,
    pub time_elapsed: f32,
    pub show_debug_info: bool,
    pub dead: bool,
    pub mission_success: bool,
    pub current_audio: Option<String>,
}

impl<'a> Entity<'a> {
    pub fn new() -> Self {
        Entity {
            transform: Transform {
                size: Vec2::new(0.0, 0.0),
                position: Vec2::new(0.0, 0.0),
                rotation: 90.0,
            },
            terrain: Vec::new(),
            flat_spots: Vec::new(),
            screen_fonts: load_fonts(),
            physics: Some(Physics::new(23200.0)), // Apollo LM total mass
            rocket_physics: Some(RocketEngine::new_apollo_lm()),
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

    pub fn initialize_with_terrain_and_position(&mut self, lander_texture_size: Vec2) {
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
        let total_mass = if let Some(rocket) = &self.rocket_physics {
            rocket.total_mass()
        } else {
            23200.0 // Default Apollo LM mass
        };
        self.physics = Some(Physics::new(total_mass));

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

pub async fn add_lander_entity<'a>(entities: &mut Vec<Entity<'a>>) {
    // Load textures first to get actual lander dimensions
    let (lander_texture, lander_accel_texture, lander_high_accel_texture) = load_lander_textures().await;

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