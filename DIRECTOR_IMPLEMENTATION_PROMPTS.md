# Director AI Implementation Prompts

## Prerequisites

Before starting these prompts, ensure:
- [ ] Project structure is set up (see SETUP_INSTRUCTIONS.md)
- [ ] sim-events crate has core types implemented (Event, Tension, SimTimestamp)
- [ ] You're in the project root directory

## Overview

These prompts implement **Director AI Phase 1: Template-Based Director**. After completing all prompts, you'll have:
- Event scoring with configurable weights
- Tension-based focus selection
- Narrative thread tracking
- Template-based commentary generation
- Camera instruction output

---

## Prompt 1: Core Types and Output Structures

```
Implement the core output types for the director crate in src/output.rs.

Based on docs/design/director_ai_design.md, create these types:

1. CameraInstruction with fields:
   - instruction_id: String
   - timestamp: SimTimestamp (from sim-events)
   - valid_until: Option<SimTimestamp>
   - camera_mode: CameraMode
   - focus: CameraFocus
   - pacing: PacingHint
   - reason: String
   - tension_id: Option<String>

2. CameraMode enum with variants:
   - FollowAgent { agent_id: String, zoom: ZoomLevel }
   - FrameLocation { location_id: String, zoom: ZoomLevel }
   - FrameMultiple { agent_ids: Vec<String>, auto_zoom: bool }
   - Cinematic { path: Vec<CameraWaypoint>, duration_ticks: u32 }
   - Overview { region: Option<String> }

3. Supporting enums: CameraFocus, PacingHint, ZoomLevel, CameraWaypoint

4. CommentaryItem with fields:
   - item_id: String
   - timestamp: SimTimestamp
   - display_duration_ticks: u32
   - commentary_type: CommentaryType
   - content: String
   - priority: f32
   - related_agents: Vec<String>
   - related_tension: Option<String>

5. CommentaryType enum: EventCaption, DramaticIrony, ContextReminder, TensionTeaser

6. DirectorOutput struct containing camera_script, commentary_queue, active_threads, highlights

7. HighlightMarker for marking notable moments

All types should derive Debug, Clone, Serialize, Deserialize.
Use #[serde(rename_all = "snake_case")] for enums.

Update src/lib.rs to export these types.
```

---

## Prompt 2: Narrative Thread Tracking

```
Implement narrative thread tracking in src/threads.rs.

Create:

1. NarrativeThread struct with:
   - thread_id: String
   - created_at_tick: u64
   - last_updated_tick: u64
   - status: ThreadStatus
   - tension_ids: Vec<String>
   - key_agents: Vec<String>
   - key_events: Vec<String>
   - thread_type: String
   - summary: String
   - hook: String
   - screen_time_ticks: u64
   - last_shown_tick: Option<u64>

2. ThreadStatus enum: Developing, Climaxing, Resolving, Dormant, Concluded

3. ThreadTracker struct that manages a collection of threads:
   - new() -> Self
   - update(&mut self, events: &[ScoredEvent], tensions: &[Tension])
   - active(&self) -> Vec<&NarrativeThread>
   - get_thread_for_tension(&self, tension_id: &str) -> Option<&NarrativeThread>
   - record_screen_time(&mut self, thread_id: &str, ticks: u64)
   - mark_concluded(&mut self, thread_id: &str)

The update method should:
- Create new threads for new tensions above severity threshold
- Update existing threads with new events
- Transition thread status based on tension status
- Mark threads dormant if no activity for N ticks

Add tests for thread lifecycle.
```

---

## Prompt 3: Event Scoring System

