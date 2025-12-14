//! Action Generation System
//!
//! Generates valid actions for each agent based on their state and surroundings.

use bevy_ecs::prelude::*;
use std::collections::HashMap;

use crate::actions::movement::{MoveAction, MovementType};
use crate::actions::communication::{CommunicationAction, CommunicationType, TargetMode};
use crate::actions::archive::{ArchiveAction, ArchiveActionType};
use crate::actions::resource::{ResourceAction, ResourceActionType, resource_weights};
use crate::actions::social::{SocialAction, SocialActionType, social_weights};
use crate::actions::faction::{FactionAction, FactionActionType, faction_weights};
use crate::actions::conflict::{ConflictAction, ConflictActionType, conflict_weights};
use crate::actions::beer::{BeerAction, BeerActionType, beer_weights};
use crate::components::agent::{AgentId, AgentName, FoodSecurity, Goals, GoalType, Intoxication, Needs, Role, SocialBelonging, Traits};
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
    /// Resource-related actions (work, trade, steal, hoard)
    Resource(ResourceAction),
    /// Social relationship actions (build trust, curry favor, gift, ostracize)
    Social(SocialAction),
    /// Faction political actions (defect, exile, challenge, support)
    Faction(FactionAction),
    /// Conflict actions (argue, fight, sabotage, assassinate)
    Conflict(ConflictAction),
    /// Beer-related actions (brew, drink, share)
    Beer(BeerAction),
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

/// System to generate resource actions for agents
///
/// Generates work, trade, steal, and hoard actions based on location and needs
pub fn generate_resource_actions(
    faction_registry: Res<FactionRegistry>,
    agents_by_location: Res<AgentsByLocation>,
    relationship_graph: Res<RelationshipGraph>,
    mut pending_actions: ResMut<PendingActions>,
    query: Query<(&AgentId, &Position, &FactionMembership, &Needs, &Traits)>,
) {
    // Build agent info map
    let agent_info: HashMap<String, (&FactionMembership, &Needs, &Traits)> = query
        .iter()
        .map(|(id, _, mem, needs, traits)| (id.0.clone(), (mem, needs, traits)))
        .collect();

    for (agent_id, position, membership, needs, traits) in query.iter() {
        // Work action - available when at a productive location
        let faction = faction_registry.get(&membership.faction_id);
        let at_territory = faction.map_or(false, |f| f.territory.contains(&position.location_id));

        if at_territory {
            let mut weight = resource_weights::WORK_BASE;
            if needs.food_security == FoodSecurity::Stressed {
                weight += resource_weights::WORK_STRESSED_BONUS;
            } else if needs.food_security == FoodSecurity::Desperate {
                weight += resource_weights::WORK_DESPERATE_BONUS;
            }

            let action = ResourceAction::work(&agent_id.0);
            pending_actions.add(
                &agent_id.0,
                WeightedAction::new(Action::Resource(action), weight, "work for faction"),
            );
        }

        // Trade action - available when other agents nearby with decent relationship
        let nearby_agents = agents_by_location.at_location(&position.location_id);
        for target_id in nearby_agents {
            if target_id == &agent_id.0 {
                continue;
            }

            let trust = relationship_graph
                .get(&agent_id.0, target_id)
                .map(|r| r.trust.overall())
                .unwrap_or(0.0);

            // Only consider trade with somewhat trusted agents
            if trust > 0.1 {
                let mut weight = resource_weights::TRADE_BASE;
                weight += trust * resource_weights::TRADE_TRUST_BONUS;

                // Cross-faction trade bonus
                if let Some((target_mem, _, _)) = agent_info.get(target_id) {
                    if target_mem.faction_id != membership.faction_id {
                        weight += resource_weights::TRADE_CROSS_FACTION_BONUS;
                    }
                }

                let action = ResourceAction::trade(
                    &agent_id.0,
                    target_id,
                    crate::actions::resource::ResourceType::Grain,
                    3,
                );
                pending_actions.add(
                    &agent_id.0,
                    WeightedAction::new(Action::Resource(action), weight, format!("trade with {}", target_id)),
                );
            }
        }

        // Steal action - available when desperate and low honesty
        if needs.food_security == FoodSecurity::Desperate && traits.honesty < 0.5 {
            for target_id in nearby_agents {
                if target_id == &agent_id.0 {
                    continue;
                }

                // Don't steal from same faction unless very desperate and low loyalty
                if let Some((target_mem, _, _)) = agent_info.get(target_id) {
                    let same_faction = target_mem.faction_id == membership.faction_id;
                    if same_faction && traits.loyalty_weight > 0.3 {
                        continue;
                    }
                }

                let mut weight = resource_weights::STEAL_BASE;
                weight += resource_weights::STEAL_DESPERATE_BONUS;
                weight -= traits.honesty * resource_weights::STEAL_HONESTY_PENALTY;
                if traits.loyalty_weight < 0.3 {
                    weight += resource_weights::STEAL_LOW_LOYALTY_BONUS;
                }

                let action = ResourceAction::steal(
                    &agent_id.0,
                    target_id,
                    crate::actions::resource::ResourceType::Grain,
                    2,
                );
                pending_actions.add(
                    &agent_id.0,
                    WeightedAction::new(Action::Resource(action), weight.max(0.01), format!("steal from {}", target_id)),
                );
            }
        }

        // Hoard action - available when stressed and low loyalty
        if needs.food_security != FoodSecurity::Secure && traits.loyalty_weight < 0.5 {
            let mut weight = resource_weights::HOARD_BASE;
            if traits.loyalty_weight < 0.3 {
                weight += resource_weights::HOARD_LOW_LOYALTY_BONUS;
            }
            if needs.food_security == FoodSecurity::Stressed {
                weight += resource_weights::HOARD_STRESSED_BONUS;
            }
            weight -= traits.loyalty_weight * resource_weights::HOARD_HIGH_LOYALTY_PENALTY;

            let action = ResourceAction::hoard(&agent_id.0, 2);
            pending_actions.add(
                &agent_id.0,
                WeightedAction::new(Action::Resource(action), weight.max(0.01), "hoard resources"),
            );
        }
    }
}

