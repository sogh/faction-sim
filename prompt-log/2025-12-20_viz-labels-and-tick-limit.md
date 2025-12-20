# Viz Labels and Tick Limit Session

**Date:** 2025-12-20
**Time:** Session start
**Hostname:** PickleJar
**Model:** Claude Opus 4.5

## User Prompt

Two feature requests: 1. Let's add text labels to things in the visualization. I need to know the names of locations, and when the mouse hovers over something I need to know what that thing is, and its coordinates in the world.  2. When running in viz mode, let's set a max for how far ahead the simulation will go based on what the user is seeing. Maybe for now it's 300 ticks. This will cap our memory and cpu overhead somewhat.

## Summary

### Feature 1: Location Text Labels and Hover Tooltips

**Files Modified:**
- `crates/viz/src/world.rs`

**Changes:**
1. Added `HoveredLocation` component to mark hovered locations
2. Added `HoverTooltip` resource to track tooltip state (hovered location ID, world position, screen position)
3. Added `TooltipContainer` and `TooltipText` UI components for the tooltip display
4. Added `format_location_name()` function to convert snake_case location IDs to "Title Case" display names
5. Modified `update_locations()` to spawn `Text2dBundle` labels as children of location markers
6. Added `spawn_tooltip_ui()` system to create the tooltip UI container at startup
7. Added `handle_location_hover()` system to detect when cursor is over a location (30px radius)
8. Added `update_tooltip_display()` system to show/hide tooltip and update its content with location name and coordinates

**Result:** Locations now have visible text labels below them, and hovering over a location shows a tooltip with its name and world coordinates.

### Feature 2: Simulation Tick Limit

**Files Modified:**
- `crates/viz/src/sim_runner.rs`
- `crates/viz/src/overlay.rs`
- `crates/viz/src/main.rs`

**Changes:**
1. Added `max_ticks_ahead` field to `SimConfig` (default: 300)
2. Added `PausedAhead { paused_at_tick, max_ticks }` variant to `SimStatus`
3. Added `--max-ticks-ahead` CLI argument
4. Added `enforce_tick_limit()` system that:
   - Pauses simulation when it's more than 300 ticks ahead of playback
   - Resumes simulation when playback catches up to within 150 ticks (half the limit)
   - Resumes from the nearest snapshot at the paused tick
5. Updated status display to show "paused - ahead of playback" state in amber color

**Result:** Simulation automatically pauses when it gets too far ahead of what the user is viewing, reducing memory and CPU overhead. It resumes automatically when playback catches up.

**Bug Fix:** Fixed a race condition where the simulation would immediately resume after pausing because `stop()` resets `last_tick_seen` to 0. The fix uses `paused_at_tick` from the status instead for the resume check, and avoids clearing output files when resuming.

### Feature 3: Agent Info Panel on Click

**Files Modified:**
- `crates/viz/src/overlay.rs`

**Changes:**
1. Added `SelectedAgentInfo` resource to track selected agent data
2. Added `AgentInfoPanel` and `AgentInfoText` UI components
3. Added `spawn_agent_info_panel()` to create a floating panel in the top-right corner
4. Added `update_agent_selection_info()` system that listens to `AgentSelectedEvent` and populates the info
5. Added `update_agent_info_panel()` system to show/hide panel and update text content

**Result:** Clicking on an agent shows an info panel displaying:
- Name
- Agent ID
- Faction
- Role
- Current location (formatted nicely)
- World coordinates

### Tests

All 41 tests pass.
