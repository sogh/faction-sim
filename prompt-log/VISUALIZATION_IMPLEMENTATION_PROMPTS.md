# Visualization Implementation Prompts

## Prerequisites

Before starting these prompts, ensure:
- [ ] Project structure is set up (see SETUP_INSTRUCTIONS.md)
- [ ] sim-events crate has core types implemented
- [ ] You're in the project root directory
- [ ] Bevy 0.14 is specified in Cargo.toml

## Overview

These prompts implement **Visualization Phase 1: Core Rendering** and **Phase 2: Director Integration**. After completing all prompts, you'll have:
- World map rendering with faction territories
- Agent sprites (placeholder) with position sync
- Smooth movement interpolation
- Manual camera controls
- Director-driven camera with mode switching
- Commentary overlay display
- File watching for live updates

---

## Prompt 1: Bevy Project Setup

```
Set up the Bevy project structure for the viz crate.

1. Update crates/viz/Cargo.toml:
   - bevy = { version = "0.14", features = ["dynamic_linking"] }  # faster iteration
   - serde, serde_json for state loading
   - notify = "6.0" for file watching
   - sim-events = { path = "../sim-events" }

2. Create src/lib.rs with SimVizPlugin:

use bevy::prelude::*;

pub mod camera;
pub mod state_loader;
pub mod world;
pub mod agents;
pub mod overlay;

pub struct SimVizPlugin;

impl Plugin for SimVizPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Emergent Simulation".into(),
                    resolution: (1280., 720.).into(),
                    ..default()
                }),
                ..default()
            }))
            .add_plugins((
                camera::CameraPlugin,
                state_loader::StateLoaderPlugin,
                world::WorldPlugin,
                agents::AgentPlugin,
                overlay::OverlayPlugin,
            ));
    }
}

3. Create src/main.rs:

use bevy::prelude::*;
use viz::SimVizPlugin;

fn main() {
    App::new()
        .add_plugins(SimVizPlugin)
        .run();
}

4. Create stub files for each module with empty plugin structs.

5. Verify it compiles and opens a window:
   cargo run -p viz
```

---

## Prompt 2: Core Resources and Types

```
Create core visualization types in src/world.rs.

1. VisualWorld resource:
   pub struct VisualWorld {
       pub current_tick: u64,
       pub season: String,
       pub bounds: WorldBounds,
   }

   pub struct WorldBounds {
       pub min: Vec2,
       pub max: Vec2,
   }

2. Location component and types:
   #[derive(Component)]
   pub struct VisualLocation {
       pub location_id: String,
       pub location_type: LocationType,
       pub controlling_faction: Option<String>,
   }

   pub enum LocationType {
       Village,
       Camp,
       Landmark,
       Bridge,
       Crossroads,
   }

3. Faction colors resource:
   #[derive(Resource)]
   pub struct FactionColors {
       pub colors: HashMap<String, Color>,
   }

   impl Default for FactionColors {
       fn default() -> Self {
           let mut colors = HashMap::new();
           colors.insert("thornwood".into(), Color::srgb(0.2, 0.6, 0.2));
           colors.insert("ironmere".into(), Color::srgb(0.4, 0.4, 0.6));
           colors.insert("saltcliff".into(), Color::srgb(0.8, 0.7, 0.5));
           colors.insert("northern_hold".into(), Color::srgb(0.5, 0.5, 0.7));
           Self { colors }
       }
   }

4. WorldPlugin that initializes these resources:
   pub struct WorldPlugin;

   impl Plugin for WorldPlugin {
       fn build(&self, app: &mut App) {
           app
               .init_resource::<VisualWorld>()
               .init_resource::<FactionColors>();
       }
   }
```

---

## Prompt 3: Camera Controller

