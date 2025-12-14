# Visualization Layer: Design & Implementation Plan

## Purpose

The visualization layer renders the simulation world and responds to Director AI instructions. It transforms abstract events and state into a watchable, engaging visual experience—a documentary of an emergent world.

This is not a game. There's no player agency over the simulation itself (though users can pause, rewind, and explore). The visualization exists to make the emergent drama legible and compelling to human observers.

---

## Design Principles

### Clarity Over Flash
The goal is understanding, not spectacle. Visual design should make relationships, factions, and tension visible at a glance. Animation serves comprehension.

### Information Density Control
Users should be able to adjust how much they see—from a clean "just watch the story" mode to a dense "show me all the stats" analysis view.

### Director-Driven, User-Overridable
The Director AI controls the camera and pacing by default, but users can take manual control at any time. The system should gracefully handle both modes.

### Performance at Scale
Hundreds of agents, continuous updates, smooth animation. Visual fidelity degrades gracefully under load rather than stuttering.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        VISUALIZATION LAYER                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐                 │
│  │   State     │───▶│   Scene     │───▶│  Renderer   │                 │
│  │   Loader    │    │   Manager   │    │   (Bevy)    │                 │
│  └─────────────┘    └─────────────┘    └─────────────┘                 │
│         │                 │                   │                         │
│         │                 ▼                   ▼                         │
│         │          ┌─────────────┐    ┌─────────────┐                  │
│         │          │   Entity    │    │   Camera    │                  │
│         │          │   Pool      │    │   System    │                  │
│         │          └─────────────┘    └─────────────┘                  │
│         │                 │                   │                         │
│         ▼                 ▼                   ▼                         │
│  ┌─────────────┐   ┌─────────────┐    ┌─────────────┐                  │
│  │  Director   │──▶│   Sprite    │    │     UI      │                  │
│  │  Interface  │   │   System    │    │   Overlay   │                  │
│  └─────────────┘   └─────────────┘    └─────────────┘                  │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘

Inputs:                              Outputs:
- current_state.json                 - Rendered frames
- camera_script.json                 - User interaction events
- commentary.json                    - Playback state
- snapshots/ (for replay)
```

---

## Core Systems

### World Representation

```rust
// The visual world, separate from simulation state
pub struct VisualWorld {
    pub map: WorldMap,
    pub agents: HashMap<String, VisualAgent>,
    pub locations: HashMap<String, VisualLocation>,
    pub effects: Vec<VisualEffect>,
    pub overlays: Vec<Overlay>,
}

pub struct WorldMap {
    pub terrain: TerrainGrid,
    pub regions: Vec<MapRegion>,
    pub paths: Vec<TravelPath>,
    pub bounds: MapBounds,
}

pub struct MapRegion {
    pub region_id: String,
    pub controlling_faction: Option<String>,
    pub visual_style: RegionStyle,
    pub boundaries: Vec<Vec2>,
    pub label_position: Vec2,
}

pub struct TravelPath {
    pub from_location: String,
    pub to_location: String,
    pub waypoints: Vec<Vec2>,
    pub path_type: PathType,  // Road, trail, river, etc.
}
```

### Agent Visualization

```rust
pub struct VisualAgent {
    pub agent_id: String,
    pub sprite: AgentSprite,
    pub position: Vec2,
    pub target_position: Option<Vec2>,
    pub movement_state: MovementState,
    pub animation_state: AnimationState,
    pub visibility: AgentVisibility,
    pub markers: Vec<VisualMarker>,
}

pub struct AgentSprite {
    pub base_body: BodyType,
    pub faction_colors: FactionColors,
    pub clothing: ClothingSet,
    pub accessories: Vec<Accessory>,
    pub expression: Expression,
}

pub enum MovementState {
    Idle,
    Walking { speed: f32, direction: Vec2 },
    Running { speed: f32, direction: Vec2 },
    Interacting { with_agent: String },
    InConversation { participants: Vec<String> },
}

pub enum AnimationState {
    Idle,
    Walking,
    Talking,
    Fighting,
    Working,
    Celebrating,
    Mourning,
    Sneaking,
}

pub struct VisualMarker {
    pub marker_type: MarkerType,
    pub acquired_at: u64,
    pub visual_element: String,  // Sprite/overlay reference
}

