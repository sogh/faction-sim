# director Crate

## Purpose

The Director AI sits between simulation and visualization. It watches raw events and active tensions, then decides **what's worth showing** and **how to show it**. Think of it as an invisible documentary filmmaker—choosing when to cut, where to point the camera, and what story threads to follow.

## Key Files

```
src/
├── lib.rs          # Public API, Director struct
├── scorer.rs       # Event prioritization with configurable weights
├── focus.rs        # Tension-based camera focus selection
├── threads.rs      # Narrative thread tracking
├── commentary.rs   # Template-based text generation
└── output.rs       # CameraInstruction, CommentaryItem, output serialization
```

## Design Doc Reference

See `/docs/design/director_ai_design.md` for the complete specification.

## Core Concepts

### The Director Does NOT:
- Control the simulation
- Decide what happens
- Generate events

### The Director DOES:
- Score events by dramatic interest
- Track developing tensions as narrative threads
- Decide which thread deserves camera focus
- Generate camera instructions for visualization
- Create commentary text (captions, dramatic irony)
- Mark highlights for later summarization

## Key Types

### CameraInstruction
```rust
pub struct CameraInstruction {
    pub instruction_id: String,
    pub timestamp: SimTimestamp,
    pub valid_until: Option<SimTimestamp>,
    pub camera_mode: CameraMode,
    pub focus: CameraFocus,
    pub pacing: PacingHint,
    pub reason: String,
    pub tension_id: Option<String>,
}

pub enum CameraMode {
    FollowAgent { agent_id: String, zoom: ZoomLevel },
    FrameLocation { location_id: String, zoom: ZoomLevel },
    FrameMultiple { agent_ids: Vec<String>, auto_zoom: bool },
    Cinematic { path: Vec<CameraWaypoint>, duration_ticks: u32 },
    Overview { region: Option<String> },
}

pub enum PacingHint {
    Slow,       // Linger, something subtle
    Normal,     // Standard pacing
    Urgent,     // Quick cuts, tension building
    Climactic,  // Key moment, hold steady
}
```

### NarrativeThread
```rust
pub struct NarrativeThread {
    pub thread_id: String,
    pub created_at_tick: u64,
    pub last_updated_tick: u64,
    pub status: ThreadStatus,
    pub tension_ids: Vec<String>,
    pub key_agents: Vec<String>,
    pub key_events: Vec<String>,
    pub thread_type: String,
    pub summary: String,
    pub screen_time_ticks: u64,
    pub last_shown_tick: Option<u64>,
}

pub enum ThreadStatus {
    Developing,
    Climaxing,
    Resolving,
    Dormant,
    Concluded,
}
```

### CommentaryItem
```rust
pub struct CommentaryItem {
    pub item_id: String,
    pub timestamp: SimTimestamp,
    pub display_duration_ticks: u32,
    pub commentary_type: CommentaryType,
    pub content: String,
    pub priority: f32,
    pub related_agents: Vec<String>,
    pub related_tension: Option<String>,
}

pub enum CommentaryType {
    EventCaption,      // "Mira arrives at the eastern bridge"
    DramaticIrony,     // "Corin doesn't know..."
    ContextReminder,   // "Three months ago..."
    TensionTeaser,     // "Winter stores are running low..."
    NarratorVoice,     // LLM-generated (Phase 3)
}
```

## Processing Loop

```rust
impl Director {
    pub fn process_tick(
        &mut self,
        events: &[Event],
        tensions: &[Tension],
        state: &WorldSnapshot,
    ) -> DirectorOutput {
        // 1. Score events
        let scored = self.scorer.score_events(events, &self.context);
        let notable: Vec<_> = scored.iter()
            .filter(|(_, score)| *score > self.threshold)
            .collect();
        
        // 2. Update narrative threads
        self.threads.update(&notable, tensions);
        
        // 3. Select focus
        let camera_script = self.focus.select(
            tensions,
            &self.threads.active(),
            &self.current_focus,
        );
        
        // 4. Generate commentary
        let commentary = self.commentary.generate(&notable, tensions);
        
        // 5. Mark highlights
        let highlights = self.mark_highlights(&notable);
        
        DirectorOutput {
            generated_at_tick: state.timestamp.tick,
            camera_script,
            commentary_queue: commentary,
            active_threads: self.threads.active(),
            highlights,
        }
    }
}
```

