//! Output Schemas
//!
//! Serialization structs for world snapshots and state output.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Timestamp for snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotTimestamp {
    pub tick: u64,
    pub date: String,
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
}

/// Faction snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionSnapshot {
    pub faction_id: String,
    pub name: String,
    pub territory: Vec<String>,
    pub headquarters: String,
    pub resources: FactionResourcesSnapshot,
    pub member_count: u32,
    pub leader: Option<String>,
    pub reader: Option<String>,
    pub archive_entry_count: usize,
    pub cohesion_score: f32,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub external_reputation: HashMap<String, f32>,
}

/// Faction resources
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FactionResourcesSnapshot {
    pub grain: u32,
    pub iron: u32,
    pub salt: u32,
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
    pub alive: bool,
    pub faction: String,
    pub role: String,
    pub location: String,
    pub traits: TraitsSnapshot,
    pub status: StatusSnapshot,
    pub needs: NeedsSnapshot,
    #[serde(skip_serializing_if = "Vec::is_empty")]
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
    pub timestamp: SnapshotTimestamp,
    pub triggered_by: String,
    pub world: WorldStateSnapshot,
    pub factions: Vec<FactionSnapshot>,
    pub agents: Vec<AgentSnapshot>,
    pub relationships: HashMap<String, HashMap<String, RelationshipSnapshot>>,
    pub locations: Vec<LocationSnapshot>,
    pub computed_metrics: ComputedMetrics,
}

impl WorldSnapshot {
    pub fn new(
        snapshot_id: impl Into<String>,
        tick: u64,
        date: impl Into<String>,
        triggered_by: impl Into<String>,
    ) -> Self {
        Self {
            snapshot_id: snapshot_id.into(),
            timestamp: SnapshotTimestamp {
                tick,
                date: date.into(),
            },
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
}