pub enum MarkerType {
    Scar { location: String },
    StolenItem { from_faction: String },
    RankInsignia { rank: String },
    Injury { severity: f32 },
    StatusEffect { effect: String },
}
```

### Location Visualization

```rust
pub struct VisualLocation {
    pub location_id: String,
    pub position: Vec2,
    pub location_type: LocationType,
    pub buildings: Vec<Building>,
    pub ambient_agents: Vec<String>,  // Agents currently here
    pub activity_level: f32,
    pub visual_state: LocationVisualState,
}

pub enum LocationType {
    Village { size: VillageSize },
    Camp { permanence: f32 },
    Landmark { landmark_type: String },
    Wilderness { terrain: String },
    Bridge,
    Crossroads,
}

pub struct Building {
    pub building_type: BuildingType,
    pub position: Vec2,
    pub faction_banner: Option<String>,
    pub state: BuildingState,  // Normal, damaged, burning, etc.
}

pub enum BuildingType {
    Hall,           // Faction HQ, where archives are kept
    House,
    Workshop,
    Granary,
    Watchtower,
    Shrine,
}
```

---

## Camera System

### Camera Controller

```rust
pub struct CameraController {
    pub current_position: Vec2,
    pub current_zoom: f32,
    pub target_position: Vec2,
    pub target_zoom: f32,
    pub mode: CameraControlMode,
    pub transition: Option<CameraTransition>,
    pub constraints: CameraConstraints,
}

pub enum CameraControlMode {
    Director {
        current_instruction: Option<CameraInstruction>,
        queue: VecDeque<CameraInstruction>,
    },
    Manual {
        return_to_director_after: Option<Duration>,
    },
    Cinematic {
        script: CinematicScript,
        progress: f32,
    },
    Overview,
}

pub struct CameraTransition {
    pub from_position: Vec2,
    pub to_position: Vec2,
    pub from_zoom: f32,
    pub to_zoom: f32,
    pub duration: f32,
    pub easing: EasingFunction,
    pub progress: f32,
}

pub struct CameraConstraints {
    pub min_zoom: f32,
    pub max_zoom: f32,
    pub bounds: Option<Rect>,
    pub forbidden_zones: Vec<Rect>,  // E.g., UI areas
}

impl CameraController {
    pub fn apply_director_instruction(&mut self, instruction: &CameraInstruction) {
        match &instruction.camera_mode {
            CameraMode::FollowAgent { agent_id, zoom } => {
                self.mode = CameraControlMode::Director {
                    current_instruction: Some(instruction.clone()),
                    queue: VecDeque::new(),
                };
                self.target_zoom = zoom.to_f32();
                // Position updated each frame based on agent location
            }
            CameraMode::FrameLocation { location_id, zoom } => {
                let location_pos = self.get_location_position(location_id);
                self.begin_transition(location_pos, zoom.to_f32(), &instruction.pacing);
            }
            CameraMode::FrameMultiple { agent_ids, auto_zoom } => {
                let bounds = self.calculate_bounding_box(agent_ids);
                let center = bounds.center();
                let zoom = if *auto_zoom {
                    self.zoom_to_fit(bounds)
                } else {
                    self.current_zoom
                };
                self.begin_transition(center, zoom, &instruction.pacing);
            }
            CameraMode::Cinematic { path, duration_ticks } => {
                self.mode = CameraControlMode::Cinematic {
                    script: CinematicScript::from_waypoints(path, *duration_ticks),
                    progress: 0.0,
                };
            }
            CameraMode::Overview { region } => {
                let (center, zoom) = match region {
                    Some(r) => self.region_view(r),
                    None => self.world_view(),
                };
                self.begin_transition(center, zoom, &instruction.pacing);
            }
        }
    }
    