## Implementation Phases

### Phase 1: Template-Based (Current)
- [x] Design doc complete
- [ ] Event scoring with configurable weights
- [ ] Tension-based focus selection
- [ ] Thread fatigue tracking
- [ ] Template-based captions
- [ ] Basic dramatic irony detection
- [ ] Camera instruction output
- [ ] Config file for weights

### Phase 2: Pattern Detection
- [ ] Multi-event sequence patterns
- [ ] Predictive hooks (anticipate betrayals)
- [ ] Social graph analysis
- [ ] Enhanced thread tracking

### Phase 3: LLM Integration
- [ ] LLM client abstraction
- [ ] Narrative summarization
- [ ] Dynamic commentary
- [ ] Chronicle generation
- [ ] Fallback to templates

## Configuration

Weights and thresholds are configurable via TOML:

```toml
[event_weights.base_scores]
betrayal = 0.9
death = 0.85
conflict = 0.7
faction = 0.6
ritual = 0.5

[event_weights.drama_tag_scores]
faction_critical = 0.3
secret_meeting = 0.25
leader_involved = 0.2

[focus]
min_tension_severity = 0.3
max_concurrent_threads = 3
thread_fatigue_threshold_ticks = 5000

[commentary]
max_queue_size = 5
min_drama_for_caption = 0.3
```

## Template System

Templates use slot-filling for commentary:

```toml
[templates.event_captions.betrayal.secret_shared_with_enemy]
templates = [
    "{primary_name} shares faction secrets with {secondary_name}",
    "At {location}, {primary_name} crosses a line that cannot be uncrossed",
]

[templates.dramatic_irony.unaware_of_betrayal]
templates = [
    "{unaware_agent} still trusts {betrayer}—for now",
    "If only {unaware_agent} knew what happened at {betrayal_location}",
]
```

## Testing Strategy

1. **Scoring tests**: Known events → expected scores
2. **Focus tests**: Given tensions → correct camera choice
3. **Template tests**: Fill templates, verify output
4. **Golden tests**: Sample event stream → expected full output
5. **Fatigue tests**: Same thread shouldn't dominate forever

```rust
#[test]
fn test_betrayal_scores_high() {
    let scorer = EventScorer::default();
    let betrayal = make_betrayal_event();
    let movement = make_movement_event();
    
    let b_score = scorer.score(&betrayal, &context);
    let m_score = scorer.score(&movement, &context);
    
    assert!(b_score > 0.8);
    assert!(m_score < 0.2);
    assert!(b_score > m_score);
}

#[test]
fn test_thread_fatigue() {
    let mut director = Director::new();
    let tension = make_high_severity_tension();
    
    // Focus on same tension many times
    for _ in 0..100 {
        director.process_tick(&[], &[tension.clone()], &state);
    }
    
    // Should eventually switch away
    let output = director.process_tick(&[], &[tension], &state);
    assert!(output.camera_script[0].reason.contains("fatigue"));
}
```

## Output Files

The director produces:

| File | Format | Description |
|------|--------|-------------|
| `camera_script.json` | JSON | Camera instructions for viz |
| `commentary.json` | JSON | Text overlays |
| `highlights.json` | JSON | Marked moments for summarization |

## Dependencies

- `sim-events`: Event and tension types
- `serde` + `serde_json`: Serialization
- `toml`: Config loading
- `tracing`: Logging decisions

## Gotchas

- Director runs *ahead* of visualization—don't assume real-time
- Camera instructions have `valid_until`—viz should check
- Thread fatigue prevents fixating on one storyline
- Templates must handle missing optional fields gracefully
- LLM integration (Phase 3) is *additive*—templates always work