/// System to generate social actions for agents
///
/// Generates build trust, curry favor, gift, and ostracize actions
pub fn generate_social_actions(
    agents_by_location: Res<AgentsByLocation>,
    relationship_graph: Res<RelationshipGraph>,
    mut pending_actions: ResMut<PendingActions>,
    query: Query<(&AgentId, &Position, &FactionMembership, &Needs, &Traits)>,
) {
    // Build agent info map
    let agent_info: HashMap<String, (&FactionMembership, &Needs, &Traits)> = query
        .iter()
        .map(|(id, _, mem, needs, traits)| (id.0.clone(), (mem, needs, traits)))
        .collect();

    for (agent_id, position, membership, needs, traits) in query.iter() {
        let nearby_agents = agents_by_location.at_location(&position.location_id);

        for target_id in nearby_agents {
            if target_id == &agent_id.0 {
                continue;
            }

            let relationship = relationship_graph.get(&agent_id.0, target_id);
            let trust = relationship.map(|r| r.trust.overall()).unwrap_or(0.0);

            let Some((target_mem, _, _)) = agent_info.get(target_id) else {
                continue;
            };

            // Build Trust action
            let mut build_trust_weight = social_weights::BUILD_TRUST_BASE;
            if needs.social_belonging != SocialBelonging::Integrated {
                build_trust_weight += social_weights::BUILD_TRUST_BELONGING_BONUS;
            }
            build_trust_weight += traits.sociability * social_weights::BUILD_TRUST_SOCIABILITY_MULT;
            if trust > 0.0 {
                build_trust_weight += social_weights::BUILD_TRUST_EXISTING_BONUS;
            }

            // Prefer same faction for trust building
            if target_mem.faction_id == membership.faction_id {
                build_trust_weight *= 1.3;
            }

            let action = SocialAction::build_trust(&agent_id.0, target_id);
            pending_actions.add(
                &agent_id.0,
                WeightedAction::new(
                    Action::Social(action),
                    build_trust_weight,
                    format!("build trust with {}", target_id),
                ),
            );

            // Curry Favor action - target higher status agents
            let target_status = target_mem.role.status_level().value();
            let my_status = membership.role.status_level().value();

            if target_status > my_status {
                let mut curry_weight = social_weights::CURRY_FAVOR_BASE;
                curry_weight += traits.ambition * social_weights::CURRY_FAVOR_AMBITION_MULT;

                if matches!(target_mem.role, Role::Leader) {
                    curry_weight += social_weights::CURRY_FAVOR_LEADER_BONUS;
                } else if matches!(target_mem.role, Role::CouncilMember | Role::Reader) {
                    curry_weight += social_weights::CURRY_FAVOR_COUNCIL_BONUS;
                }

                let action = SocialAction::curry_favor(&agent_id.0, target_id);
                pending_actions.add(
                    &agent_id.0,
                    WeightedAction::new(
                        Action::Social(action),
                        curry_weight,
                        format!("curry favor with {}", target_id),
                    ),
                );
            }

            // Gift action - for repairing relationships or gaining favor quickly
            if trust < 0.0 || (traits.sociability > 0.6 && trust < 0.5) {
                let mut gift_weight = social_weights::GIFT_BASE;
                if trust < 0.0 {
                    gift_weight += social_weights::GIFT_REPAIR_BONUS;
                }

                let action = SocialAction::gift(
                    &agent_id.0,
                    target_id,
                    social_weights::GIFT_STANDARD_COST,
                );
                pending_actions.add(
                    &agent_id.0,
                    WeightedAction::new(
                        Action::Social(action),
                        gift_weight,
                        format!("give gift to {}", target_id),
                    ),
                );
            }

            // Ostracize action - for agents with grudges or low trust
            if trust < -0.2 && traits.honesty < 0.7 {
                let mut ostracize_weight = social_weights::OSTRACIZE_BASE;
                ostracize_weight += social_weights::OSTRACIZE_GRUDGE_BONUS;
                if my_status > target_status {
                    ostracize_weight += social_weights::OSTRACIZE_STATUS_BONUS;
                }
                ostracize_weight -= traits.honesty * social_weights::OSTRACIZE_HONESTY_PENALTY;

                let action = SocialAction::ostracize(&agent_id.0, target_id);
                pending_actions.add(
                    &agent_id.0,
                    WeightedAction::new(
                        Action::Social(action),
                        ostracize_weight.max(0.01),
                        format!("ostracize {}", target_id),
                    ),
                );
            }
        }
    }
}