```
Implement the camera system in src/camera.rs.

1. Camera resources:
   #[derive(Resource)]
   pub struct CameraController {
       pub position: Vec2,
       pub zoom: f32,
       pub target_position: Vec2,
       pub target_zoom: f32,
       pub mode: CameraMode,
       pub transition: Option<CameraTransition>,
   }

   #[derive(Default, Clone)]
   pub enum CameraMode {
       #[default]
       Manual,
       Director {
           instruction: Option<CameraInstruction>,
       },
       FollowAgent {
           agent_id: String,
       },
   }

   pub struct CameraTransition {
       pub from_pos: Vec2,
       pub to_pos: Vec2,
       pub from_zoom: f32,
       pub to_zoom: f32,
       pub duration: f32,
       pub elapsed: f32,
   }

2. Camera constraints:
   pub struct CameraConstraints {
       pub min_zoom: f32,  // 0.5
       pub max_zoom: f32,  // 4.0
       pub bounds: Option<Rect>,
   }

3. CameraPlugin:
   pub struct CameraPlugin;

   impl Plugin for CameraPlugin {
       fn build(&self, app: &mut App) {
           app
               .init_resource::<CameraController>()
               .init_resource::<CameraConstraints>()
               .add_systems(Startup, setup_camera)
               .add_systems(Update, (
                   handle_camera_input,
                   update_camera_transition,
                   apply_camera_to_transform,
               ).chain());
       }
   }

4. setup_camera system:
   - Spawn Camera2dBundle
   - Initialize CameraController with default values

5. handle_camera_input system:
   - Mouse drag (middle button or right button) -> pan
   - Scroll wheel -> zoom
   - When input detected in Director mode, switch to Manual mode

6. update_camera_transition system:
   - If transition exists, advance elapsed time
   - Lerp position and zoom
   - Clear transition when complete

7. apply_camera_to_transform system:
   - Query Camera2d transform
   - Set translation from controller.position
   - Set scale from controller.zoom (inverted: higher zoom = smaller scale)

Add smooth interpolation even without active transition (for follow mode).
```

---

## Prompt 4: Manual Camera Controls

```
Implement full manual camera controls in src/camera.rs.

1. Add input handling for:
   - Pan: Right mouse button drag OR middle mouse button drag
   - Zoom: Scroll wheel (zoom toward cursor position)
   - Reset: Home key returns to world center
   - Quick zoom: +/- keys for discrete zoom steps

2. Zoom toward cursor:
   fn zoom_toward_point(
       controller: &mut CameraController,
       cursor_world_pos: Vec2,
       zoom_delta: f32,
       constraints: &CameraConstraints,
   ) {
       let old_zoom = controller.zoom;
       let new_zoom = (old_zoom * (1.0 + zoom_delta))
           .clamp(constraints.min_zoom, constraints.max_zoom);
       
       // Adjust position so cursor stays at same world point
       let zoom_ratio = new_zoom / old_zoom;
       let offset = cursor_world_pos - controller.position;
       controller.position = cursor_world_pos - offset * zoom_ratio;
       controller.zoom = new_zoom;
   }

3. Screen to world coordinate conversion:
   fn screen_to_world(
       screen_pos: Vec2,
       window: &Window,
       camera_pos: Vec2,
       camera_zoom: f32,
   ) -> Vec2 {
       let window_size = Vec2::new(window.width(), window.height());
       let ndc = (screen_pos - window_size / 2.0) / window_size * 2.0;
       let world_offset = ndc * (window_size / 2.0) / camera_zoom;
       camera_pos + Vec2::new(world_offset.x, -world_offset.y)
   }

4. Pan speed should be zoom-independent (faster pan when zoomed out):
   let pan_delta = mouse_delta / controller.zoom;

5. Add bounds checking:
   - If CameraConstraints has bounds, clamp position to stay within

6. Add keyboard shortcuts:
   - Arrow keys for discrete panning
   - Shift+Arrow for faster pan
   - Space to pause/resume (emit event for later use)

Test that:
- Zooming toward cursor keeps that world point stationary
- Pan feels consistent at different zoom levels
- Bounds prevent camera from going out of world
```

---

## Prompt 5: State Loading and File Watching