```
Implement event scoring in src/scorer.rs.

Create:

1. EventWeights struct (loadable from config):
   - base_scores: HashMap<String, f32>  // event_type -> score
   - subtype_modifiers: HashMap<String, f32>  // subtype -> multiplier
   - drama_tag_scores: HashMap<String, f32>  // tag -> additive score

2. DirectorContext struct:
   - tracked_agents: HashSet<String>
   - active_tension_events: HashSet<String>
   - current_focus: Option<String>  // agent_id being followed

3. ScoredEvent struct:
   - event: Event (or reference)
   - score: f32

4. EventScorer struct:
   - weights: EventWeights
   
   Methods:
   - from_config(path: &Path) -> Result<Self>
   - default() -> Self  // with sensible defaults from design doc
   - score(&self, event: &Event, context: &DirectorContext) -> f32
   - score_batch(&self, events: &[Event], context: &DirectorContext) -> Vec<ScoredEvent>

Scoring logic:
- Start with base_scores.get(event_type) or 0.1
- Multiply by subtype_modifiers.get(subtype) if present
- Add drama_tag_scores for each matching tag
- Multiply by 1.5 if event involves tracked_agents
- Multiply by 2.0 if event_id is in active_tension_events

Default weights (from design doc):
- betrayal: 0.9, death: 0.85, conflict: 0.7, faction: 0.6
- ritual: 0.5, cooperation: 0.4, communication: 0.3
- resource: 0.25, movement: 0.1

Add unit tests verifying:
- Betrayal scores higher than movement
- Tracked agent boost works
- Drama tags are additive
```

---

## Prompt 4: Configuration Loading

```
Create src/config.rs for loading director configuration from TOML.

1. Create DirectorConfig struct:
   - event_weights: EventWeightsConfig
   - focus: FocusConfig
   - commentary: CommentaryConfig
   - templates_path: PathBuf

2. EventWeightsConfig matching EventWeights fields

3. FocusConfig:
   - min_tension_severity: f32 (default 0.3)
   - max_concurrent_threads: usize (default 3)
   - thread_fatigue_threshold_ticks: u64 (default 5000)
   - default_camera_mode: String

4. CommentaryConfig:
   - max_queue_size: usize (default 5)
   - min_drama_for_caption: f32 (default 0.3)
   - caption_duration_ticks: u32 (default 100)

5. Implement:
   - DirectorConfig::load(path: &Path) -> Result<Self>
   - DirectorConfig::default() -> Self

Create a default config file at config/director.toml with all settings.

Example TOML structure:
[event_weights.base_scores]
betrayal = 0.9
death = 0.85

[focus]
min_tension_severity = 0.3

[commentary]
max_queue_size = 5
```

---

## Prompt 5: Focus Selection

```
Implement camera focus selection in src/focus.rs.

Create:

1. FocusSelector struct:
   - config: FocusConfig
   
2. Methods:
   - new(config: FocusConfig) -> Self
   - select_focus(
       &self,
       tensions: &[Tension],
       threads: &[NarrativeThread],
       current_focus: Option<&CameraFocus>,
       scored_events: &[ScoredEvent],
     ) -> CameraInstruction

Selection logic:

1. Filter tensions to those with severity >= min_tension_severity
2. If no viable tensions, return default_wandering_camera()
3. Check if current focus is still on an active, non-fatigued thread
   - If yes, continue with that focus (return continue_focus instruction)
4. Otherwise, select highest severity tension that isn't fatigued
5. Generate appropriate CameraInstruction based on tension type

Helper methods:
- is_fatigued(&self, tension: &Tension, threads: &[NarrativeThread]) -> bool
- focus_on_tension(&self, tension: &Tension) -> CameraInstruction
- continue_focus(&self, tension: &Tension) -> CameraInstruction
- default_wandering_camera(&self) -> CameraInstruction

The focus_on_tension method should:
- Use FollowAgent for tensions with a clear primary agent
- Use FrameMultiple for tensions involving multiple key agents
- Use FrameLocation for location-centric tensions
- Set pacing based on tension severity (higher = more urgent)

Add tests for:
- Empty tensions returns wandering camera
- Highest severity wins
- Fatigue causes switch to different tension
```

---

## Prompt 6: Template System

