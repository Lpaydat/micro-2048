//! Competition domain model with extensible tournament system.

use serde::{Deserialize, Serialize};
use linera_sdk::linera_base_types::{ChainId, Timestamp};
use crate::core::value_objects::*;

/// Represents a competition with extensible format support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Competition {
    pub competition_id: CompetitionId,
    pub competition_metadata: CompetitionMetadata,
    pub competition_format: CompetitionFormat,
    pub participation_rules: ParticipationRules,
    pub competition_lifecycle: CompetitionLifecycle,
    pub reward_structure: RewardStructure,
    pub coordination_setup: CrossChainCoordination,
}

impl Competition {
    /// Create a new competition with the given parameters
    pub fn new(
        competition_id: CompetitionId,
        title: String,
        organizer_id: ParticipantId,
        format: CompetitionFormat,
        start_time: Timestamp,
        end_time: Timestamp,
        coordinator_chain: ChainId,
        leaderboard_chain: ChainId,
    ) -> Result<Self, CompetitionError> {
        // Validate time constraints
        if start_time >= end_time {
            return Err(CompetitionError::InvalidTimeRange);
        }

        let current_time = start_time; // Assume creation happens at start time for simplicity
        let registration_opens = Timestamp::from(start_time.micros().saturating_sub(3600_000_000)); // 1 hour before
        let registration_closes = start_time;

        Ok(Self {
            competition_id,
            competition_metadata: CompetitionMetadata {
                title,
                description: None,
                organizer_id,
                category: CompetitionCategory::Casual,
                visibility: CompetitionVisibility::Public,
            },
            competition_format: format,
            participation_rules: ParticipationRules {
                max_participants: None,
                min_skill_rating: None,
                entry_requirements: Vec::new(),
                game_variant_restrictions: Vec::new(),
            },
            competition_lifecycle: CompetitionLifecycle {
                registration_opens_at: registration_opens,
                registration_closes_at: registration_closes,
                competition_starts_at: start_time,
                competition_ends_at: end_time,
                results_finalized_at: None,
                current_phase: if current_time < registration_opens {
                    CompetitionPhase::RegistrationOpen
                } else {
                    CompetitionPhase::RegistrationOpen
                },
            },
            reward_structure: RewardStructure {
                prize_pool: 0,
                distribution: Vec::new(),
                reward_type: RewardType::Points,
            },
            coordination_setup: CrossChainCoordination {
                coordinator_chain,
                participant_chains: Vec::new(),
                leaderboard_chain,
                sync_frequency: 60_000_000, // 1 minute in microseconds
            },
        })
    }

    /// Check if the competition is currently accepting registrations
    pub fn is_registration_open(&self, current_time: Timestamp) -> bool {
        current_time >= self.competition_lifecycle.registration_opens_at
            && current_time < self.competition_lifecycle.registration_closes_at
            && matches!(self.competition_lifecycle.current_phase, CompetitionPhase::RegistrationOpen)
    }

    /// Check if the competition is currently in progress
    pub fn is_in_progress(&self, current_time: Timestamp) -> bool {
        current_time >= self.competition_lifecycle.competition_starts_at
            && current_time < self.competition_lifecycle.competition_ends_at
            && matches!(self.competition_lifecycle.current_phase, CompetitionPhase::InProgress)
    }

    /// Check if the competition has concluded
    pub fn is_concluded(&self, current_time: Timestamp) -> bool {
        current_time >= self.competition_lifecycle.competition_ends_at
            || matches!(
                self.competition_lifecycle.current_phase,
                CompetitionPhase::Concluded | CompetitionPhase::ResultsFinalized | CompetitionPhase::Cancelled
            )
    }

    /// Update the competition phase based on current time
    pub fn update_phase(&mut self, current_time: Timestamp) {
        let new_phase = if current_time < self.competition_lifecycle.registration_opens_at {
            CompetitionPhase::RegistrationOpen // Waiting for registration to open
        } else if current_time < self.competition_lifecycle.registration_closes_at {
            CompetitionPhase::RegistrationOpen
        } else if current_time < self.competition_lifecycle.competition_starts_at {
            CompetitionPhase::RegistrationClosed
        } else if current_time < self.competition_lifecycle.competition_ends_at {
            CompetitionPhase::InProgress
        } else {
            CompetitionPhase::Concluded
        };

        self.competition_lifecycle.current_phase = new_phase;
    }

