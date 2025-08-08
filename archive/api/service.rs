//! GraphQL service implementation with domain model mapping.

use async_graphql::{Context, Object, SimpleObject};
use linera_sdk::{
    abi::WithServiceAbi,
    Service,
};
use std::sync::Arc;

use crate::{
    core::{models::*, value_objects::*},
    infrastructure::{contract::Game2048Abi, state::GamePlatformState},
};

/// GraphQL service for the 2048 game
pub struct Game2048Service {
    state: Arc<GamePlatformState>,
}

impl WithServiceAbi for Game2048Service {
    type Abi = Game2048Abi;
}

impl Service for Game2048Service {
    type Parameters = ();
    async fn new(_runtime: linera_sdk::service::ServiceRuntime<Self>) -> Self {
        // Simplified service creation
        Game2048Service {
            state: Arc::new(GamePlatformState::default()),
        }
    }

    async fn handle_query(&self, request: async_graphql::Request) -> async_graphql::Response {
        // Create a simple schema for now
        let schema = async_graphql::Schema::build(
            QueryRoot,
            MutationRoot,
            async_graphql::EmptySubscription,
        )
        .data(Arc::new(self.state.clone()))
        .finish();
        
        schema.execute(request).await
    }
}

/// GraphQL query root
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get participant by ID
    async fn participant(&self, ctx: &Context<'_>, id: String) -> async_graphql::Result<Option<ParticipantObject>> {
        let state = ctx.data::<Arc<GamePlatformState>>()?;
        let participant_id = ParticipantId(id.parse()?);
        
        if let Some(participant) = state.get_participant(&participant_id).await.map_err(|e| async_graphql::Error::new(e.to_string()))? {
            Ok(Some(ParticipantObject::from(participant)))
        } else {
            Ok(None)
        }
    }

    /// Get participant by username
    async fn participant_by_username(&self, ctx: &Context<'_>, username: String) -> async_graphql::Result<Option<ParticipantObject>> {
        let state = ctx.data::<Arc<GamePlatformState>>()?;
        
        if let Some(participant) = state.get_participant_by_username(&username).await.map_err(|e| async_graphql::Error::new(e.to_string()))? {
            Ok(Some(ParticipantObject::from(participant)))
        } else {
            Ok(None)
        }
    }

    /// Get active game session by ID
    async fn game_session(&self, ctx: &Context<'_>, id: String) -> async_graphql::Result<Option<GameSessionObject>> {
        let state = ctx.data::<Arc<GamePlatformState>>()?;
        let session_id = GameSessionId(id.parse()?);
        
        if let Some(session) = state.active_game_sessions.get(&session_id) {
            Ok(Some(GameSessionObject::from(session.clone())))
        } else {
            Ok(None)
        }
    }

    /// Get competition by ID
    async fn competition(&self, ctx: &Context<'_>, id: String) -> async_graphql::Result<Option<CompetitionObject>> {
        let state = ctx.data::<Arc<GamePlatformState>>()?;
        let competition_id = CompetitionId(id.parse()?);
        
        if let Some(competition) = state.active_competitions.get(&competition_id) {
            Ok(Some(CompetitionObject::from(competition.clone())))
        } else {
            Ok(None)
        }
    }

    /// Get competition participants
    async fn competition_participants(&self, ctx: &Context<'_>, competition_id: String) -> async_graphql::Result<Vec<ParticipantObject>> {
        let state = ctx.data::<Arc<GamePlatformState>>()?;
        let comp_id = CompetitionId(competition_id.parse()?);
        
        let participant_ids = state.get_competition_participants(&comp_id).await.map_err(|e| async_graphql::Error::new(e.to_string()))?;
        let mut participants = Vec::new();
        
        for participant_id in participant_ids {
            if let Some(participant) = state.get_participant(&participant_id).await.map_err(|e| async_graphql::Error::new(e.to_string()))? {
                participants.push(ParticipantObject::from(participant));
            }
        }
        
        Ok(participants)
    }

    /// Get system metrics
    async fn system_metrics(&self, ctx: &Context<'_>) -> async_graphql::Result<SystemMetricsObject> {
        let state = ctx.data::<Arc<GamePlatformState>>()?;
        let metrics = &state.system_metrics;
        Ok(SystemMetricsObject::from(metrics.clone()))
    }
}

