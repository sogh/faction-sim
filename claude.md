# Emergent Simulation Engine

## Project Overview

A terrarium-style medieval simulation where hundreds of agents with simple behavioral rules produce emergent, dramatically compelling narratives. The simulation runs headlessly and produces structured output for a separate Director AI and visualization layer.

## Core Documents

Read these before making significant changes:

- `docs/emergent_sim_design.md` — High-level vision and design principles
- `docs/simulation_output_schemas.md` — JSON schemas for all simulation output
- `docs/agent_behavioral_rules.md` — Agent decision-making rules and weighting system

## Technical Stack

- **Language**: Rust
- **ECS Framework**: Bevy ECS (using Bevy's ECS without the rendering/game loop)
- **Serialization**: serde + serde_json
- **Event Storage**: Append-only JSONL files
- **Random**: rand crate with seedable RNG for reproducibility

## Architecture Principles

### Event Sourcing
Every significant action produces an immutable event. The simulation state at any tick can be reconstructed from the event stream. Events are self-contained — they include actor state at the moment of the event, not just IDs.

### ECS Structure
- **Entities**: Agents, Locations, Factions, Archive Entries
- **Components**: Traits, Needs, Goals, Relationships, Memories, Position, FactionMembership, Status
- **Systems**: Perception, NeedUpdate, ActionGeneration, ActionSelection, ActionExecution, EventLogging, TensionDetection

### Determinism
Given the same seed and initial conditions, the simulation should produce identical results. All randomness flows through a single seeded RNG. This enables replay, debugging, and branching.

## Directory Structure

```
src/
├── main.rs              # Entry point, simulation loop
├── lib.rs               # Public API for the simulation
├── components/          # ECS components
│   ├── mod.rs
│   ├── agent.rs         # Traits, Needs, Goals
│   ├── social.rs        # Relationships, Memories, Trust
│   ├── faction.rs       # FactionMembership, Archive, Status
│   └── world.rs         # Location, Resources, Season
├── systems/             # ECS systems
│   ├── mod.rs
│   ├── perception.rs    # Update agent awareness
│   ├── needs.rs         # Update food_security, belonging
│   ├── action/          # Action generation and execution
│   │   ├── mod.rs
│   │   ├── generate.rs  # List valid actions
│   │   ├── weight.rs    # Score actions
│   │   ├── select.rs    # Probabilistic selection
│   │   └── execute.rs   # Perform action, produce event
│   ├── memory.rs        # Memory decay, propagation effects
│   ├── tension.rs       # Detect and update tensions
│   └── snapshot.rs      # Periodic state capture
├── events/              # Event types and logging
│   ├── mod.rs
│   ├── types.rs         # All event type definitions
│   └── logger.rs        # JSONL append-only writer
├── actions/             # Action definitions
│   ├── mod.rs
│   ├── movement.rs
│   ├── communication.rs
│   ├── social.rs
│   ├── resource.rs
│   ├── archive.rs
│   ├── faction.rs
│   └── conflict.rs
├── output/              # Output generation
│   ├── mod.rs
│   ├── schemas.rs       # Serialization structs matching output schemas
│   ├── snapshot.rs      # World snapshot generation
│   └── tension.rs       # Tension stream generation
└── setup/               # World initialization
    ├── mod.rs
    ├── world.rs         # Create locations, resources
    ├── factions.rs      # Create factions, assign territories
    └── agents.rs        # Spawn agents with traits
```

## Key Implementation Notes

### Trust is Multi-Dimensional
Never store trust as a single value. Always track:
- `reliability`: Do they do what they say?
- `alignment`: Do they want what I want?
- `capability`: Can they deliver?

### Memories Have Attribution
When Agent A tells Agent B about Agent C, B's memory stores:
- The content
- The source chain (who told whom)
- The fidelity (degrades with each hop)

### Actions are Weighted Probabilistically
Agents don't pick the "best" action. They pick probabilistically from weighted options. This prevents deterministic behavior while still respecting personality.

```rust
// Pseudocode for action selection
let weights: Vec<(Action, f32)> = generate_weighted_actions(agent, world);
let selected = weighted_random_choice(&mut rng, &weights);
```

### Events are Self-Contained
Each event includes enough context to understand it without loading other state:
```rust
// Good: includes actor state
actors: {
    primary: { agent_id, name, faction, role, location }
}

// Bad: just IDs requiring lookup
actors: {
    primary: "agent_mira_0042"
}
```

### Needs are Abstracted
Don't model calorie counts. Model security states:
```rust
enum FoodSecurity { Secure, Stressed, Desperate }
enum SocialBelonging { Integrated, Peripheral, Isolated }
```

## Coding Style

- Prefer clarity over cleverness
- Use descriptive names even if long: `calculate_betrayal_weight` not `calc_btrl_wt`
- Comment the *why*, not the *what*
- Each system should do one thing
- Keep action execution logic separate from action weighting logic

## Testing Strategy

- Unit test individual weight calculations
- Integration test action sequences (setup → action → verify event)
- Property test that events are self-contained (parse event without world state)
- Snapshot test output format stability
- Determinism test: same seed = same output

## Output Files

The simulation produces:
- `output/events.jsonl` — Append-only event log
- `output/snapshots/snap_{tick}.json` — Periodic full state
- `output/tensions.json` — Current tension state (overwritten)
- `output/current_state.json` — Latest world state (overwritten each tick)

## Running the Simulation

```bash
# Run with default settings
cargo run

# Run with specific seed for reproducibility
cargo run -- --seed 12345

# Run for specific number of ticks
cargo run -- --ticks 10000

# Run with snapshot interval
cargo run -- --snapshot-interval 1000
```

## Intervention System

The simulation supports pausing and injecting changes via JSON files in `interventions/`:
- Modifications are applied at the start of the next tick
- Interventions are logged as special events
- Format matches the component schemas

## Common Tasks

### Adding a New Action
1. Define action struct in `actions/{category}.rs`
2. Add precondition check in `systems/action/generate.rs`
3. Add weight calculation in `systems/action/weight.rs`
4. Add execution logic in `systems/action/execute.rs`
5. Add event type in `events/types.rs`

### Adding a New Trait
1. Add field to `components/agent.rs`
2. Update agent spawning in `setup/agents.rs`
3. Add weight modifiers in relevant `systems/action/weight.rs` rules
4. Update snapshot schema in `output/schemas.rs`

### Adding a New Tension Type
1. Add variant to tension enum in `output/tension.rs`
2. Add detection logic in `systems/tension.rs`
3. Document in `docs/simulation_output_schemas.md`

## Performance Targets

- Hundreds of agents (200-500) simulating concurrently
- 1000+ ticks per second when not writing snapshots
- Event log can grow unbounded; snapshots ~1-5MB each

## Prompt Log Rule

- Manage the folder prompt-log.
- BEFORE BEGINNING WORK:  If you aren't currently working in a prompt log, start a new markdown file there that contains the conversation topic and date time as a file name. In the file you will record the date, time, hostname, your name, and the verbatim user prompt.
- After completing work, you will summarize the work and append it to the same file. We can use the same file for an entire session, but new sessions will generally need new files. This will be a useful context storage for historical purposes.
