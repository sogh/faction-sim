# sim-events Implementation Prompts

## Prerequisites

Before starting these prompts, ensure:
- [ ] Project structure is set up (see SETUP_INSTRUCTIONS.md)
- [ ] Cargo.toml has serde, serde_json, uuid dependencies
- [ ] You're in the project root directory

## Overview

The sim-events crate defines **all shared types** for the simulation. It has no logic—only data structures and their serialization. Every other crate depends on this one.

After completing all prompts, you'll have:
- SimTimestamp and date handling
- Complete Event struct matching the JSON schema
- All event types and subtypes
- Actor structs (primary, secondary, affected)
- Tension struct with predictions
- WorldSnapshot with agents, factions, locations
- Round-trip serialization tests

**Important**: These types must serialize to JSON that exactly matches `docs/design/simulation_output_schemas.md`. The director and visualization depend on this contract.

---

## Prompt 1: Timestamp Types

```
Implement timestamp types in crates/sim-events/src/timestamp.rs.

Based on docs/design/simulation_output_schemas.md, create:

1. SimTimestamp struct:
   - tick: u64 (monotonically increasing simulation tick)
   - date: SimDate (human-readable date)

2. SimDate struct that serializes to strings like "year_3.winter.day_12":
   - year: u32
   - season: Season
   - day: u8

3. Season enum with variants:
   - Spring, Summer, Autumn, Winter
   - Should serialize to lowercase: "spring", "summer", etc.

4. Implement Display for SimDate:
   - Format: "year_{year}.{season}.day_{day}"

5. Implement FromStr for SimDate:
   - Parse "year_3.winter.day_12" back to struct

6. Custom serialization for SimDate:
   - Serialize as the display string, not as an object
   - Use serde's serialize_with or implement Serialize manually

Example JSON:
{
  "tick": 84729,
  "date": "year_3.winter.day_12"
}

7. Helper methods on SimTimestamp:
   - new(tick: u64, year: u32, season: Season, day: u8) -> Self
   - advance_tick(&mut self) - increment tick
   - advance_day(&mut self) - increment day, handle season/year rollover

8. Constants:
   - DAYS_PER_SEASON: u8 = 30
   - TICKS_PER_DAY: u64 = 100 (configurable later)

Add tests:
- Round-trip serialization
- SimDate parsing from string
- Season rollover (day 30 winter -> day 1 spring)
- Year rollover

Update src/lib.rs to export these types.
```

---

## Prompt 2: Event Type Enums

```
Implement event type enums in crates/sim-events/src/event.rs.

Based on the Event Types table in docs/design/simulation_output_schemas.md:

1. EventType enum:
   - Movement
   - Communication
   - Betrayal
   - Loyalty
   - Conflict
   - Cooperation
   - Faction
   - Archive
   - Ritual
   - Resource
   - Death
   - Birth

   Derive: Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize
   Use #[serde(rename_all = "snake_case")]

2. Create subtype constants or helper for validation:
   
   Movement subtypes: travel, flee, pursue, patrol
   Communication subtypes: share_memory, spread_rumor, lie, confess
   Betrayal subtypes: secret_shared_with_enemy, sabotage, defection, false_testimony
   Loyalty subtypes: defend_ally, sacrifice_for_faction, refuse_bribe
   Conflict subtypes: argument, fight, duel, raid
   Cooperation subtypes: trade, alliance_formed, gift, favor
   Faction subtypes: join, leave, exile, promotion, demotion
   Archive subtypes: write_entry, read_entry, destroy_entry, forge_entry
   Ritual subtypes: reading_held, reading_disrupted, reading_attended, reading_missed
   Resource subtypes: acquire, lose, trade, steal, hoard
   Death subtypes: natural, killed, executed, sacrifice
   Birth subtypes: born, arrived, created

3. Helper method:
   impl EventType {
       pub fn valid_subtypes(&self) -> &'static [&'static str] {
           match self {
               EventType::Movement => &["travel", "flee", "pursue", "patrol"],
               // ... etc
           }
       }
       
       pub fn is_valid_subtype(&self, subtype: &str) -> bool {
           self.valid_subtypes().contains(&subtype)
       }
   }

4. Common drama tags as constants:
   pub mod drama_tags {
       pub const FACTION_CRITICAL: &str = "faction_critical";
       pub const SECRET_MEETING: &str = "secret_meeting";
       pub const LEADER_INVOLVED: &str = "leader_involved";
       pub const CROSS_FACTION: &str = "cross_faction";
       pub const WINTER_CRISIS: &str = "winter_crisis";
       pub const BETRAYAL: &str = "betrayal";
       // etc.
   }

Add tests for serialization producing correct snake_case strings.
```