    /// Check if a participant can join this competition
    pub fn can_participant_join(
        &self,
        participant: &crate::core::models::participant::Participant,
        current_time: Timestamp,
    ) -> Result<(), CompetitionError> {
        // Check if registration is open
        if !self.is_registration_open(current_time) {
            return Err(CompetitionError::RegistrationClosed);
        }

        // Check if participant is active
        if !participant.is_active() {
            return Err(CompetitionError::ParticipantNotEligible("Participant is not active".to_string()));
        }

        // Check minimum skill rating requirement
        if let Some(min_rating) = self.participation_rules.min_skill_rating {
            if let Some(participant_rating) = participant.skill_metrics.skill_rating {
                if participant_rating < min_rating {
                    return Err(CompetitionError::ParticipantNotEligible(
                        format!("Minimum skill rating {} required", min_rating)
                    ));
                }
            } else {
                return Err(CompetitionError::ParticipantNotEligible(
                    "Skill rating required but not available".to_string()
                ));
            }
        }

        // Check entry requirements
        for requirement in &self.participation_rules.entry_requirements {
            match requirement {
                EntryRequirement::MinimumGamesPlayed(min_games) => {
                    if participant.participation_history.total_sessions_played < *min_games {
                        return Err(CompetitionError::ParticipantNotEligible(
                            format!("Minimum {} games played required", min_games)
                        ));
                    }
                }
                EntryRequirement::MinimumScore(min_score) => {
                    if participant.skill_metrics.personal_best_score < *min_score {
                        return Err(CompetitionError::ParticipantNotEligible(
                            format!("Minimum score {} required", min_score)
                        ));
                    }
                }
                EntryRequirement::AchievementRequired(achievement_id) => {
                    if !participant.participation_history.achievements_earned
                        .iter()
                        .any(|a| a.achievement_id == *achievement_id) {
                        return Err(CompetitionError::ParticipantNotEligible(
                            format!("Achievement '{}' required", achievement_id)
                        ));
                    }
                }
                EntryRequirement::InvitationRequired => {
                    // This would need to be checked against an invitation list
                    // For now, we'll assume it's handled elsewhere
                }
            }
        }

        Ok(())
    }

    /// Finalize the competition results
    pub fn finalize_results(&mut self, current_time: Timestamp) -> Result<(), CompetitionError> {
        if !self.is_concluded(current_time) {
            return Err(CompetitionError::CompetitionNotConcluded);
        }

        self.competition_lifecycle.current_phase = CompetitionPhase::ResultsFinalized;
        self.competition_lifecycle.results_finalized_at = Some(current_time);
        
        Ok(())
    }

    /// Cancel the competition
    pub fn cancel(&mut self, current_time: Timestamp) -> Result<(), CompetitionError> {
        if matches!(
            self.competition_lifecycle.current_phase,
            CompetitionPhase::Concluded | CompetitionPhase::ResultsFinalized
        ) {
            return Err(CompetitionError::CompetitionAlreadyFinalized);
        }

        self.competition_lifecycle.current_phase = CompetitionPhase::Cancelled;
        self.competition_lifecycle.results_finalized_at = Some(current_time);
        
        Ok(())
    }

    /// Get the expected duration of the competition in microseconds
    pub fn get_duration(&self) -> u64 {
        self.competition_lifecycle.competition_ends_at.micros() 
            - self.competition_lifecycle.competition_starts_at.micros()
    }

    /// Get the registration duration in microseconds
    pub fn get_registration_duration(&self) -> u64 {
        self.competition_lifecycle.registration_closes_at.micros()
            - self.competition_lifecycle.registration_opens_at.micros()
    }
}

/// Errors that can occur when working with competitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompetitionError {
    InvalidTimeRange,
    RegistrationClosed,
    CompetitionNotStarted,
    CompetitionEnded,
    CompetitionNotConcluded,
    CompetitionAlreadyFinalized,
    ParticipantNotEligible(String),
    MaxParticipantsReached,
    CompetitionNotFound,
    InsufficientPermissions,
    InvalidFormat(String),
}

impl std::fmt::Display for CompetitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompetitionError::InvalidTimeRange => write!(f, "Invalid time range for competition"),
            CompetitionError::RegistrationClosed => write!(f, "Registration is closed"),
            CompetitionError::CompetitionNotStarted => write!(f, "Competition has not started yet"),
            CompetitionError::CompetitionEnded => write!(f, "Competition has ended"),
            CompetitionError::CompetitionNotConcluded => write!(f, "Competition has not concluded yet"),
            CompetitionError::CompetitionAlreadyFinalized => write!(f, "Competition results are already finalized"),
            CompetitionError::ParticipantNotEligible(reason) => write!(f, "Participant not eligible: {}", reason),
            CompetitionError::MaxParticipantsReached => write!(f, "Maximum number of participants reached"),
            CompetitionError::CompetitionNotFound => write!(f, "Competition not found"),
            CompetitionError::InsufficientPermissions => write!(f, "Insufficient permissions"),
            CompetitionError::InvalidFormat(reason) => write!(f, "Invalid competition format: {}", reason),
        }
    }
}

impl std::error::Error for CompetitionError {}

