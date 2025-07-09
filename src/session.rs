//! Game session management for multi-attempt lunar lander missions.
//!
//! This module handles:
//! - 3-attempt game sessions with attempt tracking
//! - Comprehensive scoring system based on zone difficulty and fuel efficiency
//! - Session state management and progression
//! - Performance analysis and session summaries

use macroquad::logging::info;
use crate::surface::LandingZoneDifficulty;

/// Represents the result of a single landing attempt
#[derive(Debug, Clone, PartialEq)]
pub enum AttemptResult {
    Success,
    Failure,
    InProgress,
}

/// Details of a single landing attempt within a game session
#[derive(Debug, Clone)]
pub struct LandingAttempt {
    pub result: AttemptResult,
    pub score: f32,
    pub fuel_remaining: f32,  // Percentage (0-100)
    pub landing_zone: Option<LandingZoneDifficulty>,
    pub time_taken: f32,      // Time in seconds
}

impl LandingAttempt {
    /// Creates a new attempt in progress state
    pub fn new_in_progress() -> Self {
        Self {
            result: AttemptResult::InProgress,
            score: 0.0,
            fuel_remaining: 0.0,
            landing_zone: None,
            time_taken: 0.0,
        }
    }

    /// Creates a completed attempt with calculated score
    pub fn new_completed(
        result: AttemptResult,
        zone_difficulty: Option<LandingZoneDifficulty>,
        fuel_remaining_percent: f32,
        time_taken: f32,
    ) -> Self {
        let score = if result == AttemptResult::Success && zone_difficulty.is_some() {
            Self::calculate_score(zone_difficulty.unwrap(), fuel_remaining_percent, time_taken)
        } else {
            0.0
        };

        Self {
            result,
            score,
            fuel_remaining: fuel_remaining_percent,
            landing_zone: zone_difficulty,
            time_taken,
        }
    }

    /// Calculates score based on zone difficulty, fuel efficiency, and time
    ///
    /// # Scoring Formula
    /// 
    /// `Score = Base_Points × Zone_Multiplier × Fuel_Bonus × Time_Bonus`
    ///
    /// - **Base Points**: 1000
    /// - **Zone Multiplier**: 2.0 (Hard), 1.6 (Medium), 1.3 (Easy)
    /// - **Fuel Bonus**: 1.0 + (fuel_remaining / 100) - rewards fuel conservation
    /// - **Time Bonus**: 1.2 if completed under 60 seconds, 1.0 otherwise
    ///
    /// # Examples
    /// - Hard zone + 50% fuel + fast = 1000 × 2.0 × 1.5 × 1.2 = 3600 points
    /// - Easy zone + 80% fuel + slow = 1000 × 1.3 × 1.8 × 1.0 = 2340 points
    pub fn calculate_score(
        zone_difficulty: LandingZoneDifficulty,
        fuel_remaining_percent: f32,
        time_taken: f32,
    ) -> f32 {
        let base_points = 1000.0;
        
        // Zone difficulty multiplier (from existing scoring system)
        let zone_multiplier = zone_difficulty.score(); // 2.0, 1.6, 1.3
        
        // Fuel efficiency bonus: 1.0 to 2.0 based on fuel remaining
        // More fuel remaining = higher bonus
        let fuel_bonus = 1.0 + (fuel_remaining_percent / 100.0);
        
        // Time bonus: reward fast completion (under 60 seconds)
        let time_bonus = if time_taken < 60.0 { 1.2 } else { 1.0 };
        
        base_points * zone_multiplier * fuel_bonus * time_bonus
    }
}

/// Represents a complete game session of 3 landing attempts
#[derive(Debug, Clone)]
pub struct GameSession {
    pub current_attempt: usize,        // 0, 1, or 2
    pub max_attempts: usize,           // Always 3
    pub attempts: Vec<LandingAttempt>, // Results of each attempt
    pub total_score: f32,              // Cumulative score across all attempts
    pub session_complete: bool,        // True after all attempts finished
}

impl GameSession {
    /// Creates a new game session with 3 empty attempts
    pub fn new() -> Self {
        Self {
            current_attempt: 0,
            max_attempts: 3,
            attempts: vec![
                LandingAttempt::new_in_progress(),
                LandingAttempt::new_in_progress(),
                LandingAttempt::new_in_progress(),
            ],
            total_score: 0.0,
            session_complete: false,
        }
    }

    /// Gets the number of successful landings in this session
    pub fn success_count(&self) -> usize {
        self.attempts.iter()
            .filter(|attempt| attempt.result == AttemptResult::Success)
            .count()
    }

    /// Gets the number of failed landings in this session
    pub fn failure_count(&self) -> usize {
        self.attempts.iter()
            .filter(|attempt| attempt.result == AttemptResult::Failure)
            .count()
    }