---

## Prompt 3: Actor Structs

```
Implement actor structs in crates/sim-events/src/event.rs (continue from Prompt 2).

Based on the Event Schema in docs/design/simulation_output_schemas.md:

1. ActorSnapshot - represents an agent at a moment in time:
   - agent_id: String
   - name: String
   - faction: String
   - role: String
   - location: String

2. AffectedActor - for agents affected by an event:
   - agent_id: String
   - name: String
   - faction: String
   - role: String
   - relationship_to_primary: Option<String>
   
   For ritual events, also supports:
   - attended: Option<bool>
   - reason: Option<String>  // e.g., "absent_from_village"

   Use #[serde(skip_serializing_if = "Option::is_none")] for optional fields.

3. ActorSet - the actors involved in an event:
   - primary: ActorSnapshot
   - secondary: Option<ActorSnapshot>
   - affected: Vec<AffectedActor>

4. Helper constructors:
   impl ActorSet {
       pub fn primary_only(actor: ActorSnapshot) -> Self
       pub fn with_secondary(primary: ActorSnapshot, secondary: ActorSnapshot) -> Self
       pub fn all_agent_ids(&self) -> Vec<String>
   }

   impl ActorSnapshot {
       pub fn new(
           agent_id: impl Into<String>,
           name: impl Into<String>,
           faction: impl Into<String>,
           role: impl Into<String>,
           location: impl Into<String>,
       ) -> Self
   }

5. Example JSON to match:
{
  "actors": {
    "primary": {
      "agent_id": "agent_mira_0042",
      "name": "Mira of Thornwood",
      "faction": "thornwood",
      "role": "scout",
      "location": "eastern_bridge"
    },
    "secondary": {
      "agent_id": "agent_voss_0017",
      "name": "Voss the Quiet",
      "faction": "ironmere",
      "role": "spymaster",
      "location": "eastern_bridge"
    },
    "affected": [
      {
        "agent_id": "agent_corin_0003",
        "name": "Corin Thornwood",
        "faction": "thornwood",
        "role": "faction_leader",
        "relationship_to_primary": "trusts_highly"
      }
    ]
  }
}

Add tests verifying this exact JSON structure.
```

---

## Prompt 4: Event Context and Outcome

```
Implement event context and outcome in crates/sim-events/src/event.rs (continue).

1. EventContext struct:
   - trigger: String  // what caused this event
   - preconditions: Vec<String>  // human-readable conditions that enabled it
   - location_description: Option<String>

2. RelationshipChange struct:
   - from: String  // agent_id
   - to: String    // agent_id
   - dimension: String  // "reliability", "alignment", "capability"
   - old_value: f32
   - new_value: f32

3. EventOutcome struct - flexible to accommodate different event types:
   - relationship_changes: Vec<RelationshipChange>
   - state_changes: Vec<String>  // human-readable state changes
   
   // Optional fields for specific event types:
   - new_location: Option<String>  // for movement
   - travel_duration_ticks: Option<u32>  // for movement
   - information_transferred: Option<String>  // for communication/betrayal
   - entries_read: Option<Vec<ArchiveEntry>>  // for ritual
   - entries_skipped: Option<Vec<ArchiveEntry>>  // for ritual
   - memory_reinforcement: Option<Vec<MemoryReinforcement>>  // for ritual
   - memory_shared: Option<SharedMemory>  // for communication

   Use #[serde(skip_serializing_if = "Option::is_none")] liberally.
   Use #[serde(default)] for deserialization of missing fields.

4. Supporting structs:
   
   ArchiveEntry:
   - entry_id: String
   - subject: String

   MemoryReinforcement:
   - memory: String
   - agents_reinforced: u32
   - strength_delta: f32

   SharedMemory:
   - original_event: String  // event_id
   - content: String
   - source_chain: Vec<String>
   - fidelity: f32

5. Example ritual outcome JSON:
{
  "outcome": {
    "entries_read": [
      {"entry_id": "archive_tw_0012", "subject": "ironmere_broke_salt_treaty_year_1"}
    ],
    "entries_skipped": [
      {"entry_id": "archive_tw_0067", "subject": "corin_failed_harvest_negotiation"}
    ],
    "memory_reinforcement": [
      {"memory": "ironmere_untrustworthy", "agents_reinforced": 12, "strength_delta": 0.1}
    ],
    "relationship_changes": [],
    "state_changes": []
  }
}

Add tests for different outcome shapes.
```

