//! Director AI: drama detection and camera control.
//!
//! The Director AI sits between simulation and visualization. It watches raw
//! events and active tensions, then decides what's worth showing and how to
//! show it. Think of it as an invisible documentary filmmaker—choosing when
//! to cut, where to point the camera, and what story threads to follow.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐     events.jsonl      ┌──────────┐    camera_script.json
//! │  sim-core   │ ───────────────────▶  │ director │ ──────────────────────▶
//! └─────────────┘                       └──────────┘
//! ```
//!
//! # Modules
//!
//! - [`output`]: Camera instructions, commentary items, and DirectorOutput
//! - [`threads`]: Narrative thread tracking
//! - [`scorer`]: Event prioritization with configurable weights
//! - [`focus`]: Tension-based camera focus selection
//! - [`commentary`]: Template-based text generation

pub mod commentary;
pub mod config;
pub mod focus;
pub mod output;
pub mod scorer;
pub mod threads;

// Re-export output types
pub use output::{
    CameraEasing, CameraFocus, CameraInstruction, CameraMode, CameraWaypoint, CommentaryItem,
    CommentaryType, DirectorOutput, HighlightMarker, HighlightType, OutputError, OutputReader,
    OutputWriter, PacingHint, ZoomLevel, generate_commentary_id, generate_instruction_id,
};

// Re-export thread types
pub use threads::{
    generate_thread_id, NarrativeThread, ScoredEvent, ThreadStatus, ThreadTracker,
    ThreadTrackerConfig,
};

// Re-export scorer types
pub use scorer::{DirectorContext, EventScorer, EventWeights, ScorerError};

// Re-export config types
pub use config::{
    default_config_toml, CommentaryConfig, ConfigError, DefaultCameraMode, DirectorConfig,
    FocusConfig, GeneralConfig, TomlSerializeError,
};

// Re-export focus types
pub use focus::FocusSelector;

// Re-export commentary types
pub use commentary::{
    default_templates, default_templates_toml, BetrayalRecord, CommentaryGenerator,
    CommentaryTemplates, IronyDetector, IronySituation, IronyTemplate, ReminderTemplate,
    TeaserTemplate, TemplateError,
};

use std::collections::HashSet;
use std::path::Path;

use sim_events::{Event, EventType, Tension, WorldSnapshot};

/// Errors that can occur in Director operations.
#[derive(Debug)]
pub enum DirectorError {
    /// Error loading configuration
    Config(ConfigError),
    /// Error loading templates
    Template(TemplateError),
    /// Error with scorer
    Scorer(ScorerError),
}

impl std::fmt::Display for DirectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DirectorError::Config(e) => write!(f, "Config error: {}", e),
            DirectorError::Template(e) => write!(f, "Template error: {}", e),
            DirectorError::Scorer(e) => write!(f, "Scorer error: {}", e),
        }
    }
}

impl std::error::Error for DirectorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DirectorError::Config(e) => Some(e),
            DirectorError::Template(e) => Some(e),
            DirectorError::Scorer(e) => Some(e),
        }
    }
}

impl From<ConfigError> for DirectorError {
    fn from(e: ConfigError) -> Self {
        DirectorError::Config(e)
    }
}

impl From<TemplateError> for DirectorError {
    fn from(e: TemplateError) -> Self {
        DirectorError::Template(e)
    }
}

impl From<ScorerError> for DirectorError {
    fn from(e: ScorerError) -> Self {
        DirectorError::Scorer(e)
    }
}

/// The main Director AI that orchestrates drama detection and camera control.
///
/// The Director watches raw events and active tensions, then decides what's worth
/// showing and how to show it. It coordinates:
/// - Event scoring (which events are dramatically interesting)
/// - Narrative thread tracking (which storylines are developing)
/// - Camera focus selection (where to point the camera)
/// - Commentary generation (what text to overlay)
/// - Dramatic irony detection (when the audience knows more than characters)
#[derive(Debug)]
pub struct Director {
    /// Configuration settings
    config: DirectorConfig,
    /// Event scoring system
    scorer: EventScorer,
    /// Camera focus selector
    focus_selector: FocusSelector,
    /// Narrative thread tracker
    thread_tracker: ThreadTracker,
    /// Commentary generator
    commentary_generator: CommentaryGenerator,
    /// Dramatic irony detector
    irony_detector: IronyDetector,
    /// Current simulation tick
    current_tick: u64,
    /// Threshold for event notability (events must score above this)
    notability_threshold: f32,
    /// Currently tracked agents (for context)
    tracked_agents: HashSet<String>,
    /// Current camera focus
    current_focus: Option<CameraFocus>,
}

