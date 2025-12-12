//! Trust System
//!
//! Systems for processing trust updates from events and forming grudges.

use bevy_ecs::prelude::*;

use crate::components::agent::{AgentId, Goal, GoalType, Goals, Traits};
use crate::components::social::RelationshipGraph;
use crate::components::world::WorldState;

/// Constants for grudge formation
pub mod grudge_constants {
    /// Base duration for revenge goal (in ticks)
    pub const BASE_REVENGE_DURATION: u64 = 1000;
    /// Maximum multiplier from grudge_persistence trait
    pub const MAX_PERSISTENCE_MULTIPLIER: f32 = 3.0;
    /// Trust threshold below which grudge forms
    pub const GRUDGE_TRUST_THRESHOLD: f32 = -0.3;
    /// Priority for revenge goal
    pub const REVENGE_PRIORITY: f32 = 0.7;
}

/// Represents a trust-affecting event to be processed
#[derive(Debug, Clone)]
pub struct TrustEvent {
    /// Agent whose trust is affected
    pub agent_id: String,
    /// Target of the trust change
    pub target_id: String,
    /// Type of trust event
    pub event_type: TrustEventType,
    /// Original event ID (for grudge tracking)
    pub origin_event: Option<String>,
}

/// Types of events that affect trust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustEventType {
    /// Positive interaction (chat, help, shared resource)
    PositiveInteraction,
    /// Promise was kept
    PromiseKept,
    /// Promise was broken
    PromiseBroken,
    /// Target demonstrated capability
    CapabilityDemonstrated,
    /// Target failed at task
    CapabilityFailed,
    /// Betrayal (major negative - faction betrayal, theft, etc.)
    Betrayal,
    /// Shared a secret that was kept
    SecretKept,
    /// Secret was leaked
    SecretLeaked,
}

/// Resource: Queue of trust events to process
#[derive(Resource, Debug, Default)]
pub struct TrustEventQueue {
    pub events: Vec<TrustEvent>,
}

impl TrustEventQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, event: TrustEvent) {
        self.events.push(event);
    }

    pub fn drain(&mut self) -> Vec<TrustEvent> {
        std::mem::take(&mut self.events)
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// System: Process trust events and update relationships
pub fn process_trust_events(
    world_state: Res<WorldState>,
    mut trust_events: ResMut<TrustEventQueue>,
    mut relationship_graph: ResMut<RelationshipGraph>,
    mut query: Query<(&AgentId, &Traits, &mut Goals)>,
) {
    let events = trust_events.drain();

    // Build lookup for agent traits
    let traits_map: std::collections::HashMap<String, (Traits, Entity)> = query
        .iter()
        .map(|(id, traits, _)| (id.0.clone(), (traits.clone(), Entity::PLACEHOLDER)))
        .collect();

    for event in events {
        // Update trust based on event type
        let rel = relationship_graph.ensure_relationship(&event.agent_id, &event.target_id);

        match event.event_type {
            TrustEventType::PositiveInteraction => {
                rel.trust.apply_positive_interaction(1.0);
                rel.last_interaction_tick = world_state.current_tick;
            }
            TrustEventType::PromiseKept => {
                rel.trust.update_reliability(0.1);
            }
            TrustEventType::PromiseBroken => {
                rel.trust.apply_broken_promise();
            }
            TrustEventType::CapabilityDemonstrated => {
                rel.trust.apply_capability_demonstrated(1.0);
            }
            TrustEventType::CapabilityFailed => {
                rel.trust.apply_capability_failed();
            }
            TrustEventType::Betrayal => {
                rel.trust.apply_betrayal();
            }
            TrustEventType::SecretKept => {
                rel.trust.update_reliability(0.15);
            }
            TrustEventType::SecretLeaked => {
                rel.trust.update_reliability(-0.25);
            }
        }

        // Check if this should form a grudge
        let trust_overall = rel.trust.overall();
        let should_form_grudge = trust_overall < grudge_constants::GRUDGE_TRUST_THRESHOLD
            && matches!(
                event.event_type,
                TrustEventType::Betrayal
                    | TrustEventType::PromiseBroken
                    | TrustEventType::SecretLeaked
            );

        if should_form_grudge {
            // Get agent's grudge persistence to determine duration
            let persistence = traits_map
                .get(&event.agent_id)
                .map(|(t, _)| t.grudge_persistence)
                .unwrap_or(0.5);

            // Calculate revenge goal duration
            let duration_multiplier = 1.0 + persistence * (grudge_constants::MAX_PERSISTENCE_MULTIPLIER - 1.0);
            let duration = (grudge_constants::BASE_REVENGE_DURATION as f32 * duration_multiplier) as u64;

            // Find the agent and add revenge goal
            for (agent_id, _traits, mut goals) in query.iter_mut() {
                if agent_id.0 == event.agent_id {
                    // Only add if they don't already have a revenge goal against this target
                    let has_existing = goals.goals.iter().any(|g| {
                        g.goal_type == GoalType::Revenge
                            && g.target.as_ref() == Some(&event.target_id)
                    });

                    if !has_existing {
                        let mut revenge_goal = Goal::new(
                            GoalType::Revenge,
                            grudge_constants::REVENGE_PRIORITY,
                        )
                        .with_target(&event.target_id)
                        .with_expiry(world_state.current_tick + duration);

                        if let Some(ref origin) = event.origin_event {
                            revenge_goal = revenge_goal.with_origin(origin);
                        }

                        goals.add(revenge_goal);
                    }
                    break;
                }
            }
        }
    }
}

/// System: Decay grudges over time based on trait
/// Removes expired revenge goals
pub fn decay_grudges(
    world_state: Res<WorldState>,
    mut query: Query<(&AgentId, &mut Goals)>,
) {
    for (_agent_id, mut goals) in query.iter_mut() {
        goals.remove_expired(world_state.current_tick);
    }
}

/// Helper function to create a trust event
pub fn create_trust_event(
    agent_id: impl Into<String>,
    target_id: impl Into<String>,
    event_type: TrustEventType,
    origin_event: Option<String>,
) -> TrustEvent {
    TrustEvent {
        agent_id: agent_id.into(),
        target_id: target_id.into(),
        event_type,
        origin_event,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_event_queue() {
        let mut queue = TrustEventQueue::new();
        assert!(queue.is_empty());

        queue.push(TrustEvent {
            agent_id: "agent_1".to_string(),
            target_id: "agent_2".to_string(),
            event_type: TrustEventType::PositiveInteraction,
            origin_event: None,
        });

        assert!(!queue.is_empty());

        let events = queue.drain();
        assert_eq!(events.len(), 1);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_grudge_duration_calculation() {
        // Low persistence = shorter duration
        let low_persistence = 0.2;
        let low_multiplier = 1.0 + low_persistence * (grudge_constants::MAX_PERSISTENCE_MULTIPLIER - 1.0);
        let low_duration = (grudge_constants::BASE_REVENGE_DURATION as f32 * low_multiplier) as u64;

        // High persistence = longer duration
        let high_persistence = 0.9;
        let high_multiplier = 1.0 + high_persistence * (grudge_constants::MAX_PERSISTENCE_MULTIPLIER - 1.0);
        let high_duration = (grudge_constants::BASE_REVENGE_DURATION as f32 * high_multiplier) as u64;

        assert!(high_duration > low_duration);
        assert!(high_duration <= grudge_constants::BASE_REVENGE_DURATION * 3);
    }
}