```
Implement state loading in src/state_loader.rs.

1. SimulationState resource:
   #[derive(Resource, Default)]
   pub struct SimulationState {
       pub snapshot: Option<WorldSnapshot>,
       pub last_update: Option<Instant>,
       pub file_path: Option<PathBuf>,
   }

2. StateLoaderPlugin:
   pub struct StateLoaderPlugin;

   impl Plugin for StateLoaderPlugin {
       fn build(&self, app: &mut App) {
           app
               .init_resource::<SimulationState>()
               .init_resource::<FileWatcher>()
               .add_event::<StateUpdatedEvent>()
               .add_systems(Startup, setup_file_watcher)
               .add_systems(Update, (
                   check_file_updates,
                   process_state_update,
               ));
       }
   }

3. FileWatcher resource using notify crate:
   #[derive(Resource)]
   pub struct FileWatcher {
       pub watcher: Option<RecommendedWatcher>,
       pub rx: Option<Receiver<Result<notify::Event, notify::Error>>>,
       pub watch_path: Option<PathBuf>,
   }

4. setup_file_watcher system:
   - Check for command line arg or default path
   - Create watcher on the output directory
   - Store receiver in FileWatcher resource

5. check_file_updates system:
   - Poll the receiver (non-blocking)
   - On file change event for current_state.json:
     - Load and parse the file
     - Update SimulationState.snapshot
     - Send StateUpdatedEvent

6. StateUpdatedEvent:
   #[derive(Event)]
   pub struct StateUpdatedEvent {
       pub tick: u64,
   }

7. Add manual reload:
   - R key forces reload of state file

8. Error handling:
   - Log errors but don't crash
   - Show "Loading..." or "Error" in corner (later, with overlay)

9. CLI argument for state path:
   - Use clap or simple std::env::args
   - cargo run -p viz -- --state ./output/current_state.json
```

---

## Prompt 6: World Map Rendering

```
Implement world map rendering in src/world.rs.

1. Map background:
   fn spawn_map_background(
       mut commands: Commands,
       asset_server: Res<AssetServer>,
   ) {
       // Simple colored rectangle for now
       // Later: tilemap or textured terrain
       commands.spawn(SpriteBundle {
           sprite: Sprite {
               color: Color::srgb(0.3, 0.4, 0.3),  // Grass-ish
               custom_size: Some(Vec2::new(2000.0, 2000.0)),
               ..default()
           },
           transform: Transform::from_xyz(0.0, 0.0, -10.0),
           ..default()
       });
   }

2. Location markers:
   #[derive(Component)]
   pub struct LocationMarker {
       pub location_id: String,
   }

   fn spawn_location_markers(
       mut commands: Commands,
       state: Res<SimulationState>,
       faction_colors: Res<FactionColors>,
   ) {
       // Called when state updates
       // Parse locations from snapshot
       // Spawn sprite for each location
   }

   Location positions can be:
   - Hardcoded initially (define a LOCATIONS constant)
   - Later: loaded from a map config file

3. Location visuals by type:
   - Village: Larger circle with building icon
   - Camp: Triangle/tent shape
   - Bridge: Horizontal rectangle
   - Crossroads: X shape or diamond

4. Faction territory overlay:
   - Semi-transparent colored regions
   - Use mesh or sprite with alpha
   - Z-layer between background and locations

5. Territory borders:
   - Dashed or solid lines between territories
   - Color matches controlling faction

6. Location labels:
   - Text2dBundle for location names
   - Only show when zoomed in enough
   - Fade based on zoom level

7. Systems:
   - spawn_map_background (Startup)
   - update_locations (on StateUpdatedEvent)
   - update_territory_overlay (on StateUpdatedEvent)
   - update_location_labels (Update, based on zoom)
```

---

## Prompt 7: Agent Rendering

