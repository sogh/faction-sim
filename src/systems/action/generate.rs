//! Action Generation System
//!
//! Generates valid actions for each agent based on their state and surroundings.

use bevy_ecs::prelude::*;
use std::collections::HashMap;

use crate::actions::movement::{MoveAction, MovementType};
use crate::actions::communication::{CommunicationAction, CommunicationType, TargetMode};
use crate::actions::archive::{ArchiveAction, ArchiveActionType};
use crate::components::agent::{AgentId, AgentName, FoodSecurity, Needs, Role, SocialBelonging, Traits};
use crate::components::faction::{FactionMembership, FactionRegistry};
use crate::components::social::{MemoryBank, MemoryValence, RelationshipGraph};
use crate::components::world::{LocationRegistry, Position};
use crate::systems::perception::AgentsByLocation;
use crate::systems::memory::get_most_interesting_memory;

/// Enum representing all possible actions an agent can take
#[derive(Debug, Clone)]
pub enum Action {
    /// Move to an adjacent location
    Move(MoveAction),
    /// Communicate with other agents
    Communicate(CommunicationAction),
    /// Interact with faction archive
    Archive(ArchiveAction),
    /// Stay at current location (default/idle action)
    Idle,
}

/// A weighted action candidate
#[derive(Debug, Clone)]
pub struct WeightedAction {
    pub action: Action,
    pub weight: f32,
    pub reason: String,
}

impl WeightedAction {
    pub fn new(action: Action, weight: f32, reason: impl Into<String>) -> Self {
        Self {
            action,
            weight,
            reason: reason.into(),
        }
    }
}

/// Resource storing pending actions for each agent
#[derive(Resource, Debug, Default)]
pub struct PendingActions {
    /// Maps agent_id -> list of weighted action candidates
    pub actions: HashMap<String, Vec<WeightedAction>>,
}

impl PendingActions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.actions.clear();
    }

    pub fn add(&mut self, agent_id: impl Into<String>, action: WeightedAction) {
        self.actions
            .entry(agent_id.into())
            .or_default()
            .push(action);
    }

    pub fn get(&self, agent_id: &str) -> Option<&Vec<WeightedAction>> {
        self.actions.get(agent_id)
    }

    pub fn take(&mut self, agent_id: &str) -> Option<Vec<WeightedAction>> {
        self.actions.remove(agent_id)
    }
}

/// System to generate movement actions for each agent
pub fn generate_movement_actions(
    location_registry: Res<LocationRegistry>,
    faction_registry: Res<FactionRegistry>,
    mut pending_actions: ResMut<PendingActions>,
    query: Query<(&AgentId, &Position, &FactionMembership, &Needs)>,
) {
    for (agent_id, position, membership, needs) in query.iter() {
        // Get current location and its adjacencies
        let Some(current_location) = location_registry.get(&position.location_id) else {
            continue;
        };

        let adjacent_locations = location_registry.get_adjacent(&position.location_id);

        // Generate move actions for each adjacent location
        for adjacent_id in &adjacent_locations {
            let action = MoveAction::travel(&agent_id.0, adjacent_id);
            pending_actions.add(
                &agent_id.0,
                WeightedAction::new(
                    Action::Move(action),
                    0.1, // Base weight for random movement
                    format!("travel to {}", adjacent_id),
                ),
            );
        }

        // Generate return home action if not at HQ
        if let Some(faction) = faction_registry.get(&membership.faction_id) {
            if position.location_id != faction.hq_location {
                // Check if path to HQ exists (simplified: check if HQ is adjacent or reachable)
                let can_reach_hq = adjacent_locations.contains(&faction.hq_location)
                    || location_registry.path_exists(&position.location_id, &faction.hq_location);

                if can_reach_hq {
                    // Find next step toward HQ
                    let next_step = if adjacent_locations.contains(&faction.hq_location) {
                        faction.hq_location.clone()
                    } else {
                        // Get first step on path to HQ
                        location_registry
                            .next_step_toward(&position.location_id, &faction.hq_location)
                            .unwrap_or_else(|| adjacent_locations.first().cloned().unwrap_or_default())
                    };

                    if !next_step.is_empty() {
                        let action = MoveAction::return_home(&agent_id.0, &next_step);
                        let weight = if needs.social_belonging == SocialBelonging::Isolated {
                            0.5 // Higher weight if isolated
                        } else if needs.social_belonging == SocialBelonging::Peripheral {
                            0.3
                        } else {
                            0.1
                        };

                        pending_actions.add(
                            &agent_id.0,
                            WeightedAction::new(
                                Action::Move(action),
                                weight,
                                "return to faction HQ",
                            ),
                        );
                    }
                }
            }
        }

        // Always add idle option
        pending_actions.add(
            &agent_id.0,
            WeightedAction::new(
                Action::Idle,
                0.2, // Base idle weight
                "stay put",
            ),
        );
    }
}

