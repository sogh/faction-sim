//! Event Types
//!
//! All event type definitions matching the simulation output schema.

use serde::{Deserialize, Serialize};

/// Primary event type categories
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Movement,
    Communication,
    Betrayal,
    Loyalty,
    Conflict,
    Cooperation,
    Faction,
    Archive,
    Ritual,
    Resource,
    Death,
    Birth,
}

/// Movement event subtypes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MovementSubtype {
    Travel,
    Flee,
    Pursue,
    Patrol,
    ReturnHome,
}

/// Communication event subtypes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommunicationSubtype {
    ShareMemory,
    SpreadRumor,
    Lie,
    Confess,
    Recruit,
    Report,
}

/// Betrayal event subtypes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BetrayalSubtype {
    SecretSharedWithEnemy,
    Sabotage,
    Defection,
    FalseTestimony,
}

/// Loyalty event subtypes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoyaltySubtype {
    DefendAlly,
    SacrificeForFaction,
    RefuseBribe,
    ReportSuspicion,
}

/// Conflict event subtypes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictSubtype {
    Argument,
    Fight,
    Duel,
    Raid,
    Assassination,
}

/// Cooperation event subtypes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CooperationSubtype {
    Trade,
    AllianceFormed,
    Gift,
    Favor,
    BuildTrust,
}

/// Faction event subtypes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactionSubtype {
    Join,
    Leave,
    Exile,
    Promotion,
    Demotion,
    ChallengeLeader,
    SupportLeader,
}

/// Archive event subtypes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchiveSubtype {
    WriteEntry,
    ReadEntry,
    DestroyEntry,
    ForgeEntry,
}

/// Ritual event subtypes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RitualSubtype {
    ReadingHeld,
    ReadingDisrupted,
    ReadingAttended,
    ReadingMissed,
}

/// Resource event subtypes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceSubtype {
    Acquire,
    Lose,
    Trade,
    Steal,
    Hoard,
    Work,
    /// Consuming resources to satisfy a need (eating, drinking, etc.)
    Consume,
}

/// Death event subtypes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeathSubtype {
    Natural,
    Killed,
    Executed,
    Sacrifice,
}

/// Birth event subtypes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BirthSubtype {
    Born,
    Arrived,
    Created,
}

/// Combined event subtype enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventSubtype {
    Movement(MovementSubtype),
    Communication(CommunicationSubtype),
    Betrayal(BetrayalSubtype),
    Loyalty(LoyaltySubtype),
    Conflict(ConflictSubtype),
    Cooperation(CooperationSubtype),
    Faction(FactionSubtype),
    Archive(ArchiveSubtype),
    Ritual(RitualSubtype),
    Resource(ResourceSubtype),
    Death(DeathSubtype),
    Birth(BirthSubtype),
}

/// Timestamp for events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTimestamp {
    pub tick: u64,
    pub date: String,
}

impl EventTimestamp {
    pub fn new(tick: u64, date: impl Into<String>) -> Self {
        Self {
            tick,
            date: date.into(),
        }
    }
}

/// Snapshot of an agent's state at the time of an event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorSnapshot {
    pub agent_id: String,
    pub name: String,
    pub faction: String,
    pub role: String,
    pub location: String,
}

impl ActorSnapshot {
    pub fn new(
        agent_id: impl Into<String>,
        name: impl Into<String>,
        faction: impl Into<String>,
        role: impl Into<String>,
        location: impl Into<String>,
    ) -> Self {
        Self {
            agent_id: agent_id.into(),
            name: name.into(),
            faction: faction.into(),
            role: role.into(),
            location: location.into(),
        }
    }
}

/// Affected agent with additional context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectedActor {
    pub agent_id: String,
    pub name: String,
    pub faction: String,
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship_to_primary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attended: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Actors involved in an event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventActors {
    pub primary: ActorSnapshot,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary: Option<ActorSnapshot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affected: Option<Vec<AffectedActor>>,
}

impl EventActors {
    pub fn solo(primary: ActorSnapshot) -> Self {
        Self {
            primary,
            secondary: None,
            affected: None,
        }
    }

    pub fn pair(primary: ActorSnapshot, secondary: ActorSnapshot) -> Self {
        Self {
            primary,
            secondary: Some(secondary),
            affected: None,
        }
    }

    pub fn with_affected(mut self, affected: Vec<AffectedActor>) -> Self {
        self.affected = Some(affected);
        self
    }
}

/// Context for why an event occurred
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventContext {
    pub trigger: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preconditions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_description: Option<String>,
}

impl EventContext {
    pub fn new(trigger: impl Into<String>) -> Self {
        Self {
            trigger: trigger.into(),
            preconditions: Vec::new(),
            location_description: None,
        }
    }

    pub fn with_preconditions(mut self, preconditions: Vec<String>) -> Self {
        self.preconditions = preconditions;
        self
    }