```
Implement agent rendering in src/agents.rs.

1. Agent component:
   #[derive(Component)]
   pub struct VisualAgent {
       pub agent_id: String,
       pub faction: String,
       pub role: String,
       pub target_position: Vec2,
       pub move_speed: f32,
   }

2. AgentPlugin:
   pub struct AgentPlugin;

   impl Plugin for AgentPlugin {
       fn build(&self, app: &mut App) {
           app
               .add_systems(Update, (
                   sync_agents_with_state,
                   interpolate_agent_movement,
               ));
       }
   }

3. sync_agents_with_state system:
   - On StateUpdatedEvent, compare current entities with snapshot
   - Spawn new agents not yet visualized
   - Despawn agents no longer in snapshot
   - Update target_position for existing agents

4. Agent spawning (placeholder sprites):
   fn spawn_agent(
       commands: &mut Commands,
       agent: &AgentSnapshot,
       faction_colors: &FactionColors,
       position: Vec2,
   ) -> Entity {
       let color = faction_colors.colors
           .get(&agent.faction)
           .copied()
           .unwrap_or(Color::WHITE);

       commands.spawn((
           SpriteBundle {
               sprite: Sprite {
                   color,
                   custom_size: Some(Vec2::new(20.0, 30.0)),
                   ..default()
               },
               transform: Transform::from_xyz(position.x, position.y, 1.0),
               ..default()
           },
           VisualAgent {
               agent_id: agent.agent_id.clone(),
               faction: agent.faction.clone(),
               role: agent.role.clone(),
               target_position: position,
               move_speed: 100.0,
           },
       )).id()
   }

5. interpolate_agent_movement system:
   fn interpolate_agent_movement(
       time: Res<Time>,
       mut agents: Query<(&mut Transform, &VisualAgent)>,
   ) {
       for (mut transform, agent) in agents.iter_mut() {
           let current = transform.translation.truncate();
           let target = agent.target_position;
           
           if current.distance(target) > 1.0 {
               let direction = (target - current).normalize();
               let movement = direction * agent.move_speed * time.delta_seconds();
               
               if movement.length() > current.distance(target) {
                   transform.translation = target.extend(transform.translation.z);
               } else {
                   transform.translation += movement.extend(0.0);
               }
           }
       }
   }

6. Agent-to-position mapping:
   - Agents have a location_id in snapshot
   - Look up location's world position
   - Add small offset based on agent index (don't stack exactly)

7. Agent lookup resource:
   #[derive(Resource, Default)]
   pub struct AgentEntities {
       pub map: HashMap<String, Entity>,
   }
```

---

## Prompt 8: Agent Selection and Labels

```
Extend agent rendering with selection and labels.

1. Selection component:
   #[derive(Component)]
   pub struct Selected;

   #[derive(Component)]
   pub struct Hovered;

2. Agent label:
   #[derive(Component)]
   pub struct AgentLabel {
       pub agent_entity: Entity,
   }

3. Click-to-select system:
   fn handle_agent_click(
       mouse: Res<ButtonInput<MouseButton>>,
       windows: Query<&Window>,
       camera: Query<(&Camera, &GlobalTransform)>,
       agents: Query<(Entity, &Transform, &VisualAgent)>,
       mut commands: Commands,
       selected: Query<Entity, With<Selected>>,
   ) {
       if mouse.just_pressed(MouseButton::Left) {
           // Convert mouse to world coords
           // Find agent under cursor (within radius)
           // Deselect current, select new
       }
   }

4. Hover detection system:
   - Similar to click but continuous
   - Add/remove Hovered component
   - Could show tooltip or highlight

5. Selection visual:
   - Circle outline around selected agent
   - Or pulsing glow effect
   - Spawn as child entity with Selected component

6. Label spawning:
   fn spawn_agent_labels(
       mut commands: Commands,
       agents: Query<(Entity, &VisualAgent), Added<VisualAgent>>,
       asset_server: Res<AssetServer>,
   ) {
       for (entity, agent) in agents.iter() {
           let label = commands.spawn((
               Text2dBundle {
                   text: Text::from_section(
                       &agent.agent_id,  // Or name from snapshot
                       TextStyle {
                           font_size: 12.0,
                           color: Color::WHITE,
                           ..default()
                       },
                   ),
                   transform: Transform::from_xyz(0.0, 25.0, 0.1),
                   ..default()
               },
               AgentLabel { agent_entity: entity },
           )).id();
           
           commands.entity(entity).add_child(label);
       }
   }

7. Label visibility based on zoom:
   - Hide labels when zoomed out (too cluttered)
   - Show on hover regardless of zoom
   - Show for selected agent always

8. Role indicator:
   - Small icon or shape indicating role
   - Leader: crown/star
   - Scout: eye
   - Reader: book
   - Could be colored shape for now, sprites later
```

---

