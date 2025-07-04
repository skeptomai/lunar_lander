extern crate noise;

use noise::{NoiseFn, Perlin};
use plotters::prelude::*;
use rand::Rng;
use macroquad::logging::debug;

const WIDTH: u32 = 1000;
const HEIGHT: u32 = 400;

pub fn generate_terrain(num_points: usize, min_height: f64, max_height: f64, base_frequency: f64, octaves: u32, persistence: f64) -> Vec<f64> {
    let perlin = Perlin::new();
    let mut terrain = Vec::with_capacity(num_points);

    for i in 0..num_points {
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
        terrain.push(height);
    }

    // Normalize and scale the terrain to the desired height range
    let terrain_min = terrain.iter().cloned().fold(f64::INFINITY, f64::min);
    let terrain_max = terrain.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    terrain.iter_mut().for_each(|h| {
        *h = (*h - terrain_min) / (terrain_max - terrain_min); // Normalize to [0, 1]
        *h = *h * (max_height - min_height) + min_height;      // Scale to [min_height, max_height]
    });

    terrain
}

pub fn add_flat_spots(terrain: &mut Vec<f64>, min_length: usize, max_length: usize, num_spots: usize) -> Vec<(usize, usize)> {
    let mut rng = rand::thread_rng();
    let terrain_len = terrain.len();

    debug!("Creating {} flat spots with length {}-{} points", num_spots, min_length, max_length);

    // Track occupied positions to prevent overlaps
    let mut occupied_ranges: Vec<(usize, usize)> = Vec::new();
    let minimum_gap = max_length; // Ensure gaps between landing zones

    for i in 0..num_spots {
        let spot_length = rng.gen_range(min_length..=max_length);
        
        // Try to find a non-overlapping position (max 50 attempts)
        let mut attempts = 0;
        let mut start_pos;
        let mut valid_position = false;
        
        loop {
            start_pos = rng.gen_range(0..terrain_len - spot_length);
            let end_pos = start_pos + spot_length - 1;
            
            // Check if this position conflicts with existing flat spots
            let conflicts = occupied_ranges.iter().any(|(occupied_start, occupied_end)| {
                // Check for overlap or insufficient gap
                let too_close = start_pos <= occupied_end + minimum_gap && end_pos + minimum_gap >= *occupied_start;
                too_close
            });
            
            if !conflicts {
                valid_position = true;
                break;
            }
            
            attempts += 1;
            if attempts >= 50 {
                debug!("Warning: Could not find non-overlapping position for flat spot {} after 50 attempts", i + 1);
                break;
            }
        }
        
        if valid_position {
            // Calculate the average height of the section to flatten
            let avg_height: f64 = terrain[start_pos..start_pos + spot_length].iter().sum::<f64>() / spot_length as f64;

            debug!("Flat spot {}: positions {}-{} (length {}), height {:.1}", 
                     i + 1, start_pos, start_pos + spot_length - 1, spot_length, avg_height);

            for j in start_pos..start_pos + spot_length {
                terrain[j] = avg_height;
            }
            
            // Record this occupied range
            occupied_ranges.push((start_pos, start_pos + spot_length - 1));
        } else {
            debug!("Skipped flat spot {} due to overlapping constraints", i + 1);
        }
    }
    
    println!("Successfully created {} non-overlapping flat spots", occupied_ranges.len());
    
    // Return the flat spot ranges for direct reference
    occupied_ranges
}
