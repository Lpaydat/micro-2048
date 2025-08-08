//! Competition domain service for managing competition lifecycle and business rules.

use crate::core::models::*;
use crate::core::value_objects::*;
use linera_sdk::linera_base_types::{ChainId, Timestamp};
use std::collections::HashMap;

/// Domain service for competition management
pub struct CompetitionService;

impl CompetitionService {
    /// Create a new competition with validation
    pub fn create_competition(
        competition_id: CompetitionId,
        title: String,
        organizer_id: ParticipantId,
        format: CompetitionFormat,
        start_time: Timestamp,
        end_time: Timestamp,
        coordinator_chain: ChainId,
        leaderboard_chain: ChainId,
        current_time: Timestamp,
    ) -> Result<Competition, CompetitionServiceError> {
        // Validate organizer permissions (would check against admin roles in real implementation)
        Self::validate_organizer_permissions(organizer_id)?;

        // Validate time constraints
        if start_time <= current_time {
            return Err(CompetitionServiceError::InvalidStartTime);
        }

        // Create competition
        let competition = Competition::new(
            competition_id,
            title,
            organizer_id,
            format,
            start_time,
            end_time,
            coordinator_chain,
            leaderboard_chain,
        ).map_err(CompetitionServiceError::CompetitionError)?;

        Ok(competition)
    }

    /// Register a participant for a competition
    pub fn register_participant(
        competition: &Competition,
        participant: &Participant,
        current_time: Timestamp,
    ) -> Result<(), CompetitionServiceError> {
        // Check if competition allows registration
        if !competition.is_registration_open(current_time) {
            return Err(CompetitionServiceError::RegistrationClosed);
        }

        // Check participant eligibility
        competition.can_participant_join(participant, current_time)
            .map_err(CompetitionServiceError::CompetitionError)?;

        Ok(())
    }

    /// Start a competition
    pub fn start_competition(
        competition: &mut Competition,
        current_time: Timestamp,
    ) -> Result<(), CompetitionServiceError> {
        if current_time < competition.competition_lifecycle.competition_starts_at {
            return Err(CompetitionServiceError::CompetitionNotReady);
        }

        competition.update_phase(current_time);
        
        if !competition.is_in_progress(current_time) {
            return Err(CompetitionServiceError::CompetitionNotReady);
        }

        Ok(())
    }

    /// Process a game result for a competition
    pub fn process_game_result(
        competition: &Competition,
        participant_id: ParticipantId,
        session: &GameSession,
        current_time: Timestamp,
    ) -> Result<CompetitionGameResult, CompetitionServiceError> {
        // Verify competition is in progress
        if !competition.is_in_progress(current_time) {
            return Err(CompetitionServiceError::CompetitionNotActive);
        }

        // Verify session belongs to this competition
        if let Some(ref context) = session.competition_context {
            if context.competition_id != competition.competition_id {
                return Err(CompetitionServiceError::InvalidGameSession);
            }
        } else {
            return Err(CompetitionServiceError::InvalidGameSession);
        }

        // Calculate final score
        let final_score = crate::core::services::GameSessionService::calculate_final_score(session);

        // Apply competition-specific scoring rules
        let adjusted_score = Self::apply_competition_scoring_rules(competition, final_score, session);

        // Create result
        let result = CompetitionGameResult {
            competition_id: competition.competition_id,
            participant_id,
            session_id: session.session_id,
            final_score: adjusted_score,
            completion_time: session.session_lifecycle.concluded_at,
            performance_metrics: Self::calculate_performance_metrics(session),
        };

        Ok(result)
    }

    /// Calculate leaderboard rankings for a competition
    pub fn calculate_leaderboard(
        competition: &Competition,
        participant_scores: &HashMap<ParticipantId, Vec<CompetitionGameResult>>,
    ) -> Result<Vec<CompetitionRanking>, CompetitionServiceError> {
        let mut rankings = Vec::new();

        // Calculate aggregate scores for each participant
        for (participant_id, results) in participant_scores {
            let aggregate_score = Self::calculate_aggregate_score(competition, results);
            let best_completion_time = results.iter()
                .filter_map(|r| r.completion_time)
                .min();

            rankings.push(CompetitionRanking {
                rank: 0, // Will be set after sorting
                participant_id: *participant_id,
                final_score: aggregate_score,
                games_played: results.len() as u32,
                completion_time: best_completion_time,
                rewards_earned: Vec::new(), // Will be calculated after ranking
            });
        }

        // Sort by score (descending) and completion time (ascending for ties)
        rankings.sort_by(|a, b| {
            match b.final_score.cmp(&a.final_score) {
                std::cmp::Ordering::Equal => {
                    // If scores are equal, sort by completion time
                    match (a.completion_time, b.completion_time) {
                        (Some(a_time), Some(b_time)) => a_time.cmp(&b_time),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => std::cmp::Ordering::Equal,
                    }
                }
                other => other,
            }
        });

        // Assign ranks
        for (index, ranking) in rankings.iter_mut().enumerate() {
            ranking.rank = (index + 1) as u32;
        }

        // Calculate rewards
        Self::assign_rewards(competition, &mut rankings);

        Ok(rankings)
    }