    fn begin_transition(&mut self, target_pos: Vec2, target_zoom: f32, pacing: &PacingHint) {
        let duration = match pacing {
            PacingHint::Slow => 2.0,
            PacingHint::Normal => 1.0,
            PacingHint::Urgent => 0.3,
            PacingHint::Climactic => 1.5,
        };
        
        self.transition = Some(CameraTransition {
            from_position: self.current_position,
            to_position: target_pos,
            from_zoom: self.current_zoom,
            to_zoom: target_zoom,
            duration,
            easing: pacing.to_easing(),
            progress: 0.0,
        });
        
        self.target_position = target_pos;
        self.target_zoom = target_zoom;
    }
}
```

### Camera Behaviors

```rust
impl CameraController {
    pub fn update(&mut self, dt: f32, agents: &HashMap<String, VisualAgent>) {
        // Update transition if active
        if let Some(ref mut transition) = self.transition {
            transition.progress += dt / transition.duration;
            if transition.progress >= 1.0 {
                self.current_position = transition.to_position;
                self.current_zoom = transition.to_zoom;
                self.transition = None;
            } else {
                let t = transition.easing.apply(transition.progress);
                self.current_position = transition.from_position.lerp(transition.to_position, t);
                self.current_zoom = lerp(transition.from_zoom, transition.to_zoom, t);
            }
            return;
        }
        
        // Mode-specific updates
        match &self.mode {
            CameraControlMode::Director { current_instruction, .. } => {
                if let Some(instruction) = current_instruction {
                    if let CameraMode::FollowAgent { agent_id, .. } = &instruction.camera_mode {
                        if let Some(agent) = agents.get(agent_id) {
                            // Smooth follow
                            let follow_speed = 5.0;
                            self.current_position = self.current_position.lerp(
                                agent.position,
                                (follow_speed * dt).min(1.0)
                            );
                        }
                    }
                }
            }
            CameraControlMode::Cinematic { script, progress } => {
                let (pos, zoom) = script.evaluate(*progress);
                self.current_position = pos;
                self.current_zoom = zoom;
            }
            CameraControlMode::Manual { .. } => {
                // Handled by input system
            }
            CameraControlMode::Overview => {
                // Static
            }
        }
    }
    
    pub fn handle_user_input(&mut self, input: &CameraInput) {
        match input {
            CameraInput::Pan(delta) => {
                if matches!(self.mode, CameraControlMode::Manual { .. }) {
                    self.current_position += *delta / self.current_zoom;
                } else {
                    // User wants control, switch to manual
                    self.mode = CameraControlMode::Manual {
                        return_to_director_after: Some(Duration::from_secs(5)),
                    };
                    self.current_position += *delta / self.current_zoom;
                }
            }
            CameraInput::Zoom(factor) => {
                self.current_zoom = (self.current_zoom * factor)
                    .clamp(self.constraints.min_zoom, self.constraints.max_zoom);
            }
            CameraInput::ReturnToDirector => {
                self.mode = CameraControlMode::Director {
                    current_instruction: None,
                    queue: VecDeque::new(),
                };
            }
            CameraInput::ClickAgent(agent_id) => {
                self.mode = CameraControlMode::Manual {
                    return_to_director_after: None,
                };
                // Focus on clicked agent
            }
        }
    }
}
```

---

## Sprite System

### Procedural Agent Generation

With hundreds of agents, sprites must be procedurally generated with meaningful variation.

```rust
pub struct SpriteGenerator {
    base_bodies: HashMap<BodyType, SpriteSheet>,
    clothing_sets: HashMap<String, ClothingSprites>,
    accessories: HashMap<String, AccessorySprite>,
    faction_palettes: HashMap<String, ColorPalette>,
}

pub struct AgentSpriteConfig {
    pub body_type: BodyType,
    pub height_variation: f32,  // -0.2 to 0.2
    pub build: BuildType,
    pub skin_tone: SkinTone,
    pub hair_style: HairStyle,
    pub hair_color: HairColor,
}

impl SpriteGenerator {
    pub fn generate_sprite(
        &self,
        agent: &Agent,
        config: &AgentSpriteConfig,
    ) -> AgentSprite {
        let faction_colors = self.faction_palettes
            .get(&agent.faction)
            .cloned()
            .unwrap_or_default();
        
        let base_body = self.create_base_body(config);
        let clothing = self.create_clothing(agent.role, &faction_colors);
        let accessories = self.create_accessories(&agent.visual_markers);
        
        AgentSprite {
            base_body: config.body_type,
            faction_colors,
            clothing,
            accessories,
            expression: Expression::Neutral,
        }
    }
    
