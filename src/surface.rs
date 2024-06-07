extern crate noise;

use noise::{NoiseFn, Perlin};
use plotters::prelude::*;
use rand::Rng;

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

pub fn add_flat_spots(terrain: &mut Vec<f64>, min_length: usize, max_length: usize, num_spots: usize) {
    let mut rng = rand::thread_rng();
    let terrain_len = terrain.len();

    for _ in 0..num_spots {
        let spot_length = rng.gen_range(min_length..=max_length);
        let start_pos = rng.gen_range(0..terrain_len - spot_length);

        // Calculate the average height of the section to flatten
        let avg_height: f64 = terrain[start_pos..start_pos + spot_length].iter().sum::<f64>() / spot_length as f64;

        for i in start_pos..start_pos + spot_length {
            terrain[i] = avg_height;
        }
    }
}

pub fn plot_terrain(terrain: &[f64]) {
    let root = BitMapBackend::new("terrain.png", (WIDTH, HEIGHT)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    let max_height = terrain.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let min_height = terrain.iter().cloned().fold(f64::INFINITY, f64::min);

    let mut chart = ChartBuilder::on(&root)
        .caption("Generated Terrain", ("sans-serif", 50).into_font())
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0..terrain.len(), min_height..max_height)
        .unwrap();

    chart.configure_mesh().draw().unwrap();

    chart
        .draw_series(LineSeries::new(
            terrain.iter().enumerate().map(|(x, y)| (x, *y)),
            &RED,
        ))
        .unwrap();

    root.present().unwrap();
}


