//! Tension Detection System
//!
//! Identifies developing dramatic situations for the Director AI to focus on.
//! Tensions are higher-level patterns detected from agent states and relationships.

use bevy_ecs::prelude::*;
use std::collections::HashMap;

use crate::components::agent::{AgentId, AgentName, Goals, GoalType, Traits};
use crate::components::faction::{FactionMembership, FactionRegistry};
use crate::components::social::{RelationshipGraph, MemoryBank};
use crate::components::world::WorldState;
use crate::output::tension::{Tension, TensionStream, TensionType};

/// Threshold for trust to be considered "eroding" toward betrayal
const BETRAYAL_TRUST_THRESHOLD: f32 = -0.2;

/// Threshold for leader trust to indicate succession crisis
const SUCCESSION_TRUST_THRESHOLD: f32 = 0.1;

/// Minimum number of disgruntled agents for faction fracture
const FRACTURE_MIN_AGENTS: usize = 3;

/// Cross-faction trust threshold for forbidden alliance
const ALLIANCE_TRUST_THRESHOLD: f32 = 0.3;

/// Detection interval in ticks (don't run every tick for performance)
const DETECTION_INTERVAL: u64 = 10;

/// Agent data collected for tension detection
struct AgentData {
    id: String,
    name: String,
    faction_id: String,
    goals: Goals,
    traits: Traits,
}

/// System to detect new tensions and update existing ones
pub fn detect_tensions(
    world_state: Res<WorldState>,
    mut tension_stream: ResMut<TensionStream>,
    relationship_graph: Res<RelationshipGraph>,
    memory_bank: Res<MemoryBank>,
    faction_registry: Res<FactionRegistry>,
    query: Query<(&AgentId, &AgentName, &FactionMembership, &Goals, &Traits)>,
) {
    // Only run detection periodically
    if world_state.current_tick % DETECTION_INTERVAL != 0 {
        return;
    }

    let current_tick = world_state.current_tick;

    // Build lookup maps
    let mut agents_by_faction: HashMap<String, Vec<AgentData>> = HashMap::new();
    let mut all_agents: Vec<AgentData> = Vec::new();

    for (id, name, membership, goals, traits) in query.iter() {
        let agent_data = AgentData {
            id: id.0.clone(),
            name: name.0.clone(),
            faction_id: membership.faction_id.clone(),
            goals: goals.clone(),
            traits: traits.clone(),
        };
        agents_by_faction
            .entry(membership.faction_id.clone())
            .or_default()
            .push(AgentData {
                id: id.0.clone(),
                name: name.0.clone(),
                faction_id: membership.faction_id.clone(),
                goals: goals.clone(),
                traits: traits.clone(),
            });
        all_agents.push(agent_data);
    }

    // Detect tensions for each faction
    for faction_id in faction_registry.faction_ids() {
        let faction = match faction_registry.get(faction_id) {
            Some(f) => f,
            None => continue,
        };

        let faction_agents = agents_by_faction.get(faction_id).map(|v| v.as_slice()).unwrap_or(&[]);

        // 1. Detect Brewing Betrayal
        detect_brewing_betrayal(
            &mut tension_stream,
            &relationship_graph,
            faction_agents,
            &faction.leader,
            current_tick,
        );

        // 2. Detect Succession Crisis
        detect_succession_crisis(
            &mut tension_stream,
            &relationship_graph,
            faction_agents,
            &faction.leader,
            faction_id,
            &faction.name,
            current_tick,
        );

        // 3. Detect Resource Conflict
        detect_resource_conflict(
            &mut tension_stream,
            faction_id,
            &faction.name,
            &faction.resources,
            current_tick,
        );

        // 4. Detect Faction Fracture
        detect_faction_fracture(
            &mut tension_stream,
            &relationship_graph,
            faction_agents,
            &faction.leader,
            faction_id,
            &faction.name,
            current_tick,
        );
    }

    // 5. Detect Forbidden Alliances
    detect_forbidden_alliances(
        &mut tension_stream,
        &relationship_graph,
        &all_agents,
        current_tick,
    );

    // 6. Detect Revenge Arcs
    detect_revenge_arcs(
        &mut tension_stream,
        &all_agents,
        current_tick,
    );

    // 7. Detect Rising Power
    detect_rising_power(
        &mut tension_stream,
        &all_agents,
        current_tick,
    );

    // 8. Detect Secret Exposed (from memories)
    detect_secret_exposed(
        &mut tension_stream,
        &memory_bank,
        &all_agents,
        current_tick,
    );

    // 9. Detect External Threat
    detect_external_threat(
        &mut tension_stream,
        &world_state,
        current_tick,
    );

    // Update existing tensions
    update_tension_statuses(&mut tension_stream);
}

