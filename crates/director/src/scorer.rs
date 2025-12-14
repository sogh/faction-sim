//! Event scoring with configurable weights.
//!
//! Scores events by dramatic interest for prioritization.

use serde::{Deserialize, Serialize};
use sim_events::{Event, EventType};
use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::threads::ScoredEvent;

/// Weights for scoring events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventWeights {
    /// Base scores by event type name
    #[serde(default)]
    pub base_scores: HashMap<String, f32>,
    /// Multipliers for specific subtypes
    #[serde(default)]
    pub subtype_modifiers: HashMap<String, f32>,
    /// Additive scores for drama tags
    #[serde(default)]
    pub drama_tag_scores: HashMap<String, f32>,
}

impl Default for EventWeights {
    fn default() -> Self {
        let mut base_scores = HashMap::new();
        base_scores.insert("betrayal".to_string(), 0.9);
        base_scores.insert("death".to_string(), 0.85);
        base_scores.insert("conflict".to_string(), 0.7);
        base_scores.insert("faction".to_string(), 0.6);
        base_scores.insert("ritual".to_string(), 0.5);
        base_scores.insert("cooperation".to_string(), 0.4);
        base_scores.insert("communication".to_string(), 0.3);
        base_scores.insert("resource".to_string(), 0.25);
        base_scores.insert("archive".to_string(), 0.2);
        base_scores.insert("loyalty".to_string(), 0.35);
        base_scores.insert("birth".to_string(), 0.3);
        base_scores.insert("movement".to_string(), 0.1);

        let mut drama_tag_scores = HashMap::new();
        drama_tag_scores.insert("faction_critical".to_string(), 0.3);
        drama_tag_scores.insert("secret_meeting".to_string(), 0.25);
        drama_tag_scores.insert("leader_involved".to_string(), 0.2);
        drama_tag_scores.insert("cross_faction".to_string(), 0.15);
        drama_tag_scores.insert("winter_crisis".to_string(), 0.1);
        drama_tag_scores.insert("betrayal".to_string(), 0.15);
        drama_tag_scores.insert("revenge".to_string(), 0.15);
        drama_tag_scores.insert("power_struggle".to_string(), 0.15);
        drama_tag_scores.insert("death".to_string(), 0.1);

        Self {
            base_scores,
            subtype_modifiers: HashMap::new(),
            drama_tag_scores,
        }
    }
}

impl EventWeights {
    /// Gets the base score for an event type.
    pub fn base_score(&self, event_type: &EventType) -> f32 {
        let key = event_type_to_string(event_type);
        self.base_scores.get(&key).copied().unwrap_or(0.1)
    }

    /// Gets the subtype modifier if present.
    pub fn subtype_modifier(&self, subtype: &str) -> Option<f32> {
        self.subtype_modifiers.get(subtype).copied()
    }

    /// Gets the drama tag score if present.
    pub fn drama_tag_score(&self, tag: &str) -> f32 {
        self.drama_tag_scores.get(tag).copied().unwrap_or(0.0)
    }
}

/// Context for scoring events relative to current director state.
#[derive(Debug, Clone, Default)]
pub struct DirectorContext {
    /// Agents currently being tracked (e.g., following with camera)
    pub tracked_agents: HashSet<String>,
    /// Event IDs that are part of active tensions
    pub active_tension_events: HashSet<String>,
    /// Current agent being followed (if any)
    pub current_focus: Option<String>,
}

impl DirectorContext {
    /// Creates a new empty context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an agent to the tracked set.
    pub fn track_agent(&mut self, agent_id: impl Into<String>) {
        self.tracked_agents.insert(agent_id.into());
    }

    /// Adds multiple agents to the tracked set.
    pub fn track_agents(&mut self, agent_ids: impl IntoIterator<Item = impl Into<String>>) {
        for id in agent_ids {
            self.tracked_agents.insert(id.into());
        }
    }

    /// Adds an event ID to the active tension events.
    pub fn add_tension_event(&mut self, event_id: impl Into<String>) {
        self.active_tension_events.insert(event_id.into());
    }

