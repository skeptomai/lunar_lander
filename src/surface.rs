//! Procedural terrain generation with multiple landing zones.
//!
//! This module generates realistic lunar terrain using Perlin noise with integrated
//! flat landing zones of varying difficulty levels. The terrain generation ensures
//! proper spacing between zones and provides both legacy single-zone compatibility
//! and modern multi-zone functionality.

extern crate noise;

use macroquad::logging::debug;
use noise::{NoiseFn, Perlin};
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LandingZoneDifficulty {
    Hard,   // 1.0x lander width
    Medium, // 1.25x lander width  
    Easy,   // 1.5x lander width
}

impl LandingZoneDifficulty {
    /// Returns the width multiplier for this difficulty level.
    ///
    /// # Returns
    ///
    /// * `1.0` - Hard difficulty (exact lander width)
    /// * `1.25` - Medium difficulty (1.25x lander width)
    /// * `1.5` - Easy difficulty (1.5x lander width)
    pub fn width_multiplier(&self) -> f32 {
        match self {
            LandingZoneDifficulty::Hard => 1.0,
            LandingZoneDifficulty::Medium => 1.25,
            LandingZoneDifficulty::Easy => 1.5,
        }
    }
    
    /// Returns the human-readable name of this difficulty level.
    ///
    /// # Returns
    ///
    /// * `"Hard"` - Most challenging landing zones
    /// * `"Medium"` - Moderate difficulty landing zones
    /// * `"Easy"` - Most forgiving landing zones
    pub fn name(&self) -> &str {
        match self {
            LandingZoneDifficulty::Hard => "Hard",
            LandingZoneDifficulty::Medium => "Medium", 
            LandingZoneDifficulty::Easy => "Easy",
        }
    }

    /// Calculates the score for successfully landing in this zone.
    ///
    /// Score formula: 2 / width_multiplier
    /// - Hard zone (1.0x): 2/1.0 = 2.0 points
    /// - Medium zone (1.25x): 2/1.25 = 1.6 points
    /// - Easy zone (1.5x): 2/1.5 = 1.3 points
    ///
    /// # Returns
    ///
    /// The score value for landing in this zone
    pub fn score(&self) -> f32 {
        2.0 / self.width_multiplier()
    }
}

#[derive(Debug, Clone)]
pub struct LandingZone {
    pub start: usize,
    pub end: usize,
    pub difficulty: LandingZoneDifficulty,
    pub width_points: usize,
}