    /// Finalize a competition
    pub fn finalize_competition(
        competition: &mut Competition,
        final_rankings: Vec<CompetitionRanking>,
        current_time: Timestamp,
    ) -> Result<CompetitionResults, CompetitionServiceError> {
        // Ensure competition is concluded
        if !competition.is_concluded(current_time) {
            return Err(CompetitionServiceError::CompetitionNotConcluded);
        }

        // Finalize results
        competition.finalize_results(current_time)
            .map_err(CompetitionServiceError::CompetitionError)?;

        // Calculate statistics
        let total_participants = final_rankings.len() as u32;
        let completed_participants = final_rankings.iter()
            .filter(|r| r.completion_time.is_some())
            .count() as u32;
        let completion_rate = if total_participants > 0 {
            completed_participants as f64 / total_participants as f64
        } else {
            0.0
        };

        let average_score = if !final_rankings.is_empty() {
            final_rankings.iter().map(|r| r.final_score).sum::<u64>() as f64 / final_rankings.len() as f64
        } else {
            0.0
        };

        let highest_score = final_rankings.iter()
            .map(|r| r.final_score)
            .max()
            .unwrap_or(0);

        let results = CompetitionResults {
            competition_id: competition.competition_id,
            final_rankings,
            total_participants,
            completion_rate,
            average_score,
            highest_score,
            finalized_at: current_time,
        };

        Ok(results)
    }

    /// Cancel a competition
    pub fn cancel_competition(
        competition: &mut Competition,
        current_time: Timestamp,
    ) -> Result<(), CompetitionServiceError> {
        competition.cancel(current_time)
            .map_err(CompetitionServiceError::CompetitionError)
    }

    /// Validate organizer permissions
    fn validate_organizer_permissions(organizer_id: ParticipantId) -> Result<(), CompetitionServiceError> {
        // In a real implementation, this would check against admin roles
        // For now, we'll assume all participants can organize competitions
        Ok(())
    }

    /// Apply competition-specific scoring rules
    fn apply_competition_scoring_rules(
        competition: &Competition,
        base_score: u64,
        session: &GameSession,
    ) -> u64 {
        match &competition.competition_format {
            CompetitionFormat::TimeBasedLeaderboard { .. } => {
                // Standard scoring for leaderboard competitions
                base_score
            }
            CompetitionFormat::SingleElimination { .. } => {
                // Bonus for elimination tournaments
                (base_score as f64 * 1.2) as u64
            }
            CompetitionFormat::EliminationSurvival { .. } => {
                // Survival bonus based on how long the player lasted
                let survival_bonus = session.board_state.move_count as u64 * 10;
                base_score + survival_bonus
            }
            CompetitionFormat::RoundRobin { .. } => {
                // Consistency bonus for round robin
                (base_score as f64 * 1.1) as u64
            }
            CompetitionFormat::TeamBased { .. } => {
                // Team collaboration bonus
                (base_score as f64 * 1.15) as u64
            }
        }
    }

    /// Calculate aggregate score based on competition format
    fn calculate_aggregate_score(
        competition: &Competition,
        results: &[CompetitionGameResult],
    ) -> u64 {
        if results.is_empty() {
            return 0;
        }

        match &competition.competition_format {
            CompetitionFormat::TimeBasedLeaderboard { .. } => {
                // Best single score
                results.iter().map(|r| r.final_score).max().unwrap_or(0)
            }
            CompetitionFormat::SingleElimination { .. } => {
                // Latest score (final elimination result)
                results.last().map(|r| r.final_score).unwrap_or(0)
            }
            CompetitionFormat::RoundRobin { .. } => {
                // Average of all scores
                (results.iter().map(|r| r.final_score).sum::<u64>() / results.len() as u64)
            }
            CompetitionFormat::EliminationSurvival { .. } => {
                // Sum of all survival scores
                results.iter().map(|r| r.final_score).sum()
            }
            CompetitionFormat::TeamBased { .. } => {
                // Team average (would need team logic)
                results.iter().map(|r| r.final_score).max().unwrap_or(0)
            }
        }
    }

