//! Intervention System
//!
//! Allows runtime modification of the simulation by watching for JSON files
//! in the interventions/ directory.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::components::agent::{
    AgentId, AgentName, FoodSecurity, Goals, GoalType, Needs, Role, SocialBelonging, Traits,
};
use crate::components::faction::{FactionMembership, FactionRegistry};
use crate::components::social::RelationshipGraph;
use crate::components::world::{Position, WorldState};
use crate::events::types::{
    ActorSnapshot, Event, EventActors, EventContext, EventOutcome, EventSubtype, EventTimestamp,
    EventType, GeneralOutcome,
};
use crate::systems::action::TickEvents;

/// Directory to watch for intervention files
pub const INTERVENTIONS_DIR: &str = "interventions";

/// Types of interventions that can be applied
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InterventionType {
    /// Modify an existing agent's properties
    ModifyAgent {
        agent_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        traits: Option<TraitModification>,
        #[serde(skip_serializing_if = "Option::is_none")]
        needs: Option<NeedModification>,
        #[serde(skip_serializing_if = "Option::is_none")]
        goals: Option<Vec<GoalModification>>,
    },
    /// Modify a faction's properties
    ModifyFaction {
        faction_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        resources: Option<ResourceModification>,
        #[serde(skip_serializing_if = "Option::is_none")]
        new_leader: Option<String>,
    },
    /// Modify a relationship between two agents
    ModifyRelationship {
        from_agent: String,
        to_agent: String,
        reliability: Option<f32>,
        alignment: Option<f32>,
        capability: Option<f32>,
    },
    /// Move an agent to a new location
    MoveAgent {
        agent_id: String,
        location_id: String,
    },
    /// Force an agent to change factions
    ChangeFaction {
        agent_id: String,
        new_faction_id: String,
        new_role: Option<String>,
    },
    /// Add a goal to an agent
    AddGoal {
        agent_id: String,
        goal_type: String,
        target: Option<String>,
        priority: Option<f32>,
    },
}

/// Modification to agent traits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitModification {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loyalty_weight: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ambition: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub honesty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boldness: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sociability: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grudge_persistence: Option<f32>,
}

/// Modification to agent needs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeedModification {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub food_security: Option<String>, // "secure", "stressed", "desperate"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub social_belonging: Option<String>, // "integrated", "peripheral", "isolated"
}

/// Modification to goals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalModification {
    pub goal_type: String,
    pub target: Option<String>,
    pub priority: f32,
}

/// Modification to faction resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceModification {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grain: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iron: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub salt: Option<i32>,
}

/// A complete intervention request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intervention {
    /// Unique ID for this intervention
    pub id: String,
    /// Description of why this intervention is being made
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// The actual intervention to apply
    pub intervention: InterventionType,
}

/// Resource for tracking pending interventions
#[derive(Resource, Default)]
pub struct PendingInterventions {
    pub interventions: Vec<(String, Intervention)>, // (filename, intervention)
}

impl PendingInterventions {
    pub fn new() -> Self {
        Self {
            interventions: Vec::new(),
        }
    }
}

/// System to scan for and load intervention files
pub fn scan_interventions(mut pending: ResMut<PendingInterventions>) {
    // Clear any leftover from previous tick
    pending.interventions.clear();

    let interventions_path = Path::new(INTERVENTIONS_DIR);

    // Create directory if it doesn't exist
    if !interventions_path.exists() {
        if let Err(e) = fs::create_dir_all(interventions_path) {
            eprintln!("Warning: Could not create interventions directory: {}", e);
            return;
        }
    }

    // Scan for JSON files
    let entries = match fs::read_dir(interventions_path) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<Intervention>(&content) {
                    Ok(intervention) => {
                        let filename = path
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        pending.interventions.push((filename, intervention));
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Could not parse intervention file {:?}: {}",
                            path, e
                        );
                    }
                },
                Err(e) => {
                    eprintln!(
                        "Warning: Could not read intervention file {:?}: {}",
                        path, e
                    );
                }
            }
        }
    }
}

