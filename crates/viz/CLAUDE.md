# viz Crate

## Purpose

The visualization layer. Renders the simulation world using Bevy, responds to Director AI camera instructions, and provides user interaction for observation and exploration.

**This is not a game.** There's no player agency over the simulation. The visualization exists to make emergent drama legible and compelling to human observers.

## Key Files

```
src/
├── lib.rs           # Public API, SimVizPlugin
├── plugin.rs        # Bevy plugin setup, system registration
├── camera.rs        # CameraController, transitions, user input
├── sprites.rs       # Procedural sprite generation, animation
├── overlay.rs       # UI overlays, commentary display, panels
├── state_loader.rs  # Load current_state.json, file watching
└── effects.rs       # Particle effects, screen effects
```

## Design Doc Reference

See `/docs/design/visualization_design.md` for the complete specification.

## Core Concepts

### Director-Driven, User-Overridable
- Director AI controls camera by default
- User can take manual control anytime (pan, zoom, click)
- System gracefully switches between modes
- Optional: auto-return to director after idle

### Visual Clarity Over Flash
- Faction colors must be instantly distinguishable  
- Relationship lines when relevant
- Information density is user-adjustable
- Animation serves comprehension

### Paper Mario Style Sprites
- 2D characters with personality
- Procedurally generated from traits
- Accumulate visual history (scars, stolen cloaks)
- Faction-appropriate clothing and colors

## Bevy Plugin Structure

```rust
pub struct SimVizPlugin;

impl Plugin for SimVizPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .insert_resource(VisualWorld::default())
            .insert_resource(CameraController::default())
            .insert_resource(PlaybackState::default())
            .insert_resource(DirectorState::default())
            
            // Systems (ordered)
            .add_systems(Update, (
                load_simulation_state,
                process_director_instructions,
                update_camera,
                sync_agent_positions,
                update_animations,
                render_overlays,
                handle_input,
            ).chain())
            
            // Events
            .add_event::<SimulationStateUpdated>()
            .add_event::<CameraCommand>()
            
            // Sub-plugins
            .add_plugins((
                SpritePlugin,
                UIPlugin,
                EffectsPlugin,
            ));
    }
}
```

## Key Types

### CameraController
```rust
pub struct CameraController {
    pub current_position: Vec2,
    pub current_zoom: f32,
    pub target_position: Vec2,
    pub target_zoom: f32,
    pub mode: CameraControlMode,
    pub transition: Option<CameraTransition>,
}

pub enum CameraControlMode {
    Director {
        current_instruction: Option<CameraInstruction>,
        queue: VecDeque<CameraInstruction>,
    },
    Manual {
        return_to_director_after: Option<Duration>,
    },
    Overview,
}
```

### VisualAgent
```rust
pub struct VisualAgent {
    pub agent_id: String,
    pub sprite: AgentSprite,
    pub position: Vec2,
    pub target_position: Option<Vec2>,
    pub movement_state: MovementState,
    pub animation_state: AnimationState,
    pub markers: Vec<VisualMarker>,  // Scars, stolen items, etc.
}
```

### Overlays
```rust
pub enum Overlay {
    AgentLabel(AgentLabelOverlay),
    FactionTerritory(FactionTerritoryOverlay),
    RelationshipLines(RelationshipOverlay),
    Commentary(CommentaryOverlay),
    AgentDetail(AgentDetailPanel),
    MiniMap(MiniMapOverlay),
    PlaybackControls(PlaybackControls),
}
```

## File Watching

The viz layer watches for simulation output files:

```rust
pub struct StateFileWatcher {
    watcher: RecommendedWatcher,
    rx: Receiver<notify::Event>,
}

impl StateFileWatcher {
    pub fn check_updates(&mut self) -> Option<StateUpdate> {
        // Watch for:
        // - current_state.json (simulation state)
        // - camera_script.json (director instructions)
        // - commentary.json (text overlays)
    }
}
```

## Implementation Phases