    fn create_clothing(&self, role: &str, colors: &ColorPalette) -> ClothingSet {
        let base_set = match role {
            "faction_leader" => ClothingSet::noble(),
            "reader" => ClothingSet::robed(),
            "scout" => ClothingSet::light_armor(),
            "warrior" => ClothingSet::heavy_armor(),
            "laborer" => ClothingSet::simple(),
            "merchant" => ClothingSet::merchant(),
            _ => ClothingSet::common(),
        };
        
        base_set.with_colors(colors)
    }
    
    fn create_accessories(&self, markers: &[String]) -> Vec<Accessory> {
        markers.iter().filter_map(|marker| {
            match marker.as_str() {
                m if m.starts_with("scar_") => Some(Accessory::Scar {
                    location: m.strip_prefix("scar_").unwrap().to_string(),
                }),
                m if m.contains("_cloak_stolen") => {
                    let faction = m.split("_cloak").next().unwrap();
                    Some(Accessory::StolenCloak { from_faction: faction.to_string() })
                }
                "dagger" => Some(Accessory::Weapon { weapon_type: "dagger".to_string() }),
                _ => None,
            }
        }).collect()
    }
}
```

### Animation System

```rust
pub struct AnimationSystem {
    animations: HashMap<AnimationId, Animation>,
    active: HashMap<Entity, ActiveAnimation>,
}

pub struct Animation {
    pub frames: Vec<AnimationFrame>,
    pub duration: f32,
    pub looping: bool,
}

pub struct AnimationFrame {
    pub sprite_index: usize,
    pub duration: f32,
    pub events: Vec<AnimationEvent>,  // Sound, particle spawn, etc.
}

pub struct ActiveAnimation {
    pub animation_id: AnimationId,
    pub progress: f32,
    pub speed_multiplier: f32,
}

impl AnimationSystem {
    pub fn update(&mut self, dt: f32) {
        for (entity, active) in self.active.iter_mut() {
            let anim = &self.animations[&active.animation_id];
            active.progress += dt * active.speed_multiplier;
            
            if active.progress >= anim.duration {
                if anim.looping {
                    active.progress %= anim.duration;
                } else {
                    // Animation complete
                    active.progress = anim.duration;
                }
            }
        }
    }
    
    pub fn set_animation(&mut self, entity: Entity, animation_id: AnimationId) {
        self.active.insert(entity, ActiveAnimation {
            animation_id,
            progress: 0.0,
            speed_multiplier: 1.0,
        });
    }
    
    pub fn transition_to(&mut self, entity: Entity, animation_id: AnimationId, blend_time: f32) {
        // Smooth transition between animations
        // For Paper Mario style, this might be instant or have quick crossfade
    }
}

// Animation triggers based on simulation state
impl AnimationSystem {
    pub fn sync_with_state(&mut self, agent: &VisualAgent, sim_state: &AgentState) {
        let target_anim = match &agent.movement_state {
            MovementState::Idle => AnimationId::Idle,
            MovementState::Walking { .. } => AnimationId::Walk,
            MovementState::Running { .. } => AnimationId::Run,
            MovementState::Interacting { .. } => AnimationId::Interact,
            MovementState::InConversation { .. } => AnimationId::Talk,
        };
        
        // Override for emotional states
        let target_anim = if sim_state.in_combat {
            AnimationId::Fight
        } else if sim_state.mourning {
            AnimationId::Mourn
        } else {
            target_anim
        };
        
        if self.current_animation(agent.entity) != Some(target_anim) {
            self.transition_to(agent.entity, target_anim, 0.1);
        }
    }
}
```

### Visual Markers Over Time

Agents accumulate visual history:

```rust
pub struct VisualHistoryTracker {
    agent_histories: HashMap<String, AgentVisualHistory>,
}

pub struct AgentVisualHistory {
    pub scars: Vec<ScarRecord>,
    pub stolen_items: Vec<StolenItemRecord>,
    pub rank_changes: Vec<RankChangeRecord>,
}

pub struct ScarRecord {
    pub acquired_tick: u64,
    pub from_event: String,
    pub location: ScarLocation,
    pub severity: f32,
}

