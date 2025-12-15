# Director-Viz Integration Session

**Date**: 2025-12-14 19:02 UTC
**Hostname**: Windows development machine
**Model**: Claude Opus 4.5

## User Prompt

Continue implementing the director-viz integration plan. Previous session established:
- Viz as orchestrator that spawns sim-core subprocess
- Batch mode simulation (faster than real-time)
- Support for --from-snapshot flag for intervention workflow

The plan includes:
1. Wire EventLogger in sim-core
2. Add --from-snapshot flag to sim-core
3. Add restore_from_snapshot() function
4. Add SimRunner plugin to viz
5. Integrate director as library in viz
6. Add simulation status UI to overlay
7. Add intervention system to viz
8. Add CLI args to viz main.rs

## Work Completed

### sim-core Changes

**`crates/sim-core/src/main.rs`**:
- Added EventLogger initialization and wiring
- Added `--from-snapshot` and `--start-tick` CLI arguments for intervention workflow
- EventLogger logs events to `output/events.jsonl` before tick events are cleared

**`crates/sim-core/src/output/snapshot.rs`**:
- Added `load_snapshot()` function to deserialize WorldSnapshot from JSON
- Added `restore_from_snapshot()` function to restore world state from a snapshot
  - Restores agent positions, traits, needs, goals
  - Restores faction membership and relationships
  - Fixed multiple compile errors (Role enum variants, Goal::new signature, etc.)

### viz Changes

**`crates/viz/src/sim_runner.rs`** (NEW):
- `SimRunnerPlugin` - Manages sim-core subprocess lifecycle
- `SimRunner` resource - Tracks process handle, status, output
- `SimConfig` resource - Configuration for simulation (ticks, seed, snapshot_interval, auto_start)
- `SimStatus` enum - Idle, Starting, Running, Completed, Failed
- `SimulationEvent` - Events for simulation state changes
- `poll_simulation` system - Monitors subprocess output
- `handle_sim_control_input` system - S key to start/stop simulation
- `find_snapshot_at_or_before()` - Helper to locate snapshots for intervention
- Uses `Mutex` wrappers for thread-safe access to process/receiver

**`crates/viz/src/intervention.rs`** (NEW):
- `InterventionPlugin` - Handles intervention workflow
- `InterventionState` resource - Tracks pending interventions
- `InterventionEvent` - Fired when intervention is triggered
- I key handler to trigger intervention at current tick
- `process_intervention` system - Stops current sim, restarts from snapshot

**`crates/viz/src/overlay.rs`**:
- Added `SimStatusDisplay` component
- Added simulation status text to status bar
- Added `update_sim_status_display` system
- Shows: Idle, Starting, Running (with progress %), Complete, or Failed

**`crates/viz/src/main.rs`**:
- Added CLI argument parsing with clap
- Arguments: --ticks, --snapshot-interval, --seed, --auto-start, --replay, --output-dir
- SimConfig resource is populated from CLI args

**`crates/viz/src/lib.rs`**:
- Added `sim_runner` and `intervention` modules

**`crates/viz/src/plugin.rs`**:
- Registered `SimRunnerPlugin` and `InterventionPlugin`

**`crates/viz/Cargo.toml`**:
- Added clap dependency

**`Cargo.toml` (workspace)**:
- Added clap to workspace dependencies

## Architecture Summary

```
┌─────────────────────────────────────────────────────────────────────┐
│                              VIZ                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐               │
│  │ SimRunner    │  │ Director     │  │ Renderer     │               │
│  │ (subprocess) │→ │ (file-based) │→ │ (Bevy)       │               │
│  └──────────────┘  └──────────────┘  └──────────────┘               │
│         │                 │                  │                       │
│         ↓                 ↓                  ↓                       │
│  output/snapshots/   camera_script.json   Agents, Map,              │
│  output/events.jsonl commentary.json      Commentary                 │
│  output/tensions.json                                                │
└─────────────────────────────────────────────────────────────────────┘
```

## User Workflow

```bash
# Start viz with auto-start simulation
cargo run -p viz -- --ticks 2000 --auto-start

# Or replay existing output without running sim
cargo run -p viz -- --replay output/

# Keyboard controls:
# S - Start/restart simulation
# I - Intervene at current tick (restarts from nearest snapshot)
# Space - Play/pause playback
# D - Toggle director mode
```

## Build Status

All crates compile successfully:
- sim-core: 45 warnings (unused imports/variables) but no errors
- viz: Clean build
- director: Clean build
- sim-events: Clean build

---

## Session 2: Director Narration & Playback System

**Date**: 2025-12-14 (continued)
**Context**: User could see agents moving but no director narration appeared on screen

### User Prompts

1. "When running viz, I can see the agents moving around and I can select them, but I dont see any narration from the director. Come up with a plan to show the director narration as text somewhere on screen that lets viewers still watch the simulation going down."

2. "There are a couple of changes I want to make first - I cant tell how far ahead the sim is versus what tick we are currenty viewing in viz. There are some numbers at the top but i cant tell what they mean."

3. "Also the play/pause buttons didnt seem to do anything"

### Root Cause Analysis

Investigation revealed the viz crate had:
- ✅ `DirectorState.commentary_queue` - stores commentary items
- ✅ `update_commentary_display()` - renders items from the queue
- ✅ `check_director_files()` - watches output/commentary.json
- ❌ **No system called `director.process_tick()` to generate commentary**

### Work Completed

**`crates/viz/src/director_runner.rs`** (NEW):
- `DirectorRunnerPlugin` - Invokes Director AI on tick changes
- `DirectorRunner` resource - Holds Director instance and last processed tick
- `run_director_on_tick_change` system - Processes events and tensions through Director
- `load_events_since()` - Reads events.jsonl for tick range
- `load_tensions()` - Reads tensions.json
- Feeds Director output directly into `DirectorState.commentary_queue`

**`crates/viz/src/overlay.rs`**:
- Extended `PlaybackState` with `current_tick: f64` and `max_available_tick: u64`
- Added `tick_for_snapshot()` method
- Updated status bar: "▶ Tick 500/2000 | Year 1, Spring, Day 5"
- Shows play/pause icon (▶/⏸) in status

**`crates/viz/src/state_loader.rs`**:
- Added `SnapshotCache` resource - tracks available snapshot ticks
- Added `scan_available_snapshots` system - scans output/snapshots/ every 500ms
- Added `advance_playback` system - advances tick based on time and speed
- Added `load_snapshot_for_playback` system - loads correct snapshot for playback position
- Uses binary search to find nearest snapshot <= target tick

**`crates/viz/src/lib.rs`**:
- Added `pub mod director_runner;`

**`crates/viz/src/plugin.rs`**:
- Registered `DirectorRunnerPlugin`

### Commit

```
627cd59 Add DirectorRunner and playback system for viz
```

### Summary

- Director AI now generates live commentary as simulation progresses
- Play/pause buttons control playback through available snapshots
- Status bar clearly shows "Tick X/Y" with play status icon
- Playback system advances at 60 ticks/second at 1x speed