/// GraphQL mutation root
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Register a new participant (placeholder - actual implementation would use operations)
    async fn register_participant(&self, _ctx: &Context<'_>, username: String) -> async_graphql::Result<String> {
        // This would typically trigger a contract operation
        // For now, return a placeholder response
        Ok(format!("Registration request submitted for username: {}", username))
    }

    /// Create a new game session (placeholder)
    async fn create_game_session(&self, _ctx: &Context<'_>, game_variant: String) -> async_graphql::Result<String> {
        // This would typically trigger a contract operation
        Ok(format!("Game session creation request submitted for variant: {}", game_variant))
    }

    /// Make a move in a game session (placeholder)
    async fn make_move(&self, _ctx: &Context<'_>, session_id: String, direction: String) -> async_graphql::Result<String> {
        // This would typically trigger a contract operation
        Ok(format!("Move request submitted for session {} in direction {}", session_id, direction))
    }
}

/// GraphQL object for Participant
#[derive(SimpleObject)]
pub struct ParticipantObject {
    pub id: String,
    pub username: String,
    pub display_name: Option<String>,
    pub home_chain_id: String,
    pub account_created_at: String,
    pub last_activity_at: String,
    pub total_sessions_played: u32,
    pub competitions_entered: u32,
    pub personal_best_score: String,
    pub average_score: f64,
    pub skill_rating: Option<u32>,
    pub account_status: String,
}

impl From<Participant> for ParticipantObject {
    fn from(participant: Participant) -> Self {
        Self {
            id: participant.participant_id.0.to_string(),
            username: participant.display_identity.username,
            display_name: participant.display_identity.display_name,
            home_chain_id: participant.blockchain_identity.home_chain_id.to_string(),
            account_created_at: participant.participation_history.account_created_at.to_string(),
            last_activity_at: participant.participation_history.last_activity_at.to_string(),
            total_sessions_played: participant.participation_history.total_sessions_played,
            competitions_entered: participant.participation_history.competitions_entered,
            personal_best_score: participant.skill_metrics.personal_best_score.to_string(),
            average_score: participant.skill_metrics.average_score,
            skill_rating: participant.skill_metrics.skill_rating,
            account_status: format!("{:?}", participant.account_status),
        }
    }
}

/// GraphQL object for GameSession
#[derive(SimpleObject)]
pub struct GameSessionObject {
    pub id: String,
    pub participant_id: String,
    pub game_variant: String,
    pub primary_score: String,
    pub move_count: u32,
    pub highest_tile_achieved: u32,
    pub session_status: String,
    pub initiated_at: String,
    pub last_activity_at: String,
    pub concluded_at: Option<String>,
}

impl From<GameSession> for GameSessionObject {
    fn from(session: GameSession) -> Self {
        Self {
            id: session.session_id.0.to_string(),
            participant_id: session.participant_id.0.to_string(),
            game_variant: format!("{:?}", session.game_variant),
            primary_score: session.scoring_metrics.primary_score.to_string(),
            move_count: session.board_state.move_count,
            highest_tile_achieved: session.board_state.highest_tile_achieved,
            session_status: format!("{:?}", session.session_status),
            initiated_at: session.session_lifecycle.initiated_at.to_string(),
            last_activity_at: session.session_lifecycle.last_activity_at.to_string(),
            concluded_at: session.session_lifecycle.concluded_at.map(|t| t.to_string()),
        }
    }
}