/// System to apply pending interventions
pub fn apply_interventions(
    world_state: Res<WorldState>,
    mut pending: ResMut<PendingInterventions>,
    mut tick_events: ResMut<TickEvents>,
    mut faction_registry: ResMut<FactionRegistry>,
    mut relationship_graph: ResMut<RelationshipGraph>,
    mut agents: Query<(
        &AgentId,
        &AgentName,
        &mut Traits,
        &mut Needs,
        &mut Goals,
        &mut Position,
        &mut FactionMembership,
    )>,
) {
    let interventions: Vec<_> = pending.interventions.drain(..).collect();

    for (filename, intervention) in interventions {
        let success = apply_single_intervention(
            &intervention,
            &world_state,
            &mut tick_events,
            &mut faction_registry,
            &mut relationship_graph,
            &mut agents,
        );

        if success {
            // Delete the intervention file
            let filepath = Path::new(INTERVENTIONS_DIR).join(&filename);
            if let Err(e) = fs::remove_file(&filepath) {
                eprintln!("Warning: Could not delete intervention file {:?}: {}", filepath, e);
            }
        }
    }
}

/// Apply a single intervention and return whether it succeeded
fn apply_single_intervention(
    intervention: &Intervention,
    world_state: &WorldState,
    tick_events: &mut TickEvents,
    faction_registry: &mut FactionRegistry,
    relationship_graph: &mut RelationshipGraph,
    agents: &mut Query<(
        &AgentId,
        &AgentName,
        &mut Traits,
        &mut Needs,
        &mut Goals,
        &mut Position,
        &mut FactionMembership,
    )>,
) -> bool {
    match &intervention.intervention {
        InterventionType::ModifyAgent {
            agent_id,
            traits,
            needs,
            goals,
        } => {
            for (id, name, mut agent_traits, mut agent_needs, mut agent_goals, _, membership) in
                agents.iter_mut()
            {
                if &id.0 == agent_id {
                    // Apply trait modifications
                    if let Some(t) = traits {
                        if let Some(v) = t.loyalty_weight {
                            agent_traits.loyalty_weight = v.clamp(0.0, 1.0);
                        }
                        if let Some(v) = t.ambition {
                            agent_traits.ambition = v.clamp(0.0, 1.0);
                        }
                        if let Some(v) = t.honesty {
                            agent_traits.honesty = v.clamp(0.0, 1.0);
                        }
                        if let Some(v) = t.boldness {
                            agent_traits.boldness = v.clamp(0.0, 1.0);
                        }
                        if let Some(v) = t.sociability {
                            agent_traits.sociability = v.clamp(0.0, 1.0);
                        }
                        if let Some(v) = t.grudge_persistence {
                            agent_traits.grudge_persistence = v.clamp(0.0, 1.0);
                        }
                    }

                    // Apply need modifications
                    if let Some(n) = needs {
                        if let Some(ref fs) = n.food_security {
                            agent_needs.food_security = match fs.as_str() {
                                "desperate" => FoodSecurity::Desperate,
                                "stressed" => FoodSecurity::Stressed,
                                _ => FoodSecurity::Secure,
                            };
                        }
                        if let Some(ref sb) = n.social_belonging {
                            agent_needs.social_belonging = match sb.as_str() {
                                "isolated" => SocialBelonging::Isolated,
                                "peripheral" => SocialBelonging::Peripheral,
                                _ => SocialBelonging::Integrated,
                            };
                        }
                    }

                    // Apply goal modifications
                    if let Some(goal_list) = goals {
                        for g in goal_list {
                            let goal_type = match g.goal_type.as_str() {
                                "revenge" => GoalType::Revenge,
                                "challenge_leader" => GoalType::ChallengeLeader,
                                "support_leader" => GoalType::SupportLeader,
                                "rise_in_status" => GoalType::RiseInStatus,
                                "survive" => GoalType::Survive,
                                "survive_winter" => GoalType::SurviveWinter,
                                "protect" => GoalType::Protect,
                                _ => continue,
                            };
                            let mut goal = crate::components::agent::Goal::new(goal_type, g.priority);
                            goal.target = g.target.clone();
                            agent_goals.add(goal);
                        }
                    }

                    // Log intervention event
                    let event = create_intervention_event(
                        tick_events,
                        world_state,
                        &intervention.id,
                        &format!("Modified agent {}", name.0),
                        intervention.reason.as_deref(),
                        agent_id,
                        &name.0,
                        &membership.faction_id,
                    );
                    tick_events.push(event);

                    return true;
                }
            }
            eprintln!("Warning: Agent {} not found for intervention", agent_id);
            false
        }

        InterventionType::ModifyFaction {
            faction_id,
            resources,
            new_leader,
        } => {
            if let Some(faction) = faction_registry.get_mut(faction_id) {
                if let Some(r) = resources {
                    if let Some(g) = r.grain {
                        faction.resources.grain = (faction.resources.grain as i32 + g).max(0) as u32;
                    }
                    if let Some(i) = r.iron {
                        faction.resources.iron = (faction.resources.iron as i32 + i).max(0) as u32;
                    }
                    if let Some(s) = r.salt {
                        faction.resources.salt = (faction.resources.salt as i32 + s).max(0) as u32;
                    }
                }

                if let Some(leader) = new_leader {
                    faction.leader = Some(leader.clone());
                }

                let event = create_intervention_event(
                    tick_events,
                    world_state,
                    &intervention.id,
                    &format!("Modified faction {}", faction_id),
                    intervention.reason.as_deref(),
                    faction_id,
                    faction_id,
                    faction_id,
                );
                tick_events.push(event);

                true
            } else {
                eprintln!("Warning: Faction {} not found for intervention", faction_id);
                false
            }
        }

        InterventionType::ModifyRelationship {
            from_agent,
            to_agent,
            reliability,
            alignment,
            capability,
        } => {
            let rel = relationship_graph.ensure_relationship(from_agent, to_agent);

            if let Some(r) = reliability {
                rel.trust.reliability = r.clamp(-1.0, 1.0);
            }
            if let Some(a) = alignment {
                rel.trust.alignment = a.clamp(-1.0, 1.0);
            }
            if let Some(c) = capability {
                rel.trust.capability = c.clamp(-1.0, 1.0);
            }

            let event = create_intervention_event(
                tick_events,
                world_state,
                &intervention.id,
                &format!("Modified relationship {} -> {}", from_agent, to_agent),
                intervention.reason.as_deref(),
                from_agent,
                from_agent,
                "unknown",
            );
            tick_events.push(event);

            true
        }

        InterventionType::MoveAgent {
            agent_id,
            location_id,
        } => {
            for (id, name, _, _, _, mut position, membership) in agents.iter_mut() {
                if &id.0 == agent_id {
                    position.location_id = location_id.clone();

                    let event = create_intervention_event(
                        tick_events,
                        world_state,
                        &intervention.id,
                        &format!("Moved {} to {}", name.0, location_id),
                        intervention.reason.as_deref(),
                        agent_id,
                        &name.0,
                        &membership.faction_id,
                    );
                    tick_events.push(event);

                    return true;
                }
            }
            eprintln!("Warning: Agent {} not found for move intervention", agent_id);
            false
        }

        InterventionType::ChangeFaction {
            agent_id,
            new_faction_id,
            new_role,
        } => {
            for (id, name, _, _, _, _, mut membership) in agents.iter_mut() {
                if &id.0 == agent_id {
                    let old_faction = membership.faction_id.clone();
                    membership.faction_id = new_faction_id.clone();

                    if let Some(role_str) = new_role {
                        membership.role = match role_str.as_str() {
                            "leader" => Role::Leader,
                            "council_member" => Role::CouncilMember,
                            "reader" => Role::Reader,
                            "scout_captain" => Role::ScoutCaptain,
                            "healer" => Role::Healer,
                            "smith" => Role::Smith,
                            "skilled_worker" => Role::SkilledWorker,
                            "laborer" => Role::Laborer,
                            _ => Role::Newcomer,
                        };
                    } else {
                        membership.role = Role::Newcomer;
                    }

                    let event = create_intervention_event(
                        tick_events,
                        world_state,
                        &intervention.id,
                        &format!(
                            "{} changed faction from {} to {}",
                            name.0, old_faction, new_faction_id
                        ),
                        intervention.reason.as_deref(),
                        agent_id,
                        &name.0,
                        new_faction_id,
                    );
                    tick_events.push(event);

                    return true;
                }
            }
            eprintln!(
                "Warning: Agent {} not found for faction change intervention",
                agent_id
            );
            false
        }

        InterventionType::AddGoal {
            agent_id,
            goal_type,
            target,
            priority,
        } => {
            for (id, name, _, _, mut goals, _, membership) in agents.iter_mut() {
                if &id.0 == agent_id {
                    let gt = match goal_type.as_str() {
                        "revenge" => GoalType::Revenge,
                        "challenge_leader" => GoalType::ChallengeLeader,
                        "support_leader" => GoalType::SupportLeader,
                        "rise_in_status" => GoalType::RiseInStatus,
                        "survive" => GoalType::Survive,
                        "survive_winter" => GoalType::SurviveWinter,
                        "protect" => GoalType::Protect,
                        _ => {
                            eprintln!("Warning: Unknown goal type {}", goal_type);
                            return false;
                        }
                    };

                    let mut goal = crate::components::agent::Goal::new(gt, priority.unwrap_or(0.5));
                    goal.target = target.clone();
                    goals.add(goal);

                    let event = create_intervention_event(
                        tick_events,
                        world_state,
                        &intervention.id,
                        &format!("Added {} goal to {}", goal_type, name.0),
                        intervention.reason.as_deref(),
                        agent_id,
                        &name.0,
                        &membership.faction_id,
                    );
                    tick_events.push(event);

                    return true;
                }
            }
            eprintln!(
                "Warning: Agent {} not found for add goal intervention",
                agent_id
            );
            false
        }
    }
}

