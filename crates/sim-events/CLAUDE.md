# sim-events Crate

## Purpose

Shared event types and serialization for the entire simulation. This crate is a dependency for all other crates—it defines the common language they speak.

**This crate has no logic**, only data structures and their serialization. Keep it that way.

## Key Files

```
src/
├── lib.rs          # Re-exports
├── event.rs        # Event struct, EventType, actor structs
├── timestamp.rs    # SimTimestamp, tick/date handling
├── tension.rs      # Tension struct, TensionType, predictions
└── snapshot.rs     # WorldSnapshot, AgentSnapshot, FactionSnapshot
```

## Design Doc Reference

See `/docs/design/simulation_output_schemas.md` for the complete JSON schemas these types must serialize to.

## Core Types

### SimTimestamp
```rust
pub struct SimTimestamp {
    pub tick: u64,
    pub date: SimDate,  // year_N.season.day_M format
}
```

### Event
```rust
pub struct Event {
    pub event_id: String,        // "evt_00042371"
    pub timestamp: SimTimestamp,
    pub event_type: EventType,
    pub subtype: String,
    pub actors: ActorSet,
    pub context: EventContext,
    pub outcome: EventOutcome,
    pub drama_tags: Vec<String>,
    pub drama_score: f32,
    pub connected_events: Vec<String>,
}
```

### Tension
```rust
pub struct Tension {
    pub tension_id: String,      // "tens_00284"
    pub detected_at_tick: u64,
    pub last_updated_tick: u64,
    pub status: TensionStatus,
    pub tension_type: TensionType,
    pub severity: f32,
    pub confidence: f32,
    pub summary: String,
    pub key_agents: Vec<TensionAgent>,
    pub predicted_outcomes: Vec<PredictedOutcome>,
    // ...
}
```

## Serialization Requirements

All types must:
- Derive `Serialize, Deserialize` from serde
- Use `#[serde(rename_all = "snake_case")]` for enums
- Match the JSON schema exactly—the director and viz depend on this

Example:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Movement,
    Communication,
    Betrayal,
    Loyalty,
    Conflict,
    Cooperation,
    Faction,
    Archive,
    Ritual,
    Resource,
    Death,
    Birth,
}
```

## JSON Lines Format

Events serialize to JSON Lines (`.jsonl`)—one JSON object per line:
```
{"event_id":"evt_00000001","timestamp":{"tick":1,...},...}
{"event_id":"evt_00000002","timestamp":{"tick":2,...},...}
```

Use `serde_json::to_string()` (not `to_string_pretty()`) for events.

## Implementation Status

- [ ] SimTimestamp and SimDate
- [ ] EventType enum with all variants
- [ ] Event struct with all fields
- [ ] ActorSet (primary, secondary, affected)
- [ ] EventContext and EventOutcome
- [ ] TensionType enum
- [ ] Tension struct
- [ ] WorldSnapshot struct
- [ ] AgentSnapshot struct
- [ ] FactionSnapshot struct
- [ ] Serialization tests against schema examples

## Testing Strategy

1. **Round-trip tests**: Serialize → deserialize → compare
2. **Schema conformance**: Parse the example JSON from design docs
3. **Edge cases**: Empty vectors, optional fields, Unicode names

```rust
#[test]
fn test_event_roundtrip() {
    let event = Event { /* ... */ };
    let json = serde_json::to_string(&event).unwrap();
    let parsed: Event = serde_json::from_str(&json).unwrap();
    assert_eq!(event, parsed);
}

#[test]
fn test_parse_schema_example() {
    let json = include_str!("../../docs/schemas/event_example.json");
    let event: Event = serde_json::from_str(json).unwrap();
    assert_eq!(event.event_type, EventType::Betrayal);
}
```

## Dependencies

- `serde` + `serde_json`: Serialization
- `uuid`: ID generation (optional, IDs could be sequential strings)

## Gotchas

- `drama_score` is `f32`, not `f64`—matches schema, saves space
- Event IDs are strings like `"evt_00042371"`, not integers
- `connected_events` can be empty vector, not Option
- Timestamps have both `tick` (monotonic) and `date` (human-readable)