### Phase 1: Core Rendering
- [ ] Bevy project setup, window
- [ ] Map rendering with faction territories
- [ ] Agent rendering (placeholder sprites)
- [ ] Position sync from current_state.json
- [ ] Movement interpolation
- [ ] Manual camera (pan, zoom)
- [ ] File watching, hot reload

### Phase 2: Director Integration
- [ ] Load camera_script.json
- [ ] Process CameraInstruction
- [ ] Camera modes (follow, frame, overview)
- [ ] Pacing-aware transitions
- [ ] Director/manual mode switching
- [ ] Commentary overlay display

### Phase 3: Sprite System
- [ ] Art direction document
- [ ] Base body sprite sheets
- [ ] Clothing per role
- [ ] Faction color palettes
- [ ] Procedural composition
- [ ] Visual markers (scars, items)
- [ ] Animation system

### Phase 4: UI & Polish
- [ ] Agent detail panel
- [ ] Faction overview panel
- [ ] Playback controls
- [ ] Minimap
- [ ] Particle effects
- [ ] Screen effects (flash, shake)
- [ ] Sound framework

### Phase 5: Replay & Analysis
- [ ] Snapshot loading
- [ ] Event replay stepping
- [ ] Relationship graph view
- [ ] Timeline view

## Camera Transition Logic

```rust
impl CameraController {
    fn begin_transition(&mut self, target: Vec2, zoom: f32, pacing: &PacingHint) {
        let duration = match pacing {
            PacingHint::Slow => 2.0,
            PacingHint::Normal => 1.0,
            PacingHint::Urgent => 0.3,
            PacingHint::Climactic => 1.5,
        };
        
        self.transition = Some(CameraTransition {
            from: self.current_position,
            to: target,
            from_zoom: self.current_zoom,
            to_zoom: zoom,
            duration,
            easing: pacing.to_easing(),
            progress: 0.0,
        });
    }
}
```

## Performance Strategies

1. **Spatial partitioning**: Only update/render agents near camera
2. **LOD**: Distant agents become simpler shapes
3. **Batched rendering**: Group sprites by texture atlas
4. **Async loading**: Don't block render for file I/O
5. **Interpolation**: Smooth movement between sim ticks

## Testing Strategy

1. **Visual tests**: Screenshot comparison (optional)
2. **Camera tests**: Transitions reach correct position
3. **Input tests**: User input produces expected commands
4. **File loading**: Handle malformed/missing files gracefully

```rust
#[test]
fn test_camera_transition_completes() {
    let mut camera = CameraController::default();
    camera.begin_transition(Vec2::new(100.0, 100.0), 1.0, &PacingHint::Normal);
    
    // Simulate time passing
    for _ in 0..60 {
        camera.update(1.0 / 60.0, &HashMap::new());
    }
    
    assert!(camera.transition.is_none());
    assert!((camera.current_position - Vec2::new(100.0, 100.0)).length() < 0.1);
}
```

## Dependencies

- `bevy`: Game engine / renderer
- `sim-events`: Event types for display
- `director`: CameraInstruction types (or define locally)
- `notify`: File system watching
- `serde` + `serde_json`: State loading

## Asset Organization

```
assets/
├── sprites/
│   ├── bodies/        # Base body types
│   ├── clothing/      # Role-based clothing
│   ├── accessories/   # Scars, items, insignia
│   └── factions/      # Faction banners, colors
├── maps/
│   └── terrain/       # Terrain tiles
├── effects/
│   └── particles/     # Particle textures
└── fonts/
    └── medieval.ttf   # UI font
```

## Gotchas

- Bevy 0.14 coordinate system: Y-up, origin at center
- Sprite Z-ordering: higher Z = rendered on top
- Camera zoom affects world units, not screen pixels
- File watcher events can batch—handle multiple updates
- Don't assume files exist—handle missing gracefully
- Movement interpolation needs previous + current position
- Commentary queue: don't overlap, fade out old items