---

## Prompt 5: Complete Event Struct

```
Implement the complete Event struct in crates/sim-events/src/event.rs.

1. Event struct with all fields:
   - event_id: String  // "evt_00042371"
   - timestamp: SimTimestamp
   - event_type: EventType
   - subtype: String
   - actors: ActorSet
   - context: EventContext
   - outcome: EventOutcome
   - drama_tags: Vec<String>
   - drama_score: f32
   - connected_events: Vec<String>  // event_ids of related events

2. Event ID generation helper:
   pub fn generate_event_id(sequence: u64) -> String {
       format!("evt_{:08}", sequence)
   }

3. Builder pattern for Event:
   pub struct EventBuilder {
       // fields with defaults
   }

   impl EventBuilder {
       pub fn new(event_type: EventType, subtype: impl Into<String>) -> Self
       pub fn id(mut self, id: impl Into<String>) -> Self
       pub fn timestamp(mut self, ts: SimTimestamp) -> Self
       pub fn primary_actor(mut self, actor: ActorSnapshot) -> Self
       pub fn secondary_actor(mut self, actor: ActorSnapshot) -> Self
       pub fn add_affected(mut self, actor: AffectedActor) -> Self
       pub fn trigger(mut self, trigger: impl Into<String>) -> Self
       pub fn add_precondition(mut self, cond: impl Into<String>) -> Self
       pub fn drama_score(mut self, score: f32) -> Self
       pub fn add_drama_tag(mut self, tag: impl Into<String>) -> Self
       pub fn build(self) -> Event
   }

4. Helper methods on Event:
   impl Event {
       pub fn all_agent_ids(&self) -> Vec<String>
       pub fn involves_agent(&self, agent_id: &str) -> bool
       pub fn involves_faction(&self, faction: &str) -> bool
       pub fn is_high_drama(&self) -> bool  // drama_score > 0.7
   }

5. Test against the full example from the schema doc:
   - Parse the betrayal event example JSON
   - Verify all fields deserialize correctly
   - Serialize back and compare (or check key fields)

6. JSON Lines helper:
   impl Event {
       pub fn to_jsonl(&self) -> Result<String, serde_json::Error> {
           serde_json::to_string(self)  // No pretty printing
       }
       
       pub fn from_jsonl(line: &str) -> Result<Self, serde_json::Error> {
           serde_json::from_str(line)
       }
   }
```

---

## Prompt 6: Tension Types

```
Implement tension types in crates/sim-events/src/tension.rs.

Based on docs/design/simulation_output_schemas.md Tension Schema:

1. TensionType enum:
   - BrewingBetrayal
   - SuccessionCrisis
   - ResourceConflict
   - ForbiddenAlliance
   - RevengeArc
   - RisingPower
   - FactionFracture
   - ExternalThreat
   - SecretExposed
   - RitualDisruption

   Serialize as snake_case.

2. TensionStatus enum:
   - Emerging
   - Escalating
   - Critical
   - Climax
   - Resolving
   - Resolved
   - Dormant

3. TensionAgent struct:
   - agent_id: String
   - role_in_tension: String  // "potential_betrayer", "unaware_target", etc.
   - trajectory: String  // "toward_defection", "stable", etc.

4. PredictedOutcome struct:
   - outcome: String
   - probability: f32
   - impact: String  // "low", "medium", "high", "critical"
   - estimated_ticks_until: Option<u64>

5. CameraRecommendation struct:
   - primary: Option<String>  // agent_id
   - secondary: Vec<String>   // agent_ids
   - locations_of_interest: Vec<String>

6. Tension struct:
   - tension_id: String  // "tens_00284"
   - detected_at_tick: u64
   - last_updated_tick: u64
   - status: TensionStatus
   - tension_type: TensionType
   - severity: f32  // 0.0 to 1.0
   - confidence: f32  // 0.0 to 1.0
   - summary: String
   - key_agents: Vec<TensionAgent>
   - key_locations: Vec<String>
   - trigger_events: Vec<String>  // event_ids
   - predicted_outcomes: Vec<PredictedOutcome>
   - narrative_hooks: Vec<String>
   - recommended_camera_focus: CameraRecommendation
   - connected_tensions: Vec<String>  // tension_ids

7. Tension ID generation:
   pub fn generate_tension_id(sequence: u64) -> String {
       format!("tens_{:05}", sequence)
   }

8. Test against the brewing_betrayal example from schema doc.

Update src/lib.rs to export tension types.
```