    /// Calculate performance metrics for a game session
    fn calculate_performance_metrics(session: &GameSession) -> PerformanceMetrics {
        let play_time = session.get_play_time(session.session_lifecycle.concluded_at);
        let moves_per_second = if play_time > 0 {
            session.board_state.move_count as f64 / (play_time as f64 / 1_000_000.0)
        } else {
            0.0
        };

        let efficiency = if session.board_state.move_count > 0 {
            session.scoring_metrics.primary_score as f64 / session.board_state.move_count as f64
        } else {
            0.0
        };

        PerformanceMetrics {
            moves_per_second,
            score_per_move: efficiency,
            total_play_time: play_time,
            highest_tile_achieved: session.board_state.highest_tile_achieved,
        }
    }

    /// Assign rewards based on competition rankings
    fn assign_rewards(competition: &Competition, rankings: &mut [CompetitionRanking]) {
        for ranking in rankings.iter_mut() {
            // Find applicable reward tiers
            for tier in &competition.reward_structure.distribution {
                if ranking.rank >= tier.position_range.0 && ranking.rank <= tier.position_range.1 {
                    ranking.rewards_earned.push(tier.clone());
                }
            }
        }
    }
}

/// Result of processing a game within a competition
#[derive(Debug, Clone)]
pub struct CompetitionGameResult {
    pub competition_id: CompetitionId,
    pub participant_id: ParticipantId,
    pub session_id: GameSessionId,
    pub final_score: u64,
    pub completion_time: Option<Timestamp>,
    pub performance_metrics: PerformanceMetrics,
}

/// Performance metrics for a competition game
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub moves_per_second: f64,
    pub score_per_move: f64,
    pub total_play_time: u64,
    pub highest_tile_achieved: u32,
}

/// Errors that can occur in competition service operations
#[derive(Debug, Clone)]
pub enum CompetitionServiceError {
    CompetitionError(CompetitionError),
    InvalidStartTime,
    RegistrationClosed,
    CompetitionNotReady,
    CompetitionNotActive,
    CompetitionNotConcluded,
    InvalidGameSession,
    InsufficientPermissions,
    UnexpectedError(String),
}

impl std::fmt::Display for CompetitionServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompetitionServiceError::CompetitionError(err) => write!(f, "Competition error: {}", err),
            CompetitionServiceError::InvalidStartTime => write!(f, "Competition start time must be in the future"),
            CompetitionServiceError::RegistrationClosed => write!(f, "Registration is closed for this competition"),
            CompetitionServiceError::CompetitionNotReady => write!(f, "Competition is not ready to start"),
            CompetitionServiceError::CompetitionNotActive => write!(f, "Competition is not currently active"),
            CompetitionServiceError::CompetitionNotConcluded => write!(f, "Competition has not concluded yet"),
            CompetitionServiceError::InvalidGameSession => write!(f, "Game session is not valid for this competition"),
            CompetitionServiceError::InsufficientPermissions => write!(f, "Insufficient permissions for this operation"),
            CompetitionServiceError::UnexpectedError(msg) => write!(f, "Unexpected error: {}", msg),
        }
    }
}

impl std::error::Error for CompetitionServiceError {}

#[cfg(test)]
mod tests {
    use super::*;
    use linera_sdk::linera_base_types::{ChainId, Timestamp};

    #[test]
    fn test_create_competition() {
        let competition_id = CompetitionId(1);
        let organizer_id = ParticipantId(100);
        let start_time = Timestamp::from(2000000);
        let end_time = Timestamp::from(5000000);
        let current_time = Timestamp::from(1000000);
        let coordinator_chain = ChainId::root(0);
        let leaderboard_chain = ChainId::root(1);

        let result = CompetitionService::create_competition(
            competition_id,
            "Test Tournament".to_string(),
            organizer_id,
            CompetitionFormat::TimeBasedLeaderboard {
                duration: 3000000,
                ranking_criteria: RankingCriteria::Score,
            },
            start_time,
            end_time,
            coordinator_chain,
            leaderboard_chain,
            current_time,
        );

        assert!(result.is_ok());
        let competition = result.unwrap();
        assert_eq!(competition.competition_id, competition_id);
        assert_eq!(competition.competition_metadata.organizer_id, organizer_id);
    }

    #[test]
    fn test_invalid_start_time() {
        let competition_id = CompetitionId(1);
        let organizer_id = ParticipantId(100);
        let start_time = Timestamp::from(500000); // In the past
        let end_time = Timestamp::from(5000000);
        let current_time = Timestamp::from(1000000);
        let coordinator_chain = ChainId::root(0);
        let leaderboard_chain = ChainId::root(1);

        let result = CompetitionService::create_competition(
            competition_id,
            "Test Tournament".to_string(),
            organizer_id,
            CompetitionFormat::TimeBasedLeaderboard {
                duration: 3000000,
                ranking_criteria: RankingCriteria::Score,
            },
            start_time,
            end_time,
            coordinator_chain,
            leaderboard_chain,
            current_time,
        );

        assert!(matches!(result, Err(CompetitionServiceError::InvalidStartTime)));
    }

