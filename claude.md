# Emergent Medieval Simulation

## Project Vision

A terrarium-style simulation where hundreds of agents with simple behavioral rules produce unpredictable, dramatically compelling emergent narratives. The system generates history organically, which can be observed in real-time or summarized into dramatic retellings.

The core tension: mathematically emergent systems often feel alien, while human-feeling simulations typically require complex rule sets. We aim for the sweet spot—simple rules that produce genuinely human drama.

## Architecture

Monorepo with Cargo workspace. Four main crates:

| Crate | Purpose |
|-------|---------|
| `sim-events` | Shared event types, timestamps, serialization (dependency for all others) |
| `sim-core` | Agent logic, trust model, memory system, factions, world simulation |
| `director` | Watches events, detects drama, outputs camera instructions |
| `viz` | Bevy-based renderer, consumes director output, user interaction |

## Data Flow

```
┌─────────────┐     events.jsonl      ┌──────────┐    camera_script.json    ┌─────┐
│  sim-core   │ ───────────────────▶  │ director │ ──────────────────────▶  │ viz │
└─────────────┘                       └──────────┘                          └─────┘
       │                                    │
       │ snapshots/                         │ tensions.json
       ▼                                    ▼
   [analysis]                          [analysis]
```

## Key Design Principles

1. **Event Sourcing**: All state changes are immutable events. State at any moment is rebuildable from the event stream.

2. **Simulation Ahead of Camera**: Sim runs faster than visualization. Director has foresight to create dramatic irony.

3. **Separation of Concerns**:
   - Simulation decides *what happens*
   - Director decides *what matters*
   - Visualization decides *how it looks*

4. **Emergence Through Constraint**: Drama comes from incompatibilities—exclusive faction membership, scarce resources, conflicting loyalties.

## Design Documents

Detailed specifications live in `docs/design/`:
- `emergent_sim_design.md` — Core simulation concepts and agent behavior
- `simulation_output_schemas.md` — JSON schemas for events, snapshots, tensions
- `director_ai_design.md` — Drama detection and camera control
- `visualization_design.md` — Rendering, sprites, UI

**Read the relevant design doc before implementing a feature.**

## Current Phase

<!-- Update this as you progress -->
**Status**: Project scaffolding
**Next**: Implement sim-events types based on output schemas

### Milestone Tracker
- [ ] sim-events: Core types (Event, Timestamp, Tension, Snapshot)
- [ ] sim-core Phase 1: Agent behavior loop with minimal rules
- [ ] sim-core Phase 2: Trust and memory systems
- [ ] sim-core Phase 3: Factions and archive system
- [ ] director Phase 1: Template-based (scoring, focus, commentary)
- [ ] director Phase 2: Pattern detection
- [ ] director Phase 3: LLM integration
- [ ] viz Phase 1: Core rendering
- [ ] viz Phase 2: Director integration
- [ ] viz Phase 3: Sprite system

## Code Conventions

- Rust 2021 edition
- Use `thiserror` for error types
- Use `tracing` for logging (not `println!`)
- Prefer composition over deep trait hierarchies
- Events are the source of truth—state is derived
- All public types need doc comments
- Use `cargo fmt` and `cargo clippy` before committing

## Common Commands

```bash
# Build everything
cargo build --workspace

# Run tests
cargo test --workspace

# Check a specific crate
cargo check -p sim-core

# Run the simulation (once implemented)
cargo run -p sim-core --bin simulate

# Run visualization (once implemented)
cargo run -p viz

# Format and lint
cargo fmt --all
cargo clippy --workspace
```

## File Naming Conventions

- Rust modules: `snake_case.rs`
- Design docs: `snake_case.md`
- Config files: `snake_case.toml`
- JSON output: `snake_case.json` or `snake_case.jsonl`

## When Starting Work

1. Check this CLAUDE.md for current phase
2. Read the crate-specific CLAUDE.md for the area you're working on
3. Reference design docs in `docs/design/` for specifications
4. Update milestone tracker when completing features

## Prompt Log Rule

- Manage the folder prompt-log.
- BEFORE BEGINNING WORK:  If you aren't currently working in a prompt log, start a new markdown file there that contains the conversation topic and date time as a file name. In the file you will record the date, time, hostname, your name, and the verbatim user prompt.
- After completing work, you will summarize the work and append it to the same file. We can use the same file for an entire session, but new sessions will generally need new files. This will be a useful context storage for historical purposes.

## Gotchas and Notes

<!-- Add things you discover here -->
- Bevy 0.14 has breaking changes from 0.13—check migration guide if using old examples
- JSON Lines (`.jsonl`) for events: one JSON object per line, no array wrapper
- Agent IDs are prefixed with `agent_` for grep-ability
- Event IDs are prefixed with `evt_`, tensions with `tens_`, snapshots with `snap_`
