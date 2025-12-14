//! Narrative thread tracking.
//!
//! Tracks ongoing storylines for continuity and focus selection.

use serde::{Deserialize, Serialize};
use sim_events::{Event, Tension, TensionStatus};
use std::collections::HashMap;

/// Status of a narrative thread's lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ThreadStatus {
    /// Building toward something
    #[default]
    Developing,
    /// Key moment happening now
    Climaxing,
    /// Aftermath playing out
    Resolving,
    /// Nothing happening, might reactivate
    Dormant,
    /// Story finished
    Concluded,
}

impl ThreadStatus {
    /// Converts from TensionStatus to ThreadStatus.
    pub fn from_tension_status(status: TensionStatus) -> Self {
        match status {
            TensionStatus::Emerging | TensionStatus::Escalating => ThreadStatus::Developing,
            TensionStatus::Critical | TensionStatus::Climax => ThreadStatus::Climaxing,
            TensionStatus::Resolving => ThreadStatus::Resolving,
            TensionStatus::Resolved => ThreadStatus::Concluded,
            TensionStatus::Dormant => ThreadStatus::Dormant,
        }
    }
}

/// A narrative thread tracking an ongoing storyline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeThread {
    /// Unique thread identifier
    pub thread_id: String,
    /// Tick when thread was created
    pub created_at_tick: u64,
    /// Tick of last activity
    pub last_updated_tick: u64,
    /// Current status
    pub status: ThreadStatus,
    /// Related tension IDs
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tension_ids: Vec<String>,
    /// Key agents in this thread
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub key_agents: Vec<String>,
    /// Key event IDs in this thread
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub key_events: Vec<String>,
    /// Type of narrative thread (e.g., "betrayal_arc", "succession")
    pub thread_type: String,
    /// Human-readable summary
    pub summary: String,
    /// One-line teaser hook
    #[serde(default)]
    pub hook: String,
    /// Total ticks this thread has been shown
    pub screen_time_ticks: u64,
    /// Last tick when this thread was actively shown
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_shown_tick: Option<u64>,
}

impl NarrativeThread {
    /// Creates a new narrative thread.
    pub fn new(
        thread_id: impl Into<String>,
        thread_type: impl Into<String>,
        summary: impl Into<String>,
        created_at_tick: u64,
    ) -> Self {
        Self {
            thread_id: thread_id.into(),
            created_at_tick,
            last_updated_tick: created_at_tick,
            status: ThreadStatus::Developing,
            tension_ids: Vec::new(),
            key_agents: Vec::new(),
            key_events: Vec::new(),
            thread_type: thread_type.into(),
            summary: summary.into(),
            hook: String::new(),
            screen_time_ticks: 0,
            last_shown_tick: None,
        }
    }

    /// Creates a thread from a tension.
    pub fn from_tension(tension: &Tension, thread_id: impl Into<String>) -> Self {
        let mut thread = Self::new(
            thread_id,
            format!("{:?}", tension.tension_type).to_lowercase(),
            &tension.summary,
            tension.detected_at_tick,
        );
        thread.add_tension(&tension.tension_id);
        for agent in &tension.key_agents {
            thread.add_agent(&agent.agent_id);
        }
        if !tension.narrative_hooks.is_empty() {
            thread.hook = tension.narrative_hooks[0].clone();
        }
        thread.status = ThreadStatus::from_tension_status(tension.status);
        thread
    }

    /// Adds a tension to this thread.
    pub fn add_tension(&mut self, tension_id: impl Into<String>) {
        let id = tension_id.into();
        if !self.tension_ids.contains(&id) {
            self.tension_ids.push(id);
        }
    }

    /// Adds a key agent to this thread.
    pub fn add_agent(&mut self, agent_id: impl Into<String>) {
        let id = agent_id.into();
        if !self.key_agents.contains(&id) {
            self.key_agents.push(id);
        }
    }

    /// Adds a key event to this thread.
    pub fn add_event(&mut self, event_id: impl Into<String>) {
        self.key_events.push(event_id.into());
    }

