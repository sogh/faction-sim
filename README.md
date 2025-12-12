# Emergent Medieval Simulation Engine

A terrarium-style simulation where hundreds of agents with simple behavioral rules produce emergent, dramatically compelling narratives.

## What Is This?

A headless simulation engine that generates history organically. Agents navigate loyalty, survival, and ambition within a medieval world of competing factions. The simulation produces structured output for a Director AI to select what's worth visualizing and for LLMs to summarize into dramatic retellings.

## Core Concept

The tension this project explores: mathematically emergent systems often feel alien, while human-feeling simulations typically require complex rule sets. We aim for the sweet spot — simple rules that produce genuinely human drama.

## Documentation

- `docs/emergent_sim_design.md` — High-level vision and design principles
- `docs/simulation_output_schemas.md` — JSON schemas for all simulation output  
- `docs/agent_behavioral_rules.md` — Agent decision-making rules and weighting system
- `docs/implementation_prompts.md` — Phased prompts for building with Claude Code

## Project Structure

```
faction-sim/
├── README.md
├── CLAUDE.md                  # Claude Code project context
├── Cargo.toml
├── docs/
│   ├── emergent_sim_design.md
│   ├── simulation_output_schemas.md
│   ├── agent_behavioral_rules.md
│   └── implementation_prompts.md
├── src/
│   ├── main.rs                # Entry point, simulation loop
│   ├── lib.rs                 # Library exports
│   ├── components/            # ECS components
│   │   ├── agent.rs           # Traits, Needs, Goals
│   │   ├── social.rs          # Relationships, Memories, Trust
│   │   ├── faction.rs         # FactionMembership, Archive
│   │   └── world.rs           # Location, Season, WorldState
│   ├── systems/               # ECS systems
│   │   ├── perception.rs      # Agent awareness
│   │   ├── needs.rs           # Food security, belonging
│   │   ├── memory.rs          # Memory decay, propagation
│   │   ├── trust.rs           # Trust events, grudges
│   │   ├── ritual.rs          # Faction ritual readings
│   │   ├── tension.rs         # Tension detection
│   │   └── action/            # Action pipeline
│   │       ├── generate.rs    # List valid actions
│   │       ├── weight.rs      # Score by traits
│   │       ├── select.rs      # Probabilistic selection
│   │       └── execute.rs     # Perform and log events
│   ├── actions/               # Action definitions
│   │   ├── movement.rs
│   │   ├── communication.rs
│   │   └── archive.rs
│   ├── events/                # Event types and logging
│   │   ├── types.rs
│   │   └── logger.rs
│   ├── output/                # Output generation
│   │   ├── snapshot.rs        # World snapshots
│   │   ├── tension.rs         # Tension stream
│   │   └── schemas.rs         # Serialization structs
│   └── setup/                 # World initialization
│       ├── world.rs           # Locations
│       ├── factions.rs        # Factions, archives
│       └── agents.rs          # Agent spawning
└── output/                    # Generated output (gitignored)
```

## Key Features

- **Event Sourcing**: Every action produces an immutable event. Full history replay and branching.
- **Multi-Dimensional Trust**: Agents track reliability, alignment, and capability separately.
- **Memory Propagation**: Gossip spreads with attribution and fidelity loss.
- **Faction Archives**: Written records that can be read, destroyed, or forged.
- **Ritual Readings**: Scheduled gatherings that reinforce faction memory selectively.
- **Tension Detection**: Pre-computed drama indicators for the Director AI.
- **Intervention System**: Pause and modify the simulation at any point.

## Getting Started

### Prerequisites

- Rust toolchain (install via [rustup](https://rustup.rs/))

### Running the Simulation

```bash
# Build and run with default settings (1000 ticks, seed 42)
cargo run

# Run with a specific seed for reproducibility
cargo run -- --seed 12345

# Run for a specific number of ticks
cargo run -- --ticks 5000

# Adjust ritual frequency (default: 500 ticks)
cargo run -- --ritual-interval 100

# Output initial world state as JSON
cargo run -- --output-initial-state

# Combine options
cargo run -- --seed 99 --ticks 2000 --ritual-interval 200
```

### Command Line Options

| Option | Default | Description |
|--------|---------|-------------|
| `--seed` | 42 | Random seed for reproducibility |
| `--ticks` | 1000 | Number of simulation ticks to run |
| `--snapshot-interval` | 100 | Ticks between world snapshots |
| `--ritual-interval` | 500 | Ticks between faction rituals |
| `--output-initial-state` | false | Export initial locations/factions as JSON |

### Viewing Output

The simulation generates several output files in the `output/` directory:

```
output/
├── current_state.json      # Latest world snapshot (overwritten each interval)
├── tensions.json           # Active dramatic tensions for Director AI
├── snapshots/
│   ├── snap_0000000000.json    # Initial state
│   ├── snap_0000000100.json    # Periodic snapshots
│   └── ...
├── initial_locations.json  # (if --output-initial-state)
└── initial_factions.json   # (if --output-initial-state)
```

**Snapshot files** contain the full world state at that tick:
- All agents with their traits, needs, goals, and positions
- Faction information including resources and leadership
- Relationship summaries and memory counts

**Tensions file** shows detected dramatic situations:
- Brewing betrayals, succession crises, forbidden alliances
- Severity scores and predicted outcomes
- Narrative hooks for storytelling

Example: View the current tensions
```bash
cat output/tensions.json | jq .
```

Example: Check agent distribution by faction
```bash
cat output/current_state.json | jq '.agents | group_by(.faction) | map({faction: .[0].faction, count: length})'
```

### Console Output

During simulation, you'll see periodic updates:
```
[Tick  100] year_1.spring.day_11 - 212 events (moves: 21, comms: 190, RITUALS: 1)
[Tick  200] year_1.spring.day_21 - 209 events (moves: 24, comms: 184)
```

This shows:
- Current tick and in-world date
- Total events generated
- Breakdown by type (movement, communication, rituals, archive actions)

## Development

### Running Tests

```bash
cargo test
```

### Project Documentation

1. Read `docs/emergent_sim_design.md` to understand the vision
2. Review `docs/agent_behavioral_rules.md` for how agents decide
3. Follow prompts in `docs/implementation_prompts.md` to build incrementally
4. Reference `CLAUDE.md` for coding conventions and architecture

## Technical Stack

- **Language**: Rust
- **ECS**: Bevy ECS
- **Output**: JSON/JSONL for human and LLM readability
- **Target**: 200-500 agents, 1000+ ticks/second

## Current Implementation Status

The simulation engine is functional with the following systems implemented:

- **Perception**: Agents aware of others at same location
- **Needs**: Food security and social belonging states
- **Movement**: Wander, patrol, return home actions
- **Communication**: Share memories, spread information
- **Memory**: Decay over time, secondhand propagation
- **Trust**: Multi-dimensional trust with grudge formation
- **Archives**: Faction records that can be written/read
- **Rituals**: Periodic faction gatherings with archive readings
- **Tension Detection**: 10 tension types for Director AI

**63 tests** covering core functionality.

## License

MIT

---

*Built with Claude Code, December 2024*
