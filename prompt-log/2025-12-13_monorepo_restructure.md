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

### Notes

- The existing `src/` directory with the original simulation code still exists
- The new monorepo workspace compiles successfully
- Next steps: migrate existing code from `src/` into the appropriate crates