    /// Gets the highest scoring attempt in this session
    pub fn best_attempt(&self) -> Option<&LandingAttempt> {
        self.attempts.iter()
            .filter(|attempt| attempt.result == AttemptResult::Success)
            .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap())
    }

    /// Calculates average fuel efficiency across successful attempts
    pub fn average_fuel_efficiency(&self) -> f32 {
        let successful_attempts: Vec<_> = self.attempts.iter()
            .filter(|attempt| attempt.result == AttemptResult::Success)
            .collect();

        if successful_attempts.is_empty() {
            0.0
        } else {
            successful_attempts.iter()
                .map(|attempt| attempt.fuel_remaining)
                .sum::<f32>() / successful_attempts.len() as f32
        }
    }

    /// Gets a performance rating based on session results
    pub fn performance_rating(&self) -> &'static str {
        let success_count = self.success_count();
        let avg_fuel = self.average_fuel_efficiency();

        match (success_count, avg_fuel) {
            (3, fuel) if fuel >= 70.0 => "ACE PILOT",
            (3, fuel) if fuel >= 50.0 => "EXPERT",
            (3, _) => "SKILLED",
            (2, fuel) if fuel >= 60.0 => "COMPETENT",
            (2, _) => "ADEQUATE",
            (1, _) => "NOVICE",
            (0, _) => "NEEDS PRACTICE",
            _ => "UNKNOWN"
        }
    }
}

/// Manages game session state and progression
pub struct SessionManager {
    pub session: GameSession,
}

impl SessionManager {
    /// Creates a new session manager with a fresh session
    pub fn new() -> Self {
        Self {
            session: GameSession::new(),
        }
    }

    /// Completes the current attempt and updates session state
    ///
    /// # Arguments
    /// * `result` - Whether the attempt succeeded or failed
    /// * `fuel_remaining` - Fuel remaining percentage (0-100)
    /// * `zone` - Landing zone difficulty if successful
    /// * `time` - Time taken for the attempt in seconds
    pub fn complete_attempt(
        &mut self,
        result: AttemptResult,
        fuel_remaining: f32,
        zone: Option<LandingZoneDifficulty>,
        time: f32,
    ) {
        // Create completed attempt with calculated score
        let attempt = LandingAttempt::new_completed(result, zone, fuel_remaining, time);
        
        // Update session state
        self.session.attempts[self.session.current_attempt] = attempt.clone();
        self.session.total_score += attempt.score;
        self.session.current_attempt += 1;
        
        // Check if session is complete
        if self.session.current_attempt >= self.session.max_attempts {
            self.session.session_complete = true;
        }

        // Debug output for attempt completion
        info!(
            "Attempt {} completed: {:?}, Score: {:.0}, Total: {:.0}",
            self.session.current_attempt,
            attempt.result,
            attempt.score,
            self.session.total_score
        );
    }

    /// Checks if there are more attempts available in this session
    pub fn can_start_next_attempt(&self) -> bool {
        !self.session.session_complete && self.session.current_attempt < self.session.max_attempts
    }

    /// Resets to a new session (start over with 3 fresh attempts)
    pub fn reset_session(&mut self) {
        self.session = GameSession::new();
        info!("New game session started");
    }

    /// Gets the current attempt number for display (1-based)
    pub fn current_attempt_display(&self) -> usize {
        (self.session.current_attempt + 1).min(self.session.max_attempts)
    }

    /// Checks if the current attempt is the last one
    pub fn is_final_attempt(&self) -> bool {
        self.session.current_attempt >= self.session.max_attempts - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scoring_calculation() {
        // Test hard zone with high fuel efficiency
        let score = LandingAttempt::calculate_score(
            LandingZoneDifficulty::Hard,
            80.0, // 80% fuel remaining
            45.0  // Under 60 seconds
        );
        // Expected: 1000 * 2.0 * 1.8 * 1.2 = 4320
        assert_eq!(score, 4320.0);

        // Test easy zone with low fuel efficiency
        let score = LandingAttempt::calculate_score(
            LandingZoneDifficulty::Easy,
            20.0, // 20% fuel remaining
            75.0  // Over 60 seconds
        );
        // Expected: 1000 * 1.333... * 1.2 * 1.0 ≈ 1600
        assert!((score - 1600.0).abs() < 10.0);
    }

    #[test]
    fn test_session_progression() {
        let mut manager = SessionManager::new();
        
        // Start with fresh session
        assert_eq!(manager.session.current_attempt, 0);
        assert!(!manager.session.session_complete);
        
        // Complete first attempt (success)
        manager.complete_attempt(
            AttemptResult::Success,
            70.0,
            Some(LandingZoneDifficulty::Hard),
            50.0
        );
        assert_eq!(manager.session.current_attempt, 1);
        assert!(!manager.session.session_complete);
        assert!(manager.session.total_score > 0.0);
        
        // Complete remaining attempts
        manager.complete_attempt(AttemptResult::Failure, 30.0, None, 120.0);
        manager.complete_attempt(
            AttemptResult::Success,
            90.0,
            Some(LandingZoneDifficulty::Medium),
            40.0
        );
        
        // Session should be complete
        assert!(manager.session.session_complete);
        assert_eq!(manager.session.success_count(), 2);
        assert_eq!(manager.session.failure_count(), 1);
    }

    #[test]
    fn test_performance_rating() {
        let mut session = GameSession::new();
        
        // Perfect session: 3 successes with high fuel efficiency
        session.attempts = vec![
            LandingAttempt::new_completed(
                AttemptResult::Success,
                Some(LandingZoneDifficulty::Hard),
                80.0,
                45.0
            ),
            LandingAttempt::new_completed(
                AttemptResult::Success,
                Some(LandingZoneDifficulty::Medium),
                75.0,
                55.0
            ),
            LandingAttempt::new_completed(
                AttemptResult::Success,
                Some(LandingZoneDifficulty::Easy),
                85.0,
                40.0
            ),
        ];
        
        assert_eq!(session.performance_rating(), "ACE PILOT");
        assert_eq!(session.success_count(), 3);
        assert!((session.average_fuel_efficiency() - 80.0).abs() < 1.0);
    }
}