/// Results of a completed competition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionResults {
    pub competition_id: CompetitionId,
    pub final_rankings: Vec<CompetitionRanking>,
    pub total_participants: u32,
    pub completion_rate: f64, // Percentage of participants who completed
    pub average_score: f64,
    pub highest_score: u64,
    pub finalized_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionRanking {
    pub rank: u32,
    pub participant_id: ParticipantId,
    pub final_score: u64,
    pub games_played: u32,
    pub completion_time: Option<Timestamp>,
    pub rewards_earned: Vec<RewardTier>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::participant::Participant;
    use linera_sdk::linera_base_types::{ChainId, Timestamp};

    #[test]
    fn test_competition_creation() {
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
                duration: 3000000, // 3 seconds
                ranking_criteria: RankingCriteria::Score,
            },
            start_time,
            end_time,
            coordinator_chain,
            leaderboard_chain,
        ).expect("Competition creation should succeed");

        assert_eq!(competition.competition_id, competition_id);
        assert_eq!(competition.competition_metadata.title, "Test Tournament");
        assert_eq!(competition.competition_metadata.organizer_id, organizer_id);
        assert_eq!(competition.competition_lifecycle.competition_starts_at, start_time);
        assert_eq!(competition.competition_lifecycle.competition_ends_at, end_time);
        assert_eq!(competition.get_duration(), 3000000);
    }

    #[test]
    fn test_invalid_time_range() {
        let competition_id = CompetitionId(1);
        let organizer_id = ParticipantId(100);
        let start_time = Timestamp::from(5000000);
        let end_time = Timestamp::from(2000000); // End before start
        let coordinator_chain = ChainId::root(0);
        let leaderboard_chain = ChainId::root(1);

        let result = Competition::new(
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
        );

        assert!(matches!(result, Err(CompetitionError::InvalidTimeRange)));
    }

    #[test]
    fn test_competition_phase_updates() {
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

        // Test registration phase
        let registration_time = Timestamp::from(1500000);
        competition.update_phase(registration_time);
        assert!(matches!(competition.competition_lifecycle.current_phase, CompetitionPhase::RegistrationOpen));
        assert!(competition.is_registration_open(registration_time));

        // Test registration closed phase
        let pre_start_time = Timestamp::from(2000000);
        competition.update_phase(pre_start_time);
        assert!(matches!(competition.competition_lifecycle.current_phase, CompetitionPhase::RegistrationClosed));
        assert!(!competition.is_registration_open(pre_start_time));

        // Test in progress phase
        let in_progress_time = Timestamp::from(3000000);
        competition.update_phase(in_progress_time);
        assert!(matches!(competition.competition_lifecycle.current_phase, CompetitionPhase::InProgress));
        assert!(competition.is_in_progress(in_progress_time));

        // Test concluded phase
        let concluded_time = Timestamp::from(6000000);
        competition.update_phase(concluded_time);
        assert!(matches!(competition.competition_lifecycle.current_phase, CompetitionPhase::Concluded));
        assert!(competition.is_concluded(concluded_time));
    }

    #[test]
    fn test_participant_eligibility() {
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

        // Set minimum requirements
        competition.participation_rules.min_skill_rating = Some(1000);
        competition.participation_rules.entry_requirements.push(
            EntryRequirement::MinimumGamesPlayed(5)
        );

        // Create a participant that meets requirements
        let participant_id = ParticipantId(200);
        let mut participant = Participant::new(
            participant_id,
            "test_player".to_string(),
            ChainId::root(0),
            Timestamp::from(1000000),
        ).expect("Participant creation should succeed");

        participant.skill_metrics.skill_rating = Some(1200);
        participant.participation_history.total_sessions_played = 10;

        let registration_time = Timestamp::from(1500000);
        competition.update_phase(registration_time);

        // Should be eligible
        let result = competition.can_participant_join(&participant, registration_time);
        assert!(result.is_ok());

        // Test insufficient skill rating
        participant.skill_metrics.skill_rating = Some(800);
        let result = competition.can_participant_join(&participant, registration_time);
        assert!(matches!(result, Err(CompetitionError::ParticipantNotEligible(_))));

        // Test insufficient games played
        participant.skill_metrics.skill_rating = Some(1200);
        participant.participation_history.total_sessions_played = 3;
        let result = competition.can_participant_join(&participant, registration_time);
        assert!(matches!(result, Err(CompetitionError::ParticipantNotEligible(_))));
    }

    #[test]
    fn test_competition_finalization() {
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

        let finalization_time = Timestamp::from(6000000);
        competition.update_phase(finalization_time);

        // Should be able to finalize after conclusion
        let result = competition.finalize_results(finalization_time);
        assert!(result.is_ok());
        assert!(matches!(competition.competition_lifecycle.current_phase, CompetitionPhase::ResultsFinalized));
        assert_eq!(competition.competition_lifecycle.results_finalized_at, Some(finalization_time));

        // Should not be able to finalize again
        let result = competition.finalize_results(finalization_time);
        assert!(matches!(result, Err(CompetitionError::CompetitionAlreadyFinalized)));
    }
}