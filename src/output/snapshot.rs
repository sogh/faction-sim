//! Snapshot Generation
//!
//! System for generating world snapshots at regular intervals.

use bevy_ecs::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::components::agent::{AgentId, AgentName, Alive, Goals, Needs, Traits};
use crate::components::faction::{FactionMembership, FactionRegistry};
use crate::components::social::RelationshipGraph;
use crate::components::world::{LocationRegistry, Position, WorldState};

use super::schemas::*;

/// Resource to track snapshot generation
#[derive(Resource)]
pub struct SnapshotGenerator {
    next_snapshot_id: u64,
    snapshot_interval: u64,
    last_snapshot_tick: u64,
}

impl SnapshotGenerator {
    pub fn new(snapshot_interval: u64) -> Self {
        Self {
            next_snapshot_id: 1,
            snapshot_interval,
            last_snapshot_tick: 0,
        }
    }

    pub fn should_snapshot(&self, current_tick: u64) -> bool {
        current_tick == 0 ||
        (current_tick > 0 && current_tick % self.snapshot_interval == 0)
    }

    pub fn next_id(&mut self) -> String {
        let id = format!("snap_{:06}", self.next_snapshot_id);
        self.next_snapshot_id += 1;
        id
    }

    pub fn mark_snapshot(&mut self, tick: u64) {
        self.last_snapshot_tick = tick;
    }

    pub fn snapshot_count(&self) -> u64 {
        self.next_snapshot_id - 1
    }
}