impl Director {
    /// Creates a new Director with the given configuration.
    pub fn new(config: DirectorConfig) -> Result<Self, DirectorError> {
        let scorer = EventScorer::new(config.event_weights.clone());
        let focus_selector = FocusSelector::new(config.focus.clone());
        let thread_tracker = ThreadTracker::with_config(config.threads.clone());
        let commentary_generator = CommentaryGenerator::new(
            default_templates(),
            config.commentary.clone(),
        );
        let irony_detector = IronyDetector::new();

        Ok(Self {
            notability_threshold: config.focus.min_event_score,
            config,
            scorer,
            focus_selector,
            thread_tracker,
            commentary_generator,
            irony_detector,
            current_tick: 0,
            tracked_agents: HashSet::new(),
            current_focus: None,
        })
    }

    /// Creates a Director from a configuration file.
    pub fn from_config_file(path: &Path) -> Result<Self, DirectorError> {
        let config = DirectorConfig::from_file(path)?;
        Self::new(config)
    }

    /// Creates a Director with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(DirectorConfig::default()).expect("Default config should always work")
    }

    /// Processes a single tick of simulation data.
    ///
    /// This is the main entry point for the Director. It:
    /// 1. Builds context from current thread state
    /// 2. Scores all events
    /// 3. Filters to notable events (score > threshold)
    /// 4. Updates thread tracker with notable events and tensions
    /// 5. Processes events for irony detection
    /// 6. Selects camera focus
    /// 7. Generates commentary (captions + irony + teasers)
    /// 8. Marks highlights
    /// 9. Updates current_tick
    /// 10. Returns DirectorOutput
    pub fn process_tick(
        &mut self,
        events: &[Event],
        tensions: &[Tension],
        state: &WorldSnapshot,
    ) -> DirectorOutput {
        self.current_tick = state.timestamp.tick;

        // 1. Build context from current thread state
        let context = self.build_context(tensions);

        // 2. Score all events
        let scored_events = self.scorer.score_batch(events, &context);

        // 3. Filter to notable events
        let notable_events: Vec<ScoredEvent> = scored_events
            .into_iter()
            .filter(|se| se.score >= self.notability_threshold)
            .collect();

        // 4. Update thread tracker with notable events and tensions
        self.thread_tracker.update(&notable_events, tensions);

        // 5. Process events for irony detection (record new betrayals)
        for event in events {
            if event.event_type == EventType::Betrayal {
                self.irony_detector.record_betrayal(event);
            }
        }

        // 6. Select camera focus
        // Clone active threads since select_focus expects &[NarrativeThread]
        let active_threads: Vec<NarrativeThread> = self
            .thread_tracker
            .active()
            .into_iter()
            .cloned()
            .collect();
        let camera_instruction = self.focus_selector.select_focus(
            tensions,
            &active_threads,
            self.current_focus.as_ref(),
            &notable_events,
            state.timestamp.clone(),
        );

        // Update tracked agents based on camera focus
        self.update_tracked_agents(&camera_instruction);
        self.current_focus = Some(camera_instruction.focus.clone());

        // 7. Generate commentary
        let mut commentary_queue = Vec::new();

        // Update generator tick
        self.commentary_generator.set_current_tick(self.current_tick);

        // Generate captions for notable events
        for scored in &notable_events {
            if let Some(caption) = self.commentary_generator.caption_event(&scored.event, state.timestamp.clone()) {
                commentary_queue.push(caption);
            }
        }

        // Detect and generate irony commentary
        let irony_situations = self.irony_detector.detect_irony(state);
        for situation in &irony_situations {
            if let Some(irony_commentary) = self.commentary_generator.generate_irony(situation, state.timestamp.clone()) {
                commentary_queue.push(irony_commentary);
            }
        }

        // Generate tension teasers
        for tension in tensions {
            if tension.is_active() && tension.severity >= self.config.focus.min_tension_severity {
                if let Some(teaser) = self.commentary_generator.generate_teaser(tension, state.timestamp.clone()) {
                    commentary_queue.push(teaser);
                }
            }
        }

        // Sort commentary by priority and limit to max queue size
        commentary_queue.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());
        commentary_queue.truncate(self.config.commentary.max_queue_size);

        // 8. Mark highlights
        let highlights = self.mark_highlights(&notable_events, state.timestamp.clone());

        // 9. Build output
        DirectorOutput {
            generated_at_tick: self.current_tick,
            camera_script: vec![camera_instruction],
            commentary_queue,
            active_threads,
            highlights,
        }
    }

    /// Builds scoring context from current state.
    fn build_context(&self, tensions: &[Tension]) -> DirectorContext {
        let mut context = DirectorContext::new();

        // Add tracked agents
        for agent_id in &self.tracked_agents {
            context.track_agent(agent_id);
        }

        // Add active tension events (trigger events that led to this tension)
        for tension in tensions {
            if tension.is_active() {
                for trigger_event in &tension.trigger_events {
                    context.add_tension_event(trigger_event);
                }
            }
        }

        context
    }

    /// Marks notable events as highlights for later summarization.
    fn mark_highlights(
        &self,
        notable_events: &[ScoredEvent],
        _timestamp: sim_events::SimTimestamp,
    ) -> Vec<HighlightMarker> {
        notable_events
            .iter()
            .filter(|se| se.score >= 0.7) // Only high-scoring events become highlights
            .map(|se| {
                // Map event types to appropriate highlight types
                let highlight_type = match se.event.event_type {
                    EventType::Betrayal => HighlightType::TurningPoint,
                    EventType::Death => HighlightType::Climax,
                    EventType::Conflict => HighlightType::KeyMoment,
                    EventType::Faction => HighlightType::TurningPoint,
                    EventType::Ritual => HighlightType::KeyMoment,
                    _ => HighlightType::KeyMoment,
                };

                // Clip window: 50 ticks before to 50 ticks after the event
                let tick = se.event.timestamp.tick;
                let clip_start = tick.saturating_sub(50);
                let clip_end = tick + 50;

                HighlightMarker::new(
                    &se.event.event_id,
                    highlight_type,
                    clip_start,
                    clip_end,
                )
                .with_description(format!(
                    "{:?} event involving {}",
                    se.event.event_type,
                    se.event.actors.primary.name
                ))
            })
            .collect()
    }

    /// Updates tracked agents based on the camera instruction.
    fn update_tracked_agents(&mut self, instruction: &CameraInstruction) {
        // Clear old tracked agents and add new ones from camera focus
        self.tracked_agents.clear();

        for agent_id in instruction.focus.agent_ids() {
            self.tracked_agents.insert(agent_id.to_string());
        }
    }

    /// Returns a reference to the configuration.
    pub fn config(&self) -> &DirectorConfig {
        &self.config
    }

    /// Returns the current tick.
    pub fn current_tick(&self) -> u64 {
        self.current_tick
    }

    /// Returns the number of active threads.
    pub fn active_thread_count(&self) -> usize {
        self.thread_tracker.active().len()
    }

    /// Returns the number of tracked betrayals.
    pub fn tracked_betrayal_count(&self) -> usize {
        self.irony_detector.betrayal_count()
    }

    /// Cleans up old data (betrayals, dormant threads, etc.)
    pub fn cleanup(&mut self, max_betrayal_age_ticks: u64) {
        self.irony_detector.cleanup(self.current_tick, max_betrayal_age_ticks);
    }
}