/// Create an event for logging an intervention
fn create_intervention_event(
    tick_events: &mut TickEvents,
    world_state: &WorldState,
    intervention_id: &str,
    description: &str,
    reason: Option<&str>,
    agent_id: &str,
    agent_name: &str,
    faction: &str,
) -> Event {
    let event_id = tick_events.generate_id();
    let timestamp = EventTimestamp {
        tick: world_state.current_tick,
        date: world_state.formatted_date(),
    };

    let actor = ActorSnapshot {
        agent_id: agent_id.to_string(),
        name: agent_name.to_string(),
        faction: faction.to_string(),
        role: "intervention_target".to_string(),
        location: "unknown".to_string(),
    };

    let full_description = match reason {
        Some(r) => format!("{} (Reason: {})", description, r),
        None => description.to_string(),
    };

    Event {
        event_id,
        timestamp,
        event_type: EventType::Birth, // Using Birth as closest to "external creation/modification"
        subtype: EventSubtype::Birth(crate::events::types::BirthSubtype::Created),
        actors: EventActors {
            primary: actor,
            secondary: None,
            affected: None,
        },
        context: EventContext {
            trigger: format!("intervention:{}", intervention_id),
            preconditions: Vec::new(),
            location_description: None,
        },
        outcome: EventOutcome::General(GeneralOutcome {
            description: Some(full_description),
            state_changes: Vec::new(),
        }),
        drama_tags: vec!["intervention".to_string(), "external_modification".to_string()],
        drama_score: 0.3,
        connected_events: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intervention_parsing() {
        let json = r#"{
            "id": "test_001",
            "reason": "Test intervention",
            "intervention": {
                "type": "modify_agent",
                "agent_id": "agent_mira_0001",
                "traits": {
                    "loyalty_weight": 0.2,
                    "ambition": 0.9
                },
                "needs": {
                    "food_security": "desperate"
                }
            }
        }"#;

        let intervention: Intervention = serde_json::from_str(json).unwrap();
        assert_eq!(intervention.id, "test_001");

        match intervention.intervention {
            InterventionType::ModifyAgent {
                agent_id,
                traits,
                needs,
                goals: _,
            } => {
                assert_eq!(agent_id, "agent_mira_0001");
                let t = traits.unwrap();
                assert_eq!(t.loyalty_weight, Some(0.2));
                assert_eq!(t.ambition, Some(0.9));
                let n = needs.unwrap();
                assert_eq!(n.food_security, Some("desperate".to_string()));
            }
            _ => panic!("Wrong intervention type"),
        }
    }

    #[test]
    fn test_modify_relationship_parsing() {
        let json = r#"{
            "id": "rel_001",
            "intervention": {
                "type": "modify_relationship",
                "from_agent": "agent_001",
                "to_agent": "agent_002",
                "reliability": -0.5,
                "alignment": -0.3
            }
        }"#;

        let intervention: Intervention = serde_json::from_str(json).unwrap();

        match intervention.intervention {
            InterventionType::ModifyRelationship {
                from_agent,
                to_agent,
                reliability,
                alignment,
                capability,
            } => {
                assert_eq!(from_agent, "agent_001");
                assert_eq!(to_agent, "agent_002");
                assert_eq!(reliability, Some(-0.5));
                assert_eq!(alignment, Some(-0.3));
                assert_eq!(capability, None);
            }
            _ => panic!("Wrong intervention type"),
        }
    }

    #[test]
    fn test_change_faction_parsing() {
        let json = r#"{
            "id": "faction_001",
            "reason": "Defection scenario",
            "intervention": {
                "type": "change_faction",
                "agent_id": "agent_traitor",
                "new_faction_id": "ironmere",
                "new_role": "member"
            }
        }"#;

        let intervention: Intervention = serde_json::from_str(json).unwrap();

        match intervention.intervention {
            InterventionType::ChangeFaction {
                agent_id,
                new_faction_id,
                new_role,
            } => {
                assert_eq!(agent_id, "agent_traitor");
                assert_eq!(new_faction_id, "ironmere");
                assert_eq!(new_role, Some("member".to_string()));
            }
            _ => panic!("Wrong intervention type"),
        }
    }
}