---

## Prompt 7: Agent Snapshot for World State

```
Implement agent snapshot types in crates/sim-events/src/snapshot.rs.

Based on the World Snapshot schema:

1. AgentTraits struct:
   - boldness: f32
   - loyalty_weight: f32
   - memory_strength: f32
   - grudge_persistence: f32

2. PhysicalState struct:
   - health: f32
   - hunger: f32
   - exhaustion: f32

3. Goal struct:
   - goal: String
   - priority: f32

4. AgentSnapshot struct (full agent state):
   - agent_id: String
   - name: String
   - alive: bool
   - faction: Option<String>  // None if factionless
   - role: String
   - location: String
   - traits: AgentTraits
   - physical_state: PhysicalState
   - goals: Vec<Goal>
   - inventory: Vec<String>
   - visual_markers: Vec<String>

5. Trust struct:
   - reliability: f32
   - alignment: f32
   - capability: f32
   - last_interaction_tick: Option<u64>
   - memory_count: u32
   - significant_memories: Vec<String>

6. RelationshipMap type alias:
   pub type RelationshipMap = HashMap<String, HashMap<String, Trust>>;
   // agent_id -> (other_agent_id -> Trust)

7. Helper methods:
   impl AgentSnapshot {
       pub fn is_alive(&self) -> bool
       pub fn has_faction(&self) -> bool
       pub fn highest_priority_goal(&self) -> Option<&Goal>
   }

   impl Trust {
       pub fn overall(&self) -> f32 {
           // Simple average or weighted combination
           (self.reliability + self.alignment + self.capability) / 3.0
       }
       
       pub fn is_positive(&self) -> bool {
           self.overall() > 0.5
       }
   }

Add tests for serialization matching schema examples.
```

---

## Prompt 8: Faction and Location Snapshots

```
Implement faction and location snapshots in crates/sim-events/src/snapshot.rs (continue).

1. FactionSnapshot struct:
   - faction_id: String
   - territory: Vec<String>  // location_ids
   - headquarters: String  // location_id
   - resources: FactionResources
   - member_count: u32
   - leader: Option<String>  // agent_id
   - reader: Option<String>  // agent_id (for ritual readings)
   - archive_entry_count: u32
   - cohesion_score: f32
   - external_reputation: HashMap<String, f32>  // faction_id -> reputation

2. FactionResources struct:
   - grain: u32
   - iron: u32
   - salt: u32
   // Extensible for other resources

3. LocationSnapshot struct:
   - location_id: String
   - location_type: String  // "village", "camp", "neutral_territory", etc.
   - controlling_faction: Option<String>
   - agents_present: Vec<String>
   - resources: HashMap<String, u32>
   - properties: Vec<String>  // "hidden_meeting_spot", "trade_route", etc.

4. WorldState struct (global state):
   - season: String
   - global_resources: HashMap<String, u32>
   - active_threats: Vec<String>

5. ComputedMetrics struct:
   - faction_power_balance: HashMap<String, f32>
   - war_probability_30_days: HashMap<String, f32>  // "faction1_vs_faction2" -> prob
   - agents_at_defection_risk: Vec<String>
   - factions_at_collapse_risk: Vec<String>

6. Helper methods:
   impl FactionSnapshot {
       pub fn total_resources(&self) -> u32
       pub fn has_leader(&self) -> bool
       pub fn controls_location(&self, location_id: &str) -> bool
   }

   impl LocationSnapshot {
       pub fn is_neutral(&self) -> bool
       pub fn agent_count(&self) -> usize
   }
```

---

## Prompt 9: Complete World Snapshot