    /// Sets the current focus agent.
    pub fn set_focus(&mut self, agent_id: impl Into<String>) {
        self.current_focus = Some(agent_id.into());
    }

    /// Checks if an agent is being tracked.
    pub fn is_tracked(&self, agent_id: &str) -> bool {
        self.tracked_agents.contains(agent_id)
    }

    /// Checks if an event is part of an active tension.
    pub fn is_tension_event(&self, event_id: &str) -> bool {
        self.active_tension_events.contains(event_id)
    }
}

/// Scores events for dramatic interest.
#[derive(Debug, Clone)]
pub struct EventScorer {
    /// Scoring weights
    weights: EventWeights,
    /// Boost multiplier for tracked agents
    tracked_agent_boost: f32,
    /// Boost multiplier for tension-related events
    tension_event_boost: f32,
}

impl EventScorer {
    /// Creates a new scorer with the given weights.
    pub fn new(weights: EventWeights) -> Self {
        Self {
            weights,
            tracked_agent_boost: 1.5,
            tension_event_boost: 2.0,
        }
    }

    /// Loads weights from a TOML config file.
    pub fn from_config(path: &Path) -> Result<Self, ScorerError> {
        let content = std::fs::read_to_string(path).map_err(ScorerError::IoError)?;
        let weights: EventWeights = toml::from_str(&content).map_err(ScorerError::TomlError)?;
        Ok(Self::new(weights))
    }

    /// Sets the tracked agent boost multiplier.
    pub fn with_tracked_boost(mut self, boost: f32) -> Self {
        self.tracked_agent_boost = boost;
        self
    }

    /// Sets the tension event boost multiplier.
    pub fn with_tension_boost(mut self, boost: f32) -> Self {
        self.tension_event_boost = boost;
        self
    }

    /// Scores a single event.
    pub fn score(&self, event: &Event, context: &DirectorContext) -> f32 {
        // Start with base score for event type
        let mut score = self.weights.base_score(&event.event_type);

        // Apply subtype modifier if present
        let subtype_str = subtype_to_string(&event.subtype);
        if let Some(modifier) = self.weights.subtype_modifier(&subtype_str) {
            score *= modifier;
        }

        // Add drama tag scores (additive)
        for tag in &event.drama_tags {
            score += self.weights.drama_tag_score(tag);
        }

        // Boost if involves tracked agents
        let involves_tracked = event
            .all_agent_ids()
            .iter()
            .any(|id| context.is_tracked(id));
        if involves_tracked {
            score *= self.tracked_agent_boost;
        }

        // Boost if event is part of active tension
        if context.is_tension_event(&event.event_id) {
            score *= self.tension_event_boost;
        }

        // Cap at 1.0 but allow natural scores to accumulate
        score.min(1.5)
    }

    /// Scores a batch of events.
    pub fn score_batch<'a>(
        &self,
        events: &'a [Event],
        context: &DirectorContext,
    ) -> Vec<ScoredEvent<'a>> {
        events
            .iter()
            .map(|e| ScoredEvent::new(e, self.score(e, context)))
            .collect()
    }

    /// Returns a reference to the weights.
    pub fn weights(&self) -> &EventWeights {
        &self.weights
    }
}

impl Default for EventScorer {
    fn default() -> Self {
        Self::new(EventWeights::default())
    }
}

/// Errors that can occur during scoring.
#[derive(Debug)]
pub enum ScorerError {
    /// IO error reading config file
    IoError(std::io::Error),
    /// Error parsing TOML config
    TomlError(toml::de::Error),
}

impl std::fmt::Display for ScorerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScorerError::IoError(e) => write!(f, "IO error: {}", e),
            ScorerError::TomlError(e) => write!(f, "TOML parse error: {}", e),
        }
    }
}

impl std::error::Error for ScorerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ScorerError::IoError(e) => Some(e),
            ScorerError::TomlError(e) => Some(e),
        }
    }
}

