//! Perception System
//!
//! Updates each agent's awareness of nearby agents at the same location.

use bevy_ecs::prelude::*;
use std::collections::HashMap;

use crate::components::agent::AgentId;
use crate::components::world::Position;

/// Component tracking which agents an agent can perceive
#[derive(Component, Debug, Clone, Default)]
pub struct VisibleAgents {
    /// List of agent IDs visible at the current location
    pub agents: Vec<String>,
}

impl VisibleAgents {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a specific agent is visible
    pub fn can_see(&self, agent_id: &str) -> bool {
        self.agents.iter().any(|id| id == agent_id)
    }

    /// Get the number of visible agents
    pub fn count(&self) -> usize {
        self.agents.len()
    }

    /// Check if this agent is alone
    pub fn is_alone(&self) -> bool {
        self.agents.is_empty()
    }
}

/// Resource tracking agents by location for efficient perception queries
#[derive(Resource, Debug, Default)]
pub struct AgentsByLocation {
    /// Maps location_id -> list of agent IDs present
    locations: HashMap<String, Vec<String>>,
}

impl AgentsByLocation {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get agents at a specific location
    pub fn at_location(&self, location_id: &str) -> &[String] {
        self.locations
            .get(location_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Get the number of agents at a location
    pub fn count_at(&self, location_id: &str) -> usize {
        self.locations
            .get(location_id)
            .map(|v| v.len())
            .unwrap_or(0)
    }

    /// Clear all location data (called before rebuilding)
    pub fn clear(&mut self) {
        self.locations.clear();
    }

    /// Add an agent to a location
    pub fn add(&mut self, location_id: impl Into<String>, agent_id: impl Into<String>) {
        self.locations
            .entry(location_id.into())
            .or_default()
            .push(agent_id.into());
    }
}

/// System to build the AgentsByLocation index
/// This runs first to create an efficient lookup structure
pub fn build_location_index(
    mut agents_by_location: ResMut<AgentsByLocation>,
    query: Query<(&AgentId, &Position)>,
) {
    agents_by_location.clear();

    for (agent_id, position) in query.iter() {
        agents_by_location.add(&position.location_id, &agent_id.0);
    }
}

/// System to update each agent's perception of visible agents
/// Runs after build_location_index to use the cached data
pub fn update_perception(
    agents_by_location: Res<AgentsByLocation>,
    mut query: Query<(&AgentId, &Position, &mut VisibleAgents)>,
) {
    for (agent_id, position, mut visible) in query.iter_mut() {
        // Get all agents at this location
        let agents_here = agents_by_location.at_location(&position.location_id);

        // Update visible agents, excluding self
        visible.agents = agents_here
            .iter()
            .filter(|id| *id != &agent_id.0)
            .cloned()
            .collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::agent::Alive;

    #[test]
    fn test_visible_agents_basic() {
        let mut visible = VisibleAgents::new();
        assert!(visible.is_alone());
        assert_eq!(visible.count(), 0);

        visible.agents.push("agent_001".to_string());
        visible.agents.push("agent_002".to_string());

        assert!(!visible.is_alone());
        assert_eq!(visible.count(), 2);
        assert!(visible.can_see("agent_001"));
        assert!(!visible.can_see("agent_999"));
    }

    #[test]
    fn test_agents_by_location() {
        let mut abl = AgentsByLocation::new();

        abl.add("village", "agent_001");
        abl.add("village", "agent_002");
        abl.add("forest", "agent_003");

        assert_eq!(abl.count_at("village"), 2);
        assert_eq!(abl.count_at("forest"), 1);
        assert_eq!(abl.count_at("castle"), 0);

        let village_agents = abl.at_location("village");
        assert_eq!(village_agents.len(), 2);
        assert!(village_agents.contains(&"agent_001".to_string()));
        assert!(village_agents.contains(&"agent_002".to_string()));

        abl.clear();
        assert_eq!(abl.count_at("village"), 0);
    }

    #[test]
    fn test_perception_system_integration() {
        let mut world = World::new();

        // Add resources
        world.insert_resource(AgentsByLocation::new());

        // Spawn agents at same location
        world.spawn((
            AgentId("agent_001".to_string()),
            Position {
                location_id: "village".to_string(),
            },
            VisibleAgents::new(),
            Alive::new(),
        ));

        world.spawn((
            AgentId("agent_002".to_string()),
            Position {
                location_id: "village".to_string(),
            },
            VisibleAgents::new(),
            Alive::new(),
        ));

        // Spawn agent at different location
        world.spawn((
            AgentId("agent_003".to_string()),
            Position {
                location_id: "forest".to_string(),
            },
            VisibleAgents::new(),
            Alive::new(),
        ));

        // Create and run schedule
        let mut schedule = Schedule::default();
        schedule.add_systems((build_location_index, update_perception).chain());
        schedule.run(&mut world);

        // Check perception results
        let mut query = world.query::<(&AgentId, &VisibleAgents)>();
        let results: Vec<_> = query.iter(&world).collect();

        for (agent_id, visible) in results {
            match agent_id.0.as_str() {
                "agent_001" => {
                    assert_eq!(visible.count(), 1);
                    assert!(visible.can_see("agent_002"));
                    assert!(!visible.can_see("agent_003"));
                }
                "agent_002" => {
                    assert_eq!(visible.count(), 1);
                    assert!(visible.can_see("agent_001"));
                    assert!(!visible.can_see("agent_003"));
                }
                "agent_003" => {
                    assert_eq!(visible.count(), 0);
                    assert!(visible.is_alone());
                }
                _ => panic!("Unexpected agent"),
            }
        }
    }
}