impl Default for Director {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_events::{
        ActorSet, ActorSnapshot, AffectedActor, BetrayalSubtype, EventContext, EventOutcome,
        EventSubtype, GeneralOutcome, MovementSubtype, Season, SimTimestamp, TensionStatus,
        TensionType, RelationshipSnapshot,
    };

    fn test_timestamp(tick: u64) -> SimTimestamp {
        SimTimestamp::new(tick, 1, Season::Spring, 10)
    }

    fn make_betrayal_event(tick: u64) -> Event {
        let primary = ActorSnapshot::new(
            "agent_mira",
            "Mira",
            "thornwood",
            "scout",
            "eastern_bridge",
        );
        let secondary = ActorSnapshot::new(
            "agent_voss",
            "Voss",
            "ironmere",
            "spymaster",
            "eastern_bridge",
        );
        let mut actors = ActorSet::with_secondary(primary, secondary);
        actors.affected.push(AffectedActor::new("agent_corin", "Corin", "thornwood", "leader"));

        Event {
            event_id: format!("evt_{:05}", tick),
            timestamp: test_timestamp(tick),
            event_type: EventType::Betrayal,
            subtype: EventSubtype::Betrayal(BetrayalSubtype::SecretSharedWithEnemy),
            actors,
            context: EventContext::new("trust_eroded"),
            outcome: EventOutcome::General(GeneralOutcome::default()),
            drama_tags: vec!["betrayal".to_string(), "faction_critical".to_string()],
            drama_score: 0.85,
            connected_events: vec![],
        }
    }

