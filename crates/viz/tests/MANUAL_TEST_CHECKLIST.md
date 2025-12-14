# Visualization Manual Test Checklist

This checklist should be run through before releases to verify visualization functionality.

## Setup

1. Build the visualization:
   ```bash
   cargo build -p viz --release
   ```

2. Run with sample state:
   ```bash
   cargo run -p viz -- --state=output/current_state.json
   ```

---

## Camera Controls

### Pan
- [ ] Right mouse button drag pans the camera
- [ ] Middle mouse button drag pans the camera
- [ ] Arrow keys pan the camera
- [ ] Shift+Arrow keys pan faster
- [ ] Pan speed scales with zoom (faster when zoomed out)

### Zoom
- [ ] Scroll wheel zooms in/out
- [ ] Zoom centers on cursor position (world point under cursor stays fixed)
- [ ] +/- keys zoom in discrete steps
- [ ] Zoom respects min/max constraints (0.25x to 4x)

### Reset
- [ ] Home key returns camera to world center at 1x zoom
- [ ] Transition to home is smooth (not instant)

---

## Agents

### Rendering
- [ ] Agents appear at correct locations
- [ ] Agent colors match faction colors
- [ ] Dead agents are not rendered

### Movement
- [ ] Agents move smoothly when location changes (interpolation)
- [ ] Multiple agents at same location are offset (not stacked)

### Selection
- [ ] Left-click on agent selects it
- [ ] Selected agent shows yellow highlight
- [ ] Clicking empty space deselects
- [ ] Only one agent selected at a time

### Hover
- [ ] Hovering over agent shows visual feedback
- [ ] Hover detection radius is reasonable (~20 world units)

---

## Director Mode

### Toggle
- [ ] D key toggles director mode on/off
- [ ] Status bar shows current mode (Director/Manual)

### Camera Control
- [ ] Director mode follows camera_script.json instructions
- [ ] FollowAgent mode tracks agent position
- [ ] FrameMultiple mode shows all specified agents
- [ ] Overview mode zooms out
- [ ] Transitions use pacing hint (slow/normal/urgent/climactic)

### Manual Override
- [ ] Any manual input (pan/zoom) temporarily overrides director
- [ ] Status shows "Manual (override)" during override
- [ ] After timeout (~5 seconds), returns to director mode

---

## Commentary

### Display
- [ ] Commentary appears in bottom panel
- [ ] EventCaption shows in white
- [ ] DramaticIrony shows in gold/italic style with "//" prefix
- [ ] TensionTeaser shows in red-tinted color
- [ ] ContextReminder shows in gray

### Behavior
- [ ] Commentary fades out after display_duration
- [ ] New commentary doesn't overlap with existing
- [ ] Maximum of ~3 items shown at once

---

## UI Overlays

### Status Bar (Top)
- [ ] Shows current tick number
- [ ] Shows date (Year, Season, Day)
- [ ] Shows camera mode
- [ ] Updates when state changes

### Playback Controls (Bottom)
- [ ] Play/Pause button works (click and Space key)
- [ ] Speed buttons work (0.5x, 1x, 2x)
- [ ] 1, 2, 3 keys change speed
- [ ] Active speed button is highlighted

---

## Debug Overlay

- [ ] F3 toggles debug overlay
- [ ] Shows FPS counter
- [ ] Shows camera position and zoom
- [ ] Shows agent count
- [ ] Shows current tick
- [ ] Text turns red when FPS < 30
- [ ] Text turns yellow when FPS < 55

---

## File Watching

- [ ] Changes to current_state.json are detected
- [ ] Changes to camera_script.json update director
- [ ] Changes to commentary.json add new items
- [ ] R key forces manual reload
- [ ] Error loading shows in status bar

---

## World Rendering

### Map
- [ ] Background map is visible (green terrain)
- [ ] Locations are rendered at correct positions

### Locations
- [ ] Village locations show as larger shapes
- [ ] Bridge locations show as horizontal rectangles
- [ ] Crossroads show appropriate shape
- [ ] Location colors match controlling faction

---

## Performance

- [ ] FPS stays above 60 with 0-10 agents
- [ ] FPS stays above 30 with 50+ agents
- [ ] No visible stuttering during camera pan
- [ ] No stuttering during agent movement

---

## Known Issues

Document any issues found during testing:

1. _[Issue description]_
2. _[Issue description]_

---

## Test Environment

- **Date**: ____________
- **Version**: ____________
- **OS**: ____________
- **GPU**: ____________
- **Tester**: ____________