impl VisualHistoryTracker {
    pub fn process_event(&mut self, event: &Event) {
        match event.event_type.as_str() {
            "conflict" if event.subtype == "fight" || event.subtype == "duel" => {
                // Participants might get scars
                for agent_id in event.all_agent_ids() {
                    if rand::random::<f32>() < 0.3 {  // 30% chance of visible scar
                        self.add_scar(&agent_id, event);
                    }
                }
            }
            "betrayal" if event.subtype == "defection" => {
                // Defector might take faction items
                let primary = &event.actors.primary;
                self.add_stolen_item(&primary.agent_id, StolenItemRecord {
                    item_type: "cloak".to_string(),
                    from_faction: primary.faction.clone(),
                    acquired_tick: event.timestamp.tick,
                });
            }
            "faction" if event.subtype == "promotion" => {
                // Update rank insignia
                let primary = &event.actors.primary;
                self.add_rank_change(&primary.agent_id, RankChangeRecord {
                    new_rank: event.outcome.get("new_role").cloned(),
                    tick: event.timestamp.tick,
                });
            }
            _ => {}
        }
    }
}
```

---

## UI Overlay System

### Overlay Types

```rust
pub enum Overlay {
    AgentLabel(AgentLabelOverlay),
    FactionTerritory(FactionTerritoryOverlay),
    RelationshipLines(RelationshipOverlay),
    TensionIndicator(TensionIndicatorOverlay),
    Commentary(CommentaryOverlay),
    EventPopup(EventPopupOverlay),
    MiniMap(MiniMapOverlay),
    Timeline(TimelineOverlay),
    AgentDetail(AgentDetailPanel),
}

pub struct AgentLabelOverlay {
    pub show_names: bool,
    pub show_faction: bool,
    pub show_role: bool,
    pub filter: AgentFilter,
}

pub struct FactionTerritoryOverlay {
    pub show_borders: bool,
    pub show_color_wash: bool,
    pub opacity: f32,
}

pub struct RelationshipOverlay {
    pub focused_agent: Option<String>,
    pub show_positive: bool,
    pub show_negative: bool,
    pub min_strength: f32,
}

pub struct CommentaryOverlay {
    pub current_items: VecDeque<CommentaryDisplay>,
    pub position: CommentaryPosition,
    pub style: CommentaryStyle,
}

pub struct CommentaryDisplay {
    pub item: CommentaryItem,
    pub entered_at: f32,
    pub opacity: f32,
}

pub enum CommentaryPosition {
    Bottom,
    TopLeft,
    Subtitle,
}
```

### Information Panel

```rust
pub struct AgentDetailPanel {
    pub agent_id: String,
    pub visible: bool,
    pub expanded_sections: HashSet<String>,
}

impl AgentDetailPanel {
    pub fn render(&self, agent: &Agent, relationships: &RelationshipMap, ui: &mut Ui) {
        ui.panel("agent_detail", |ui| {
            // Header
            ui.heading(&agent.name);
            ui.label(&format!("{} of {}", agent.role, agent.faction));
            
            // Trust summary (collapsed by default)
            if ui.collapsing_header("Relationships") {
                for (other_id, rel) in relationships.get(&agent.agent_id) {
                    ui.horizontal(|ui| {
                        ui.label(&other_id);
                        ui.colored_bar("reliability", rel.reliability, Color::BLUE);
                        ui.colored_bar("alignment", rel.alignment, Color::GREEN);
                    });
                }
            }
            
            // Goals
            if ui.collapsing_header("Goals") {
                for goal in &agent.goals {
                    ui.label(&format!("{}: {:.0}%", goal.goal, goal.priority * 100.0));
                }
            }
            
            // Recent memories
            if ui.collapsing_header("Recent Memories") {
                // Show last N significant memories
            }
        });
    }
}
```

### Playback Controls

```rust
pub struct PlaybackControls {
    pub state: PlaybackState,
    pub speed: f32,
    pub current_tick: u64,
    pub max_tick: u64,
}

pub enum PlaybackState {
    Playing,
    Paused,
    FastForward { multiplier: f32 },
    Rewinding,
    Scrubbing { target_tick: u64 },
}

