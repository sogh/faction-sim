//! Snapshot Types
//!
//! Serialization structs for world snapshots and state output.
//!
//! Snapshots capture the complete state of the simulation at a point in time,
//! used for analysis, visualization, and debugging.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::SimTimestamp;

/// Generates a snapshot ID with the given sequence number.
pub fn generate_snapshot_id(sequence: u64) -> String {
    format!("snap_{:06}", sequence)
}

/// Global world state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldStateSnapshot {
    pub season: String,
    pub global_resources: GlobalResources,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub active_threats: Vec<String>,
}

/// Global resource totals
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalResources {
    pub total_grain: u32,
    pub total_iron: u32,
    pub total_salt: u32,
    pub total_beer: u32,
}

/// Faction snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionSnapshot {
    pub faction_id: String,
    pub name: String,
    #[serde(default)]
    pub territory: Vec<String>,
    pub headquarters: String,
    #[serde(default)]
    pub resources: FactionResourcesSnapshot,
    #[serde(default)]
    pub member_count: u32,
    #[serde(default)]
    pub leader: Option<String>,
    #[serde(default)]
    pub reader: Option<String>,
    #[serde(default)]
    pub archive_entry_count: usize,
    #[serde(default)]
    pub cohesion_score: f32,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub external_reputation: HashMap<String, f32>,
}

/// Faction resources
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FactionResourcesSnapshot {
    pub grain: u32,
    pub iron: u32,
    pub salt: u32,
    pub beer: u32,
}

/// Agent traits snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitsSnapshot {
    pub boldness: f32,
    pub loyalty_weight: f32,
    pub grudge_persistence: f32,
    pub ambition: f32,
    pub honesty: f32,
    pub sociability: f32,
    pub group_preference: f32,
}

/// Agent status snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusSnapshot {
    pub level: u8,
    pub role_title: String,
    pub influence_score: f32,
    pub social_reach: u32,
    pub trusted_by_count: u32,
    pub trusts_count: u32,
}

/// Agent needs snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeedsSnapshot {
    pub food_security: String,
    pub social_belonging: String,
}

/// Agent goal snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalSnapshot {
    pub goal: String,
    pub priority: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

/// Full agent snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSnapshot {
    pub agent_id: String,
    pub name: String,
    #[serde(default)]
    pub alive: bool,
    pub faction: String,
    pub role: String,
    pub location: String,
    #[serde(default)]
    pub traits: TraitsSnapshot,
    #[serde(default)]
    pub status: StatusSnapshot,
    #[serde(default)]
    pub needs: NeedsSnapshot,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub goals: Vec<GoalSnapshot>,
}

/// Relationship snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipSnapshot {
    pub reliability: f32,
    pub alignment: f32,
    pub capability: f32,
    pub last_interaction_tick: u64,
    pub memory_count: u32,
}

/// Location snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationSnapshot {
    pub location_id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub location_type: String,
    pub controlling_faction: Option<String>,
    pub agents_present: Vec<String>,
    pub resources: LocationResourcesSnapshot,
    pub properties: Vec<String>,
}

/// Location resources
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocationResourcesSnapshot {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grain_production: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iron_production: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub salt_production: Option<u32>,
}

/// Social network hub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialHub {
    pub agent_id: String,
    pub faction: String,
    pub influence_score: f32,
    pub role: String,
    pub connections: u32,
}

/// Social network bridge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialBridge {
    pub agent_id: String,
    pub connects: Vec<String>,
    pub bridge_strength: f32,
    pub known_to_faction: bool,
}

/// Isolated agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialIsolate {
    pub agent_id: String,
    pub faction: String,
    pub connections: u32,
    pub belonging: String,
    pub risk: String,
}

/// Social network analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SocialNetworkSnapshot {
    #[serde(default)]
    pub hubs: Vec<SocialHub>,
    #[serde(default)]
    pub bridges: Vec<SocialBridge>,
    #[serde(default)]
    pub isolates: Vec<SocialIsolate>,
}

/// Computed metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComputedMetrics {
    #[serde(default)]
    pub faction_power_balance: HashMap<String, f32>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub war_probability_30_days: HashMap<String, f32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub agents_at_defection_risk: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub factions_at_collapse_risk: Vec<String>,
    #[serde(default)]
    pub social_network: SocialNetworkSnapshot,
}

/// Complete world snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub snapshot_id: String,
    pub timestamp: SimTimestamp,
    pub triggered_by: String,
    pub world: WorldStateSnapshot,
    pub factions: Vec<FactionSnapshot>,
    pub agents: Vec<AgentSnapshot>,
    pub relationships: HashMap<String, HashMap<String, RelationshipSnapshot>>,
    pub locations: Vec<LocationSnapshot>,
    pub computed_metrics: ComputedMetrics,
}