    fn make_movement_event(tick: u64) -> Event {
        let actor = ActorSnapshot::new(
            "agent_mira",
            "Mira",
            "thornwood",
            "scout",
            "village_center",
        );

        Event {
            event_id: format!("evt_{:05}", tick),
            timestamp: test_timestamp(tick),
            event_type: EventType::Movement,
            subtype: EventSubtype::Movement(MovementSubtype::Travel),
            actors: ActorSet::primary_only(actor),
            context: EventContext::new("patrol"),
            outcome: EventOutcome::General(GeneralOutcome::default()),
            drama_tags: vec![],
            drama_score: 0.1,
            connected_events: vec![],
        }
    }

    fn make_tension() -> Tension {
        let mut tension = Tension::new(
            "tens_00001",
            TensionType::BrewingBetrayal,
            1000,
            "Mira's loyalty is wavering",
        );
        tension.severity = 0.7;
        tension.status = TensionStatus::Escalating;
        tension.add_agent_inline("agent_mira", "potential_traitor", "uncertain");
        tension.add_narrative_hook("Something is wrong with Mira");
        tension.add_trigger_event("evt_00999");
        tension
    }

    fn make_world_snapshot(tick: u64) -> WorldSnapshot {
        let mut snapshot = WorldSnapshot::new(
            format!("snap_{:05}", tick),
            test_timestamp(tick),
            "scheduled",
        );

        // Add some agents
        snapshot.agents.push(sim_events::AgentSnapshot::new(
            "agent_mira", "Mira", "thornwood", "scout", "eastern_bridge"
        ));
        snapshot.agents.push(sim_events::AgentSnapshot::new(
            "agent_corin", "Corin", "thornwood", "leader", "thornwood_hall"
        ));
        snapshot.agents.push(sim_events::AgentSnapshot::new(
            "agent_voss", "Voss", "ironmere", "spymaster", "ironmere_keep"
        ));

        // Add relationship where Corin trusts Mira
        let mut corin_rels = std::collections::HashMap::new();
        corin_rels.insert(
            "agent_mira".to_string(),
            RelationshipSnapshot::new(0.8, 0.6, 0.5),
        );
        snapshot.relationships.insert("agent_corin".to_string(), corin_rels);

        snapshot
    }

    #[test]
    fn test_director_creation() {
        let director = Director::with_defaults();
        assert_eq!(director.current_tick(), 0);
        assert_eq!(director.active_thread_count(), 0);
    }

    #[test]
    fn test_director_from_config() {
        let config = DirectorConfig::default();
        let director = Director::new(config).unwrap();
        assert_eq!(director.current_tick(), 0);
    }

    #[test]
    fn test_process_tick_empty() {
        let mut director = Director::with_defaults();
        let state = make_world_snapshot(1000);

        let output = director.process_tick(&[], &[], &state);

        assert_eq!(output.generated_at_tick, 1000);
        assert_eq!(output.camera_script.len(), 1);
        assert!(output.commentary_queue.is_empty());
        assert!(output.highlights.is_empty());
    }

    #[test]
    fn test_process_tick_with_notable_event() {
        let mut director = Director::with_defaults();
        let event = make_betrayal_event(1000);
        let state = make_world_snapshot(1000);

        let output = director.process_tick(&[event], &[], &state);

        assert_eq!(output.generated_at_tick, 1000);
        // High-drama event should generate commentary
        assert!(!output.commentary_queue.is_empty());
        // Betrayal should be a highlight
        assert!(!output.highlights.is_empty());
    }

    #[test]
    fn test_process_tick_filters_low_drama() {
        let mut director = Director::with_defaults();
        let event = make_movement_event(1000);
        let state = make_world_snapshot(1000);

        let output = director.process_tick(&[event], &[], &state);

        // Low-drama event should not generate commentary
        assert!(output.commentary_queue.is_empty());
        assert!(output.highlights.is_empty());
    }

