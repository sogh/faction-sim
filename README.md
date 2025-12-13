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
├── tuning.toml                # Simulation tuning parameters
├── docs/
│   ├── emergent_sim_design.md
│   ├── simulation_output_schemas.md
│   ├── agent_behavioral_rules.md
│   └── implementation_prompts.md
├── src/
│   ├── main.rs                # Entry point, simulation loop
│   ├── lib.rs                 # Library exports
│   ├── config.rs              # Tuning configuration loader
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
│   │   ├── archive.rs
│   │   ├── resource.rs        # Work, trade, steal, hoard
│   │   ├── social.rs          # Build trust, gift, ostracize
│   │   ├── faction.rs         # Defect, exile, challenge leader
│   │   └── conflict.rs        # Argue, fight, sabotage
│   ├── events/                # Event types and logging
│   │   ├── types.rs
│   │   ├── logger.rs
│   │   └── drama.rs           # Drama scoring for Director AI
│   ├── output/                # Output generation
│   │   ├── snapshot.rs        # World snapshots
│   │   ├── tension.rs         # Tension stream
│   │   ├── stats.rs           # Statistics output
│   │   └── schemas.rs         # Serialization structs
│   ├── interventions/         # Runtime modification system
│   │   └── mod.rs             # JSON-based interventions
│   └── setup/                 # World initialization
│       ├── world.rs           # Locations
│       ├── factions.rs        # Factions, archives
│       └── agents.rs          # Agent spawning
├── interventions/             # Drop JSON files here for runtime changes
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
├── stats.json              # Simulation statistics for analysis
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

## Tuning Configuration

The simulation behavior can be customized via `tuning.toml` without recompiling. Edit this file to adjust agent behavior weights.

### Configuration Sections

| Section | Description |
|---------|-------------|
| `[simulation]` | Default ticks, snapshot interval, ritual interval |
| `[agents]` | Agents per faction, specialist count |
| `[movement]` | Travel, patrol, wander, flee base weights and trait modifiers |
| `[communication]` | Share memory, gossip, lie weights and relationship modifiers |
| `[resource]` | Work, trade, steal, hoard weights and need-based modifiers |
| `[social]` | Build trust, gift, ostracize weights |
| `[faction]` | Defect, exile, challenge leader weights |
| `[conflict]` | Argue, fight, sabotage, assassinate weights |
| `[archive]` | Write, read, destroy, forge weights |
| `[memory]` | Decay rates, fidelity multipliers |
| `[trust]` | Trust decay, grudge threshold |
| `[drama]` | Drama score thresholds and multipliers |

### Example: Making Agents More Aggressive

```toml
[conflict]
argue_base = 0.25      # Increase from 0.15
fight_base = 0.10      # Increase from 0.05
boldness_fight_bonus = 0.5  # Bold agents fight more
```

### Example: Increasing Faction Loyalty

```toml
[faction]
defect_base = 0.005    # Decrease from 0.02
loyalty_defect_penalty = 0.6  # Increase from 0.4

[communication]
same_faction_bonus = 0.4  # Increase from 0.2
```

If `tuning.toml` is missing or has parsing errors, the simulation uses built-in defaults.

## Runtime Interventions

You can modify the simulation at runtime by dropping JSON files into the `interventions/` directory. The simulation checks this directory each tick and applies any valid interventions.

### Intervention Types

**ModifyAgent** - Change agent traits, needs, or goals:
```json
{
    "id": "boost_ambition_001",
    "reason": "Testing ambitious behavior",
    "intervention": {
        "type": "modify_agent",
        "agent_id": "agent_thornwood_0001",
        "traits": {
            "ambition": 0.95,
            "boldness": 0.9
        },
        "needs": {
            "food_security": "desperate"
        }
    }
}
```

**ModifyRelationship** - Change trust between agents:
```json
{
    "id": "create_rivalry",
    "intervention": {
        "type": "modify_relationship",
        "from_agent": "agent_001",
        "to_agent": "agent_002",
        "reliability": -0.8,
        "alignment": -0.5
    }
}
```

**MoveAgent** - Relocate an agent:
```json
{
    "id": "exile_move",
    "intervention": {
        "type": "move_agent",
        "agent_id": "agent_traitor",
        "location_id": "wilderness_north"
    }
}
```

**ChangeFaction** - Force faction change:
```json
{
    "id": "defection_scenario",
    "intervention": {
        "type": "change_faction",
        "agent_id": "agent_spy",
        "new_faction_id": "ironmere",
        "new_role": "newcomer"
    }
}
```

**AddGoal** - Add a goal to an agent:
```json
{
    "id": "revenge_arc",
    "intervention": {
        "type": "add_goal",
        "agent_id": "agent_victim",
        "goal_type": "revenge",
        "target": "agent_betrayer",
        "priority": 0.9
    }
}
```

### How It Works

1. Drop a `.json` file into `interventions/`
2. The simulation reads and applies it at the start of the next tick
3. The intervention is logged as a special event
4. The file is automatically deleted after processing

This enables:
- Interactive debugging and testing
- Setting up specific scenarios
- Creating dramatic situations for the Director AI
- Live adjustments without restarting

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

The simulation engine is fully functional with all major systems implemented:

- **Perception**: Agents aware of others at same location
- **Needs**: Food security and social belonging states
- **Movement**: Travel, patrol, wander, flee actions
- **Communication**: Share memories, spread rumors, lie, confess
- **Resource**: Work, trade, steal, hoard actions
- **Social**: Build trust, curry favor, gift, ostracize
- **Faction**: Defect, exile, challenge/support leader
- **Conflict**: Argue, fight, sabotage, assassinate
- **Memory**: Decay over time, secondhand propagation
- **Trust**: Multi-dimensional trust with grudge formation
- **Archives**: Faction records that can be written/read/destroyed/forged
- **Rituals**: Periodic faction gatherings with archive readings
- **Tension Detection**: 10 tension types for Director AI
- **Drama Scoring**: Automatic scoring and tagging of dramatic events
- **Intervention System**: Runtime modification via JSON files
- **Tuning Config**: External configuration without recompiling
- **Statistics**: Event analysis and faction health metrics

**98 tests** covering core functionality and determinism verification.

## License

MIT

---

*Built with Claude Code, December 2024-2025*