## Prompt 9: Director Integration - Loading Instructions

```
Implement Director integration for camera control.

1. Director state resource:
   #[derive(Resource, Default)]
   pub struct DirectorState {
       pub camera_script: Vec<CameraInstruction>,
       pub commentary_queue: Vec<CommentaryItem>,
       pub current_instruction_index: usize,
       pub enabled: bool,
   }

2. Extend FileWatcher to also watch:
   - camera_script.json
   - commentary.json

3. DirectorUpdatedEvent:
   #[derive(Event)]
   pub struct DirectorUpdatedEvent;

4. Load director files system:
   fn load_director_files(
       mut director_state: ResMut<DirectorState>,
       mut events: EventReader<FileChangedEvent>,
   ) {
       for event in events.read() {
           if event.path.ends_with("camera_script.json") {
               // Parse and update camera_script
           }
           if event.path.ends_with("commentary.json") {
               // Parse and update commentary_queue
           }
       }
   }

5. Process camera instructions system:
   fn process_director_instructions(
       mut director: ResMut<DirectorState>,
       mut camera: ResMut<CameraController>,
       state: Res<SimulationState>,
       agents: Query<(&VisualAgent, &Transform)>,
       time: Res<Time>,
   ) {
       if !director.enabled {
           return;
       }

       // Find instruction valid for current tick
       let current_tick = state.snapshot
           .as_ref()
           .map(|s| s.timestamp.tick)
           .unwrap_or(0);

       // Apply instruction to camera
       if let Some(instruction) = director.current_instruction() {
           apply_camera_instruction(&mut camera, instruction, &agents);
       }
   }

6. apply_camera_instruction function:
   fn apply_camera_instruction(
       camera: &mut CameraController,
       instruction: &CameraInstruction,
       agents: &Query<(&VisualAgent, &Transform)>,
   ) {
       match &instruction.camera_mode {
           CameraMode::FollowAgent { agent_id, zoom } => {
               // Find agent position
               // Set camera to follow mode
               camera.mode = CameraMode::FollowAgent {
                   agent_id: agent_id.clone(),
               };
               camera.target_zoom = zoom_level_to_f32(zoom);
           }
           CameraMode::FrameLocation { location_id, zoom } => {
               // Look up location position
               // Start transition
           }
           CameraMode::FrameMultiple { agent_ids, auto_zoom } => {
               // Calculate bounding box
               // Center camera, adjust zoom
           }
           CameraMode::Overview { region } => {
               // Zoom out to show region/world
           }
           _ => {}
       }
   }

7. Toggle Director mode:
   - D key toggles director.enabled
   - When disabled, camera stays in manual mode
   - When enabled, camera follows instructions
   - Any manual input temporarily overrides (with timeout)
```

---

## Prompt 10: Director Integration - Camera Behaviors