/// Detect brewing betrayal: agent with low trust in leader + high ambition
fn detect_brewing_betrayal(
    tension_stream: &mut TensionStream,
    relationships: &RelationshipGraph,
    faction_agents: &[AgentData],
    leader_id: &Option<String>,
    current_tick: u64,
) {
    let leader = match leader_id {
        Some(id) => id,
        None => return, // No leader, no betrayal
    };

    for agent in faction_agents {
        if &agent.id == leader {
            continue; // Leader can't betray themselves
        }

        // Check trust toward leader
        if let Some(rel) = relationships.get(&agent.id, leader) {
            let trust = rel.trust.overall();

            // Low trust + high ambition = brewing betrayal
            if trust < BETRAYAL_TRUST_THRESHOLD && agent.traits.ambition > 0.6 {
                let tension_id = format!("betrayal_{}_vs_{}", agent.id, leader);

                // Check if tension already exists
                if let Some(existing) = tension_stream.get_mut(&tension_id) {
                    let new_severity = (0.5 - trust) * agent.traits.ambition;
                    existing.update_severity(new_severity.clamp(0.3, 1.0), current_tick);
                } else {
                    // Create new tension
                    let severity = (0.5 - trust) * agent.traits.ambition;
                    let mut tension = Tension::new(
                        &tension_id,
                        TensionType::BrewingBetrayal,
                        current_tick,
                        format!("{} harbors resentment toward leadership", agent.name),
                    );
                    tension.severity = severity.clamp(0.3, 0.8);
                    tension.confidence = 0.6;
                    tension.add_agent(&agent.id, "potential_betrayer", "escalating");
                    tension.add_agent(leader, "target", "unaware");
                    tension.add_predicted_outcome("open_defiance", 0.3, "high");
                    tension.add_predicted_outcome("faction_defection", 0.2, "very_high");
                    tension.narrative_hooks.push("Will ambition overcome loyalty?".to_string());
                    tension_stream.upsert(tension);
                }
            }
        }
    }
}

/// Detect succession crisis: no leader or leader has low trust from faction
fn detect_succession_crisis(
    tension_stream: &mut TensionStream,
    relationships: &RelationshipGraph,
    faction_agents: &[AgentData],
    leader_id: &Option<String>,
    faction_id: &str,
    faction_name: &str,
    current_tick: u64,
) {
    let tension_id = format!("succession_{}", faction_id);

    match leader_id {
        None => {
            // No leader - definite succession crisis
            if tension_stream.get(&tension_id).is_none() {
                let mut tension = Tension::new(
                    &tension_id,
                    TensionType::SuccessionCrisis,
                    current_tick,
                    format!("{} has no leader", faction_name),
                );
                tension.severity = 0.8;
                tension.confidence = 1.0;
                tension.add_predicted_outcome("power_struggle", 0.6, "high");
                tension.add_predicted_outcome("external_intervention", 0.2, "very_high");
                tension_stream.upsert(tension);
            }
        }
        Some(leader) => {
            // Check average trust in leader
            let mut trust_sum = 0.0;
            let mut trust_count = 0;

            for agent in faction_agents {
                if &agent.id == leader {
                    continue;
                }
                if let Some(rel) = relationships.get(&agent.id, leader) {
                    trust_sum += rel.trust.overall();
                    trust_count += 1;
                }
            }

            if trust_count > 0 {
                let avg_trust = trust_sum / trust_count as f32;

                if avg_trust < SUCCESSION_TRUST_THRESHOLD {
                    // Leader has low trust - succession crisis brewing
                    if let Some(existing) = tension_stream.get_mut(&tension_id) {
                        let severity = 0.5 + (SUCCESSION_TRUST_THRESHOLD - avg_trust);
                        existing.update_severity(severity.clamp(0.3, 0.9), current_tick);
                    } else {
                        let mut tension = Tension::new(
                            &tension_id,
                            TensionType::SuccessionCrisis,
                            current_tick,
                            format!("{} leadership under question", faction_name),
                        );
                        tension.severity = 0.5;
                        tension.confidence = 0.7;
                        tension.add_agent(leader, "contested_leader", "defensive");
                        tension.add_predicted_outcome("leadership_challenge", 0.4, "high");
                        tension.add_predicted_outcome("gradual_legitimacy_loss", 0.3, "medium");
                        tension_stream.upsert(tension);
                    }
                } else if let Some(existing) = tension_stream.get_mut(&tension_id) {
                    // Trust recovered - de-escalate
                    existing.update_severity(0.1, current_tick);
                }
            }
        }
    }
}