/// System to generate patrol actions for scouts
pub fn generate_patrol_actions(
    location_registry: Res<LocationRegistry>,
    faction_registry: Res<FactionRegistry>,
    mut pending_actions: ResMut<PendingActions>,
    query: Query<(&AgentId, &Position, &FactionMembership)>,
) {
    for (agent_id, position, membership) in query.iter() {
        // Only scouts generate patrol actions
        if !matches!(membership.role, Role::ScoutCaptain) {
            continue;
        }

        let Some(faction) = faction_registry.get(&membership.faction_id) else {
            continue;
        };

        // Get adjacent locations within faction territory for patrol
        let adjacent_locations = location_registry.get_adjacent(&position.location_id);

        for adjacent_id in &adjacent_locations {
            // Prefer patrolling within territory
            let in_territory = faction.territory.contains(adjacent_id);
            let base_weight = if in_territory { 0.4 } else { 0.15 };

            let action = MoveAction::patrol(&agent_id.0, adjacent_id);
            pending_actions.add(
                &agent_id.0,
                WeightedAction::new(
                    Action::Move(action),
                    base_weight,
                    format!("patrol to {}", adjacent_id),
                ),
            );
        }
    }
}

/// System to generate communication actions for agents
///
/// Generates share_memory actions when:
/// - Agent has interesting memories to share
/// - There are other agents at the same location
pub fn generate_communication_actions(
    world_state: Res<crate::components::world::WorldState>,
    agents_by_location: Res<AgentsByLocation>,
    memory_bank: Res<MemoryBank>,
    relationship_graph: Res<RelationshipGraph>,
    mut pending_actions: ResMut<PendingActions>,
    query: Query<(&AgentId, &AgentName, &Position, &FactionMembership, &Traits, &Needs)>,
) {
    // Build a map of agent_id -> (name, faction_id) for target info
    let agent_info: HashMap<String, (String, String)> = query
        .iter()
        .map(|(id, name, _, membership, _, _)| {
            (id.0.clone(), (name.0.clone(), membership.faction_id.clone()))
        })
        .collect();

    for (agent_id, _name, position, membership, traits, _needs) in query.iter() {
        // Get agents at the same location
        let nearby_agents: Vec<String> = agents_by_location.at_location(&position.location_id).to_vec();

        // Skip if alone
        if nearby_agents.len() <= 1 {
            continue;
        }

        // Check if this agent has interesting memories to share
        let interesting_memory = get_most_interesting_memory(
            &memory_bank,
            &agent_id.0,
            world_state.current_tick,
        );

        if let Some(memory) = interesting_memory {
            // Generate share actions for each nearby agent
            for target_id in &nearby_agents {
                if target_id == &agent_id.0 {
                    continue;
                }

                let Some((target_name, target_faction)) = agent_info.get(target_id) else {
                    continue;
                };

                // Determine target mode based on group_preference
                let target_mode = if traits.group_preference > 0.7 && nearby_agents.len() >= 4 {
                    TargetMode::Group
                } else {
                    TargetMode::Individual
                };

                // Base weight for gossip (from behavioral rules)
                let base_weight = 0.4;

                // Determine reason based on memory valence
                let reason = match memory.valence {
                    crate::components::social::MemoryValence::Negative => {
                        format!("share negative gossip about {} with {}", memory.subject, target_name)
                    }
                    crate::components::social::MemoryValence::Positive => {
                        format!("share good news about {} with {}", memory.subject, target_name)
                    }
                    crate::components::social::MemoryValence::Neutral => {
                        format!("share information about {} with {}", memory.subject, target_name)
                    }
                };

                let action = CommunicationAction::share_memory(
                    &agent_id.0,
                    target_id,
                    &memory.memory_id,
                    target_mode,
                );

                pending_actions.add(
                    &agent_id.0,
                    WeightedAction::new(
                        Action::Communicate(action),
                        base_weight,
                        reason,
                    ),
                );

                // If targeting group, only add one action
                if target_mode == TargetMode::Group {
                    break;
                }
            }
        }
    }
}