impl WorldSnapshot {
    /// Creates a new WorldSnapshot with minimal data.
    pub fn new(
        snapshot_id: impl Into<String>,
        timestamp: SimTimestamp,
        triggered_by: impl Into<String>,
    ) -> Self {
        Self {
            snapshot_id: snapshot_id.into(),
            timestamp,
            triggered_by: triggered_by.into(),
            world: WorldStateSnapshot {
                season: "spring".to_string(),
                global_resources: GlobalResources::default(),
                active_threats: Vec::new(),
            },
            factions: Vec::new(),
            agents: Vec::new(),
            relationships: HashMap::new(),
            locations: Vec::new(),
            computed_metrics: ComputedMetrics::default(),
        }
    }

    /// Finds an agent by ID.
    pub fn find_agent(&self, agent_id: &str) -> Option<&AgentSnapshot> {
        self.agents.iter().find(|a| a.agent_id == agent_id)
    }

    /// Finds a faction by ID.
    pub fn find_faction(&self, faction_id: &str) -> Option<&FactionSnapshot> {
        self.factions.iter().find(|f| f.faction_id == faction_id)
    }

    /// Finds a location by ID.
    pub fn find_location(&self, location_id: &str) -> Option<&LocationSnapshot> {
        self.locations.iter().find(|l| l.location_id == location_id)
    }

    /// Gets the relationship between two agents.
    pub fn get_relationship(&self, from: &str, to: &str) -> Option<&RelationshipSnapshot> {
        self.relationships.get(from).and_then(|m| m.get(to))
    }

    /// Returns the number of living agents.
    pub fn living_agent_count(&self) -> usize {
        self.agents.iter().filter(|a| a.alive).count()
    }

    /// Returns agents at a specific location.
    pub fn agents_at_location(&self, location_id: &str) -> Vec<&AgentSnapshot> {
        self.agents.iter().filter(|a| a.location == location_id).collect()
    }

    /// Returns agents belonging to a faction.
    pub fn faction_members(&self, faction_id: &str) -> Vec<&AgentSnapshot> {
        self.agents.iter().filter(|a| a.faction == faction_id && a.alive).collect()
    }

    /// Serializes the snapshot to pretty JSON.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Serializes the snapshot to compact JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserializes a snapshot from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl AgentSnapshot {
    /// Creates a new AgentSnapshot with required fields.
    pub fn new(
        agent_id: impl Into<String>,
        name: impl Into<String>,
        faction: impl Into<String>,
        role: impl Into<String>,
        location: impl Into<String>,
    ) -> Self {
        Self {
            agent_id: agent_id.into(),
            name: name.into(),
            alive: true,
            faction: faction.into(),
            role: role.into(),
            location: location.into(),
            traits: TraitsSnapshot::default(),
            status: StatusSnapshot::default(),
            needs: NeedsSnapshot::default(),
            goals: Vec::new(),
        }
    }
}

impl Default for TraitsSnapshot {
    fn default() -> Self {
        Self {
            boldness: 0.5,
            loyalty_weight: 0.5,
            grudge_persistence: 0.5,
            ambition: 0.5,
            honesty: 0.5,
            sociability: 0.5,
            group_preference: 0.5,
        }
    }
}

impl Default for StatusSnapshot {
    fn default() -> Self {
        Self {
            level: 1,
            role_title: "member".to_string(),
            influence_score: 0.0,
            social_reach: 0,
            trusted_by_count: 0,
            trusts_count: 0,
        }
    }
}

impl Default for NeedsSnapshot {
    fn default() -> Self {
        Self {
            food_security: "satisfied".to_string(),
            social_belonging: "satisfied".to_string(),
        }
    }
}

impl FactionSnapshot {
    /// Creates a new FactionSnapshot with required fields.
    pub fn new(
        faction_id: impl Into<String>,
        name: impl Into<String>,
        headquarters: impl Into<String>,
    ) -> Self {
        Self {
            faction_id: faction_id.into(),
            name: name.into(),
            territory: Vec::new(),
            headquarters: headquarters.into(),
            resources: FactionResourcesSnapshot::default(),
            member_count: 0,
            leader: None,
            reader: None,
            archive_entry_count: 0,
            cohesion_score: 0.5,
            external_reputation: HashMap::new(),
        }
    }
}

impl LocationSnapshot {
    /// Creates a new LocationSnapshot with required fields.
    pub fn new(
        location_id: impl Into<String>,
        name: impl Into<String>,
        location_type: impl Into<String>,
    ) -> Self {
        Self {
            location_id: location_id.into(),
            name: name.into(),
            location_type: location_type.into(),
            controlling_faction: None,
            agents_present: Vec::new(),
            resources: LocationResourcesSnapshot::default(),
            properties: Vec::new(),
        }
    }
}