impl PlaybackControls {
    pub fn render(&self, ui: &mut Ui) -> PlaybackCommand {
        ui.horizontal(|ui| {
            if ui.button("⏮").clicked() {
                return PlaybackCommand::JumpToStart;
            }
            if ui.button("⏪").clicked() {
                return PlaybackCommand::StepBack(100);
            }
            
            let play_pause = if matches!(self.state, PlaybackState::Playing) {
                "⏸"
            } else {
                "▶"
            };
            if ui.button(play_pause).clicked() {
                return PlaybackCommand::TogglePlayPause;
            }
            
            if ui.button("⏩").clicked() {
                return PlaybackCommand::StepForward(100);
            }
            if ui.button("⏭").clicked() {
                return PlaybackCommand::JumpToEnd;
            }
            
            // Speed selector
            ui.label(&format!("{:.1}x", self.speed));
            if ui.button("-").clicked() {
                return PlaybackCommand::SetSpeed(self.speed * 0.5);
            }
            if ui.button("+").clicked() {
                return PlaybackCommand::SetSpeed(self.speed * 2.0);
            }
            
            // Timeline scrubber
            let mut tick = self.current_tick as f32;
            if ui.slider(&mut tick, 0.0..=self.max_tick as f32) {
                return PlaybackCommand::SeekTo(tick as u64);
            }
        });
        
        PlaybackCommand::None
    }
}
```

---

## Event Visualization

### Event Effects

```rust
pub struct EventVisualizer {
    effect_templates: HashMap<String, EventEffectTemplate>,
}

pub struct EventEffectTemplate {
    pub camera_behavior: Option<CameraOverride>,
    pub particle_effects: Vec<ParticleEffectConfig>,
    pub sound_cue: Option<String>,
    pub agent_animations: Vec<(String, AnimationId)>,  // (role, animation)
    pub screen_effect: Option<ScreenEffect>,
    pub ui_popup: Option<PopupConfig>,
}

pub enum ScreenEffect {
    Flash { color: Color, duration: f32 },
    Shake { intensity: f32, duration: f32 },
    Vignette { color: Color, duration: f32 },
    SlowMotion { factor: f32, duration: f32 },
}

impl EventVisualizer {
    pub fn visualize_event(&self, event: &Event, world: &mut VisualWorld) {
        let key = format!("{}_{}", event.event_type, event.subtype);
        let template = self.effect_templates.get(&key)
            .or_else(|| self.effect_templates.get(&event.event_type));
        
        if let Some(template) = template {
            // Spawn particle effects
            for particle_config in &template.particle_effects {
                let position = self.get_event_position(event, world);
                world.effects.push(VisualEffect::Particles {
                    config: particle_config.clone(),
                    position,
                    spawn_time: world.current_time,
                });
            }
            
            // Trigger agent animations
            for (role, animation) in &template.agent_animations {
                if let Some(agent_id) = self.get_agent_by_role(event, role) {
                    if let Some(agent) = world.agents.get_mut(&agent_id) {
                        agent.animation_state = animation.to_state();
                    }
                }
            }
            
            // Screen effects
            if let Some(effect) = &template.screen_effect {
                world.effects.push(VisualEffect::Screen(effect.clone()));
            }
            
            // UI popup
            if let Some(popup) = &template.ui_popup {
                world.overlays.push(Overlay::EventPopup(EventPopupOverlay {
                    event_id: event.event_id.clone(),
                    config: popup.clone(),
                    entered_at: world.current_time,
                }));
            }
        }
    }
}