    /// Sets the hook.
    pub fn with_hook(mut self, hook: impl Into<String>) -> Self {
        self.hook = hook.into();
        self
    }

    /// Records screen time for this thread.
    pub fn record_screen_time(&mut self, ticks: u64, current_tick: u64) {
        self.screen_time_ticks += ticks;
        self.last_shown_tick = Some(current_tick);
    }

    /// Updates the thread's last activity tick.
    pub fn touch(&mut self, current_tick: u64) {
        self.last_updated_tick = current_tick;
    }

    /// Checks if this thread is active (not dormant or concluded).
    pub fn is_active(&self) -> bool {
        !matches!(self.status, ThreadStatus::Dormant | ThreadStatus::Concluded)
    }

    /// Checks if this thread involves a specific tension.
    pub fn involves_tension(&self, tension_id: &str) -> bool {
        self.tension_ids.iter().any(|id| id == tension_id)
    }

    /// Checks if this thread involves a specific agent.
    pub fn involves_agent(&self, agent_id: &str) -> bool {
        self.key_agents.iter().any(|id| id == agent_id)
    }

    /// Updates thread status based on tension status.
    pub fn update_from_tension(&mut self, tension: &Tension, current_tick: u64) {
        self.status = ThreadStatus::from_tension_status(tension.status);
        self.last_updated_tick = current_tick;

        // Add any new agents from the tension
        for agent in &tension.key_agents {
            self.add_agent(&agent.agent_id);
        }
    }
}

/// An event with its computed score.
#[derive(Debug, Clone)]
pub struct ScoredEvent<'a> {
    /// Reference to the original event
    pub event: &'a Event,
    /// Computed drama/importance score
    pub score: f32,
}

impl<'a> ScoredEvent<'a> {
    /// Creates a new scored event.
    pub fn new(event: &'a Event, score: f32) -> Self {
        Self { event, score }
    }
}

/// Configuration for the thread tracker.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThreadTrackerConfig {
    /// Minimum tension severity to create a thread
    pub min_severity_for_thread: f32,
    /// Ticks of inactivity before marking thread dormant
    pub dormant_threshold_ticks: u64,
    /// Maximum number of threads to track
    pub max_threads: usize,
}

impl Default for ThreadTrackerConfig {
    fn default() -> Self {
        Self {
            min_severity_for_thread: 0.3,
            dormant_threshold_ticks: 5000,
            max_threads: 20,
        }
    }
}

/// Manages a collection of narrative threads.
#[derive(Debug, Clone)]
pub struct ThreadTracker {
    /// Active threads indexed by thread ID
    threads: HashMap<String, NarrativeThread>,
    /// Maps tension IDs to thread IDs
    tension_to_thread: HashMap<String, String>,
    /// Configuration
    config: ThreadTrackerConfig,
    /// Next thread sequence number
    next_sequence: u64,
    /// Current tick for tracking
    current_tick: u64,
}

impl ThreadTracker {
    /// Creates a new thread tracker with default config.
    pub fn new() -> Self {
        Self::with_config(ThreadTrackerConfig::default())
    }

    /// Creates a new thread tracker with the given config.
    pub fn with_config(config: ThreadTrackerConfig) -> Self {
        Self {
            threads: HashMap::new(),
            tension_to_thread: HashMap::new(),
            config,
            next_sequence: 1,
            current_tick: 0,
        }
    }

    /// Updates threads based on new events and tensions.
    ///
    /// This method:
    /// - Creates new threads for new tensions above severity threshold
    /// - Updates existing threads with new events
    /// - Transitions thread status based on tension status
    /// - Marks threads dormant if no activity for N ticks
    pub fn update(&mut self, events: &[ScoredEvent], tensions: &[Tension]) {
        // Update current tick from tensions
        if let Some(tension) = tensions.first() {
            self.current_tick = tension.last_updated_tick;
        }

        // Process tensions - create or update threads
        for tension in tensions {
            if tension.severity >= self.config.min_severity_for_thread {
                self.process_tension(tension);
            }
        }

        // Add events to relevant threads
        for scored in events {
            self.process_event(scored);
        }

        // Mark dormant threads based on inactivity
        self.check_dormant_threads();

        // Prune if over max
        self.prune_old_threads();
    }

