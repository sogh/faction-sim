# sim-core Crate

## Purpose

The simulation engine. Agents with behavioral rules living in a world of factions, scarcity, and political intrigue. This crate runs the simulation and emits events—it knows nothing about visualization or drama detection.

## Key Files

```
src/
├── lib.rs          # Public API, World struct
├── agent.rs        # Agent struct, behavior loop
├── trust.rs        # Multi-dimensional trust model
├── memory.rs       # Agent memory, propagation rules
├── faction.rs      # Faction struct, archive, rituals
└── world.rs        # World state, tick execution, event emission
```

## Design Doc Reference

See `/docs/design/emergent_sim_design.md` for the core concepts.

## Core Concepts

### Agents
Individual characters with:
- **Traits**: boldness, loyalty_weight, memory_strength, grudge_persistence
- **Physical state**: health, hunger, exhaustion
- **Goals**: prioritized objectives that drive behavior
- **Inventory**: items they carry
- **Relationships**: trust values toward other agents

### Trust Model (Multi-dimensional)
Not a single "trust" value—three separate dimensions:

| Dimension | Question |
|-----------|----------|
| Reliability | Do they do what they say? |
| Alignment | Do they want what I want? |
| Capability | Can they actually deliver? |

These can diverge: trust someone's intentions but not competence.

### Memory System
- **Firsthand**: Events witnessed directly, full fidelity
- **Secondhand**: Told by others, degraded fidelity
- **Source attribution**: "A claims C is a traitor"—if A becomes unreliable, re-evaluate
- **Faction filtering**: In-group sources believed more readily

### Factions
- Exclusive membership (no dual enrollment)
- Territory and resources
- **Archive**: Written records, can be forged/destroyed
- **Reader**: Agent who conducts ritual readings
- **Ritual readings**: Reinforce selected memories, shape faction beliefs

## Key Types

```rust
pub struct Agent {
    pub id: AgentId,
    pub name: String,
    pub faction: Option<FactionId>,
    pub role: Role,
    pub location: LocationId,
    pub traits: AgentTraits,
    pub physical_state: PhysicalState,
    pub goals: Vec<Goal>,
    pub inventory: Vec<Item>,
}

pub struct Trust {
    pub reliability: f32,  // 0.0 - 1.0
    pub alignment: f32,
    pub capability: f32,
}

pub struct Memory {
    pub event_id: EventId,
    pub source: MemorySource,  // Firsthand, Secondhand { from, chain }
    pub fidelity: f32,
    pub acquired_tick: u64,
}
```

## Simulation Loop

```rust
impl World {
    pub fn tick(&mut self) -> Vec<Event> {
        let mut events = Vec::new();
        
        // 1. Environmental updates (season, resources)
        events.extend(self.update_environment());
        
        // 2. Each agent acts
        for agent_id in self.agent_order() {
            if let Some(action) = self.decide_action(agent_id) {
                events.extend(self.execute_action(agent_id, action));
            }
        }
        
        // 3. Scheduled events (rituals, etc.)
        events.extend(self.process_scheduled());
        
        // 4. Update relationships based on events
        self.propagate_consequences(&events);
        
        self.current_tick += 1;
        events
    }
}
```

## Behavior Decision

Agents choose actions based on:
1. **Needs**: Hunger, safety, belonging
2. **Goals**: Explicit objectives with priorities
3. **Relationships**: Trust toward relevant agents
4. **Opportunities**: What's available at current location

**Keep rules simple.** Complexity emerges from interaction, not from complicated individual logic.

## Implementation Phases

### Phase 1: Minimal Agent Loop
- [ ] Agent struct with basic traits
- [ ] Location system (agents exist somewhere)
- [ ] Movement action (travel between locations)
- [ ] Event emission for actions
- [ ] Basic tick loop

### Phase 2: Trust and Memory
- [ ] Trust struct with three dimensions
- [ ] Relationship storage (agent → agent → trust)
- [ ] Memory struct with source tracking
- [ ] Memory propagation (sharing information)
- [ ] Trust updates based on observed events

### Phase 3: Factions
- [ ] Faction struct with territory, resources
- [ ] Faction membership, roles
- [ ] Archive system (write, read, destroy entries)
- [ ] Ritual reading system
- [ ] Faction-level resource management

### Phase 4: Full Behavior
- [ ] Goal system with priorities
- [ ] Need-based action selection
- [ ] Betrayal conditions and execution
- [ ] Cross-faction interactions
- [ ] Environmental pressure (seasons, scarcity)

## Testing Strategy

1. **Unit tests**: Individual components (trust updates, memory fidelity decay)
2. **Scenario tests**: Set up situation, run N ticks, verify outcomes
3. **Invariant tests**: Run long simulations, check nothing impossible happens
4. **Emergence tests**: Verify interesting behaviors emerge (betrayals happen, factions conflict)

```rust
#[test]
fn test_broken_promise_reduces_reliability_trust() {
    let mut world = World::new_test();
    let alice = world.add_agent("Alice", "thornwood");
    let bob = world.add_agent("Bob", "thornwood");
    
    // Bob promises Alice something
    world.create_promise(bob, alice, "help_harvest");
    
    // Bob breaks the promise
    world.break_promise(bob, alice, "help_harvest");
    
    // Alice's reliability trust in Bob should decrease
    let trust = world.get_trust(alice, bob);
    assert!(trust.reliability < 0.5);
}
```

## Event Emission

Every significant action must emit an event. The simulation's "memory" is the event log.

```rust
fn execute_movement(&mut self, agent_id: AgentId, destination: LocationId) -> Event {
    let agent = self.agents.get_mut(&agent_id).unwrap();
    let from = agent.location.clone();
    agent.location = destination.clone();
    
    Event {
        event_id: self.next_event_id(),
        timestamp: self.current_timestamp(),
        event_type: EventType::Movement,
        subtype: "travel".into(),
        actors: ActorSet::primary(self.agent_snapshot(agent_id)),
        context: EventContext {
            trigger: "agent_decision".into(),
            preconditions: vec![],
        },
        outcome: EventOutcome {
            new_location: Some(destination),
            // ...
        },
        drama_tags: vec![],
        drama_score: 0.05,  // Movement is low drama
        connected_events: vec![],
    }
}
```

## Dependencies

- `sim-events`: Event types, serialization
- `rand`: Random decisions, weighted choices
- `tracing`: Logging agent decisions

## Gotchas

- Agent order matters for fairness—shuffle each tick
- Don't let agents act on information they shouldn't have
- Memory fidelity degrades on sharing, not over time
- Trust updates should be gradual, not instant flips
- Keep action logic simple—complexity emerges from interactions