/// System to generate faction political actions
///
/// Generates defect, exile, challenge leader, and support leader actions
pub fn generate_faction_actions(
    faction_registry: Res<FactionRegistry>,
    relationship_graph: Res<RelationshipGraph>,
    mut pending_actions: ResMut<PendingActions>,
    query: Query<(&AgentId, &Position, &FactionMembership, &Needs, &Traits, &Goals)>,
) {
    // Collect faction member counts and leader trust
    let mut faction_leader_trust: HashMap<String, Vec<f32>> = HashMap::new();

    for (agent_id, _, membership, _, _, _) in query.iter() {
        if let Some(faction) = faction_registry.get(&membership.faction_id) {
            if let Some(leader_id) = &faction.leader {
                if leader_id != &agent_id.0 {
                    let trust = relationship_graph
                        .get(&agent_id.0, leader_id)
                        .map(|r| r.trust.overall())
                        .unwrap_or(0.0);
                    faction_leader_trust
                        .entry(membership.faction_id.clone())
                        .or_default()
                        .push(trust);
                }
            }
        }
    }

    // Calculate average leader trust per faction
    let avg_leader_trust: HashMap<String, f32> = faction_leader_trust
        .iter()
        .map(|(f, trusts)| {
            let avg = if trusts.is_empty() {
                0.5
            } else {
                trusts.iter().sum::<f32>() / trusts.len() as f32
            };
            (f.clone(), avg)
        })
        .collect();

    for (agent_id, _position, membership, needs, traits, goals) in query.iter() {
        let faction = faction_registry.get(&membership.faction_id);

        // Get leader trust
        let leader_trust = faction.and_then(|f| f.leader.as_ref()).map(|leader_id| {
            relationship_graph
                .get(&agent_id.0, leader_id)
                .map(|r| r.trust.overall())
                .unwrap_or(0.0)
        }).unwrap_or(0.0);

        // Defect action - rare, requires poor conditions
        if needs.social_belonging == SocialBelonging::Isolated
            || (needs.social_belonging == SocialBelonging::Peripheral && leader_trust < -0.1)
        {
            // Find potential new factions
            for new_faction in faction_registry.all_factions() {
                if new_faction.id.0 == membership.faction_id {
                    continue;
                }

                let mut weight = faction_weights::DEFECT_BASE;
                weight += (1.0 - traits.loyalty_weight) * faction_weights::DEFECT_LOW_LOYALTY_BONUS;

                if needs.social_belonging == SocialBelonging::Isolated {
                    weight += faction_weights::DEFECT_ISOLATED_BONUS;
                }
                if leader_trust < -0.2 {
                    weight += faction_weights::DEFECT_LEADER_DISTRUST_BONUS;
                }
                if traits.loyalty_weight > 0.7 {
                    weight -= faction_weights::DEFECT_HIGH_LOYALTY_PENALTY;
                }

                let action = FactionAction::defect(
                    &agent_id.0,
                    &membership.faction_id,
                    &new_faction.id.0,
                );
                pending_actions.add(
                    &agent_id.0,
                    WeightedAction::new(
                        Action::Faction(action),
                        weight.max(0.001),
                        format!("defect to {}", new_faction.name),
                    ),
                );
            }
        }

        // Exile action - for leaders/council only
        if matches!(membership.role, Role::Leader | Role::CouncilMember) {
            // Could add exile generation here based on trust toward specific agents
            // For now, skipping detailed exile generation
        }

        // Challenge Leader action - requires high ambition and weak leader
        let avg_trust = avg_leader_trust.get(&membership.faction_id).copied().unwrap_or(0.5);
        let leader_is_weak = avg_trust < faction_weights::WEAK_LEADER_TRUST_THRESHOLD;

        if traits.ambition > 0.6 && !membership.is_leader() {
            let mut weight = faction_weights::CHALLENGE_BASE;
            weight += traits.ambition * faction_weights::CHALLENGE_AMBITION_MULT;

            if leader_is_weak {
                weight += faction_weights::CHALLENGE_WEAK_LEADER_BONUS;
            }
            if traits.loyalty_weight > 0.7 {
                weight -= faction_weights::CHALLENGE_HIGH_LOYALTY_PENALTY;
            }

            // Check if agent has ChallengeLeader goal
            if goals.has_goal(&GoalType::ChallengeLeader) {
                weight *= 3.0; // Strongly boost if this is an active goal
            }

            let action = FactionAction::challenge_leader(&agent_id.0, &membership.faction_id);
            pending_actions.add(
                &agent_id.0,
                WeightedAction::new(
                    Action::Faction(action),
                    weight.max(0.001),
                    "challenge faction leader",
                ),
            );
        }

        // Support Leader action - for loyal agents when leader exists
        if let Some(faction) = faction {
            if faction.leader.is_some() && !membership.is_leader() {
                let mut weight = faction_weights::SUPPORT_BASE;
                weight += traits.loyalty_weight * faction_weights::SUPPORT_HIGH_LOYALTY_BONUS;

                if leader_trust > 0.3 {
                    weight += faction_weights::SUPPORT_LEADER_TRUST_BONUS;
                }

                // Check if agent has SupportLeader goal
                if goals.has_goal(&GoalType::SupportLeader) {
                    weight *= 2.0;
                }

                let action = FactionAction::support_leader(&agent_id.0, &membership.faction_id);
                pending_actions.add(
                    &agent_id.0,
                    WeightedAction::new(
                        Action::Faction(action),
                        weight,
                        "support faction leader",
                    ),
                );
            }
        }
    }
}