/// Detect resource conflict when faction resources are critical
fn detect_resource_conflict(
    tension_stream: &mut TensionStream,
    faction_id: &str,
    faction_name: &str,
    resources: &crate::components::faction::FactionResources,
    current_tick: u64,
) {
    let tension_id = format!("resources_{}", faction_id);

    if resources.is_critical() {
        if let Some(existing) = tension_stream.get_mut(&tension_id) {
            // Already tracking - update severity based on how critical
            let severity = if resources.grain < 50 { 0.9 } else { 0.6 };
            existing.update_severity(severity, current_tick);
        } else {
            let mut tension = Tension::new(
                &tension_id,
                TensionType::ResourceConflict,
                current_tick,
                format!("{} facing resource scarcity", faction_name),
            );
            tension.severity = 0.6;
            tension.confidence = 0.9;
            tension.add_predicted_outcome("resource_raid", 0.3, "medium");
            tension.add_predicted_outcome("internal_hoarding", 0.4, "medium");
            tension.add_predicted_outcome("desperate_measures", 0.2, "high");
            tension.narrative_hooks.push("Scarcity breeds conflict".to_string());
            tension_stream.upsert(tension);
        }
    } else if let Some(existing) = tension_stream.get_mut(&tension_id) {
        // Resources recovered
        existing.update_severity(0.05, current_tick);
    }
}

/// Detect faction fracture: multiple agents have negative sentiment toward leadership
fn detect_faction_fracture(
    tension_stream: &mut TensionStream,
    relationships: &RelationshipGraph,
    faction_agents: &[AgentData],
    leader_id: &Option<String>,
    faction_id: &str,
    faction_name: &str,
    current_tick: u64,
) {
    let leader = match leader_id {
        Some(id) => id,
        None => return, // No leader, no fracture (that's a succession crisis)
    };

    // Count agents with negative trust toward leader
    let mut disgruntled: Vec<String> = Vec::new();

    for agent in faction_agents {
        if &agent.id == leader {
            continue;
        }
        if let Some(rel) = relationships.get(&agent.id, leader) {
            if rel.trust.is_negative() {
                disgruntled.push(agent.id.clone());
            }
        }
    }

    let tension_id = format!("fracture_{}", faction_id);

    if disgruntled.len() >= FRACTURE_MIN_AGENTS {
        let severity = (disgruntled.len() as f32 / faction_agents.len() as f32).clamp(0.3, 0.9);

        if let Some(existing) = tension_stream.get_mut(&tension_id) {
            existing.update_severity(severity, current_tick);
        } else {
            let mut tension = Tension::new(
                &tension_id,
                TensionType::FactionFracture,
                current_tick,
                format!("Discontent spreading within {}", faction_name),
            );
            tension.severity = severity;
            tension.confidence = 0.8;
            for agent_id in disgruntled.iter().take(5) {
                tension.add_agent(agent_id, "dissident", "deepening");
            }
            tension.add_agent(leader, "authority_figure", "challenged");
            tension.add_predicted_outcome("faction_split", 0.3, "very_high");
            tension.add_predicted_outcome("mass_defection", 0.2, "very_high");
            tension.add_predicted_outcome("internal_reform", 0.3, "medium");
            tension.narrative_hooks.push("The cracks begin to show".to_string());
            tension_stream.upsert(tension);
        }
    } else if let Some(existing) = tension_stream.get_mut(&tension_id) {
        // Discontent subsiding
        existing.update_severity(0.1, current_tick);
    }
}