/// Converts an EventType to its lowercase string representation.
fn event_type_to_string(event_type: &EventType) -> String {
    match event_type {
        EventType::Movement => "movement",
        EventType::Communication => "communication",
        EventType::Betrayal => "betrayal",
        EventType::Loyalty => "loyalty",
        EventType::Conflict => "conflict",
        EventType::Cooperation => "cooperation",
        EventType::Faction => "faction",
        EventType::Archive => "archive",
        EventType::Ritual => "ritual",
        EventType::Resource => "resource",
        EventType::Death => "death",
        EventType::Birth => "birth",
    }
    .to_string()
}

/// Converts an EventSubtype to its string representation.
fn subtype_to_string(subtype: &sim_events::EventSubtype) -> String {
    use sim_events::EventSubtype::*;
    match subtype {
        Movement(s) => format!("{:?}", s).to_lowercase(),
        Communication(s) => format!("{:?}", s).to_lowercase(),
        Betrayal(s) => format!("{:?}", s).to_lowercase(),
        Loyalty(s) => format!("{:?}", s).to_lowercase(),
        Conflict(s) => format!("{:?}", s).to_lowercase(),
        Cooperation(s) => format!("{:?}", s).to_lowercase(),
        Faction(s) => format!("{:?}", s).to_lowercase(),
        Archive(s) => format!("{:?}", s).to_lowercase(),
        Ritual(s) => format!("{:?}", s).to_lowercase(),
        Resource(s) => format!("{:?}", s).to_lowercase(),
        Death(s) => format!("{:?}", s).to_lowercase(),
        Birth(s) => format!("{:?}", s).to_lowercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_events::{
        ActorSet, ActorSnapshot, EventContext, EventOutcome, GeneralOutcome, MovementSubtype,
        BetrayalSubtype, EventSubtype, Season, SimTimestamp,
    };

    fn make_movement_event(id: &str, agent_id: &str) -> Event {
        let actor = ActorSnapshot::new(agent_id, "Test", "faction", "scout", "loc");
        Event {
            event_id: id.to_string(),
            timestamp: SimTimestamp::new(1000, 1, Season::Spring, 10),
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

    fn make_betrayal_event(id: &str, agent_id: &str) -> Event {
        let actor = ActorSnapshot::new(agent_id, "Betrayer", "faction", "spy", "loc");
        Event {
            event_id: id.to_string(),
            timestamp: SimTimestamp::new(1000, 1, Season::Spring, 10),
            event_type: EventType::Betrayal,
            subtype: EventSubtype::Betrayal(BetrayalSubtype::SecretSharedWithEnemy),
            actors: ActorSet::primary_only(actor),
            context: EventContext::new("trust_broken"),
            outcome: EventOutcome::General(GeneralOutcome::default()),
            drama_tags: vec![
                "betrayal".to_string(),
                "faction_critical".to_string(),
                "secret_meeting".to_string(),
            ],
            drama_score: 0.87,
            connected_events: vec![],
        }
    }

    #[test]
    fn test_event_weights_default() {
        let weights = EventWeights::default();

        assert_eq!(weights.base_score(&EventType::Betrayal), 0.9);
        assert_eq!(weights.base_score(&EventType::Death), 0.85);
        assert_eq!(weights.base_score(&EventType::Movement), 0.1);
    }

    #[test]
    fn test_event_weights_drama_tags() {
        let weights = EventWeights::default();

        assert_eq!(weights.drama_tag_score("faction_critical"), 0.3);
        assert_eq!(weights.drama_tag_score("secret_meeting"), 0.25);
        assert_eq!(weights.drama_tag_score("unknown_tag"), 0.0);
    }

    #[test]
    fn test_director_context() {
        let mut context = DirectorContext::new();
        context.track_agent("agent_mira");
        context.add_tension_event("evt_00042");
        context.set_focus("agent_mira");

        assert!(context.is_tracked("agent_mira"));
        assert!(!context.is_tracked("agent_corin"));
        assert!(context.is_tension_event("evt_00042"));
        assert_eq!(context.current_focus, Some("agent_mira".to_string()));
    }

    #[test]
    fn test_scorer_betrayal_higher_than_movement() {
        let scorer = EventScorer::default();
        let context = DirectorContext::new();

        let movement = make_movement_event("evt_1", "agent_1");
        let betrayal = make_betrayal_event("evt_2", "agent_2");

        let m_score = scorer.score(&movement, &context);
        let b_score = scorer.score(&betrayal, &context);

        assert!(b_score > 0.8, "Betrayal should score > 0.8, got {}", b_score);
        assert!(m_score < 0.2, "Movement should score < 0.2, got {}", m_score);
        assert!(b_score > m_score, "Betrayal should score higher than movement");
    }

    #[test]
    fn test_scorer_tracked_agent_boost() {
        let scorer = EventScorer::default();
        let mut context = DirectorContext::new();

        let event = make_movement_event("evt_1", "agent_mira");

        let base_score = scorer.score(&event, &context);

        context.track_agent("agent_mira");
        let boosted_score = scorer.score(&event, &context);

        assert!(boosted_score > base_score, "Tracked agent should boost score");
        assert!((boosted_score / base_score - 1.5).abs() < 0.01, "Boost should be 1.5x");
    }

    #[test]
    fn test_scorer_tension_event_boost() {
        let scorer = EventScorer::default();
        let mut context = DirectorContext::new();

        let event = make_movement_event("evt_1", "agent_1");

        let base_score = scorer.score(&event, &context);

        context.add_tension_event("evt_1");
        let boosted_score = scorer.score(&event, &context);

        assert!(boosted_score > base_score, "Tension event should boost score");
        assert!((boosted_score / base_score - 2.0).abs() < 0.01, "Boost should be 2.0x");
    }

    #[test]
    fn test_scorer_drama_tags_additive() {
        let scorer = EventScorer::default();
        let context = DirectorContext::new();

        let mut event = make_movement_event("evt_1", "agent_1");
        let base_score = scorer.score(&event, &context);

        event.drama_tags = vec!["faction_critical".to_string()]; // +0.3
        let with_one_tag = scorer.score(&event, &context);

        event.drama_tags = vec![
            "faction_critical".to_string(),  // +0.3
            "secret_meeting".to_string(),    // +0.25
        ];
        let with_two_tags = scorer.score(&event, &context);

        assert!(with_one_tag > base_score);
        assert!(with_two_tags > with_one_tag);
        assert!((with_one_tag - base_score - 0.3).abs() < 0.01);
        assert!((with_two_tags - base_score - 0.55).abs() < 0.01);
    }

    #[test]
    fn test_scorer_batch() {
        let scorer = EventScorer::default();
        let context = DirectorContext::new();

        let events = vec![
            make_movement_event("evt_1", "agent_1"),
            make_betrayal_event("evt_2", "agent_2"),
            make_movement_event("evt_3", "agent_3"),
        ];

        let scored = scorer.score_batch(&events, &context);

        assert_eq!(scored.len(), 3);
        assert_eq!(scored[0].event.event_id, "evt_1");
        assert_eq!(scored[1].event.event_id, "evt_2");
        assert!(scored[1].score > scored[0].score); // Betrayal > Movement
    }

    #[test]
    fn test_scorer_with_custom_boosts() {
        let scorer = EventScorer::default()
            .with_tracked_boost(2.0)
            .with_tension_boost(3.0);

        let mut context = DirectorContext::new();
        context.track_agent("agent_1");

        let event = make_movement_event("evt_1", "agent_1");
        let score = scorer.score(&event, &context);

        // Base movement (0.1) * tracked boost (2.0) = 0.2
        assert!((score - 0.2).abs() < 0.01);
    }

    #[test]
    fn test_event_weights_serialization() {
        let weights = EventWeights::default();
        let toml = toml::to_string(&weights).unwrap();

        assert!(toml.contains("betrayal"));
        assert!(toml.contains("movement"));

        let parsed: EventWeights = toml::from_str(&toml).unwrap();
        assert_eq!(parsed.base_score(&EventType::Betrayal), 0.9);
    }

    #[test]
    fn test_director_context_track_multiple() {
        let mut context = DirectorContext::new();
        context.track_agents(vec!["agent_1", "agent_2", "agent_3"]);

        assert!(context.is_tracked("agent_1"));
        assert!(context.is_tracked("agent_2"));
        assert!(context.is_tracked("agent_3"));
        assert!(!context.is_tracked("agent_4"));
    }
}
