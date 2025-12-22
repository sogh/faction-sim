# History Panel and Agent History Fix

**Date:** 2025-12-20
**Model:** Claude Opus 4.5

## User Prompt

> I see director commentary happening in the viz, but nothing shows up in the history panel (key H). I also still dont see anything in the agent history. No locations or recent events. Investigate and fix that.

## Investigation

Traced the data flow for both issues:

1. **Commentary History Panel (H key)**: `CommentaryHistory` resource populated in `update_commentary_display()` when new commentary items arrive from director.

2. **Agent Event History**: `AgentEventHistory` resource populated in `track_agent_events()` by reading from `EventWatcher.recent_events`.

## Root Cause

Both `CommentaryHistory` and `AgentEventHistory` used `#[derive(Default)]` which set their `max_items`/`max_per_agent` fields to `0` (the default for `usize`).

This caused the `add()` methods to immediately pop items after adding them:
```rust
if events.len() > self.max_per_agent {  // 1 > 0 = true
    events.pop_back();  // Immediately removes the event we just added!
}
```

Both structs had a `new()` method with correct default values (100 and 20 respectively), but `init_resource::<T>()` calls `Default::default()`, not `new()`.

## Fix

Changed both structs from `#[derive(Resource, Default)]` to `#[derive(Resource)]` with a manual `impl Default` that sets the correct values:

### overlay.rs - CommentaryHistory
- Changed to manual `impl Default` with `max_items: 100`

### live_commentary.rs - AgentEventHistory
- Changed to manual `impl Default` with `max_per_agent: 20`

## Files Modified

- `crates/viz/src/overlay.rs`
- `crates/viz/src/live_commentary.rs`

## Verification

- `cargo check -p viz` compiles successfully