/// System to generate archive actions for agents at faction HQ
///
/// Generates write_entry actions when:
/// - Agent is at their faction HQ
/// - Agent can write to archive (leader, reader, or council member)
/// - Agent has a significant memory worth recording
pub fn generate_archive_actions(
    world_state: Res<crate::components::world::WorldState>,
    faction_registry: Res<FactionRegistry>,
    memory_bank: Res<MemoryBank>,
    mut pending_actions: ResMut<PendingActions>,
    query: Query<(&AgentId, &AgentName, &Position, &FactionMembership, &Traits)>,
) {
    use crate::actions::archive::archive_weights;

    for (agent_id, agent_name, position, membership, traits) in query.iter() {
        // Check if agent is at their faction HQ
        let Some(faction) = faction_registry.get(&membership.faction_id) else {
            continue;
        };

        if position.location_id != faction.hq_location {
            continue;
        }

        // Check if agent can write to archive
        if !membership.can_write_archive() {
            continue;
        }

        // Find a significant memory to write
        let Some(memories) = memory_bank.get_memories(&agent_id.0) else {
            continue;
        };

        // Find the most significant unrecorded memory
        let significant_memory = memories
            .iter()
            .filter(|m| m.emotional_weight > 0.3 && m.fidelity > 0.5)
            .max_by(|a, b| {
                a.emotional_weight
                    .partial_cmp(&b.emotional_weight)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

        if let Some(memory) = significant_memory {
            let mut weight = archive_weights::WRITE_BASE;

            // Bonus for significant events
            if memory.emotional_weight > 0.6 {
                weight += archive_weights::SIGNIFICANT_EVENT_BONUS;
            }

            // Bonus if memory reflects well on self
            if memory.subject == agent_id.0 && memory.valence == MemoryValence::Positive {
                weight += archive_weights::SELF_FAVORABLE_BONUS;
            }

            let action = ArchiveAction::write_entry(
                &agent_id.0,
                &membership.faction_id,
                &memory.memory_id,
            );

            pending_actions.add(
                &agent_id.0,
                WeightedAction::new(
                    Action::Archive(action),
                    weight,
                    format!("record {} in archive", memory.subject),
                ),
            );
        }

        // Generate read archive action (for anyone at HQ)
        let archive = faction_registry.get_archive(&membership.faction_id);
        if archive.map_or(false, |a| a.entry_count() > 0) {
            let action = ArchiveAction::read_archive(&agent_id.0, &membership.faction_id);
            pending_actions.add(
                &agent_id.0,
                WeightedAction::new(
                    Action::Archive(action),
                    archive_weights::READ_BASE,
                    "read faction archive".to_string(),
                ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pending_actions() {
        let mut pending = PendingActions::new();

        pending.add(
            "agent_001",
            WeightedAction::new(Action::Idle, 0.5, "test"),
        );
        pending.add(
            "agent_001",
            WeightedAction::new(
                Action::Move(MoveAction::travel("agent_001", "village")),
                0.3,
                "travel",
            ),
        );

        let actions = pending.get("agent_001").unwrap();
        assert_eq!(actions.len(), 2);

        let taken = pending.take("agent_001").unwrap();
        assert_eq!(taken.len(), 2);
        assert!(pending.get("agent_001").is_none());
    }

    #[test]
    fn test_weighted_action() {
        let action = WeightedAction::new(Action::Idle, 0.5, "resting");
        assert_eq!(action.weight, 0.5);
        assert_eq!(action.reason, "resting");
    }
}