/// GraphQL object for Competition
#[derive(SimpleObject)]
pub struct CompetitionObject {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub organizer_id: String,
    pub category: String,
    pub visibility: String,
    pub format: String,
    pub registration_opens_at: String,
    pub registration_closes_at: String,
    pub competition_starts_at: String,
    pub competition_ends_at: String,
    pub current_phase: String,
}

impl From<Competition> for CompetitionObject {
    fn from(competition: Competition) -> Self {
        Self {
            id: competition.competition_id.0.to_string(),
            title: competition.competition_metadata.title,
            description: competition.competition_metadata.description,
            organizer_id: competition.competition_metadata.organizer_id.0.to_string(),
            category: format!("{:?}", competition.competition_metadata.category),
            visibility: format!("{:?}", competition.competition_metadata.visibility),
            format: format!("{:?}", competition.competition_format),
            registration_opens_at: competition.competition_lifecycle.registration_opens_at.to_string(),
            registration_closes_at: competition.competition_lifecycle.registration_closes_at.to_string(),
            competition_starts_at: competition.competition_lifecycle.competition_starts_at.to_string(),
            competition_ends_at: competition.competition_lifecycle.competition_ends_at.to_string(),
            current_phase: format!("{:?}", competition.competition_lifecycle.current_phase),
        }
    }
}

/// GraphQL object for SystemMetrics
#[derive(SimpleObject)]
pub struct SystemMetricsObject {
    pub active_sessions_count: u32,
    pub total_participants: u32,
    pub active_competitions: u32,
    pub cross_chain_messages_pending: u32,
    pub last_updated: String,
}

impl From<SystemMetrics> for SystemMetricsObject {
    fn from(metrics: SystemMetrics) -> Self {
        Self {
            active_sessions_count: metrics.active_sessions_count,
            total_participants: metrics.total_participants,
            active_competitions: metrics.active_competitions,
            cross_chain_messages_pending: metrics.cross_chain_messages_pending,
            last_updated: metrics.last_updated.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use linera_sdk::linera_base_types::{ChainId, Timestamp};

    #[test]
    fn test_participant_object_conversion() {
        let participant_id = ParticipantId(1);
        let username = "test_user".to_string();
        let home_chain = ChainId::root(0);
        let current_time = Timestamp::from(1000000);

        let participant = Participant::new(participant_id, username.clone(), home_chain, current_time)
            .expect("Participant creation should succeed");

        let participant_obj = ParticipantObject::from(participant);
        
        assert_eq!(participant_obj.id, "1");
        assert_eq!(participant_obj.username, username);
        assert_eq!(participant_obj.total_sessions_played, 0);
        assert_eq!(participant_obj.personal_best_score, "0");
    }

    #[test]
    fn test_game_session_object_conversion() {
        let session_id = GameSessionId(1);
        let participant_id = ParticipantId(100);
        let current_time = Timestamp::from(1000000);

        let session = GameSession::new(
            session_id,
            participant_id,
            GameVariant::Classic2048,
            current_time,
            None,
        );

        let session_obj = GameSessionObject::from(session);
        
        assert_eq!(session_obj.id, "1");
        assert_eq!(session_obj.participant_id, "100");
        assert_eq!(session_obj.move_count, 0);
        assert_eq!(session_obj.primary_score, "0");
    }

    #[test]
    fn test_system_metrics_object_conversion() {
        let metrics = SystemMetrics {
            active_sessions_count: 10,
            total_participants: 100,
            active_competitions: 5,
            cross_chain_messages_pending: 2,
            last_updated: Timestamp::from(1000000),
        };

        let metrics_obj = SystemMetricsObject::from(metrics);
        
        assert_eq!(metrics_obj.active_sessions_count, 10);
        assert_eq!(metrics_obj.total_participants, 100);
        assert_eq!(metrics_obj.active_competitions, 5);
        assert_eq!(metrics_obj.cross_chain_messages_pending, 2);
    }
}