```
Implement the template system in src/commentary.rs.

Create:

1. CommentaryTemplates struct loadable from TOML:
   - event_captions: HashMap<String, Vec<String>>  // "betrayal.secret_shared" -> templates
   - dramatic_irony: Vec<IronyTemplate>
   - context_reminders: Vec<ReminderTemplate>
   - tension_teasers: Vec<TeaserTemplate>

2. IronyTemplate:
   - pattern: String  // e.g., "agent_unaware_of_betrayal"
   - templates: Vec<String>
   - required_context: Vec<String>

3. CommentaryGenerator:
   - templates: CommentaryTemplates
   - config: CommentaryConfig

4. Methods:
   - load_templates(path: &Path) -> Result<CommentaryTemplates>
   - caption_event(&self, event: &Event) -> Option<CommentaryItem>
   - generate_irony(&self, situation: &IronySituation) -> Option<CommentaryItem>
   - generate_teaser(&self, tension: &Tension) -> Option<CommentaryItem>

5. Template filling:
   - fill_template(&self, template: &str, event: &Event) -> String
   
   Supports placeholders:
   - {primary_name}, {primary_faction}, {primary_role}
   - {secondary_name}, {secondary_faction}
   - {location}
   - {affected_names} (comma-separated)

Create templates/commentary.toml with:

[event_captions]
"betrayal.secret_shared_with_enemy" = [
    "{primary_name} shares faction secrets with {secondary_name}",
    "At {location}, {primary_name} crosses a line that cannot be uncrossed",
]
"betrayal.defection" = [
    "{primary_name} abandons {primary_faction}",
    "A traitor reveals themselves: {primary_name} defects",
]
"death.killed" = [
    "{primary_name} has fallen",
    "Death claims {primary_name}",
]
"ritual.reading_held" = [
    "The faithful gather at {location}",
    "{primary_name} opens the book of {primary_faction}",
]
"movement.travel" = [
    "{primary_name} journeys to {location}",
]

[dramatic_irony]
[[dramatic_irony.patterns]]
pattern = "unaware_of_betrayal"
templates = [
    "{unaware_agent} still trusts {betrayer}—for now",
    "If only {unaware_agent} knew what {betrayer} did",
]

Add tests for template filling with various events.
```

---

## Prompt 7: Irony Detection

```
Implement dramatic irony detection in src/commentary.rs (extend existing).

Create:

1. IronySituation struct:
   - situation_type: String
   - unaware_agent: AgentSnapshot
   - betrayer: Option<AgentSnapshot>
   - secret_info: String
   - betrayal_event: Option<String>  // event_id

2. IronyDetector struct:
   - recent_betrayals: Vec<BetrayalRecord>
   
   BetrayalRecord:
   - event_id: String
   - betrayer_id: String
   - affected_ids: Vec<String>
   - tick: u64
   - discovered_by: HashSet<String>

3. Methods:
   - record_betrayal(&mut self, event: &Event)
   - detect_irony(&self, state: &WorldSnapshot) -> Vec<IronySituation>
   - mark_discovered(&mut self, betrayal_event_id: &str, discoverer_id: &str)

Detection logic for "unaware_of_betrayal":
1. For each recorded betrayal not yet discovered by affected parties
2. Check if any affected agent still has high trust in the betrayer
3. If so, create IronySituation

The detect_irony method checks the snapshot's relationship data:
- Look up trust from affected agent toward betrayer
- If reliability trust > 0.5, they're still unaware = irony opportunity

Integrate with CommentaryGenerator:
- generate_all_commentary(&self, events: &[ScoredEvent], state: &WorldSnapshot, irony_detector: &IronyDetector) -> Vec<CommentaryItem>

Add tests:
- Betrayal creates irony situation
- Irony clears when trust drops
- Multiple affected agents can have separate irony situations
```

---

## Prompt 8: Main Director Struct

```
Implement the main Director struct in src/lib.rs.

Create:

1. Director struct:
   - config: DirectorConfig
   - scorer: EventScorer
   - focus_selector: FocusSelector
   - thread_tracker: ThreadTracker
   - commentary_generator: CommentaryGenerator
   - irony_detector: IronyDetector
   - current_tick: u64
   - notability_threshold: f32

2. DirectorContext (for scoring):
   - tracked_agents: HashSet<String>
   - active_tension_events: HashSet<String>

3. Methods:
   - new(config: DirectorConfig) -> Result<Self>
   - from_config_file(path: &Path) -> Result<Self>
   - default() -> Self
   
   - process_tick(
       &mut self,
       events: &[Event],
       tensions: &[Tension],
       state: &WorldSnapshot,
     ) -> DirectorOutput

4. process_tick implementation:
   a. Build context from current thread state
   b. Score all events
   c. Filter to notable events (score > threshold)
   d. Update thread tracker with notable events and tensions
   e. Process events for irony detection
   f. Select camera focus
   g. Generate commentary (captions + irony + teasers)
   h. Mark highlights
   i. Update current_tick
   j. Return DirectorOutput

5. Helper methods:
   - build_context(&self) -> DirectorContext
   - mark_highlights(&self, events: &[ScoredEvent]) -> Vec<HighlightMarker>
   - update_tracked_agents(&mut self, instruction: &CameraInstruction)

Add integration test:
- Create sample events and tensions
- Process multiple ticks
- Verify camera instructions make sense
- Verify commentary is generated for high-drama events
```

