extern crate noise;

use noise::{NoiseFn, Perlin};
use plotters::prelude::*;
use rand::Rng;
use rand::seq::SliceRandom;
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

    debug!("Creating {} flat spots with length {}-{} points using zone-based placement", num_spots, min_length, max_length);

    // Zone-based approach: Divide terrain into non-overlapping zones
    let max_zones = 6; // Total available zones across the terrain
    let zone_size = terrain_len / max_zones; // Each zone gets equal space
    let buffer_size = max_length; // Buffer between zones to prevent overlap
    
    debug!("Zone-based placement: {} zones of {} points each, {} point buffer", max_zones, zone_size, buffer_size);
    
    // Create available zones (each zone can fit a landing spot + buffer)
    let mut available_zones: Vec<usize> = (0..max_zones).collect();
    available_zones.shuffle(&mut rng);
    
    // Guarantee at least 2 zones, up to requested number
    let zones_to_use = std::cmp::max(2, std::cmp::min(num_spots, available_zones.len()));
    let selected_zones = &available_zones[0..zones_to_use];
    
    debug!("Selected {} zones for landing spots: {:?}", zones_to_use, selected_zones);
    
    let mut occupied_ranges: Vec<(usize, usize)> = Vec::new();
    
    for (i, &zone_id) in selected_zones.iter().enumerate() {
        // Calculate zone boundaries with buffer
        let zone_start = zone_id * zone_size;
        let zone_end = std::cmp::min((zone_id + 1) * zone_size - buffer_size, terrain_len);
        
        // Ensure zone is large enough for a landing spot
        let usable_zone_size = zone_end.saturating_sub(zone_start);
        if usable_zone_size < min_length {
            debug!("Zone {} too small ({} points), skipping", zone_id, usable_zone_size);
            continue;
        }
        
        // Random spot size within zone constraints
        let max_spot_in_zone = std::cmp::min(max_length, usable_zone_size);
        let spot_length = rng.gen_range(min_length..=max_spot_in_zone);
        
        // Random position within the zone
        let max_start_pos = zone_end - spot_length;
        let start_pos = if max_start_pos > zone_start {
            rng.gen_range(zone_start..=max_start_pos)
        } else {
            zone_start
        };
        let end_pos = start_pos + spot_length - 1;
        
        // Calculate the average height of the section to flatten
        let avg_height: f64 = terrain[start_pos..=end_pos].iter().sum::<f64>() / spot_length as f64;

        debug!("Flat spot {}: zone {} positions {}-{} (length {}), height {:.1}", 
                 i + 1, zone_id, start_pos, end_pos, spot_length, avg_height);

        // Flatten the terrain
        for j in start_pos..=end_pos {
            terrain[j] = avg_height;
        }
        
        // Record this occupied range
        occupied_ranges.push((start_pos, end_pos));
    }
    
    println!("Successfully created {} guaranteed non-overlapping flat spots using zone-based placement", occupied_ranges.len());
    
    // Verify we got at least 2 (this should always be true with zone-based approach)
    if occupied_ranges.len() < 2 {
        panic!("CRITICAL ERROR: Zone-based placement failed to create minimum 2 landing zones! Got {}", occupied_ranges.len());
    }
    
    occupied_ranges
}