// Effect configurations
impl EventVisualizer {
    pub fn default_effects() -> HashMap<String, EventEffectTemplate> {
        let mut effects = HashMap::new();
        
        effects.insert("betrayal".to_string(), EventEffectTemplate {
            camera_behavior: Some(CameraOverride::ZoomIn { duration: 0.5 }),
            particle_effects: vec![],
            sound_cue: Some("betrayal_sting".to_string()),
            agent_animations: vec![
                ("primary".to_string(), AnimationId::Guilty),
            ],
            screen_effect: Some(ScreenEffect::Vignette {
                color: Color::rgba(0.5, 0.0, 0.0, 0.3),
                duration: 2.0,
            }),
            ui_popup: Some(PopupConfig {
                text_template: "{primary_name} betrays {affected_faction}".to_string(),
                duration: 3.0,
                style: PopupStyle::Dramatic,
            }),
        });
        
        effects.insert("death_killed".to_string(), EventEffectTemplate {
            camera_behavior: Some(CameraOverride::Hold { duration: 2.0 }),
            particle_effects: vec![],
            sound_cue: Some("death_toll".to_string()),
            agent_animations: vec![
                ("primary".to_string(), AnimationId::Death),
            ],
            screen_effect: Some(ScreenEffect::Flash {
                color: Color::WHITE,
                duration: 0.1,
            }),
            ui_popup: Some(PopupConfig {
                text_template: "{primary_name} has fallen".to_string(),
                duration: 4.0,
                style: PopupStyle::Somber,
            }),
        });
        
        effects.insert("ritual_reading_held".to_string(), EventEffectTemplate {
            camera_behavior: Some(CameraOverride::FrameLocation {
                location_key: "location".to_string(),
            }),
            particle_effects: vec![ParticleEffectConfig {
                effect_type: "candle_glow".to_string(),
                count: 5,
            }],
            sound_cue: Some("ritual_ambient".to_string()),
            agent_animations: vec![
                ("primary".to_string(), AnimationId::Reading),
            ],
            screen_effect: None,
            ui_popup: None,
        });
        
        effects
    }
}
```

---

## Phase 1: Core Rendering

### Goal
Get the world visible: map, agents moving, basic camera. No Director integration yet—just visualize current state.

### Deliverables

1. **World map rendering**
   - Terrain grid with faction territory coloring
   - Location markers (villages, camps, landmarks)
   - Travel paths between locations

2. **Agent rendering**
   - Placeholder sprites (colored shapes by faction)
   - Position sync from `current_state.json`
   - Basic movement interpolation

3. **Manual camera**
   - Pan with mouse drag
   - Zoom with scroll wheel
   - Click to center on agent/location

4. **State loading**
   - Load and parse `current_state.json`
   - Hot-reload on file change (for live sim connection)

### Milestones

| Milestone | Description | Estimate |
|-----------|-------------|----------|
| 1.1 | Bevy project setup, window, basic rendering | 2-3 days |
| 1.2 | Map rendering, territory visualization | 3-4 days |
| 1.3 | Agent spawning, position sync | 3-4 days |
| 1.4 | Movement interpolation, basic animation | 3-4 days |
| 1.5 | Manual camera controls | 2-3 days |
| 1.6 | State file loading, hot reload | 2-3 days |

---

## Phase 2: Director Integration

### Goal
Connect to Director output. Camera follows instructions, commentary appears, events have visual feedback.

### Deliverables

1. **Director interface**
   - Load `camera_script.json`
   - Process camera instructions
   - Handle user override → manual mode

2. **Camera behaviors**
   - Follow agent smoothly
   - Frame multiple agents
   - Location transitions
   - Pacing-aware movement speed

3. **Commentary display**
   - Subtitle-style text overlay
   - Queue management (don't overlap)
   - Fade in/out

4. **Basic event visualization**
   - Popup for significant events
   - Simple screen effects

### Milestones

| Milestone | Description | Estimate |
|-----------|-------------|----------|
| 2.1 | Director file loading, instruction parsing | 2-3 days |
| 2.2 | Camera mode implementations | 4-5 days |
| 2.3 | Director/manual mode switching | 2-3 days |
| 2.4 | Commentary overlay system | 3-4 days |
| 2.5 | Event popup system | 2-3 days |
| 2.6 | Integration testing with Director output | 3-4 days |

---

## Phase 3: Sprite System

### Goal
Replace placeholder sprites with proper Paper Mario-style procedural agents.

### Deliverables

1. **Base sprite sheets**
   - Body types (3-4 variations)
   - Clothing sets per role
   - Faction color palettes

2. **Procedural generation**
   - Config-driven sprite composition
   - Deterministic from agent ID (reproducible)
   - Faction-appropriate styling

3. **Visual markers**
   - Scar overlays
   - Rank insignia
   - Stolen items visible

4. **Animation**
   - Walk cycles
   - Idle variations
   - Interaction poses
   - Combat animations

### Milestones

| Milestone | Description | Estimate |
|-----------|-------------|----------|
| 3.1 | Art direction document, reference gathering | 2-3 days |
| 3.2 | Base body sprite sheets | 5-7 days |
| 3.3 | Clothing and accessory sprites | 5-7 days |
| 3.4 | Procedural composition system | 3-4 days |
| 3.5 | Animation system implementation | 4-5 days |
| 3.6 | Visual marker system | 3-4 days |

---

## Phase 4: UI & Polish

### Goal
Full information display, playback controls, polished experience.

### Deliverables

1. **Agent detail panel**
   - Click to inspect any agent
   - Relationship visualization
   - Goal and memory display

2. **Faction overview**
   - Resource display
   - Member list
   - Territory summary

3. **Playback controls**
   - Play/pause
   - Speed control
   - Timeline scrubbing
   - Jump to snapshot

4. **Minimap**
   - Overview of world
   - Tension hotspots highlighted
   - Quick navigation

5. **Visual polish**
   - Particle effects
   - Screen effects for drama
   - Sound integration (framework)

### Milestones

| Milestone | Description | Estimate |
|-----------|-------------|----------|
| 4.1 | Agent detail panel | 4-5 days |
| 4.2 | Faction overview panel | 3-4 days |
| 4.3 | Playback controls, timeline | 4-5 days |
| 4.4 | Minimap implementation | 3-4 days |
| 4.5 | Particle effect system | 3-4 days |
| 4.6 | Screen effects, visual drama | 3-4 days |
| 4.7 | Sound system framework | 2-3 days |

---

## Phase 5: Replay & Analysis

### Goal
Support historical playback from snapshots and events, plus analysis tooling.

### Deliverables

1. **Snapshot loading**
   - Jump to any saved snapshot
   - Reconstruct visual state

2. **Event replay**
   - Step through events
   - Event highlighting

3. **Relationship graph view**
   - Network visualization
   - Filter by faction, trust dimension
   - Highlight tensions

4. **Timeline view**
   - Event density over time
   - Tension arcs
   - Click to jump

### Milestones

| Milestone | Description | Estimate |
|-----------|-------------|----------|
| 5.1 | Snapshot loading, state reconstruction | 4-5 days |
| 5.2 | Event replay, stepping | 3-4 days |
| 5.3 | Relationship graph visualization | 5-6 days |
| 5.4 | Timeline view | 4-5 days |
| 5.5 | Analysis mode polish | 3-4 days |

---

## Technical Considerations

### Bevy Architecture

```rust
// Main plugin structure
pub struct SimVizPlugin;

