# Prompt Log: Monorepo Restructure

**Date**: 2025-12-13
**Host**: Windows workstation
**Assistant**: Claude Opus 4.5

## User Prompt

> Read SETUP_INSTRUCTIONS.md and follow it.

## Summary

The user requested to restructure the existing faction-sim project into a Cargo workspace monorepo with four crates:
- `sim-events` - Shared event types and serialization
- `sim-core` - Core simulation logic
- `director` - Director AI for drama detection
- `viz` - Bevy-based visualization

## Work Log

### Completed Steps

1. **Created directory structure**:
   - `crates/sim-core/src/`
   - `crates/sim-events/src/`
   - `crates/director/src/`
   - `crates/viz/src/`
   - `docs/design/`
   - `docs/schemas/`
   - `tools/`

2. **Moved design docs** to `docs/design/`:
   - `emergent_sim_design.md`
   - `simulation_output_schemas.md`
   - `agent_behavioral_rules.md`
   - `implementation_prompts.md`

3. **Created root workspace Cargo.toml** with workspace dependencies

4. **Created crate Cargo.toml files** for all four crates

5. **Created lib.rs files** with module declarations for each crate

6. **Created module stub files** (TODO: implement) for each crate

7. **Copied CLAUDE.md files** to root and each crate directory

8. **Updated .gitignore** with additional patterns

9. **Verified setup** with `cargo build --workspace` - successful

### Final Structure

```
faction-sim/
├── CLAUDE.md
├── Cargo.toml (workspace)
├── crates/
│   ├── sim-core/
│   │   ├── CLAUDE.md
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── agent.rs
│   │       ├── trust.rs
│   │       ├── memory.rs
│   │       ├── faction.rs
│   │       └── world.rs
│   ├── sim-events/
│   │   ├── CLAUDE.md
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── event.rs
│   │       ├── timestamp.rs
│   │       ├── tension.rs
│   │       └── snapshot.rs
│   ├── director/
│   │   ├── CLAUDE.md
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── scorer.rs
│   │       ├── focus.rs
│   │       ├── threads.rs
│   │       ├── commentary.rs
│   │       └── output.rs
│   └── viz/
│       ├── CLAUDE.md
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── camera.rs
│           ├── sprites.rs
│           ├── overlay.rs
│           ├── state_loader.rs
│           └── plugin.rs
├── docs/
│   ├── design/
│   └── schemas/
└── tools/
```

---

## Second Session: Code Migration

**User Prompt**: `remove that .tar.gz and start migrating the src/ files into the appropriate crates and make sure to build to make sure it still works when done`

### Migration Completed

1. **Removed** `project_scaffold.tar.gz`

2. **Migrated sim-events crate**:
   - Created `event.rs` with all event types (EventType, EventSubtype, Event, etc.)
   - Created `timestamp.rs` with EventTimestamp and SnapshotTimestamp
   - Created `tension.rs` with tension data types (TensionType, Tension, etc.)
   - Created `snapshot.rs` with world snapshot types (WorldSnapshot, AgentSnapshot, etc.)
   - Updated `lib.rs` with proper re-exports

3. **Migrated sim-core crate**:
   - Copied all source files from `src/` to `crates/sim-core/src/`
   - Updated `main.rs` to use `sim_core` instead of `emergent_sim`
   - Added required dependencies (bevy_ecs, clap, toml)
   - Enabled `small_rng` feature for rand crate

4. **Removed old src/ directory**

5. **Verified full build**: `cargo build --workspace` succeeds

6. **Verified simulation runs**: `cargo run -p sim-core -- --ticks 100` works

### Final Working Structure

```
faction-sim/
├── Cargo.toml (workspace)
├── crates/
│   ├── sim-events/     # Pure event/snapshot data types
│   ├── sim-core/       # Full simulation (migrated from src/)
│   ├── director/       # Stub - drama detection
│   └── viz/            # Stub - visualization
├── docs/design/        # Design documentation
└── output/             # Simulation output
```

### Commands

```bash
# Build all crates
cargo build --workspace

# Run simulation
cargo run -p sim-core -- --seed 42 --ticks 1000
```