impl RelationshipSnapshot {
    /// Creates a new RelationshipSnapshot.
    pub fn new(reliability: f32, alignment: f32, capability: f32) -> Self {
        Self {
            reliability,
            alignment,
            capability,
            last_interaction_tick: 0,
            memory_count: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Season, SimTimestamp};

    #[test]
    fn test_generate_snapshot_id() {
        assert_eq!(generate_snapshot_id(1), "snap_000001");
        assert_eq!(generate_snapshot_id(42371), "snap_042371");
        assert_eq!(generate_snapshot_id(999999), "snap_999999");
    }

    #[test]
    fn test_world_snapshot_new() {
        let ts = SimTimestamp::new(1000, 1, Season::Spring, 10);
        let snapshot = WorldSnapshot::new("snap_000001", ts, "scheduled");

        assert_eq!(snapshot.snapshot_id, "snap_000001");
        assert_eq!(snapshot.triggered_by, "scheduled");
        assert!(snapshot.agents.is_empty());
    }

    #[test]
    fn test_agent_snapshot_new() {
        let agent = AgentSnapshot::new(
            "agent_001",
            "Test Agent",
            "thornwood",
            "scout",
            "village_a",
        );

        assert_eq!(agent.agent_id, "agent_001");
        assert!(agent.alive);
        assert_eq!(agent.faction, "thornwood");
    }

    #[test]
    fn test_faction_snapshot_new() {
        let faction = FactionSnapshot::new("thornwood", "The Thornwood Council", "thornwood_village");

        assert_eq!(faction.faction_id, "thornwood");
        assert_eq!(faction.headquarters, "thornwood_village");
        assert!(faction.leader.is_none());
    }

    #[test]
    fn test_location_snapshot_new() {
        let location = LocationSnapshot::new("loc_market", "Central Market", "market");

        assert_eq!(location.location_id, "loc_market");
        assert_eq!(location.location_type, "market");
        assert!(location.controlling_faction.is_none());
    }

    #[test]
    fn test_world_snapshot_find_agent() {
        let ts = SimTimestamp::new(1000, 1, Season::Spring, 10);
        let mut snapshot = WorldSnapshot::new("snap_000001", ts, "scheduled");

        snapshot.agents.push(AgentSnapshot::new(
            "agent_001", "Alice", "thornwood", "scout", "village_a"
        ));
        snapshot.agents.push(AgentSnapshot::new(
            "agent_002", "Bob", "ironmere", "trader", "market"
        ));

        let agent = snapshot.find_agent("agent_001");
        assert!(agent.is_some());
        assert_eq!(agent.unwrap().name, "Alice");

        assert!(snapshot.find_agent("nonexistent").is_none());
    }

    #[test]
    fn test_world_snapshot_agents_at_location() {
        let ts = SimTimestamp::new(1000, 1, Season::Spring, 10);
        let mut snapshot = WorldSnapshot::new("snap_000001", ts, "scheduled");

        snapshot.agents.push(AgentSnapshot::new(
            "agent_001", "Alice", "thornwood", "scout", "market"
        ));
        snapshot.agents.push(AgentSnapshot::new(
            "agent_002", "Bob", "ironmere", "trader", "market"
        ));
        snapshot.agents.push(AgentSnapshot::new(
            "agent_003", "Carol", "thornwood", "leader", "village_a"
        ));

        let at_market = snapshot.agents_at_location("market");
        assert_eq!(at_market.len(), 2);
    }

    #[test]
    fn test_world_snapshot_serialization() {
        let ts = SimTimestamp::new(1000, 1, Season::Spring, 10);
        let mut snapshot = WorldSnapshot::new("snap_000001", ts, "scheduled");

        snapshot.agents.push(AgentSnapshot::new(
            "agent_001", "Alice", "thornwood", "scout", "market"
        ));
        snapshot.factions.push(FactionSnapshot::new(
            "thornwood", "The Thornwood Council", "thornwood_village"
        ));

        let json = snapshot.to_json().unwrap();
        assert!(json.contains("snap_000001"));
        assert!(json.contains("agent_001"));
        assert!(json.contains("thornwood"));

        // Verify roundtrip
        let parsed = WorldSnapshot::from_json(&json).unwrap();
        assert_eq!(parsed.snapshot_id, "snap_000001");
        assert_eq!(parsed.agents.len(), 1);
    }

    #[test]
    fn test_relationship_snapshot() {
        let rel = RelationshipSnapshot::new(0.8, 0.6, 0.5);

        assert_eq!(rel.reliability, 0.8);
        assert_eq!(rel.alignment, 0.6);
        assert_eq!(rel.capability, 0.5);
    }

    #[test]
    fn test_defaults() {
        let traits = TraitsSnapshot::default();
        assert_eq!(traits.boldness, 0.5);

        let status = StatusSnapshot::default();
        assert_eq!(status.level, 1);

        let needs = NeedsSnapshot::default();
        assert_eq!(needs.food_security, "satisfied");
    }
}