/// Detect forbidden alliances: cross-faction positive relationships
fn detect_forbidden_alliances(
    tension_stream: &mut TensionStream,
    relationships: &RelationshipGraph,
    all_agents: &[AgentData],
    current_tick: u64,
) {
    // Check all pairs of agents from different factions
    for (i, agent1) in all_agents.iter().enumerate() {
        for agent2 in all_agents.iter().skip(i + 1) {
            if agent1.faction_id == agent2.faction_id {
                continue; // Same faction - not forbidden
            }

            // Check if they have positive trust
            if let Some(rel) = relationships.get(&agent1.id, &agent2.id) {
                if rel.trust.overall() > ALLIANCE_TRUST_THRESHOLD {
                    let tension_id = format!("alliance_{}_{}", agent1.id, agent2.id);

                    if tension_stream.get(&tension_id).is_none() {
                        let mut tension = Tension::new(
                            &tension_id,
                            TensionType::ForbiddenAlliance,
                            current_tick,
                            format!(
                                "{} and {} form unlikely bond across faction lines",
                                agent1.name, agent2.name
                            ),
                        );
                        tension.severity = 0.4;
                        tension.confidence = 0.7;
                        tension.add_agent(&agent1.id, "ally", "committed");
                        tension.add_agent(&agent2.id, "ally", "committed");
                        tension.add_predicted_outcome("secret_cooperation", 0.5, "medium");
                        tension.add_predicted_outcome("exposed_and_punished", 0.3, "high");
                        tension.add_predicted_outcome("defection_together", 0.2, "very_high");
                        tension.narrative_hooks.push("Loyalty divided".to_string());
                        tension_stream.upsert(tension);
                    } else if let Some(existing) = tension_stream.get_mut(&tension_id) {
                        // Update based on trust strength
                        let severity = (rel.trust.overall() - ALLIANCE_TRUST_THRESHOLD + 0.3).clamp(0.3, 0.8);
                        existing.update_severity(severity, current_tick);
                    }
                }
            }
        }
    }
}

/// Detect revenge arcs: agents with active revenge goals
fn detect_revenge_arcs(
    tension_stream: &mut TensionStream,
    all_agents: &[AgentData],
    current_tick: u64,
) {
    for agent in all_agents {
        if let Some(revenge_goal) = agent.goals.get_goal(&GoalType::Revenge) {
            if let Some(target) = &revenge_goal.target {
                let tension_id = format!("revenge_{}_vs_{}", agent.id, target);

                if tension_stream.get(&tension_id).is_none() {
                    let severity = revenge_goal.priority * agent.traits.grudge_persistence;
                    let mut tension = Tension::new(
                        &tension_id,
                        TensionType::RevengeArc,
                        current_tick,
                        format!("{} seeks revenge", agent.name),
                    );
                    tension.severity = severity.clamp(0.4, 0.9);
                    tension.confidence = 0.9;
                    tension.add_agent(&agent.id, "avenger", "hunting");
                    tension.add_agent(target, "target", "unaware");
                    if let Some(origin) = &revenge_goal.origin_event {
                        tension.add_trigger_event(origin);
                    }
                    tension.add_predicted_outcome("confrontation", 0.5, "high");
                    tension.add_predicted_outcome("sabotage", 0.3, "medium");
                    tension.add_predicted_outcome("forgiveness", 0.1, "medium");
                    tension.narrative_hooks.push("Vengeance is a patient hunter".to_string());
                    tension_stream.upsert(tension);
                } else if let Some(existing) = tension_stream.get_mut(&tension_id) {
                    // Goal still active - update
                    let severity = revenge_goal.priority * agent.traits.grudge_persistence;
                    existing.update_severity(severity.clamp(0.4, 0.9), current_tick);
                }
            }
        }
    }
}

/// Detect rising power: ambitious agents gaining influence
fn detect_rising_power(
    tension_stream: &mut TensionStream,
    all_agents: &[AgentData],
    current_tick: u64,
) {
    for agent in all_agents {
        // High ambition + challenging leader goal = rising power tension
        if agent.traits.ambition > 0.7 && agent.goals.has_goal(&GoalType::ChallengeLeader) {
            let tension_id = format!("rising_{}", agent.id);

            if tension_stream.get(&tension_id).is_none() {
                let mut tension = Tension::new(
                    &tension_id,
                    TensionType::RisingPower,
                    current_tick,
                    format!("{} amasses influence", agent.name),
                );
                tension.severity = 0.5 + (agent.traits.ambition - 0.5);
                tension.confidence = 0.6;
                tension.add_agent(&agent.id, "aspirant", "ascending");
                tension.add_predicted_outcome("successful_challenge", 0.3, "very_high");
                tension.add_predicted_outcome("blocked_by_incumbent", 0.4, "medium");
                tension.add_predicted_outcome("faction_split", 0.2, "very_high");
                tension.narrative_hooks.push("The climb to power begins".to_string());
                tension_stream.upsert(tension);
            }
        }
    }
}