    /// Processes a tension, creating or updating the corresponding thread.
    fn process_tension(&mut self, tension: &Tension) {
        if let Some(thread_id) = self.tension_to_thread.get(&tension.tension_id) {
            // Update existing thread
            if let Some(thread) = self.threads.get_mut(thread_id) {
                thread.update_from_tension(tension, self.current_tick);
            }
        } else if self.threads.len() < self.config.max_threads {
            // Create new thread for this tension
            let thread_id = generate_thread_id(self.next_sequence);
            self.next_sequence += 1;

            let thread = NarrativeThread::from_tension(tension, &thread_id);
            self.tension_to_thread
                .insert(tension.tension_id.clone(), thread_id.clone());
            self.threads.insert(thread_id, thread);
        }
    }

    /// Processes an event, adding it to relevant threads.
    fn process_event(&mut self, scored: &ScoredEvent) {
        let event = scored.event;

        // Add event to threads that involve any of the event's agents
        for thread in self.threads.values_mut() {
            let involves_thread_agent = event
                .all_agent_ids()
                .iter()
                .any(|id| thread.involves_agent(id));

            if involves_thread_agent {
                thread.add_event(&event.event_id);
                thread.touch(event.timestamp.tick);

                // Add any new agents from the event
                for agent_id in event.all_agent_ids() {
                    thread.add_agent(agent_id);
                }
            }
        }
    }

    /// Checks and marks dormant threads.
    fn check_dormant_threads(&mut self) {
        for thread in self.threads.values_mut() {
            if thread.is_active() {
                let ticks_since_update = self.current_tick.saturating_sub(thread.last_updated_tick);
                if ticks_since_update > self.config.dormant_threshold_ticks {
                    thread.status = ThreadStatus::Dormant;
                }
            }
        }
    }

    /// Removes old concluded threads if over the limit.
    fn prune_old_threads(&mut self) {
        if self.threads.len() <= self.config.max_threads {
            return;
        }

        // Remove concluded threads first
        let concluded: Vec<_> = self
            .threads
            .iter()
            .filter(|(_, t)| t.status == ThreadStatus::Concluded)
            .map(|(id, _)| id.clone())
            .collect();

        for id in concluded {
            if self.threads.len() <= self.config.max_threads {
                break;
            }
            self.remove_thread(&id);
        }
    }

    /// Removes a thread and its tension mappings.
    fn remove_thread(&mut self, thread_id: &str) {
        if let Some(thread) = self.threads.remove(thread_id) {
            for tension_id in &thread.tension_ids {
                self.tension_to_thread.remove(tension_id);
            }
        }
    }

    /// Returns all active threads.
    pub fn active(&self) -> Vec<&NarrativeThread> {
        self.threads
            .values()
            .filter(|t| t.is_active())
            .collect()
    }

    /// Returns all threads (including dormant and concluded).
    pub fn all(&self) -> Vec<&NarrativeThread> {
        self.threads.values().collect()
    }

    /// Gets the thread for a specific tension.
    pub fn get_thread_for_tension(&self, tension_id: &str) -> Option<&NarrativeThread> {
        self.tension_to_thread
            .get(tension_id)
            .and_then(|id| self.threads.get(id))
    }

    /// Gets a thread by ID.
    pub fn get_thread(&self, thread_id: &str) -> Option<&NarrativeThread> {
        self.threads.get(thread_id)
    }

    /// Gets a mutable thread by ID.
    pub fn get_thread_mut(&mut self, thread_id: &str) -> Option<&mut NarrativeThread> {
        self.threads.get_mut(thread_id)
    }

    /// Records screen time for a thread.
    pub fn record_screen_time(&mut self, thread_id: &str, ticks: u64) {
        if let Some(thread) = self.threads.get_mut(thread_id) {
            thread.record_screen_time(ticks, self.current_tick);
        }
    }