/// Generate a complete world snapshot
pub fn generate_snapshot(world: &mut World, triggered_by: &str) -> WorldSnapshot {
    let world_state = world.resource::<WorldState>();
    let tick = world_state.current_tick;
    let date = world_state.formatted_date();
    let season = format!("{:?}", world_state.current_season).to_lowercase();
    let active_threats = world_state.active_threats.clone();

    // Get snapshot ID
    let snapshot_id = {
        let mut generator = world.resource_mut::<SnapshotGenerator>();
        generator.next_id()
    };

    let mut snapshot = WorldSnapshot::new(&snapshot_id, tick, &date, triggered_by);
    snapshot.world.season = season;
    snapshot.world.active_threats = active_threats;

    // Collect faction data
    let faction_registry = world.resource::<FactionRegistry>();
    let mut global_grain = 0u32;
    let mut global_iron = 0u32;
    let mut global_salt = 0u32;

    for faction in faction_registry.all_factions() {
        global_grain += faction.resources.grain;
        global_iron += faction.resources.iron;
        global_salt += faction.resources.salt;

        let archive = faction_registry.get_archive(&faction.id.0);
        let archive_count = archive.map(|a| a.entry_count()).unwrap_or(0);

        snapshot.factions.push(FactionSnapshot {
            faction_id: faction.id.0.clone(),
            name: faction.name.clone(),
            territory: faction.territory.clone(),
            headquarters: faction.hq_location.clone(),
            resources: FactionResourcesSnapshot {
                grain: faction.resources.grain,
                iron: faction.resources.iron,
                salt: faction.resources.salt,
            },
            member_count: faction.member_count,
            leader: faction.leader.clone(),
            reader: faction.reader.clone(),
            archive_entry_count: archive_count,
            cohesion_score: 0.8, // Placeholder - compute from relationships
            external_reputation: HashMap::new(), // Placeholder
        });
    }

    snapshot.world.global_resources = GlobalResources {
        total_grain: global_grain,
        total_iron: global_iron,
        total_salt: global_salt,
    };

    // Collect agent data
    let mut agents_by_location: HashMap<String, Vec<String>> = HashMap::new();

    {
        let mut query = world.query::<(
            &AgentId,
            &AgentName,
            &Alive,
            &FactionMembership,
            &Position,
            &Traits,
            &Needs,
            &Goals,
        )>();

        for (agent_id, name, alive, membership, position, traits, needs, goals) in query.iter(world)
        {
            // Track agents at each location
            agents_by_location
                .entry(position.location_id.clone())
                .or_default()
                .push(agent_id.0.clone());

            let goals_snapshot: Vec<GoalSnapshot> = goals
                .goals
                .iter()
                .map(|g| GoalSnapshot {
                    goal: format!("{:?}", g.goal_type).to_lowercase(),
                    priority: g.priority,
                    target: g.target.clone(),
                })
                .collect();

            snapshot.agents.push(AgentSnapshot {
                agent_id: agent_id.0.clone(),
                name: name.0.clone(),
                alive: alive.is_alive(),
                faction: membership.faction_id.clone(),
                role: format!("{:?}", membership.role).to_lowercase(),
                location: position.location_id.clone(),
                traits: TraitsSnapshot {
                    boldness: traits.boldness,
                    loyalty_weight: traits.loyalty_weight,
                    grudge_persistence: traits.grudge_persistence,
                    ambition: traits.ambition,
                    honesty: traits.honesty,
                    sociability: traits.sociability,
                    group_preference: traits.group_preference,
                },
                status: StatusSnapshot {
                    level: membership.status_level,
                    role_title: format!("{:?}", membership.role).to_lowercase(),
                    influence_score: 0.5, // Placeholder
                    social_reach: 0,      // Placeholder
                    trusted_by_count: 0,  // Computed below
                    trusts_count: 0,      // Computed below
                },
                needs: NeedsSnapshot {
                    food_security: format!("{:?}", needs.food_security).to_lowercase(),
                    social_belonging: format!("{:?}", needs.social_belonging).to_lowercase(),
                },
                goals: goals_snapshot,
            });
        }
    }

    // Collect relationship data
    let relationship_graph = world.resource::<RelationshipGraph>();

    // Build relationship snapshots and count trust connections
    let mut trust_counts: HashMap<String, (u32, u32)> = HashMap::new(); // (trusted_by, trusts)

    for agent in &snapshot.agents {
        let agent_relationships = relationship_graph.relationships_for(&agent.agent_id);

        if !agent_relationships.is_empty() {
            let mut rel_map: HashMap<String, RelationshipSnapshot> = HashMap::new();

            for rel in agent_relationships {
                // Count trusts
                trust_counts.entry(agent.agent_id.clone()).or_insert((0, 0)).1 += 1;
                trust_counts.entry(rel.target_id.clone()).or_insert((0, 0)).0 += 1;

                rel_map.insert(
                    rel.target_id.clone(),
                    RelationshipSnapshot {
                        reliability: rel.trust.reliability,
                        alignment: rel.trust.alignment,
                        capability: rel.trust.capability,
                        last_interaction_tick: rel.last_interaction_tick,
                        memory_count: rel.memory_count,
                    },
                );
            }

            snapshot.relationships.insert(agent.agent_id.clone(), rel_map);
        }
    }

    // Update agent trust counts
    for agent in &mut snapshot.agents {
        if let Some(&(trusted_by, trusts)) = trust_counts.get(&agent.agent_id) {
            agent.status.trusted_by_count = trusted_by;
            agent.status.trusts_count = trusts;
            agent.status.social_reach = trusted_by + trusts;
        }
    }

    // Collect location data
    let location_registry = world.resource::<LocationRegistry>();

    for location in location_registry.all_locations() {
        let agents_present = agents_by_location
            .get(&location.id)
            .cloned()
            .unwrap_or_default();

        let resources = if location.resources.grain_production > 0
            || location.resources.iron_production > 0
            || location.resources.salt_production > 0
        {
            LocationResourcesSnapshot {
                grain_production: if location.resources.grain_production > 0 {
                    Some(location.resources.grain_production)
                } else {
                    None
                },
                iron_production: if location.resources.iron_production > 0 {
                    Some(location.resources.iron_production)
                } else {
                    None
                },
                salt_production: if location.resources.salt_production > 0 {
                    Some(location.resources.salt_production)
                } else {
                    None
                },
            }
        } else {
            LocationResourcesSnapshot::default()
        };

        snapshot.locations.push(LocationSnapshot {
            location_id: location.id.clone(),
            name: location.name.clone(),
            location_type: format!("{:?}", location.location_type).to_lowercase(),
            controlling_faction: location.controlling_faction.clone(),
            agents_present,
            resources,
            properties: location.properties.iter().map(|p| format!("{:?}", p).to_lowercase()).collect(),
        });
    }

    // Compute metrics
    snapshot.computed_metrics = compute_metrics(&snapshot);

    snapshot
}