/// Detect secret exposed: when secret memories are shared
fn detect_secret_exposed(
    tension_stream: &mut TensionStream,
    memory_bank: &MemoryBank,
    all_agents: &[AgentData],
    current_tick: u64,
) {
    // Look for recently created memories about secrets
    for agent in all_agents {
        if let Some(memories) = memory_bank.get_memories(&agent.id) {
            for memory in memories {
                // Check for recently shared secrets (memories that reference secrets)
                if memory.is_secret && !memory.source_chain.is_empty() {
                    // This is a secondhand secret - someone shared it
                    let tension_id = format!("secret_{}_{}", memory.subject, current_tick / 100);

                    if tension_stream.get(&tension_id).is_none() {
                        let mut tension = Tension::new(
                            &tension_id,
                            TensionType::SecretExposed,
                            current_tick,
                            format!("Secret about {} is spreading", memory.subject),
                        );
                        tension.severity = 0.6;
                        tension.confidence = 0.8;
                        tension.add_agent(&memory.subject, "exposed", "vulnerable");
                        if let Some(source) = memory.source_chain.last() {
                            tension.add_agent(&source.agent_id, "revealer", "active");
                        }
                        tension.add_predicted_outcome("reputation_damage", 0.5, "medium");
                        tension.add_predicted_outcome("retaliation", 0.3, "high");
                        tension.add_predicted_outcome("confession", 0.2, "medium");
                        tension.narrative_hooks.push("Secrets have a way of surfacing".to_string());
                        tension_stream.upsert(tension);
                    }
                }
            }
        }
    }
}

/// Detect external threat from world state
fn detect_external_threat(
    tension_stream: &mut TensionStream,
    world_state: &WorldState,
    current_tick: u64,
) {
    for threat in &world_state.active_threats {
        let tension_id = format!("threat_{}", threat.replace(' ', "_"));

        if tension_stream.get(&tension_id).is_none() {
            let mut tension = Tension::new(
                &tension_id,
                TensionType::ExternalThreat,
                current_tick,
                format!("External threat: {}", threat),
            );
            tension.severity = 0.7;
            tension.confidence = 1.0;
            tension.add_predicted_outcome("unified_response", 0.4, "medium");
            tension.add_predicted_outcome("exploitation_by_faction", 0.3, "high");
            tension.add_predicted_outcome("casualties", 0.3, "very_high");
            tension.narrative_hooks.push("External forces gather".to_string());
            tension_stream.upsert(tension);
        }
    }
}

/// Update tension statuses and cleanup resolved ones
fn update_tension_statuses(tension_stream: &mut TensionStream) {
    // Cleanup resolved tensions periodically
    tension_stream.cleanup_resolved();
}