/// System to generate conflict actions
///
/// Generates argue, fight, sabotage, and assassinate actions
pub fn generate_conflict_actions(
    agents_by_location: Res<AgentsByLocation>,
    relationship_graph: Res<RelationshipGraph>,
    mut pending_actions: ResMut<PendingActions>,
    query: Query<(&AgentId, &Position, &FactionMembership, &Needs, &Traits, &Goals)>,
) {
    for (agent_id, position, membership, needs, traits, goals) in query.iter() {
        let nearby_agents = agents_by_location.at_location(&position.location_id);

        // Check for revenge goal
        let revenge_goal = goals.get_goal(&GoalType::Revenge);
        let revenge_target = revenge_goal.and_then(|g| g.target.clone());

        for target_id in nearby_agents {
            if target_id == &agent_id.0 {
                continue;
            }

            let trust = relationship_graph
                .get(&agent_id.0, target_id)
                .map(|r| r.trust.overall())
                .unwrap_or(0.0);

            let has_grudge = trust < -0.2;
            let is_revenge_target = revenge_target.as_ref() == Some(target_id);

            // Argue action - verbal conflict
            if has_grudge || is_revenge_target {
                let mut weight = conflict_weights::ARGUE_BASE;
                weight += conflict_weights::ARGUE_NEGATIVE_REL_BONUS;
                if is_revenge_target {
                    weight += conflict_weights::ARGUE_GRUDGE_BONUS;
                }
                weight += traits.boldness * conflict_weights::ARGUE_BOLDNESS_MULT;

                let action = ConflictAction::argue(
                    &agent_id.0,
                    target_id,
                    Some("grievance".to_string()),
                );
                pending_actions.add(
                    &agent_id.0,
                    WeightedAction::new(
                        Action::Conflict(action),
                        weight,
                        format!("argue with {}", target_id),
                    ),
                );
            }

            // Fight action - physical violence (rare)
            if (is_revenge_target || trust < -0.4) && traits.boldness > 0.5 {
                let mut weight = conflict_weights::FIGHT_BASE;
                if is_revenge_target {
                    weight += conflict_weights::FIGHT_REVENGE_BONUS;
                }
                weight += traits.boldness * conflict_weights::FIGHT_BOLDNESS_MULT;
                if traits.boldness < 0.3 {
                    weight -= conflict_weights::FIGHT_LOW_BOLDNESS_PENALTY;
                }

                let action = ConflictAction::fight(
                    &agent_id.0,
                    target_id,
                    Some("hostility".to_string()),
                );
                pending_actions.add(
                    &agent_id.0,
                    WeightedAction::new(
                        Action::Conflict(action),
                        weight.max(0.001),
                        format!("fight {}", target_id),
                    ),
                );
            }

            // Sabotage action - sneaky retaliation
            if has_grudge && traits.honesty < 0.5 {
                let mut weight = conflict_weights::SABOTAGE_BASE;
                if is_revenge_target {
                    weight += conflict_weights::SABOTAGE_REVENGE_BONUS;
                }
                weight -= traits.honesty * conflict_weights::SABOTAGE_HONESTY_PENALTY;
                weight += conflict_weights::SABOTAGE_NEGATIVE_REL_BONUS;

                let action = ConflictAction::sabotage(
                    &agent_id.0,
                    target_id,
                    Some("revenge".to_string()),
                );
                pending_actions.add(
                    &agent_id.0,
                    WeightedAction::new(
                        Action::Conflict(action),
                        weight.max(0.001),
                        format!("sabotage {}", target_id),
                    ),
                );
            }

            // Assassinate action - extreme violence (very rare)
            if is_revenge_target
                && trust < conflict_weights::ASSASSINATE_MIN_DISTRUST
                && needs.social_belonging == SocialBelonging::Isolated
            {
                let mut weight = conflict_weights::ASSASSINATE_BASE;
                weight += conflict_weights::ASSASSINATE_REVENGE_BONUS;
                weight += conflict_weights::ASSASSINATE_ISOLATED_BONUS;

                if needs.food_security == FoodSecurity::Desperate {
                    weight += conflict_weights::ASSASSINATE_DESPERATE_BONUS;
                }

                let action = ConflictAction::assassinate(
                    &agent_id.0,
                    target_id,
                    revenge_goal.and_then(|g| g.origin_event.clone()).unwrap_or_default(),
                );
                pending_actions.add(
                    &agent_id.0,
                    WeightedAction::new(
                        Action::Conflict(action),
                        weight.max(0.0001),
                        format!("assassinate {}", target_id),
                    ),
                );
            }
        }
    }
}