```
Implement Director-driven camera behaviors.

1. Follow agent behavior:
   fn update_follow_camera(
       mut camera: ResMut<CameraController>,
       agents: Query<(&VisualAgent, &Transform)>,
       time: Res<Time>,
   ) {
       if let CameraMode::FollowAgent { agent_id } = &camera.mode {
           // Find the agent
           for (agent, transform) in agents.iter() {
               if agent.agent_id == *agent_id {
                   let target = transform.translation.truncate();
                   
                   // Smooth follow with slight lag
                   let follow_speed = 3.0;
                   camera.position = camera.position.lerp(
                       target,
                       (follow_speed * time.delta_seconds()).min(1.0),
                   );
                   break;
               }
           }
       }
   }

2. Pacing-aware transitions:
   fn begin_director_transition(
       camera: &mut CameraController,
       target_pos: Vec2,
       target_zoom: f32,
       pacing: &PacingHint,
   ) {
       let duration = match pacing {
           PacingHint::Slow => 2.0,
           PacingHint::Normal => 1.0,
           PacingHint::Urgent => 0.3,
           PacingHint::Climactic => 1.5,
       };

       camera.transition = Some(CameraTransition {
           from_pos: camera.position,
           to_pos: target_pos,
           from_zoom: camera.zoom,
           to_zoom: target_zoom,
           duration,
           elapsed: 0.0,
       });
   }

3. Easing functions:
   fn ease_in_out(t: f32) -> f32 {
       if t < 0.5 {
           2.0 * t * t
       } else {
           1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
       }
   }

   Apply easing to transition progress for smoother movement.

4. Manual override with timeout:
   #[derive(Resource)]
   pub struct ManualOverride {
       pub active: bool,
       pub started_at: Option<Instant>,
       pub timeout: Duration,
   }

   - When user provides input during Director mode:
     - Set override.active = true
     - Record started_at
   - After timeout expires, return to Director mode
   - Show "Manual Control" indicator in UI

5. Return to Director:
   - Spacebar or specific key returns to Director immediately
   - Or wait for timeout

6. Frame multiple agents:
   fn calculate_framing(
       agent_ids: &[String],
       agents: &Query<(&VisualAgent, &Transform)>,
       padding: f32,
   ) -> (Vec2, f32) {
       let positions: Vec<Vec2> = agent_ids.iter()
           .filter_map(|id| {
               agents.iter()
                   .find(|(a, _)| a.agent_id == *id)
                   .map(|(_, t)| t.translation.truncate())
           })
           .collect();

       if positions.is_empty() {
           return (Vec2::ZERO, 1.0);
       }

       let min = positions.iter().fold(Vec2::MAX, |a, &b| a.min(b));
       let max = positions.iter().fold(Vec2::MIN, |a, &b| a.max(b));
       let center = (min + max) / 2.0;
       let size = (max - min) + Vec2::splat(padding * 2.0);
       let zoom = (800.0 / size.max_element()).clamp(0.5, 2.0);

       (center, zoom)
   }
```

---

## Prompt 11: Commentary Overlay

```
Implement commentary display in src/overlay.rs.

1. Overlay plugin:
   pub struct OverlayPlugin;

   impl Plugin for OverlayPlugin {
       fn build(&self, app: &mut App) {
           app
               .add_systems(Startup, setup_overlay_ui)
               .add_systems(Update, (
                   update_commentary_display,
                   fade_commentary,
                   update_status_bar,
               ));
       }
   }

2. Commentary display component:
   #[derive(Component)]
   pub struct CommentaryDisplay {
       pub items: VecDeque<DisplayedCommentary>,
       pub max_items: usize,
   }

   pub struct DisplayedCommentary {
       pub item: CommentaryItem,
       pub entity: Entity,
       pub opacity: f32,
       pub time_remaining: f32,
   }

3. Setup UI structure:
   fn setup_overlay_ui(mut commands: Commands) {
       // Root UI node
       commands.spawn(NodeBundle {
           style: Style {
               width: Val::Percent(100.0),
               height: Val::Percent(100.0),
               flex_direction: FlexDirection::Column,
               justify_content: JustifyContent::SpaceBetween,
               ..default()
           },
           ..default()
       }).with_children(|parent| {
           // Top bar (status info)
           spawn_status_bar(parent);
           
           // Bottom area (commentary)
           spawn_commentary_area(parent);
       });
   }

4. Commentary area (bottom of screen):
   fn spawn_commentary_area(parent: &mut ChildBuilder) {
       parent.spawn((
           NodeBundle {
               style: Style {
                   width: Val::Percent(100.0),
                   height: Val::Px(100.0),
                   padding: UiRect::all(Val::Px(20.0)),
                   justify_content: JustifyContent::Center,
                   align_items: AlignItems::End,
                   ..default()
               },
               background_color: Color::srgba(0.0, 0.0, 0.0, 0.5).into(),
               ..default()
           },
           CommentaryContainer,
       ));
   }

5. Update commentary system:
   fn update_commentary_display(
       mut commands: Commands,
       director: Res<DirectorState>,
       mut container: Query<Entity, With<CommentaryContainer>>,
       mut displayed: Local<VecDeque<DisplayedCommentary>>,
       time: Res<Time>,
   ) {
       // Add new items from director queue
       for item in &director.commentary_queue {
           if !displayed.iter().any(|d| d.item.item_id == item.item_id) {
               // Spawn text node
               // Add to displayed queue
           }
       }

       // Remove old items
       displayed.retain(|d| d.time_remaining > 0.0);
   }

6. Fade effect:
   fn fade_commentary(
       mut texts: Query<(&mut Text, &DisplayedCommentary)>,
       time: Res<Time>,
   ) {
       // Fade in when new
       // Fade out when time_remaining < 1.0
   }

7. Commentary styling:
   - Semi-transparent dark background
   - White or cream text
   - Subtle text shadow for readability
   - Larger font for EventCaption
   - Italic for DramaticIrony
   - Different positioning for different types

8. Status bar (top):
   - Current tick / date
   - Camera mode indicator (Director / Manual)
   - Playback speed (if implemented)
```