    #[test]
    fn test_calculate_leaderboard() {
        let competition_id = CompetitionId(1);
        let organizer_id = ParticipantId(100);
        let start_time = Timestamp::from(2000000);
        let end_time = Timestamp::from(5000000);
        let coordinator_chain = ChainId::root(0);
        let leaderboard_chain = ChainId::root(1);

        let competition = Competition::new(
            competition_id,
            "Test Tournament".to_string(),
            organizer_id,
            CompetitionFormat::TimeBasedLeaderboard {
                duration: 3000000,
                ranking_criteria: RankingCriteria::Score,
            },
            start_time,
            end_time,
            coordinator_chain,
            leaderboard_chain,
        ).expect("Competition creation should succeed");

        // Create test results
        let mut participant_scores = HashMap::new();
        
        let participant1 = ParticipantId(1);
        let participant2 = ParticipantId(2);
        
        participant_scores.insert(participant1, vec![
            CompetitionGameResult {
                competition_id,
                participant_id: participant1,
                session_id: GameSessionId(1),
                final_score: 1500,
                completion_time: Some(Timestamp::from(3000000)),
                performance_metrics: PerformanceMetrics {
                    moves_per_second: 1.0,
                    score_per_move: 10.0,
                    total_play_time: 1000000,
                    highest_tile_achieved: 1024,
                },
            }
        ]);

        participant_scores.insert(participant2, vec![
            CompetitionGameResult {
                competition_id,
                participant_id: participant2,
                session_id: GameSessionId(2),
                final_score: 2000,
                completion_time: Some(Timestamp::from(3500000)),
                performance_metrics: PerformanceMetrics {
                    moves_per_second: 1.2,
                    score_per_move: 12.0,
                    total_play_time: 1200000,
                    highest_tile_achieved: 2048,
                },
            }
        ]);

        let result = CompetitionService::calculate_leaderboard(&competition, &participant_scores);
        assert!(result.is_ok());

        let rankings = result.unwrap();
        assert_eq!(rankings.len(), 2);
        
        // Participant 2 should be first (higher score)
        assert_eq!(rankings[0].participant_id, participant2);
        assert_eq!(rankings[0].rank, 1);
        assert_eq!(rankings[0].final_score, 2000);
        
        // Participant 1 should be second
        assert_eq!(rankings[1].participant_id, participant1);
        assert_eq!(rankings[1].rank, 2);
        assert_eq!(rankings[1].final_score, 1500);
    }

    #[test]
    fn test_process_game_result() {
        let competition_id = CompetitionId(1);
        let organizer_id = ParticipantId(100);
        let start_time = Timestamp::from(2000000);
        let end_time = Timestamp::from(5000000);
        let coordinator_chain = ChainId::root(0);
        let leaderboard_chain = ChainId::root(1);

        let mut competition = Competition::new(
            competition_id,
            "Test Tournament".to_string(),
            organizer_id,
            CompetitionFormat::TimeBasedLeaderboard {
                duration: 3000000,
                ranking_criteria: RankingCriteria::Score,
            },
            start_time,
            end_time,
            coordinator_chain,
            leaderboard_chain,
        ).expect("Competition creation should succeed");

        // Set competition to in progress
        let current_time = Timestamp::from(3000000);
        competition.update_phase(current_time);

        // Create a game session with competition context
        let participant_id = ParticipantId(200);
        let session_id = GameSessionId(1);
        let mut session = GameSession::new(
            session_id,
            participant_id,
            GameVariant::Classic2048,
            current_time,
            Some(CompetitionContext {
                competition_id,
                competition_phase: CompetitionPhase::InProgress,
                participant_rank: None,
                elimination_threshold: None,
            }),
        );
        session.scoring_metrics.primary_score = 1000;
        session.complete_session(SessionStatus::CompletedSuccessfully, current_time);

        let result = CompetitionService::process_game_result(
            &competition,
            participant_id,
            &session,
            current_time,
        );

        assert!(result.is_ok());
        let game_result = result.unwrap();
        assert_eq!(game_result.competition_id, competition_id);
        assert_eq!(game_result.participant_id, participant_id);
        assert_eq!(game_result.session_id, session_id);
        assert_eq!(game_result.final_score, 1000); // No bonus for leaderboard format
    }
}