```
Implement the complete WorldSnapshot in crates/sim-events/src/snapshot.rs.

1. WorldSnapshot struct:
   - snapshot_id: String  // "snap_003847"
   - timestamp: SimTimestamp
   - triggered_by: String  // "periodic_1000_ticks", "significant_event", etc.
   - world: WorldState
   - factions: Vec<FactionSnapshot>
   - agents: Vec<AgentSnapshot>
   - relationships: RelationshipMap
   - locations: Vec<LocationSnapshot>
   - computed_metrics: ComputedMetrics

2. Snapshot ID generation:
   pub fn generate_snapshot_id(sequence: u64) -> String {
       format!("snap_{:06}", sequence)
   }

3. Helper methods:
   impl WorldSnapshot {
       pub fn get_agent(&self, agent_id: &str) -> Option<&AgentSnapshot>
       pub fn get_faction(&self, faction_id: &str) -> Option<&FactionSnapshot>
       pub fn get_location(&self, location_id: &str) -> Option<&LocationSnapshot>
       pub fn agents_at_location(&self, location_id: &str) -> Vec<&AgentSnapshot>
       pub fn faction_members(&self, faction_id: &str) -> Vec<&AgentSnapshot>
       pub fn get_trust(&self, from: &str, to: &str) -> Option<&Trust>
       pub fn living_agents(&self) -> impl Iterator<Item = &AgentSnapshot>
   }

4. Index structs for faster lookups (optional but useful):
   pub struct SnapshotIndex {
       agents_by_id: HashMap<String, usize>,
       agents_by_location: HashMap<String, Vec<usize>>,
       agents_by_faction: HashMap<String, Vec<usize>>,
   }

   impl WorldSnapshot {
       pub fn build_index(&self) -> SnapshotIndex
   }

5. Test with full example from schema:
   - Create a snapshot programmatically
   - Serialize to JSON
   - Verify structure matches schema
   - Deserialize back and verify equality
```

---

## Prompt 10: Library Exports and Integration

```
Finalize the sim-events crate with proper exports and integration tests.

1. Update src/lib.rs with organized exports:

//! Shared event types for the emergent simulation.
//!
//! This crate defines the data structures used for communication between
//! simulation, director, and visualization layers.

mod timestamp;
mod event;
mod tension;
mod snapshot;

// Re-export main types
pub use timestamp::{SimTimestamp, SimDate, Season};
pub use event::{
    Event, EventType, EventBuilder,
    ActorSnapshot, ActorSet, AffectedActor,
    EventContext, EventOutcome,
    RelationshipChange, SharedMemory, ArchiveEntry, MemoryReinforcement,
    generate_event_id, drama_tags,
};
pub use tension::{
    Tension, TensionType, TensionStatus,
    TensionAgent, PredictedOutcome, CameraRecommendation,
    generate_tension_id,
};
pub use snapshot::{
    WorldSnapshot, WorldState, ComputedMetrics,
    AgentSnapshot, AgentTraits, PhysicalState, Goal, Trust,
    FactionSnapshot, FactionResources,
    LocationSnapshot,
    RelationshipMap, generate_snapshot_id,
};

2. Create tests/integration_test.rs:

use sim_events::*;

#[test]
fn test_create_movement_event() {
    let event = EventBuilder::new(EventType::Movement, "travel")
        .id("evt_00000001")
        .timestamp(SimTimestamp::new(1, 1, Season::Spring, 1))
        .primary_actor(ActorSnapshot::new(
            "agent_test_001",
            "Test Agent",
            "thornwood",
            "scout",
            "village_a",
        ))
        .trigger("scheduled_patrol")
        .drama_score(0.1)
        .build();
    
    assert_eq!(event.event_type, EventType::Movement);
    assert_eq!(event.subtype, "travel");
}

#[test]
fn test_create_betrayal_event() {
    // Create the full betrayal event from the schema example
    // Verify it serializes correctly
}

#[test]
fn test_tension_round_trip() {
    // Create tension, serialize, deserialize, compare
}

#[test]
fn test_snapshot_round_trip() {
    // Create minimal snapshot, serialize, deserialize, compare
}

#[test]
fn test_jsonl_format() {
    let event = /* create event */;
    let line = event.to_jsonl().unwrap();
    
    // Verify no newlines in output
    assert!(!line.contains('\n'));
    
    // Verify can parse back
    let parsed = Event::from_jsonl(&line).unwrap();
    assert_eq!(event.event_id, parsed.event_id);
}

3. Create tests/schema_conformance.rs:
   - Include the JSON examples from schema doc as string literals
   - Parse them to verify our types are compatible
   - This catches schema drift

4. Add doc comments to all public types with examples:
   /// A point in simulation time.
   ///
   /// # Example
   /// ```
   /// use sim_events::{SimTimestamp, Season};
   /// let ts = SimTimestamp::new(100, 1, Season::Spring, 15);
   /// assert_eq!(ts.tick, 100);
   /// ```
   pub struct SimTimestamp { ... }