/// System to generate beer-related actions (brew, drink, share)
///
/// Generates actions for brewing beer from grain, drinking for social benefits,
/// and sharing beer with others to build trust.
pub fn generate_beer_actions(
    faction_registry: Res<FactionRegistry>,
    agents_by_location: Res<AgentsByLocation>,
    mut pending_actions: ResMut<PendingActions>,
    query: Query<(&AgentId, &Position, &FactionMembership, &Needs, &Traits, &Intoxication)>,
) {
    for (agent_id, position, membership, needs, traits, intoxication) in query.iter() {
        let Some(faction) = faction_registry.get(&membership.faction_id) else {
            continue;
        };

        let at_territory = faction.territory.contains(&position.location_id);

        // Brew action - available when at territory with sufficient grain
        if at_territory && faction.resources.grain >= beer_weights::BREW_GRAIN_THRESHOLD {
            let mut weight = beer_weights::BREW_BASE;

            // Bonus if we have lots of grain (> 200 per territory)
            let territory_count = faction.territory.len().max(1) as u32;
            let grain_per_territory = faction.resources.grain / territory_count;
            if grain_per_territory > 200 {
                weight += beer_weights::BREW_EXCESS_GRAIN_BONUS;
            }

            // Penalty if grain is scarce
            if faction.resources.is_critical() {
                weight -= beer_weights::BREW_SCARCE_GRAIN_PENALTY;
            }

            // Sociable agents brew more
            weight += traits.sociability * 0.1;

            let action = BeerAction::brew(&agent_id.0, 1);
            pending_actions.add(
                &agent_id.0,
                WeightedAction::new(
                    Action::Beer(action),
                    weight.max(0.01),
                    "brew beer",
                ),
            );
        }

        // Drink action - available when faction has beer and not too intoxicated
        if faction.resources.beer > 0 && intoxication.level < beer_weights::DRINK_MAX_INTOX {
            let mut weight = beer_weights::DRINK_BASE;
            weight += traits.sociability * beer_weights::DRINK_SOCIABILITY_BONUS;

            // Bonus for seeking belonging (peripheral/isolated agents)
            if needs.social_belonging != SocialBelonging::Integrated {
                weight += beer_weights::DRINK_BELONGING_BONUS;
            }

            // Bold agents drink more
            weight += traits.boldness * 0.1;

            let action = BeerAction::drink(&agent_id.0);
            pending_actions.add(
                &agent_id.0,
                WeightedAction::new(
                    Action::Beer(action),
                    weight,
                    "drink beer",
                ),
            );
        }

        // Share action - available when beer exists and others nearby
        if faction.resources.beer > 1 {
            let nearby_agents = agents_by_location.at_location(&position.location_id);

            for target_id in nearby_agents {
                if target_id == &agent_id.0 {
                    continue;
                }

                let mut weight = beer_weights::SHARE_BASE;
                weight += traits.sociability * beer_weights::SHARE_SOCIABILITY_BONUS;

                // Generous (low ambition) agents share more
                weight += (1.0 - traits.ambition) * 0.1;

                let action = BeerAction::share(&agent_id.0, target_id);
                pending_actions.add(
                    &agent_id.0,
                    WeightedAction::new(
                        Action::Beer(action),
                        weight,
                        format!("share beer with {}", target_id),
                    ),
                );
            }
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