    pub fn with_location(mut self, description: impl Into<String>) -> Self {
        self.location_description = Some(description.into());
        self
    }
}

/// Flexible outcome data for events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventOutcome {
    Movement(MovementOutcome),
    Communication(CommunicationOutcome),
    Relationship(RelationshipOutcome),
    Archive(ArchiveOutcome),
    General(GeneralOutcome),
}

/// Archive event outcome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveOutcome {
    /// The entry ID that was created, read, destroyed, or forged
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_id: Option<String>,
    /// Content of the entry (for write/forge events)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Subject of the entry
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// Whether the action was authentic (false for forged entries)
    pub is_authentic: bool,
}

/// Movement event outcome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementOutcome {
    pub new_location: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub travel_duration_ticks: Option<u32>,
}

/// Communication event outcome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationOutcome {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_shared: Option<MemorySharedInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipient_state_change: Option<RecipientStateChange>,
}

/// Information about a shared memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySharedInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_event: Option<String>,
    pub content: String,
    pub source_chain: Vec<String>,
    pub fidelity: f32,
}

/// State change in the recipient of communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipientStateChange {
    pub new_memory_added: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust_impact: Option<TrustImpact>,
}

/// Trust change from an event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustImpact {
    pub toward: String,
    pub dimension: String,
    pub delta: f32,
    pub reason: String,
}

/// Relationship change outcome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipOutcome {
    pub relationship_changes: Vec<RelationshipChange>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub state_changes: Vec<String>,
}

/// A single relationship change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipChange {
    pub from: String,
    pub to: String,
    pub dimension: String,
    pub old_value: f32,
    pub new_value: f32,
}

/// General outcome for simple events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralOutcome {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub state_changes: Vec<String>,
}

impl Default for GeneralOutcome {
    fn default() -> Self {
        Self {
            description: None,
            state_changes: Vec::new(),
        }
    }
}

/// Drama tags for categorizing events
pub type DramaTags = Vec<String>;

/// A complete simulation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_id: String,
    pub timestamp: EventTimestamp,
    pub event_type: EventType,
    pub subtype: EventSubtype,
    pub actors: EventActors,
    pub context: EventContext,
    pub outcome: EventOutcome,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub drama_tags: DramaTags,
    #[serde(default)]
    pub drama_score: f32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub connected_events: Vec<String>,
}

impl Event {
    /// Create a new event with required fields
    pub fn new(
        event_id: impl Into<String>,
        timestamp: EventTimestamp,
        event_type: EventType,
        subtype: EventSubtype,
        actors: EventActors,
        context: EventContext,
        outcome: EventOutcome,
    ) -> Self {
        Self {
            event_id: event_id.into(),
            timestamp,
            event_type,
            subtype,
            actors,
            context,
            outcome,
            drama_tags: Vec::new(),
            drama_score: 0.0,
            connected_events: Vec::new(),
        }
    }

    pub fn with_drama(mut self, score: f32, tags: Vec<String>) -> Self {
        self.drama_score = score;
        self.drama_tags = tags;
        self
    }

    pub fn with_connected_events(mut self, events: Vec<String>) -> Self {
        self.connected_events = events;
        self
    }
}

/// Helper to create a simple movement event
pub fn create_movement_event(
    event_id: impl Into<String>,
    tick: u64,
    date: impl Into<String>,
    actor: ActorSnapshot,
    trigger: impl Into<String>,
    new_location: impl Into<String>,
) -> Event {
    Event::new(
        event_id,
        EventTimestamp::new(tick, date),
        EventType::Movement,
        EventSubtype::Movement(MovementSubtype::Travel),
        EventActors::solo(actor),
        EventContext::new(trigger),
        EventOutcome::Movement(MovementOutcome {
            new_location: new_location.into(),
            travel_duration_ticks: None,
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_serialization() {
        let actor = ActorSnapshot::new(
            "agent_mira_0042",
            "Mira of Thornwood",
            "thornwood",
            "scout",
            "thornwood_village",
        );

        let event = create_movement_event(
            "evt_00000001",
            100,
            "year_1.spring.day_10",
            actor,
            "scheduled_patrol",
            "eastern_bridge",
        );

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("evt_00000001"));
        assert!(json.contains("movement"));
        assert!(json.contains("eastern_bridge"));

        // Verify it can be parsed back
        let parsed: Event = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.event_id, "evt_00000001");
    }

    #[test]
    fn test_event_with_drama() {
        let actor = ActorSnapshot::new("agent_1", "Test", "faction", "role", "loc");
        let event = create_movement_event("evt_1", 1, "day_1", actor, "test", "loc2")
            .with_drama(0.5, vec!["test_tag".to_string()]);

        assert_eq!(event.drama_score, 0.5);
        assert_eq!(event.drama_tags.len(), 1);
    }
}