impl Plugin for SimVizPlugin {
    fn build(&self, app: &mut App) {
        app
            // Core state
            .insert_resource(VisualWorld::default())
            .insert_resource(CameraController::default())
            .insert_resource(PlaybackState::default())
            
            // Systems
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
            .add_event::<SimulationEvent>()
            .add_event::<CameraCommand>()
            
            // Plugins
            .add_plugins((
                SpritePlugin,
                UIPlugin,
                EffectsPlugin,
            ));
    }
}
```

### Performance Strategies

1. **Spatial partitioning**: Only update/render agents near camera
2. **LOD system**: Distant agents become simpler shapes
3. **Batched rendering**: Group sprites by texture atlas
4. **Async state loading**: Don't block render for file I/O
5. **Interpolation**: Smooth movement between discrete sim ticks

### File Watching

```rust
pub struct StateFileWatcher {
    watcher: RecommendedWatcher,
    rx: Receiver<notify::Result<Event>>,
    last_state: Option<WorldSnapshot>,
    last_director: Option<DirectorOutput>,
}

impl StateFileWatcher {
    pub fn check_updates(&mut self) -> StateUpdates {
        let mut updates = StateUpdates::default();
        
        while let Ok(event) = self.rx.try_recv() {
            if let Ok(event) = event {
                for path in event.paths {
                    if path.ends_with("current_state.json") {
                        if let Ok(state) = self.load_state(&path) {
                            updates.new_state = Some(state);
                        }
                    }
                    if path.ends_with("camera_script.json") {
                        if let Ok(director) = self.load_director(&path) {
                            updates.new_director = Some(director);
                        }
                    }
                }
            }
        }
        
        updates
    }
}
```

---

## Open Questions

1. **Art style details**: Paper Mario is the reference, but how stylized? Outlines? Color palette constraints? This needs an art direction pass.

2. **Sound**: The framework is planned but sound design is its own project. What's the minimum viable audio? Just event stings? Ambient per-location?

3. **Multi-window**: Should analysis views (relationship graph, timeline) be overlays or separate windows? Bevy supports multiple windows but it adds complexity.

4. **Mobile/Web**: Is browser deployment a goal? Bevy compiles to WASM but performance and file I/O work differently.

5. **Recording**: Should we support video export of sessions? Screen recording externally works, but built-in export with compression would be cleaner.

---

*Document version 0.1 — December 2024*