---

## Prompt 12: Playback Controls UI

```
Implement basic playback controls.

1. PlaybackState resource:
   #[derive(Resource)]
   pub struct PlaybackState {
       pub playing: bool,
       pub speed: f32,
       pub current_tick: u64,
   }

2. Playback controls UI:
   fn spawn_playback_controls(parent: &mut ChildBuilder) {
       parent.spawn(NodeBundle {
           style: Style {
               position_type: PositionType::Absolute,
               bottom: Val::Px(120.0),
               left: Val::Px(20.0),
               padding: UiRect::all(Val::Px(10.0)),
               column_gap: Val::Px(10.0),
               ..default()
           },
           background_color: Color::srgba(0.1, 0.1, 0.1, 0.8).into(),
           ..default()
       }).with_children(|parent| {
           // Play/Pause button
           spawn_button(parent, "▶", PlayPauseButton);
           
           // Speed buttons
           spawn_button(parent, "0.5x", SpeedButton(0.5));
           spawn_button(parent, "1x", SpeedButton(1.0));
           spawn_button(parent, "2x", SpeedButton(2.0));
           
           // Tick display
           spawn_tick_display(parent);
       });
   }

3. Button interaction system:
   fn handle_playback_buttons(
       interaction: Query<(&Interaction, &PlaybackButton), Changed<Interaction>>,
       mut playback: ResMut<PlaybackState>,
   ) {
       for (interaction, button) in interaction.iter() {
           if *interaction == Interaction::Pressed {
               match button {
                   PlaybackButton::PlayPause => {
                       playback.playing = !playback.playing;
                   }
                   PlaybackButton::Speed(s) => {
                       playback.speed = *s;
                   }
               }
           }
       }
   }

4. Keyboard shortcuts:
   - Space: Play/pause
   - 1, 2, 3: Speed presets
   - Left/Right arrows: Step back/forward (when paused)

5. Tick display update:
   fn update_tick_display(
       state: Res<SimulationState>,
       mut query: Query<&mut Text, With<TickDisplay>>,
   ) {
       if let Some(snapshot) = &state.snapshot {
           for mut text in query.iter_mut() {
               text.sections[0].value = format!(
                   "Tick: {} | {}",
                   snapshot.timestamp.tick,
                   snapshot.timestamp.date,
               );
           }
       }
   }

6. Visual feedback:
   - Highlight active speed button
   - Change play/pause icon based on state
   - Subtle animation on button press
```

---

## Prompt 13: Debug Overlay