---

## Prompt 9: JSON Output

```
Implement JSON output writing in src/output.rs (extend existing).

Add methods to DirectorOutput:

1. write_camera_script(&self, path: &Path) -> Result<()>
   - Writes camera_script as pretty JSON

2. write_commentary(&self, path: &Path) -> Result<()>
   - Writes commentary_queue as pretty JSON

3. write_highlights(&self, path: &Path) -> Result<()>
   - Writes highlights as pretty JSON

4. write_all(&self, output_dir: &Path) -> Result<()>
   - Writes all three files to the directory

Create OutputWriter struct for streaming output:

1. OutputWriter:
   - output_dir: PathBuf
   - camera_script_file: File
   - commentary_file: File
   
2. Methods:
   - new(output_dir: &Path) -> Result<Self>
   - write_tick(&mut self, output: &DirectorOutput) -> Result<()>
   - flush(&mut self) -> Result<()>

The streaming writer appends to files, allowing the visualization
to tail them in real-time.

Add a simple CLI in src/main.rs (optional bin target):
- Load events from events.jsonl
- Load tensions from tensions.json
- Load state from current_state.json
- Run director
- Write output

cargo run -p director -- --events events.jsonl --tensions tensions.json --state current_state.json --output ./director_output/
```

---

## Prompt 10: Testing with Sample Data

```
Create comprehensive tests for the director crate.

1. Create tests/fixtures/ directory with sample data:

tests/fixtures/sample_events.jsonl - 20-30 events including:
- Movement events (low drama)
- A betrayal event (high drama)
- A ritual reading
- Some communication/rumors
- A death event

tests/fixtures/sample_tensions.json - 2-3 tensions:
- A brewing_betrayal tension
- A resource_conflict tension

tests/fixtures/sample_state.json - Minimal world snapshot

2. Create tests/integration_tests.rs:

#[test]
fn test_full_director_pipeline() {
    // Load fixtures
    // Create director with default config
    // Process tick
    // Verify output has camera instructions
    // Verify high-drama events get commentary
}

#[test]
fn test_betrayal_gets_focus() {
    // Create tensions with betrayal highest severity
    // Verify camera focuses on betrayal agents
}

#[test]
fn test_thread_fatigue_switches_focus() {
    // Run many ticks focusing on same tension
    // Verify eventually switches
}

#[test]
fn test_irony_detection() {
    // Create betrayal event
    // Create state where victim still trusts betrayer
    // Verify irony commentary generated
}

#[test]
fn test_template_filling() {
    // Test various event types fill templates correctly
}

3. Add golden test:
- Process known input
- Compare output to saved expected output
- Update expected output intentionally when behavior changes
```

---

## Verification Checklist

After completing all prompts, verify:

- [ ] `cargo build -p director` succeeds
- [ ] `cargo test -p director` passes
- [ ] Config loads from TOML
- [ ] Templates load and fill correctly
- [ ] Events are scored (betrayal > movement)
- [ ] Tensions drive focus selection
- [ ] Thread fatigue prevents fixation
- [ ] Irony detection finds obvious cases
- [ ] JSON output is valid and matches schema
- [ ] Integration test passes with sample data

---

## Next Steps

After Phase 1 is complete:
- Phase 2: Pattern detection (multi-event sequences)
- Phase 3: LLM integration (narrative summaries)

See docs/design/director_ai_design.md for Phase 2 and 3 specifications.