/// Generates procedural terrain with 1-3 randomly placed landing zones of varying difficulty.
///
/// This function creates realistic lunar terrain using multi-octave Perlin noise while
/// integrating flat landing zones. The zones are strategically placed to avoid overlaps
/// and provide a good gameplay experience.
///
/// # Arguments
///
/// * `num_points` - Number of terrain points to generate
/// * `min_height` - Minimum terrain height
/// * `max_height` - Maximum terrain height
/// * `base_frequency` - Base frequency for Perlin noise
/// * `octaves` - Number of noise octaves for terrain detail
/// * `persistence` - Persistence factor for noise octaves
/// * `lander_width_points` - Width of the lander in terrain points
///
/// # Returns
///
/// A tuple containing:
/// * `Vec<f64>` - Generated terrain heights
/// * `Vec<LandingZone>` - Landing zones with positions and difficulty levels
///
/// # Zone Generation
///
/// - Generates 1-3 landing zones randomly
/// - Each zone has a random difficulty (Hard/Medium/Easy)
/// - Zones are spaced at least 150 points apart
/// - Fallback to single zone if spacing constraints can't be met
///
/// # Example
///
/// ```rust
/// let (terrain, zones) = generate_terrain_with_multiple_landing_zones(
///     1000, 0.0, 100.0, 0.01, 6, 0.5, 36
/// );
/// println!("Generated {} zones", zones.len());
/// ```
pub fn generate_terrain_with_multiple_landing_zones(
    num_points: usize,
    min_height: f64,
    max_height: f64,
    base_frequency: f64,
    octaves: u32,
    persistence: f64,
    lander_width_points: usize,
) -> (Vec<f64>, Vec<LandingZone>) {
    let perlin = Perlin::new();
    let mut terrain = Vec::with_capacity(num_points);
    let mut rng = rand::thread_rng();

    // Create 1-3 landing zones with different difficulties
    let num_zones = rng.gen_range(1..=3);
    let mut landing_zones = Vec::new();
    
    // Define difficulty levels to choose from
    let difficulties = [
        LandingZoneDifficulty::Hard,
        LandingZoneDifficulty::Medium,
        LandingZoneDifficulty::Easy,
    ];
    
    // Calculate spacing to ensure zones don't overlap
    let zone_spacing = 150; // Minimum points between zones
    let max_zone_width = (lander_width_points as f32 * 1.5) as usize; // Largest possible zone
    let available_space = num_points - 200; // Leave margins on both sides
    let total_space_needed = num_zones * (max_zone_width + zone_spacing);
    
    if total_space_needed > available_space {
        // Fallback to single zone if we can't fit multiple zones
        let difficulty = difficulties[rng.gen_range(0..difficulties.len())];
        let width_points = (lander_width_points as f32 * difficulty.width_multiplier()) as usize;
        let start = rng.gen_range(100..(num_points - width_points - 100));
        let end = start + width_points - 1;
        
        landing_zones.push(LandingZone {
            start,
            end,
            difficulty,
            width_points,
        });
    } else {
        // Generate multiple non-overlapping zones
        let mut positions = Vec::new();
        
        for _i in 0..num_zones {
            let difficulty = difficulties[rng.gen_range(0..difficulties.len())];
            let width_points = (lander_width_points as f32 * difficulty.width_multiplier()) as usize;
            
            // Find a position that doesn't overlap with existing zones
            let mut attempts = 0;
            loop {
                let start = rng.gen_range(100..(num_points - width_points - 100));
                let end = start + width_points - 1;
                
                // Check if this overlaps with any existing zone
                let overlaps = positions.iter().any(|(existing_start, existing_end)| {
                    !(end + zone_spacing < *existing_start || start > *existing_end + zone_spacing)
                });
                
                if !overlaps || attempts > 50 {
                    positions.push((start, end));
                    landing_zones.push(LandingZone {
                        start,
                        end,
                        difficulty,
                        width_points,
                    });
                    break;
                }
                attempts += 1;
            }
        }
    }
    
    // Sort zones by position for easier processing
    landing_zones.sort_by_key(|zone| zone.start);
    
    debug!("Generated {} landing zones:", landing_zones.len());
    for (i, zone) in landing_zones.iter().enumerate() {
        debug!("  Zone {}: {} difficulty, positions {}-{} ({} points)", 
               i + 1, zone.difficulty.name(), zone.start, zone.end, zone.width_points);
    }

    // Generate terrain using Perlin noise with integrated flat spots
    let mut zone_heights = Vec::new();
    
    for i in 0..num_points {
        let height = {
            // Check if this point is in any landing zone
            let mut in_zone = None;
            for (zone_idx, zone) in landing_zones.iter().enumerate() {
                if i >= zone.start && i <= zone.end {
                    in_zone = Some(zone_idx);
                    break;
                }
            }
            
            if let Some(zone_idx) = in_zone {
                let zone = &landing_zones[zone_idx];
                if i == zone.start {
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
                    
                    // Store this height for the entire flat section
                    if zone_heights.len() <= zone_idx {
                        zone_heights.resize(zone_idx + 1, 0.0);
                    }
                    zone_heights[zone_idx] = height;
                    height
                } else {
                    // Rest of flat spot - use the same height as the first point
                    zone_heights[zone_idx]
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
            }
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

    (terrain, landing_zones)
}

/// Legacy function for backward compatibility with single flat spot generation.
///
/// This function wraps the modern multi-zone generation and returns only the first
/// landing zone in the legacy format. Use `generate_terrain_with_multiple_landing_zones`
/// for full multi-zone functionality.
///
/// # Arguments
///
/// * `num_points` - Number of terrain points to generate
/// * `min_height` - Minimum terrain height
/// * `max_height` - Maximum terrain height
/// * `base_frequency` - Base frequency for Perlin noise
/// * `octaves` - Number of noise octaves for terrain detail
/// * `persistence` - Persistence factor for noise octaves
/// * `lander_width_points` - Width of the lander in terrain points
///
/// # Returns
///
/// A tuple containing:
/// * `Vec<f64>` - Generated terrain heights
/// * `(usize, usize)` - Flat spot range (start_index, end_index)
pub fn generate_terrain_with_flat_spot(
    num_points: usize,
    min_height: f64,
    max_height: f64,
    base_frequency: f64,
    octaves: u32,
    persistence: f64,
    lander_width_points: usize,
) -> (Vec<f64>, (usize, usize)) {
    // Use the new multiple landing zones function and convert to legacy format
    let (terrain, landing_zones) = generate_terrain_with_multiple_landing_zones(
        num_points,
        min_height,
        max_height,
        base_frequency,
        octaves,
        persistence,
        lander_width_points,
    );
    
    // Return the first (or only) landing zone for backward compatibility
    let flat_spot_range = if !landing_zones.is_empty() {
        (landing_zones[0].start, landing_zones[0].end)
    } else {
        (0, 0) // Fallback, shouldn't happen
    };
    
    (terrain, flat_spot_range)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scoring_system() {
        // Test scoring formula: 2 / width_multiplier

        // Hard zone (1.0x width): 2/1.0 = 2.0
        let hard = LandingZoneDifficulty::Hard;
        assert_eq!(hard.score(), 2.0);

        // Medium zone (1.25x width): 2/1.25 = 1.6
        let medium = LandingZoneDifficulty::Medium;
        assert_eq!(medium.score(), 1.6);

        // Easy zone (1.5x width): 2/1.5 = 1.333...
        let easy = LandingZoneDifficulty::Easy;
        assert!((easy.score() - 1.333333).abs() < 0.001); // Float comparison with tolerance
    }
}