    #[test]
    fn test_process_tick_with_tension() {
        let mut director = Director::with_defaults();
        let tension = make_tension();
        let state = make_world_snapshot(1000);

        let output = director.process_tick(&[], &[tension], &state);

        // Should focus camera based on tension
        assert_eq!(output.camera_script.len(), 1);
        // May generate tension teaser
    }

    #[test]
    fn test_process_tick_creates_threads() {
        let mut director = Director::with_defaults();
        let tension = make_tension();
        let state = make_world_snapshot(1000);

        director.process_tick(&[], &[tension], &state);

        // Should have created a thread for the tension
        assert!(director.active_thread_count() > 0);
    }

    #[test]
    fn test_process_tick_tracks_betrayals() {
        let mut director = Director::with_defaults();
        let event = make_betrayal_event(1000);
        let state = make_world_snapshot(1000);

        director.process_tick(&[event], &[], &state);

        // Should have tracked the betrayal
        assert_eq!(director.tracked_betrayal_count(), 1);
    }

    #[test]
    fn test_process_tick_detects_irony() {
        let mut director = Director::with_defaults();
        let event = make_betrayal_event(1000);
        let state = make_world_snapshot(1000);

        let output = director.process_tick(&[event], &[], &state);

        // Corin still trusts Mira, so there should be dramatic irony
        let irony_items: Vec<_> = output
            .commentary_queue
            .iter()
            .filter(|c| c.commentary_type == CommentaryType::DramaticIrony)
            .collect();
        assert!(!irony_items.is_empty());
    }

    #[test]
    fn test_process_multiple_ticks() {
        let mut director = Director::with_defaults();

        // Tick 1000: Betrayal
        let event1 = make_betrayal_event(1000);
        let state1 = make_world_snapshot(1000);
        let output1 = director.process_tick(&[event1], &[], &state1);
        assert_eq!(output1.generated_at_tick, 1000);

        // Tick 1001: Movement
        let event2 = make_movement_event(1001);
        let state2 = make_world_snapshot(1001);
        let output2 = director.process_tick(&[event2], &[], &state2);
        assert_eq!(output2.generated_at_tick, 1001);

        // Director should remember the betrayal
        assert_eq!(director.tracked_betrayal_count(), 1);
    }

    #[test]
    fn test_director_cleanup() {
        let mut director = Director::with_defaults();
        let event = make_betrayal_event(1000);
        let state = make_world_snapshot(1000);

        director.process_tick(&[event], &[], &state);
        assert_eq!(director.tracked_betrayal_count(), 1);

        // Simulate time passing
        director.current_tick = 100000;
        director.cleanup(50000);

        // Old betrayal should be cleaned up
        assert_eq!(director.tracked_betrayal_count(), 0);
    }

    #[test]
    fn test_commentary_queue_limited() {
        let mut director = Director::new(DirectorConfig {
            commentary: CommentaryConfig {
                max_queue_size: 2,
                ..CommentaryConfig::default()
            },
            ..DirectorConfig::default()
        }).unwrap();

        // Create multiple high-drama events
        let events: Vec<Event> = (0..5)
            .map(|i| make_betrayal_event(1000 + i))
            .collect();
        let state = make_world_snapshot(1004);

        let output = director.process_tick(&events, &[], &state);

        // Queue should be limited to max_queue_size
        assert!(output.commentary_queue.len() <= 2);
    }

    #[test]
    fn test_highlights_for_high_drama() {
        let mut director = Director::with_defaults();
        let betrayal = make_betrayal_event(1000);
        let movement = make_movement_event(1001);
        let state = make_world_snapshot(1001);

        let output = director.process_tick(&[betrayal, movement], &[], &state);

        // Only betrayal should be highlighted (high drama)
        // Betrayal events map to TurningPoint highlight type
        assert!(!output.highlights.is_empty());
        assert!(output.highlights.iter().any(|h| h.highlight_type == HighlightType::TurningPoint));
    }

    #[test]
    fn test_build_context() {
        let director = Director::with_defaults();
        let tensions = vec![make_tension()];

        let context = director.build_context(&tensions);

        // Context should have tension events
        assert!(context.is_tension_event("evt_00999"));
    }
}
