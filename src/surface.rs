extern crate noise;

use macroquad::logging::debug;
use noise::{NoiseFn, Perlin};
use rand::Rng;

pub fn generate_terrain_with_flat_spot(
    num_points: usize,
    min_height: f64,
    max_height: f64,
    base_frequency: f64,
    octaves: u32,
    persistence: f64,
    lander_width_points: usize,
) -> (Vec<f64>, (usize, usize)) {
    let perlin = Perlin::new();
    let mut terrain = Vec::with_capacity(num_points);
    let mut rng = rand::thread_rng();

    // Pick one random position for flat spot (simple approach - anywhere in terrain)
    let flat_spot_start = rng.gen_range(100..(num_points - lander_width_points - 100));
    let flat_spot_end = flat_spot_start + lander_width_points - 1;

    let mut flat_height = 0.0;

    // Generate terrain using standard Perlin noise WITH integrated flat spot
    for i in 0..num_points {
        let height = if i >= flat_spot_start && i <= flat_spot_end {
            // We're in the flat spot region
            if i == flat_spot_start {
                // First point of flat spot - generate the height using Perlin noise
                let mut height = 0.0;
                let mut amplitude = 1.0;
                let mut frequency = base_frequency;
                let mut max_amplitude = 0.0;

                for _ in 0..octaves {
                    height += perlin.get([i as f64 * frequency, 0.0]) * amplitude;
                    max_amplitude += amplitude;
                    amplitude *= persistence;
                    frequency *= 2.0;
                }
                height /= max_amplitude; // Normalize
                flat_height = height; // Store this height for the entire flat section
                height
            } else {
                // Rest of flat spot - use the same height as the first point
                flat_height
            }
        } else {
            // Normal terrain - generate Perlin noise
            let mut height = 0.0;
            let mut amplitude = 1.0;
            let mut frequency = base_frequency;
            let mut max_amplitude = 0.0;

            for _ in 0..octaves {
                height += perlin.get([i as f64 * frequency, 0.0]) * amplitude;
                max_amplitude += amplitude;
                amplitude *= persistence;
                frequency *= 2.0;
            }
            height /= max_amplitude; // Normalize
            height
        };
        
        terrain.push(height);
    }

    // Normalize and scale the terrain to the desired height range
    let terrain_min = terrain.iter().cloned().fold(f64::INFINITY, f64::min);
    let terrain_max = terrain.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    terrain.iter_mut().for_each(|h| {
        *h = (*h - terrain_min) / (terrain_max - terrain_min); // Normalize to [0, 1]
        *h = *h * (max_height - min_height) + min_height; // Scale to [min_height, max_height]
    });

    debug!(
        "Generated flat landing spot at positions {}-{} ({} points) at height {:.1}",
        flat_spot_start, flat_spot_end, lander_width_points, terrain[flat_spot_start]
    );

    (terrain, (flat_spot_start, flat_spot_end))
}