/// System to serialize tension stream to file (runs less frequently)
pub fn output_tensions(
    world_state: Res<WorldState>,
    tension_stream: Res<TensionStream>,
) {
    // Only output every 100 ticks
    if world_state.current_tick % 100 != 0 {
        return;
    }

    let json = tension_stream.to_json();
    if let Err(e) = std::fs::write("output/tensions.json", json) {
        eprintln!("Warning: Could not write tensions.json: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::agent::{Agent, Goal, Role};
    use crate::components::faction::{Faction, FactionResources};
    use crate::components::social::{Relationship, Trust};

    #[test]
    fn test_detection_constants() {
        assert!(BETRAYAL_TRUST_THRESHOLD < 0.0);
        assert!(SUCCESSION_TRUST_THRESHOLD > 0.0);
        assert!(FRACTURE_MIN_AGENTS >= 2);
        assert!(ALLIANCE_TRUST_THRESHOLD > 0.0);
    }

    #[test]
    fn test_detection_interval() {
        // Detection should run periodically, not every tick
        assert!(DETECTION_INTERVAL > 1);
        assert!(DETECTION_INTERVAL <= 50);
    }

    /// Helper to create a test world with agents configured for tension detection
    fn setup_test_world() -> World {
        let mut world = World::new();

        // Insert required resources
        world.insert_resource(WorldState::new());
        world.insert_resource(TensionStream::new());
        world.insert_resource(RelationshipGraph::new());
        world.insert_resource(MemoryBank::new());

        // Create faction registry with one faction
        let mut faction_registry = FactionRegistry::new();
        let mut faction = Faction::new("test_faction", "Test Faction", "test_hq");
        faction.leader = Some("leader_001".to_string());
        faction.resources = FactionResources::new(500, 100, 100); // Not critical
        faction_registry.register(faction);
        world.insert_resource(faction_registry);

        world
    }

    /// Spawn a test agent with given parameters
    fn spawn_agent(
        world: &mut World,
        id: &str,
        name: &str,
        faction_id: &str,
        role: Role,
        ambition: f32,
        grudge_persistence: f32,
    ) -> Entity {
        let mut goals = Goals::new();
        // Default goal
        goals.add(Goal::new(GoalType::Survive, 0.5));

        world.spawn((
            Agent,
            AgentId(id.to_string()),
            AgentName(name.to_string()),
            FactionMembership::new(faction_id, role),
            goals,
            Traits {
                ambition,
                grudge_persistence,
                boldness: 0.5,
                loyalty_weight: 0.5,
                honesty: 0.5,
                sociability: 0.5,
                group_preference: 0.5,
            },
        )).id()
    }

    #[test]
    fn test_brewing_betrayal_detection() {
        let mut world = setup_test_world();

        // Spawn leader
        spawn_agent(&mut world, "leader_001", "Leader", "test_faction", Role::Leader, 0.5, 0.5);

        // Spawn ambitious agent with low trust toward leader
        spawn_agent(&mut world, "ambitious_001", "Ambitious Agent", "test_faction", Role::Laborer, 0.8, 0.5);

        // Set up negative trust relationship
        {
            let mut graph = world.resource_mut::<RelationshipGraph>();
            let mut rel = Relationship::new("leader_001");
            rel.trust = Trust::new(-0.5, -0.3, 0.0); // Low reliability and alignment
            graph.set("ambitious_001", rel);
        }

        // Set tick to detection interval
        world.resource_mut::<WorldState>().current_tick = DETECTION_INTERVAL;

        // Run detection system
        let mut schedule = Schedule::default();
        schedule.add_systems(detect_tensions);
        schedule.run(&mut world);

        // Check that brewing betrayal tension was detected
        let tension_stream = world.resource::<TensionStream>();
        let tensions: Vec<_> = tension_stream.active_tensions().collect();

        assert!(!tensions.is_empty(), "Should detect at least one tension");
        assert!(
            tensions.iter().any(|t| t.tension_type == TensionType::BrewingBetrayal),
            "Should detect BrewingBetrayal tension"
        );

        // Verify tension details
        let betrayal = tensions.iter().find(|t| t.tension_type == TensionType::BrewingBetrayal).unwrap();
        assert!(betrayal.severity >= 0.3, "Severity should be at least 0.3");
        assert!(!betrayal.key_agents.is_empty(), "Should have key agents");
    }

    #[test]
    fn test_succession_crisis_no_leader() {
        let mut world = World::new();

        // Insert required resources
        world.insert_resource(WorldState::new());
        world.insert_resource(TensionStream::new());
        world.insert_resource(RelationshipGraph::new());
        world.insert_resource(MemoryBank::new());

        // Create faction with NO leader
        let mut faction_registry = FactionRegistry::new();
        let mut faction = Faction::new("leaderless", "Leaderless Faction", "hq");
        faction.leader = None; // No leader!
        faction_registry.register(faction);
        world.insert_resource(faction_registry);

        // Spawn some agents in the faction
        spawn_agent(&mut world, "agent_001", "Agent 1", "leaderless", Role::Laborer, 0.5, 0.5);
        spawn_agent(&mut world, "agent_002", "Agent 2", "leaderless", Role::Laborer, 0.5, 0.5);

        // Set tick to detection interval
        world.resource_mut::<WorldState>().current_tick = DETECTION_INTERVAL;

        // Run detection
        let mut schedule = Schedule::default();
        schedule.add_systems(detect_tensions);
        schedule.run(&mut world);

        // Check for succession crisis
        let tension_stream = world.resource::<TensionStream>();
        let tensions: Vec<_> = tension_stream.active_tensions().collect();

        assert!(
            tensions.iter().any(|t| t.tension_type == TensionType::SuccessionCrisis),
            "Should detect SuccessionCrisis when faction has no leader"
        );

        let crisis = tensions.iter().find(|t| t.tension_type == TensionType::SuccessionCrisis).unwrap();
        assert_eq!(crisis.severity, 0.8, "No-leader crisis should have high severity");
        assert_eq!(crisis.confidence, 1.0, "No-leader crisis should have full confidence");
    }

    #[test]
    fn test_revenge_arc_detection() {
        let mut world = setup_test_world();

        // Spawn target agent
        spawn_agent(&mut world, "target_001", "Target", "test_faction", Role::Laborer, 0.5, 0.5);

        // Spawn avenger with revenge goal
        let avenger_entity = spawn_agent(
            &mut world,
            "avenger_001",
            "Avenger",
            "test_faction",
            Role::Laborer,
            0.5,
            0.8, // High grudge persistence
        );

        // Add revenge goal to the avenger
        {
            let mut goals = world.get_mut::<Goals>(avenger_entity).unwrap();
            goals.add(
                Goal::new(GoalType::Revenge, 0.9)
                    .with_target("target_001")
                    .with_origin("betrayal_event_001")
            );
        }

        // Set tick to detection interval
        world.resource_mut::<WorldState>().current_tick = DETECTION_INTERVAL;

        // Run detection
        let mut schedule = Schedule::default();
        schedule.add_systems(detect_tensions);
        schedule.run(&mut world);

        // Check for revenge arc
        let tension_stream = world.resource::<TensionStream>();
        let tensions: Vec<_> = tension_stream.active_tensions().collect();

        assert!(
            tensions.iter().any(|t| t.tension_type == TensionType::RevengeArc),
            "Should detect RevengeArc when agent has revenge goal"
        );

        let revenge = tensions.iter().find(|t| t.tension_type == TensionType::RevengeArc).unwrap();
        assert!(revenge.severity >= 0.4, "Revenge arc should have moderate severity");
        assert!(!revenge.trigger_events.is_empty(), "Should have trigger event from goal origin");
    }

    #[test]
    fn test_forbidden_alliance_detection() {
        let mut world = World::new();

        // Insert required resources
        world.insert_resource(WorldState::new());
        world.insert_resource(TensionStream::new());
        world.insert_resource(RelationshipGraph::new());
        world.insert_resource(MemoryBank::new());

        // Create two factions
        let mut faction_registry = FactionRegistry::new();
        let mut faction1 = Faction::new("faction_a", "Faction A", "hq_a");
        faction1.leader = Some("leader_a".to_string());
        let mut faction2 = Faction::new("faction_b", "Faction B", "hq_b");
        faction2.leader = Some("leader_b".to_string());
        faction_registry.register(faction1);
        faction_registry.register(faction2);
        world.insert_resource(faction_registry);

        // Spawn agents in different factions
        spawn_agent(&mut world, "agent_a", "Agent A", "faction_a", Role::Laborer, 0.5, 0.5);
        spawn_agent(&mut world, "agent_b", "Agent B", "faction_b", Role::Laborer, 0.5, 0.5);

        // Create positive cross-faction relationship
        {
            let mut graph = world.resource_mut::<RelationshipGraph>();
            let mut rel = Relationship::new("agent_b");
            rel.trust = Trust::new(0.5, 0.5, 0.4); // Positive trust across factions
            graph.set("agent_a", rel);
        }

        // Set tick to detection interval
        world.resource_mut::<WorldState>().current_tick = DETECTION_INTERVAL;

        // Run detection
        let mut schedule = Schedule::default();
        schedule.add_systems(detect_tensions);
        schedule.run(&mut world);

        // Check for forbidden alliance
        let tension_stream = world.resource::<TensionStream>();
        let tensions: Vec<_> = tension_stream.active_tensions().collect();

        assert!(
            tensions.iter().any(|t| t.tension_type == TensionType::ForbiddenAlliance),
            "Should detect ForbiddenAlliance for cross-faction positive relationship"
        );
    }

    #[test]
    fn test_resource_conflict_detection() {
        let mut world = World::new();

        // Insert required resources
        world.insert_resource(WorldState::new());
        world.insert_resource(TensionStream::new());
        world.insert_resource(RelationshipGraph::new());
        world.insert_resource(MemoryBank::new());

        // Create faction with CRITICAL resources
        let mut faction_registry = FactionRegistry::new();
        let mut faction = Faction::new("starving", "Starving Faction", "hq");
        faction.leader = Some("leader".to_string());
        faction.resources = FactionResources::new(50, 10, 10); // Critical grain!
        faction_registry.register(faction);
        world.insert_resource(faction_registry);

        // Spawn an agent
        spawn_agent(&mut world, "agent_001", "Hungry Agent", "starving", Role::Laborer, 0.5, 0.5);

        // Set tick to detection interval
        world.resource_mut::<WorldState>().current_tick = DETECTION_INTERVAL;

        // Run detection
        let mut schedule = Schedule::default();
        schedule.add_systems(detect_tensions);
        schedule.run(&mut world);

        // Check for resource conflict
        let tension_stream = world.resource::<TensionStream>();
        let tensions: Vec<_> = tension_stream.active_tensions().collect();

        assert!(
            tensions.iter().any(|t| t.tension_type == TensionType::ResourceConflict),
            "Should detect ResourceConflict when faction resources are critical"
        );

        let conflict = tensions.iter().find(|t| t.tension_type == TensionType::ResourceConflict).unwrap();
        assert!(conflict.severity >= 0.6, "Resource conflict should have high severity");
    }

    #[test]
    fn test_faction_fracture_detection() {
        let mut world = setup_test_world();

        // Spawn leader
        spawn_agent(&mut world, "leader_001", "Leader", "test_faction", Role::Leader, 0.5, 0.5);

        // Spawn 4 disgruntled agents (need at least FRACTURE_MIN_AGENTS = 3)
        for i in 1..=4 {
            spawn_agent(
                &mut world,
                &format!("disgruntled_{:03}", i),
                &format!("Disgruntled {}", i),
                "test_faction",
                Role::Laborer,
                0.5,
                0.5,
            );
        }

        // Set up negative trust toward leader for all disgruntled agents
        {
            let mut graph = world.resource_mut::<RelationshipGraph>();
            for i in 1..=4 {
                let mut rel = Relationship::new("leader_001");
                rel.trust = Trust::new(-0.3, -0.2, 0.0); // Negative trust
                graph.set(&format!("disgruntled_{:03}", i), rel);
            }
        }

        // Set tick to detection interval
        world.resource_mut::<WorldState>().current_tick = DETECTION_INTERVAL;

        // Run detection
        let mut schedule = Schedule::default();
        schedule.add_systems(detect_tensions);
        schedule.run(&mut world);

        // Check for faction fracture
        let tension_stream = world.resource::<TensionStream>();
        let tensions: Vec<_> = tension_stream.active_tensions().collect();

        assert!(
            tensions.iter().any(|t| t.tension_type == TensionType::FactionFracture),
            "Should detect FactionFracture when multiple agents distrust leader"
        );
    }

    #[test]
    fn test_multiple_tensions_simultaneously() {
        let mut world = World::new();

        // Insert required resources
        world.insert_resource(WorldState::new());
        world.insert_resource(TensionStream::new());
        world.insert_resource(RelationshipGraph::new());
        world.insert_resource(MemoryBank::new());

        // Create faction with critical resources AND no leader
        let mut faction_registry = FactionRegistry::new();
        let mut faction = Faction::new("doomed", "Doomed Faction", "hq");
        faction.leader = None; // Succession crisis
        faction.resources = FactionResources::new(30, 5, 5); // Resource conflict
        faction_registry.register(faction);
        world.insert_resource(faction_registry);

        // Spawn agent with revenge goal
        let avenger = spawn_agent(&mut world, "avenger", "Avenger", "doomed", Role::Laborer, 0.5, 0.9);
        {
            let mut goals = world.get_mut::<Goals>(avenger).unwrap();
            goals.add(Goal::new(GoalType::Revenge, 0.8).with_target("enemy"));
        }

        // Set tick to detection interval
        world.resource_mut::<WorldState>().current_tick = DETECTION_INTERVAL;

        // Run detection
        let mut schedule = Schedule::default();
        schedule.add_systems(detect_tensions);
        schedule.run(&mut world);

        // Should detect multiple tensions
        let tension_stream = world.resource::<TensionStream>();
        let tensions: Vec<_> = tension_stream.active_tensions().collect();

        assert!(tensions.len() >= 3, "Should detect at least 3 tensions (succession, resource, revenge)");

        let types: Vec<_> = tensions.iter().map(|t| t.tension_type).collect();
        assert!(types.contains(&TensionType::SuccessionCrisis));
        assert!(types.contains(&TensionType::ResourceConflict));
        assert!(types.contains(&TensionType::RevengeArc));
    }
}