    /// Marks a thread as concluded.
    pub fn mark_concluded(&mut self, thread_id: &str) {
        if let Some(thread) = self.threads.get_mut(thread_id) {
            thread.status = ThreadStatus::Concluded;
        }
    }

    /// Returns the number of threads.
    pub fn len(&self) -> usize {
        self.threads.len()
    }

    /// Returns true if there are no threads.
    pub fn is_empty(&self) -> bool {
        self.threads.is_empty()
    }

    /// Sets the current tick.
    pub fn set_current_tick(&mut self, tick: u64) {
        self.current_tick = tick;
    }
}

impl Default for ThreadTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Generates a thread ID.
pub fn generate_thread_id(sequence: u64) -> String {
    format!("thread_{:05}", sequence)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_events::{
        ActorSet, ActorSnapshot, EventContext, EventOutcome, EventType, GeneralOutcome,
        MovementSubtype, Season, SimTimestamp, TensionType, EventSubtype,
    };

    fn make_test_tension(id: &str, severity: f32, status: TensionStatus) -> Tension {
        let mut tension = Tension::new(id, TensionType::BrewingBetrayal, 1000, "Test tension");
        tension.severity = severity;
        tension.status = status;
        tension.add_agent_inline("agent_mira", "betrayer", "uncertain");
        tension.add_narrative_hook("Trouble is brewing");
        tension
    }

    fn make_test_event(id: &str, tick: u64, agent_id: &str) -> Event {
        let actor = ActorSnapshot::new(agent_id, "Test", "faction", "role", "loc");
        Event {
            event_id: id.to_string(),
            timestamp: SimTimestamp::new(tick, 1, Season::Spring, 10),
            event_type: EventType::Movement,
            subtype: EventSubtype::Movement(MovementSubtype::Travel),
            actors: ActorSet::primary_only(actor),
            context: EventContext::new("test"),
            outcome: EventOutcome::General(GeneralOutcome::default()),
            drama_tags: vec![],
            drama_score: 0.1,
            connected_events: vec![],
        }
    }

    #[test]
    fn test_thread_status_serialization() {
        assert_eq!(
            serde_json::to_string(&ThreadStatus::Developing).unwrap(),
            r#""developing""#
        );
        assert_eq!(
            serde_json::to_string(&ThreadStatus::Climaxing).unwrap(),
            r#""climaxing""#
        );
        assert_eq!(
            serde_json::to_string(&ThreadStatus::Concluded).unwrap(),
            r#""concluded""#
        );
    }

    #[test]
    fn test_thread_status_from_tension() {
        assert_eq!(
            ThreadStatus::from_tension_status(TensionStatus::Emerging),
            ThreadStatus::Developing
        );
        assert_eq!(
            ThreadStatus::from_tension_status(TensionStatus::Climax),
            ThreadStatus::Climaxing
        );
        assert_eq!(
            ThreadStatus::from_tension_status(TensionStatus::Resolved),
            ThreadStatus::Concluded
        );
    }

    #[test]
    fn test_narrative_thread_creation() {
        let thread = NarrativeThread::new(
            "thread_00001",
            "betrayal_arc",
            "Mira considers defecting to Ironmere",
            1000,
        )
        .with_hook("A spy among us");

        assert_eq!(thread.thread_id, "thread_00001");
        assert_eq!(thread.thread_type, "betrayal_arc");
        assert_eq!(thread.status, ThreadStatus::Developing);
        assert_eq!(thread.hook, "A spy among us");
        assert!(thread.is_active());
    }

    #[test]
    fn test_narrative_thread_from_tension() {
        let tension = make_test_tension("tens_00001", 0.8, TensionStatus::Escalating);
        let thread = NarrativeThread::from_tension(&tension, "thread_00001");

        assert_eq!(thread.tension_ids, vec!["tens_00001"]);
        assert!(thread.involves_agent("agent_mira"));
        assert_eq!(thread.hook, "Trouble is brewing");
        assert_eq!(thread.status, ThreadStatus::Developing);
    }

    #[test]
    fn test_thread_add_tension() {
        let mut thread = NarrativeThread::new("thread_00001", "conflict", "Test", 1000);
        thread.add_tension("tens_00001");
        thread.add_tension("tens_00002");
        thread.add_tension("tens_00001"); // Duplicate

        assert_eq!(thread.tension_ids.len(), 2);
        assert!(thread.involves_tension("tens_00001"));
        assert!(thread.involves_tension("tens_00002"));
        assert!(!thread.involves_tension("tens_00003"));
    }

    #[test]
    fn test_thread_add_agent() {
        let mut thread = NarrativeThread::new("thread_00001", "conflict", "Test", 1000);
        thread.add_agent("agent_mira");
        thread.add_agent("agent_corin");
        thread.add_agent("agent_mira"); // Duplicate

        assert_eq!(thread.key_agents.len(), 2);
        assert!(thread.involves_agent("agent_mira"));
    }

    #[test]
    fn test_thread_screen_time() {
        let mut thread = NarrativeThread::new("thread_00001", "conflict", "Test", 1000);
        thread.record_screen_time(100, 1100);
        thread.record_screen_time(50, 1200);

        assert_eq!(thread.screen_time_ticks, 150);
        assert_eq!(thread.last_shown_tick, Some(1200));
    }

    #[test]
    fn test_thread_is_active() {
        let mut thread = NarrativeThread::new("thread_00001", "conflict", "Test", 1000);
        assert!(thread.is_active());

        thread.status = ThreadStatus::Climaxing;
        assert!(thread.is_active());

        thread.status = ThreadStatus::Dormant;
        assert!(!thread.is_active());

        thread.status = ThreadStatus::Concluded;
        assert!(!thread.is_active());
    }

    #[test]
    fn test_generate_thread_id() {
        assert_eq!(generate_thread_id(1), "thread_00001");
        assert_eq!(generate_thread_id(12345), "thread_12345");
    }

    #[test]
    fn test_thread_serialization() {
        let mut thread = NarrativeThread::new(
            "thread_00001",
            "betrayal_arc",
            "Mira's loyalty is tested",
            1000,
        );
        thread.add_tension("tens_00001");
        thread.add_agent("agent_mira");
        thread.add_event("evt_00042");

        let json = serde_json::to_string(&thread).unwrap();
        assert!(json.contains("betrayal_arc"));
        assert!(json.contains("agent_mira"));

        let parsed: NarrativeThread = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.thread_id, "thread_00001");
        assert_eq!(parsed.tension_ids.len(), 1);
    }

    #[test]
    fn test_thread_tracker_new() {
        let tracker = ThreadTracker::new();
        assert!(tracker.is_empty());
        assert_eq!(tracker.len(), 0);
    }

    #[test]
    fn test_thread_tracker_creates_thread_for_tension() {
        let mut tracker = ThreadTracker::new();
        let tension = make_test_tension("tens_00001", 0.5, TensionStatus::Escalating);

        tracker.update(&[], &[tension]);

        assert_eq!(tracker.len(), 1);
        let thread = tracker.get_thread_for_tension("tens_00001");
        assert!(thread.is_some());
        assert!(thread.unwrap().involves_tension("tens_00001"));
    }

    #[test]
    fn test_thread_tracker_ignores_low_severity() {
        let mut tracker = ThreadTracker::new();
        let tension = make_test_tension("tens_00001", 0.1, TensionStatus::Emerging);

        tracker.update(&[], &[tension]);

        assert!(tracker.is_empty());
    }

    #[test]
    fn test_thread_tracker_updates_thread_status() {
        let mut tracker = ThreadTracker::new();
        let tension1 = make_test_tension("tens_00001", 0.5, TensionStatus::Escalating);
        tracker.update(&[], &[tension1]);

        let mut tension2 = make_test_tension("tens_00001", 0.9, TensionStatus::Climax);
        tension2.last_updated_tick = 2000;
        tracker.update(&[], &[tension2]);

        let thread = tracker.get_thread_for_tension("tens_00001").unwrap();
        assert_eq!(thread.status, ThreadStatus::Climaxing);
    }

    #[test]
    fn test_thread_tracker_adds_events() {
        let mut tracker = ThreadTracker::new();
        let tension = make_test_tension("tens_00001", 0.5, TensionStatus::Escalating);
        tracker.update(&[], &[tension]);

        let event = make_test_event("evt_00001", 1500, "agent_mira");
        let scored = vec![ScoredEvent::new(&event, 0.5)];
        tracker.update(&scored, &[]);

        let thread = tracker.get_thread_for_tension("tens_00001").unwrap();
        assert!(thread.key_events.contains(&"evt_00001".to_string()));
    }

    #[test]
    fn test_thread_tracker_active_threads() {
        let mut tracker = ThreadTracker::new();

        let t1 = make_test_tension("tens_00001", 0.5, TensionStatus::Escalating);
        let t2 = make_test_tension("tens_00002", 0.6, TensionStatus::Resolved);

        tracker.update(&[], &[t1, t2]);

        let active = tracker.active();
        assert_eq!(active.len(), 1);
    }

    #[test]
    fn test_thread_tracker_record_screen_time() {
        let mut tracker = ThreadTracker::new();
        let tension = make_test_tension("tens_00001", 0.5, TensionStatus::Escalating);
        tracker.update(&[], &[tension]);

        let thread_id = tracker
            .get_thread_for_tension("tens_00001")
            .unwrap()
            .thread_id
            .clone();
        tracker.record_screen_time(&thread_id, 100);

        let thread = tracker.get_thread(&thread_id).unwrap();
        assert_eq!(thread.screen_time_ticks, 100);
    }

    #[test]
    fn test_thread_tracker_mark_concluded() {
        let mut tracker = ThreadTracker::new();
        let tension = make_test_tension("tens_00001", 0.5, TensionStatus::Escalating);
        tracker.update(&[], &[tension]);

        let thread_id = tracker
            .get_thread_for_tension("tens_00001")
            .unwrap()
            .thread_id
            .clone();
        tracker.mark_concluded(&thread_id);

        let thread = tracker.get_thread(&thread_id).unwrap();
        assert_eq!(thread.status, ThreadStatus::Concluded);
        assert!(!thread.is_active());
    }

    #[test]
    fn test_thread_tracker_dormant_after_inactivity() {
        let mut tracker = ThreadTracker::with_config(ThreadTrackerConfig {
            min_severity_for_thread: 0.3,
            dormant_threshold_ticks: 100,
            max_threads: 20,
        });

        let mut tension = make_test_tension("tens_00001", 0.5, TensionStatus::Escalating);
        tension.last_updated_tick = 1000;
        tracker.set_current_tick(1000);
        tracker.update(&[], &[tension]);

        // Thread should be active
        let thread = tracker.get_thread_for_tension("tens_00001").unwrap();
        assert!(thread.is_active());

        // Advance time past threshold
        tracker.set_current_tick(2000);
        tracker.update(&[], &[]);

        let thread = tracker.get_thread_for_tension("tens_00001").unwrap();
        assert_eq!(thread.status, ThreadStatus::Dormant);
    }

    #[test]
    fn test_thread_tracker_max_threads() {
        let mut tracker = ThreadTracker::with_config(ThreadTrackerConfig {
            min_severity_for_thread: 0.3,
            dormant_threshold_ticks: 5000,
            max_threads: 2,
        });

        let t1 = make_test_tension("tens_00001", 0.5, TensionStatus::Escalating);
        let t2 = make_test_tension("tens_00002", 0.6, TensionStatus::Escalating);
        let t3 = make_test_tension("tens_00003", 0.7, TensionStatus::Escalating);

        tracker.update(&[], &[t1, t2, t3]);

        // Should only have 2 threads (max)
        assert_eq!(tracker.len(), 2);
    }
}