```
Implement a debug overlay for development.

1. DebugOverlay resource:
   #[derive(Resource)]
   pub struct DebugOverlay {
       pub enabled: bool,
       pub show_agent_ids: bool,
       pub show_positions: bool,
       pub show_fps: bool,
       pub show_camera_info: bool,
   }

2. Toggle with F3 key:
   fn toggle_debug_overlay(
       keyboard: Res<ButtonInput<KeyCode>>,
       mut debug: ResMut<DebugOverlay>,
   ) {
       if keyboard.just_pressed(KeyCode::F3) {
           debug.enabled = !debug.enabled;
       }
   }

3. Debug info display:
   fn update_debug_display(
       mut commands: Commands,
       debug: Res<DebugOverlay>,
       camera: Res<CameraController>,
       state: Res<SimulationState>,
       agents: Query<&VisualAgent>,
       time: Res<Time>,
       mut fps_history: Local<VecDeque<f32>>,
   ) {
       if !debug.enabled {
           return;
       }

       // Calculate FPS
       fps_history.push_back(1.0 / time.delta_seconds());
       if fps_history.len() > 60 {
           fps_history.pop_front();
       }
       let avg_fps: f32 = fps_history.iter().sum::<f32>() / fps_history.len() as f32;

       // Build debug text
       let text = format!(
           "FPS: {:.0}\n\
            Camera: ({:.0}, {:.0}) zoom: {:.2}\n\
            Mode: {:?}\n\
            Agents: {}\n\
            Tick: {}",
           avg_fps,
           camera.position.x, camera.position.y,
           camera.zoom,
           camera.mode,
           agents.iter().count(),
           state.snapshot.as_ref().map(|s| s.timestamp.tick).unwrap_or(0),
       );

       // Update debug text entity
   }

4. Agent debug visualization:
   - Show agent_id above each agent (when enabled)
   - Show movement vectors
   - Show target positions

5. Camera debug:
   - Show camera bounds rectangle
   - Show world origin crosshair
   - Show click-to-world coordinate on click

6. Performance warnings:
   - Red text if FPS drops below 30
   - Warning if agent count exceeds threshold
```

---

## Prompt 14: Integration Testing

```
Create integration tests for the visualization.

1. Test state loading:
   // tests/state_loading.rs
   #[test]
   fn test_parse_sample_state() {
       let json = include_str!("fixtures/sample_state.json");
       let snapshot: WorldSnapshot = serde_json::from_str(json).unwrap();
       
       assert!(!snapshot.agents.is_empty());
       assert!(!snapshot.factions.is_empty());
   }

2. Test coordinate conversion:
   #[test]
   fn test_screen_to_world() {
       // Test various screen positions convert correctly
   }

   #[test]
   fn test_zoom_toward_cursor() {
       // Verify world point under cursor stays stationary
   }

3. Create test fixtures:
   tests/fixtures/sample_state.json - Minimal world state
   tests/fixtures/sample_camera_script.json - Camera instructions
   tests/fixtures/sample_commentary.json - Commentary items

4. Headless rendering test (optional):
   - Use bevy's headless mode
   - Verify systems don't panic
   - Check entity counts after state load

5. Manual test checklist:
   Create tests/MANUAL_TEST_CHECKLIST.md:
   
   ## Camera
   - [ ] Pan with right mouse button
   - [ ] Zoom with scroll wheel
   - [ ] Zoom centers on cursor
   - [ ] Camera respects bounds
   
   ## Agents
   - [ ] Agents appear at correct positions
   - [ ] Agents move smoothly to new positions
   - [ ] Agent colors match factions
   - [ ] Click selects agent
   
   ## Director
   - [ ] D key toggles director mode
   - [ ] Camera follows FollowAgent instruction
   - [ ] Manual input overrides director
   - [ ] Returns to director after timeout
   
   ## Commentary
   - [ ] Commentary appears at bottom
   - [ ] Commentary fades out
   - [ ] Multiple items queue correctly
   
   ## File Watching
   - [ ] Changing state file updates visualization
   - [ ] Changing camera_script updates camera
   - [ ] R key forces reload

6. Sample data generator:
   Create tools/generate_test_data.rs (or Python script)
   - Generate sample state with N agents
   - Generate camera script with various instructions
   - Generate commentary queue
```

---

## Verification Checklist

After completing all prompts, verify:

- [ ] `cargo build -p viz` succeeds
- [ ] `cargo run -p viz` opens window with map background
- [ ] Manual camera pan/zoom works
- [ ] State file loads and agents appear
- [ ] File watching triggers updates
- [ ] Agent movement interpolates smoothly
- [ ] Clicking agent selects it
- [ ] Director mode follows instructions
- [ ] D key toggles Director/Manual
- [ ] Commentary displays and fades
- [ ] Debug overlay (F3) shows info
- [ ] Playback controls respond

---

## Next Steps

After Phase 1 & 2 complete:
- Phase 3: Sprite system (procedural Paper Mario style)
- Phase 4: UI polish (detail panels, minimap)
- Phase 5: Replay and analysis

See docs/design/visualization_design.md for full specifications.