5. Verify:
   cargo build -p sim-events
   cargo test -p sim-events
   cargo doc -p sim-events --open
```

---

## Prompt 11: Sample Data Fixtures

```
Create sample data fixtures for testing other crates.

1. Create crates/sim-events/tests/fixtures/ directory

2. Create sample_events.jsonl with diverse events:
   - 3-5 movement events (low drama)
   - 2-3 communication events (gossip, sharing memories)
   - 1 betrayal event (high drama)
   - 1 ritual reading event
   - 1 death event
   - 1 faction join event
   
   Each event should be a single line of valid JSON.
   Events should reference consistent agent/location IDs.

3. Create sample_tensions.json:
   - 1 brewing_betrayal tension (high severity)
   - 1 resource_conflict tension (medium severity)
   
   Tensions should reference events from sample_events.jsonl.

4. Create sample_state.json - a minimal but complete WorldSnapshot:
   - 2 factions (thornwood, ironmere)
   - 5-6 agents total
   - 3-4 locations
   - Some relationships with varying trust levels
   - Basic computed metrics

5. Create a fixtures module:
   // crates/sim-events/src/fixtures.rs
   
   pub fn sample_events() -> Vec<Event> {
       let jsonl = include_str!("../tests/fixtures/sample_events.jsonl");
       jsonl.lines()
           .filter(|l| !l.is_empty())
           .map(|l| Event::from_jsonl(l).unwrap())
           .collect()
   }
   
   pub fn sample_tensions() -> Vec<Tension> {
       let json = include_str!("../tests/fixtures/sample_tensions.json");
       serde_json::from_str(json).unwrap()
   }
   
   pub fn sample_snapshot() -> WorldSnapshot {
       let json = include_str!("../tests/fixtures/sample_state.json");
       serde_json::from_str(json).unwrap()
   }

6. Feature-gate fixtures for tests:
   // In Cargo.toml
   [features]
   test-fixtures = []
   
   // In lib.rs
   #[cfg(feature = "test-fixtures")]
   pub mod fixtures;

7. Other crates can use:
   [dev-dependencies]
   sim-events = { path = "../sim-events", features = ["test-fixtures"] }

This gives director and viz crates ready-made test data.
```

---

## Verification Checklist

After completing all prompts, verify:

- [ ] `cargo build -p sim-events` succeeds with no warnings
- [ ] `cargo test -p sim-events` all tests pass
- [ ] `cargo doc -p sim-events` generates documentation
- [ ] SimTimestamp serializes to `{"tick": N, "date": "year_X.season.day_Y"}`
- [ ] EventType serializes to snake_case strings
- [ ] Event can round-trip through JSON
- [ ] Tension can round-trip through JSON
- [ ] WorldSnapshot can round-trip through JSON
- [ ] JSONL format has no newlines within events
- [ ] Sample fixtures load correctly
- [ ] All public types have doc comments

---

## Type Reference

Quick reference for the key types:

```rust
// Timestamps
SimTimestamp { tick: u64, date: SimDate }
SimDate { year: u32, season: Season, day: u8 }
Season::Spring | Summer | Autumn | Winter

// Events
Event { event_id, timestamp, event_type, subtype, actors, context, outcome, drama_tags, drama_score, connected_events }
EventType::Movement | Communication | Betrayal | Loyalty | Conflict | Cooperation | Faction | Archive | Ritual | Resource | Death | Birth
ActorSnapshot { agent_id, name, faction, role, location }
ActorSet { primary, secondary, affected }

// Tensions
Tension { tension_id, status, tension_type, severity, confidence, summary, key_agents, predicted_outcomes, ... }
TensionType::BrewingBetrayal | SuccessionCrisis | ResourceConflict | ...
TensionStatus::Emerging | Escalating | Critical | Climax | Resolving | Resolved | Dormant

// Snapshots
WorldSnapshot { snapshot_id, timestamp, world, factions, agents, relationships, locations, computed_metrics }
AgentSnapshot { agent_id, name, alive, faction, role, location, traits, physical_state, goals, inventory, visual_markers }
FactionSnapshot { faction_id, territory, headquarters, resources, member_count, leader, reader, ... }
Trust { reliability, alignment, capability, ... }
```

---

## Next Steps

After sim-events is complete:
1. **sim-core** can build agents and emit events using these types
2. **director** can process events and tensions
3. **viz** can load and display snapshots

This crate should rarely change once stable—it's the contract between all other components.