/// Compute derived metrics from snapshot data
fn compute_metrics(snapshot: &WorldSnapshot) -> ComputedMetrics {
    let mut metrics = ComputedMetrics::default();

    // Faction power balance (based on resources and member count)
    let total_power: f32 = snapshot
        .factions
        .iter()
        .map(|f| faction_power(f))
        .sum();

    for faction in &snapshot.factions {
        let power = faction_power(faction);
        metrics.faction_power_balance.insert(
            faction.faction_id.clone(),
            if total_power > 0.0 { power / total_power } else { 0.0 },
        );
    }

    // Find social hubs (agents with high connection counts)
    let mut agents_by_connections: Vec<_> = snapshot
        .agents
        .iter()
        .map(|a| (a, a.status.social_reach))
        .collect();
    agents_by_connections.sort_by(|a, b| b.1.cmp(&a.1));

    for (agent, _) in agents_by_connections.iter().take(5) {
        if agent.status.social_reach > 10 {
            metrics.social_network.hubs.push(SocialHub {
                agent_id: agent.agent_id.clone(),
                faction: agent.faction.clone(),
                influence_score: agent.status.influence_score,
                role: agent.role.clone(),
                connections: agent.status.social_reach,
            });
        }
    }

    // Find isolates
    for agent in &snapshot.agents {
        if agent.status.social_reach <= 2 && agent.alive {
            metrics.social_network.isolates.push(SocialIsolate {
                agent_id: agent.agent_id.clone(),
                faction: agent.faction.clone(),
                connections: agent.status.social_reach,
                belonging: agent.needs.social_belonging.clone(),
                risk: if agent.needs.social_belonging == "isolated" {
                    "death_unnoticed".to_string()
                } else {
                    "low".to_string()
                },
            });
        }
    }

    metrics
}

/// Calculate faction power score
fn faction_power(faction: &FactionSnapshot) -> f32 {
    let resource_score = (faction.resources.grain as f32 * 0.3)
        + (faction.resources.iron as f32 * 0.5)
        + (faction.resources.salt as f32 * 0.2);
    let member_score = faction.member_count as f32 * 10.0;
    let territory_score = faction.territory.len() as f32 * 20.0;

    resource_score + member_score + territory_score
}

/// Write snapshot to file
pub fn write_snapshot(snapshot: &WorldSnapshot, path: impl AsRef<Path>) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(snapshot)?;
    fs::write(path, json)?;
    Ok(())
}

/// Write snapshot to snapshots directory
pub fn write_snapshot_to_dir(snapshot: &WorldSnapshot) -> std::io::Result<()> {
    let path = format!("output/snapshots/snap_{:06}.json", snapshot.timestamp.tick);
    write_snapshot(snapshot, path)
}

/// Write current state (overwrites each time)
pub fn write_current_state(snapshot: &WorldSnapshot) -> std::io::Result<()> {
    write_snapshot(snapshot, "output/current_state.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_serialization() {
        let snapshot = WorldSnapshot::new("snap_000001", 100, "year_1.spring.day_10", "test");

        let json = serde_json::to_string_pretty(&snapshot).unwrap();
        assert!(json.contains("snap_000001"));
        assert!(json.contains("year_1.spring.day_10"));

        // Verify it can be parsed back
        let parsed: WorldSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.snapshot_id, "snap_000001");
    }

    #[test]
    fn test_faction_power_calculation() {
        let faction = FactionSnapshot {
            faction_id: "test".to_string(),
            name: "Test".to_string(),
            territory: vec!["loc1".to_string(), "loc2".to_string()],
            headquarters: "loc1".to_string(),
            resources: FactionResourcesSnapshot {
                grain: 100,
                iron: 100,
                salt: 100,
            },
            member_count: 50,
            leader: None,
            reader: None,
            archive_entry_count: 0,
            cohesion_score: 0.8,
            external_reputation: HashMap::new(),
        };

        let power = faction_power(&faction);
        assert!(power > 0.0);
    }
}
