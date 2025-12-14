//! Event Types
//!
//! All event type definitions matching the simulation output schema.

use serde::{Deserialize, Serialize};

/// Primary event type categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

impl EventType {
    /// Returns the valid subtypes for this event type.
    pub fn valid_subtypes(&self) -> &'static [&'static str] {
        match self {
            EventType::Movement => &["travel", "flee", "pursue", "patrol"],
            EventType::Communication => &["share_memory", "spread_rumor", "lie", "confess"],
            EventType::Betrayal => &["secret_shared_with_enemy", "sabotage", "defection", "false_testimony"],
            EventType::Loyalty => &["defend_ally", "sacrifice_for_faction", "refuse_bribe"],
            EventType::Conflict => &["argument", "fight", "duel", "raid"],
            EventType::Cooperation => &["trade", "alliance_formed", "gift", "favor"],
            EventType::Faction => &["join", "leave", "exile", "promotion", "demotion"],
            EventType::Archive => &["write_entry", "read_entry", "destroy_entry", "forge_entry"],
            EventType::Ritual => &["reading_held", "reading_disrupted", "reading_attended", "reading_missed"],
            EventType::Resource => &["acquire", "lose", "trade", "steal", "hoard"],
            EventType::Death => &["natural", "killed", "executed", "sacrifice"],
            EventType::Birth => &["born", "arrived", "created"],
        }
    }

    /// Checks if the given subtype is valid for this event type.
    pub fn is_valid_subtype(&self, subtype: &str) -> bool {
        self.valid_subtypes().contains(&subtype)
    }

    /// Returns all event type variants.
    pub fn all() -> &'static [EventType] {
        &[
            EventType::Movement,
            EventType::Communication,
            EventType::Betrayal,
            EventType::Loyalty,
            EventType::Conflict,
            EventType::Cooperation,
            EventType::Faction,
            EventType::Archive,
            EventType::Ritual,
            EventType::Resource,
            EventType::Death,
            EventType::Birth,
        ]
    }
}

/// Common drama tags for categorizing events.
pub mod drama_tags {
    /// Event is critical to faction stability or survival
    pub const FACTION_CRITICAL: &str = "faction_critical";
    /// Event involves a secret meeting
    pub const SECRET_MEETING: &str = "secret_meeting";
    /// Event involves a faction leader
    pub const LEADER_INVOLVED: &str = "leader_involved";
    /// Event crosses faction boundaries
    pub const CROSS_FACTION: &str = "cross_faction";
    /// Event occurs during winter crisis
    pub const WINTER_CRISIS: &str = "winter_crisis";
    /// Event involves betrayal
    pub const BETRAYAL: &str = "betrayal";
    /// Event involves revenge
    pub const REVENGE: &str = "revenge";
    /// Event involves a power struggle
    pub const POWER_STRUGGLE: &str = "power_struggle";
    /// Event involves resource scarcity
    pub const RESOURCE_SCARCITY: &str = "resource_scarcity";
    /// Event involves forbidden alliance
    pub const FORBIDDEN_ALLIANCE: &str = "forbidden_alliance";
    /// Event involves selective history (at ritual)
    pub const SELECTIVE_HISTORY: &str = "selective_history";
    /// An agent was absent from an important event
    pub const ABSENT_AGENT: &str = "absent_agent";
    /// Event involves rumor spreading
    pub const RUMOR_SPREADING: &str = "rumor_spreading";
    /// Event erodes a leader's reputation
    pub const LEADER_REPUTATION_EROSION: &str = "leader_reputation_erosion";
    /// Event reveals a secret
    pub const SECRET_REVEALED: &str = "secret_revealed";
    /// Death event
    pub const DEATH: &str = "death";
    /// Birth/arrival event
    pub const NEW_ARRIVAL: &str = "new_arrival";
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

/// Affected agent with additional context.
///
/// Used for agents who are affected by an event but are not the primary
/// or secondary actors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectedActor {
    pub agent_id: String,
    pub name: String,
    pub faction: String,
    pub role: String,
    /// Relationship to the primary actor (e.g., "trusts_highly", "rival")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship_to_primary: Option<String>,
    /// For ritual events: whether the agent attended
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attended: Option<bool>,
    /// Reason for absence or other context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl AffectedActor {
    /// Creates a new AffectedActor with required fields.
    pub fn new(
        agent_id: impl Into<String>,
        name: impl Into<String>,
        faction: impl Into<String>,
        role: impl Into<String>,
    ) -> Self {
        Self {
            agent_id: agent_id.into(),
            name: name.into(),
            faction: faction.into(),
            role: role.into(),
            relationship_to_primary: None,
            attended: None,
            reason: None,
        }
    }

    /// Sets the relationship to the primary actor.
    pub fn with_relationship(mut self, relationship: impl Into<String>) -> Self {
        self.relationship_to_primary = Some(relationship.into());
        self
    }

    /// Sets attendance status (for ritual events).
    pub fn with_attendance(mut self, attended: bool, reason: Option<String>) -> Self {
        self.attended = Some(attended);
        self.reason = reason;
        self
    }
}

/// Actors involved in an event.
///
/// Contains the primary actor (required), optional secondary actor,
/// and a list of affected actors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorSet {
    /// The primary actor performing the action
    pub primary: ActorSnapshot,
    /// Optional secondary actor (e.g., target of communication)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary: Option<ActorSnapshot>,
    /// Other agents affected by this event
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected: Vec<AffectedActor>,
}

impl ActorSet {
    /// Creates an ActorSet with only a primary actor.
    pub fn primary_only(actor: ActorSnapshot) -> Self {
        Self {
            primary: actor,
            secondary: None,
            affected: Vec::new(),
        }
    }

    /// Creates an ActorSet with primary and secondary actors.
    pub fn with_secondary(primary: ActorSnapshot, secondary: ActorSnapshot) -> Self {
        Self {
            primary,
            secondary: Some(secondary),
            affected: Vec::new(),
        }
    }

    /// Adds affected actors to the set.
    pub fn with_affected(mut self, affected: Vec<AffectedActor>) -> Self {
        self.affected = affected;
        self
    }

    /// Returns all agent IDs involved in this event.
    pub fn all_agent_ids(&self) -> Vec<&str> {
        let mut ids = vec![self.primary.agent_id.as_str()];
        if let Some(ref secondary) = self.secondary {
            ids.push(secondary.agent_id.as_str());
        }
        for affected in &self.affected {
            ids.push(affected.agent_id.as_str());
        }
        ids
    }

    /// Checks if a specific agent is involved in this event.
    pub fn involves_agent(&self, agent_id: &str) -> bool {
        self.all_agent_ids().contains(&agent_id)
    }
}

/// Type alias for backwards compatibility
pub type EventActors = ActorSet;

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

/// A complete simulation event.
///
/// Events are the atomic units of simulation history. Each event captures
/// a specific action or occurrence, along with its actors, context, and outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Unique identifier (e.g., "evt_00042371")
    pub event_id: String,
    /// When the event occurred
    pub timestamp: crate::SimTimestamp,
    /// Primary event category
    pub event_type: EventType,
    /// Specific subtype within the category
    pub subtype: EventSubtype,
    /// Actors involved in the event
    pub actors: ActorSet,
    /// Context explaining why the event occurred
    pub context: EventContext,
    /// Results and state changes from the event
    pub outcome: EventOutcome,
    /// Tags for drama categorization
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub drama_tags: DramaTags,
    /// Drama score from 0.0 to 1.0
    #[serde(default)]
    pub drama_score: f32,
    /// Related event IDs
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub connected_events: Vec<String>,
}

impl Event {
    /// Create a new event with required fields.
    pub fn new(
        event_id: impl Into<String>,
        timestamp: crate::SimTimestamp,
        event_type: EventType,
        subtype: EventSubtype,
        actors: ActorSet,
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

    /// Sets the drama score and tags.
    pub fn with_drama(mut self, score: f32, tags: Vec<String>) -> Self {
        self.drama_score = score;
        self.drama_tags = tags;
        self
    }

    /// Sets connected events.
    pub fn with_connected_events(mut self, events: Vec<String>) -> Self {
        self.connected_events = events;
        self
    }

    /// Returns all agent IDs involved in this event.
    pub fn all_agent_ids(&self) -> Vec<&str> {
        self.actors.all_agent_ids()
    }

    /// Checks if a specific agent is involved in this event.
    pub fn involves_agent(&self, agent_id: &str) -> bool {
        self.actors.involves_agent(agent_id)
    }

    /// Checks if a specific faction is involved in this event.
    pub fn involves_faction(&self, faction: &str) -> bool {
        self.actors.primary.faction == faction
            || self.actors.secondary.as_ref().map_or(false, |s| s.faction == faction)
            || self.actors.affected.iter().any(|a| a.faction == faction)
    }

    /// Returns true if this is a high-drama event (score > 0.7).
    pub fn is_high_drama(&self) -> bool {
        self.drama_score > 0.7
    }

    /// Serializes the event to a JSON line (for JSONL format).
    pub fn to_jsonl(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserializes an event from a JSON line.
    pub fn from_jsonl(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line)
    }
}

/// Generates an event ID with the given sequence number.
pub fn generate_event_id(sequence: u64) -> String {
    format!("evt_{:08}", sequence)
}

/// Builder for creating events with a fluent API.
///
/// # Example
///
/// ```
/// use sim_events::*;
///
/// let event = EventBuilder::new(EventType::Movement, "travel")
///     .id("evt_00000001")
///     .timestamp(SimTimestamp::new(100, 1, Season::Spring, 10))
///     .primary_actor(ActorSnapshot::new("agent_1", "Test", "faction", "scout", "loc"))
///     .trigger("scheduled_patrol")
///     .drama_score(0.1)
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct EventBuilder {
    event_id: Option<String>,
    timestamp: Option<crate::SimTimestamp>,
    event_type: EventType,
    subtype: String,
    primary: Option<ActorSnapshot>,
    secondary: Option<ActorSnapshot>,
    affected: Vec<AffectedActor>,
    trigger: String,
    preconditions: Vec<String>,
    location_description: Option<String>,
    outcome: Option<EventOutcome>,
    drama_tags: Vec<String>,
    drama_score: f32,
    connected_events: Vec<String>,
}

impl EventBuilder {
    /// Creates a new EventBuilder with the required event type and subtype.
    pub fn new(event_type: EventType, subtype: impl Into<String>) -> Self {
        Self {
            event_id: None,
            timestamp: None,
            event_type,
            subtype: subtype.into(),
            primary: None,
            secondary: None,
            affected: Vec::new(),
            trigger: String::new(),
            preconditions: Vec::new(),
            location_description: None,
            outcome: None,
            drama_tags: Vec::new(),
            drama_score: 0.0,
            connected_events: Vec::new(),
        }
    }

    /// Sets the event ID.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.event_id = Some(id.into());
        self
    }

    /// Sets the timestamp.
    pub fn timestamp(mut self, ts: crate::SimTimestamp) -> Self {
        self.timestamp = Some(ts);
        self
    }

    /// Sets the primary actor.
    pub fn primary_actor(mut self, actor: ActorSnapshot) -> Self {
        self.primary = Some(actor);
        self
    }

    /// Sets the secondary actor.
    pub fn secondary_actor(mut self, actor: ActorSnapshot) -> Self {
        self.secondary = Some(actor);
        self
    }

    /// Adds an affected actor.
    pub fn add_affected(mut self, actor: AffectedActor) -> Self {
        self.affected.push(actor);
        self
    }

    /// Sets the trigger.
    pub fn trigger(mut self, trigger: impl Into<String>) -> Self {
        self.trigger = trigger.into();
        self
    }

    /// Adds a precondition.
    pub fn add_precondition(mut self, cond: impl Into<String>) -> Self {
        self.preconditions.push(cond.into());
        self
    }

    /// Sets the location description.
    pub fn location_description(mut self, desc: impl Into<String>) -> Self {
        self.location_description = Some(desc.into());
        self
    }

    /// Sets the outcome.
    pub fn outcome(mut self, outcome: EventOutcome) -> Self {
        self.outcome = Some(outcome);
        self
    }

    /// Sets the drama score.
    pub fn drama_score(mut self, score: f32) -> Self {
        self.drama_score = score;
        self
    }

    /// Adds a drama tag.
    pub fn add_drama_tag(mut self, tag: impl Into<String>) -> Self {
        self.drama_tags.push(tag.into());
        self
    }

    /// Adds a connected event.
    pub fn add_connected_event(mut self, event_id: impl Into<String>) -> Self {
        self.connected_events.push(event_id.into());
        self
    }

    /// Builds the Event.
    ///
    /// # Panics
    ///
    /// Panics if required fields (event_id, timestamp, primary actor) are not set.
    pub fn build(self) -> Event {
        let primary = self.primary.expect("EventBuilder: primary actor is required");
        let timestamp = self.timestamp.expect("EventBuilder: timestamp is required");
        let event_id = self.event_id.expect("EventBuilder: event_id is required");

        let actors = ActorSet {
            primary,
            secondary: self.secondary,
            affected: self.affected,
        };

        let context = EventContext {
            trigger: self.trigger,
            preconditions: self.preconditions,
            location_description: self.location_description,
        };

        // Use provided outcome or default to GeneralOutcome
        let outcome = self.outcome.unwrap_or_else(|| {
            EventOutcome::General(GeneralOutcome::default())
        });

        // Convert subtype string to EventSubtype
        let subtype = string_to_event_subtype(&self.event_type, &self.subtype);

        Event {
            event_id,
            timestamp,
            event_type: self.event_type,
            subtype,
            actors,
            context,
            outcome,
            drama_tags: self.drama_tags,
            drama_score: self.drama_score,
            connected_events: self.connected_events,
        }
    }
}

/// Converts a string subtype to the appropriate EventSubtype enum variant.
fn string_to_event_subtype(event_type: &EventType, subtype: &str) -> EventSubtype {
    match event_type {
        EventType::Movement => EventSubtype::Movement(match subtype {
            "travel" => MovementSubtype::Travel,
            "flee" => MovementSubtype::Flee,
            "pursue" => MovementSubtype::Pursue,
            "patrol" => MovementSubtype::Patrol,
            _ => MovementSubtype::Travel,
        }),
        EventType::Communication => EventSubtype::Communication(match subtype {
            "share_memory" => CommunicationSubtype::ShareMemory,
            "spread_rumor" => CommunicationSubtype::SpreadRumor,
            "lie" => CommunicationSubtype::Lie,
            "confess" => CommunicationSubtype::Confess,
            _ => CommunicationSubtype::ShareMemory,
        }),
        EventType::Betrayal => EventSubtype::Betrayal(match subtype {
            "secret_shared_with_enemy" => BetrayalSubtype::SecretSharedWithEnemy,
            "sabotage" => BetrayalSubtype::Sabotage,
            "defection" => BetrayalSubtype::Defection,
            "false_testimony" => BetrayalSubtype::FalseTestimony,
            _ => BetrayalSubtype::Defection,
        }),
        EventType::Loyalty => EventSubtype::Loyalty(match subtype {
            "defend_ally" => LoyaltySubtype::DefendAlly,
            "sacrifice_for_faction" => LoyaltySubtype::SacrificeForFaction,
            "refuse_bribe" => LoyaltySubtype::RefuseBribe,
            _ => LoyaltySubtype::DefendAlly,
        }),
        EventType::Conflict => EventSubtype::Conflict(match subtype {
            "argument" => ConflictSubtype::Argument,
            "fight" => ConflictSubtype::Fight,
            "duel" => ConflictSubtype::Duel,
            "raid" => ConflictSubtype::Raid,
            _ => ConflictSubtype::Argument,
        }),
        EventType::Cooperation => EventSubtype::Cooperation(match subtype {
            "trade" => CooperationSubtype::Trade,
            "alliance_formed" => CooperationSubtype::AllianceFormed,
            "gift" => CooperationSubtype::Gift,
            "favor" => CooperationSubtype::Favor,
            _ => CooperationSubtype::Trade,
        }),
        EventType::Faction => EventSubtype::Faction(match subtype {
            "join" => FactionSubtype::Join,
            "leave" => FactionSubtype::Leave,
            "exile" => FactionSubtype::Exile,
            "promotion" => FactionSubtype::Promotion,
            "demotion" => FactionSubtype::Demotion,
            _ => FactionSubtype::Join,
        }),
        EventType::Archive => EventSubtype::Archive(match subtype {
            "write_entry" => ArchiveSubtype::WriteEntry,
            "read_entry" => ArchiveSubtype::ReadEntry,
            "destroy_entry" => ArchiveSubtype::DestroyEntry,
            "forge_entry" => ArchiveSubtype::ForgeEntry,
            _ => ArchiveSubtype::WriteEntry,
        }),
        EventType::Ritual => EventSubtype::Ritual(match subtype {
            "reading_held" => RitualSubtype::ReadingHeld,
            "reading_disrupted" => RitualSubtype::ReadingDisrupted,
            "reading_attended" => RitualSubtype::ReadingAttended,
            "reading_missed" => RitualSubtype::ReadingMissed,
            _ => RitualSubtype::ReadingHeld,
        }),
        EventType::Resource => EventSubtype::Resource(match subtype {
            "acquire" => ResourceSubtype::Acquire,
            "lose" => ResourceSubtype::Lose,
            "trade" => ResourceSubtype::Trade,
            "steal" => ResourceSubtype::Steal,
            "hoard" => ResourceSubtype::Hoard,
            _ => ResourceSubtype::Acquire,
        }),
        EventType::Death => EventSubtype::Death(match subtype {
            "natural" => DeathSubtype::Natural,
            "killed" => DeathSubtype::Killed,
            "executed" => DeathSubtype::Executed,
            "sacrifice" => DeathSubtype::Sacrifice,
            _ => DeathSubtype::Natural,
        }),
        EventType::Birth => EventSubtype::Birth(match subtype {
            "born" => BirthSubtype::Born,
            "arrived" => BirthSubtype::Arrived,
            "created" => BirthSubtype::Created,
            _ => BirthSubtype::Arrived,
        }),
    }
}

/// Helper to create a simple movement event.
pub fn create_movement_event(
    event_id: impl Into<String>,
    timestamp: crate::SimTimestamp,
    actor: ActorSnapshot,
    trigger: impl Into<String>,
    new_location: impl Into<String>,
) -> Event {
    Event::new(
        event_id,
        timestamp,
        EventType::Movement,
        EventSubtype::Movement(MovementSubtype::Travel),
        ActorSet::primary_only(actor),
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
    use crate::{Season, SimTimestamp};

    #[test]
    fn test_event_type_serialization() {
        assert_eq!(serde_json::to_string(&EventType::Movement).unwrap(), r#""movement""#);
        assert_eq!(serde_json::to_string(&EventType::Communication).unwrap(), r#""communication""#);
        assert_eq!(serde_json::to_string(&EventType::Betrayal).unwrap(), r#""betrayal""#);
        assert_eq!(serde_json::to_string(&EventType::Loyalty).unwrap(), r#""loyalty""#);
        assert_eq!(serde_json::to_string(&EventType::Conflict).unwrap(), r#""conflict""#);
        assert_eq!(serde_json::to_string(&EventType::Cooperation).unwrap(), r#""cooperation""#);
        assert_eq!(serde_json::to_string(&EventType::Faction).unwrap(), r#""faction""#);
        assert_eq!(serde_json::to_string(&EventType::Archive).unwrap(), r#""archive""#);
        assert_eq!(serde_json::to_string(&EventType::Ritual).unwrap(), r#""ritual""#);
        assert_eq!(serde_json::to_string(&EventType::Resource).unwrap(), r#""resource""#);
        assert_eq!(serde_json::to_string(&EventType::Death).unwrap(), r#""death""#);
        assert_eq!(serde_json::to_string(&EventType::Birth).unwrap(), r#""birth""#);
    }

    #[test]
    fn test_event_type_deserialization() {
        assert_eq!(serde_json::from_str::<EventType>(r#""movement""#).unwrap(), EventType::Movement);
        assert_eq!(serde_json::from_str::<EventType>(r#""betrayal""#).unwrap(), EventType::Betrayal);
        assert_eq!(serde_json::from_str::<EventType>(r#""death""#).unwrap(), EventType::Death);
    }

    #[test]
    fn test_event_type_valid_subtypes() {
        assert!(EventType::Movement.is_valid_subtype("travel"));
        assert!(EventType::Movement.is_valid_subtype("flee"));
        assert!(!EventType::Movement.is_valid_subtype("invalid"));

        assert!(EventType::Betrayal.is_valid_subtype("secret_shared_with_enemy"));
        assert!(EventType::Betrayal.is_valid_subtype("defection"));
        assert!(!EventType::Betrayal.is_valid_subtype("travel"));

        assert!(EventType::Death.is_valid_subtype("natural"));
        assert!(EventType::Death.is_valid_subtype("killed"));
        assert!(!EventType::Death.is_valid_subtype("murdered")); // not a valid subtype
    }

    #[test]
    fn test_event_type_all_variants() {
        let all = EventType::all();
        assert_eq!(all.len(), 12);
        assert!(all.contains(&EventType::Movement));
        assert!(all.contains(&EventType::Death));
    }

    #[test]
    fn test_drama_tags_constants() {
        assert_eq!(drama_tags::FACTION_CRITICAL, "faction_critical");
        assert_eq!(drama_tags::BETRAYAL, "betrayal");
        assert_eq!(drama_tags::WINTER_CRISIS, "winter_crisis");
    }

    #[test]
    fn test_event_serialization() {
        let actor = ActorSnapshot::new(
            "agent_mira_0042",
            "Mira of Thornwood",
            "thornwood",
            "scout",
            "thornwood_village",
        );

        let timestamp = SimTimestamp::new(100, 1, Season::Spring, 10);
        let event = create_movement_event(
            "evt_00000001",
            timestamp,
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
        let timestamp = SimTimestamp::new(1, 1, Season::Spring, 1);
        let event = create_movement_event("evt_1", timestamp, actor, "test", "loc2")
            .with_drama(0.5, vec!["test_tag".to_string()]);

        assert_eq!(event.drama_score, 0.5);
        assert_eq!(event.drama_tags.len(), 1);
    }

    #[test]
    fn test_actor_set_primary_only() {
        let actor = ActorSnapshot::new("agent_1", "Test", "faction", "role", "loc");
        let actors = ActorSet::primary_only(actor);

        assert_eq!(actors.primary.agent_id, "agent_1");
        assert!(actors.secondary.is_none());
        assert!(actors.affected.is_empty());
    }

    #[test]
    fn test_actor_set_with_secondary() {
        let primary = ActorSnapshot::new("agent_1", "Primary", "faction1", "role1", "loc");
        let secondary = ActorSnapshot::new("agent_2", "Secondary", "faction2", "role2", "loc");
        let actors = ActorSet::with_secondary(primary, secondary);

        assert_eq!(actors.primary.agent_id, "agent_1");
        assert_eq!(actors.secondary.as_ref().unwrap().agent_id, "agent_2");
    }

    #[test]
    fn test_actor_set_all_agent_ids() {
        let primary = ActorSnapshot::new("agent_1", "P", "f", "r", "l");
        let secondary = ActorSnapshot::new("agent_2", "S", "f", "r", "l");
        let affected = vec![
            AffectedActor::new("agent_3", "A1", "f", "r"),
            AffectedActor::new("agent_4", "A2", "f", "r"),
        ];
        let actors = ActorSet::with_secondary(primary, secondary).with_affected(affected);

        let ids = actors.all_agent_ids();
        assert_eq!(ids.len(), 4);
        assert!(ids.contains(&"agent_1"));
        assert!(ids.contains(&"agent_2"));
        assert!(ids.contains(&"agent_3"));
        assert!(ids.contains(&"agent_4"));
    }

    #[test]
    fn test_actor_set_involves_agent() {
        let primary = ActorSnapshot::new("agent_1", "P", "f", "r", "l");
        let actors = ActorSet::primary_only(primary);

        assert!(actors.involves_agent("agent_1"));
        assert!(!actors.involves_agent("agent_2"));
    }

    #[test]
    fn test_affected_actor_builder() {
        let affected = AffectedActor::new("agent_1", "Test", "faction", "role")
            .with_relationship("trusts_highly")
            .with_attendance(false, Some("absent_from_village".to_string()));

        assert_eq!(affected.agent_id, "agent_1");
        assert_eq!(affected.relationship_to_primary, Some("trusts_highly".to_string()));
        assert_eq!(affected.attended, Some(false));
        assert_eq!(affected.reason, Some("absent_from_village".to_string()));
    }

    #[test]
    fn test_actor_set_serialization() {
        let primary = ActorSnapshot::new("agent_mira_0042", "Mira", "thornwood", "scout", "eastern_bridge");
        let secondary = ActorSnapshot::new("agent_voss_0017", "Voss", "ironmere", "spymaster", "eastern_bridge");
        let affected = vec![
            AffectedActor::new("agent_corin_0003", "Corin", "thornwood", "faction_leader")
                .with_relationship("trusts_highly"),
        ];
        let actors = ActorSet::with_secondary(primary, secondary).with_affected(affected);

        let json = serde_json::to_string_pretty(&actors).unwrap();
        assert!(json.contains("agent_mira_0042"));
        assert!(json.contains("agent_voss_0017"));
        assert!(json.contains("agent_corin_0003"));
        assert!(json.contains("trusts_highly"));

        // Verify roundtrip
        let parsed: ActorSet = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.primary.agent_id, "agent_mira_0042");
    }

    #[test]
    fn test_event_involves_faction() {
        let actor = ActorSnapshot::new("agent_1", "Test", "thornwood", "scout", "loc");
        let timestamp = SimTimestamp::new(1, 1, Season::Spring, 1);
        let event = create_movement_event("evt_1", timestamp, actor, "test", "loc2");

        assert!(event.involves_faction("thornwood"));
        assert!(!event.involves_faction("ironmere"));
    }

    #[test]
    fn test_event_is_high_drama() {
        let actor = ActorSnapshot::new("agent_1", "Test", "faction", "role", "loc");
        let timestamp = SimTimestamp::new(1, 1, Season::Spring, 1);

        let low_drama = create_movement_event("evt_1", timestamp.clone(), actor.clone(), "test", "loc2")
            .with_drama(0.3, vec![]);
        let high_drama = create_movement_event("evt_2", timestamp, actor, "test", "loc2")
            .with_drama(0.85, vec![]);

        assert!(!low_drama.is_high_drama());
        assert!(high_drama.is_high_drama());
    }

    #[test]
    fn test_event_jsonl() {
        let actor = ActorSnapshot::new("agent_1", "Test", "faction", "role", "loc");
        let timestamp = SimTimestamp::new(1, 1, Season::Spring, 1);
        let event = create_movement_event("evt_1", timestamp, actor, "test", "loc2");

        let line = event.to_jsonl().unwrap();
        assert!(!line.contains('\n')); // No newlines in JSONL

        let parsed = Event::from_jsonl(&line).unwrap();
        assert_eq!(parsed.event_id, "evt_1");
    }

    #[test]
    fn test_generate_event_id() {
        assert_eq!(generate_event_id(1), "evt_00000001");
        assert_eq!(generate_event_id(42371), "evt_00042371");
        assert_eq!(generate_event_id(99999999), "evt_99999999");
    }

    #[test]
    fn test_event_builder() {
        let event = EventBuilder::new(EventType::Movement, "travel")
            .id("evt_00000001")
            .timestamp(SimTimestamp::new(100, 1, Season::Spring, 10))
            .primary_actor(ActorSnapshot::new("agent_1", "Test Agent", "thornwood", "scout", "village_a"))
            .trigger("scheduled_patrol")
            .drama_score(0.1)
            .add_drama_tag("routine")
            .build();

        assert_eq!(event.event_id, "evt_00000001");
        assert_eq!(event.event_type, EventType::Movement);
        assert_eq!(event.drama_score, 0.1);
        assert_eq!(event.actors.primary.agent_id, "agent_1");
    }

    #[test]
    fn test_event_builder_with_secondary() {
        let event = EventBuilder::new(EventType::Communication, "share_memory")
            .id("evt_00000002")
            .timestamp(SimTimestamp::new(200, 1, Season::Spring, 15))
            .primary_actor(ActorSnapshot::new("agent_1", "Alice", "thornwood", "scout", "market"))
            .secondary_actor(ActorSnapshot::new("agent_2", "Bob", "thornwood", "trader", "market"))
            .trigger("casual_conversation")
            .add_precondition("same_location")
            .add_precondition("no_hostility")
            .drama_score(0.2)
            .build();

        assert!(event.actors.secondary.is_some());
        assert_eq!(event.actors.secondary.unwrap().agent_id, "agent_2");
        assert_eq!(event.context.preconditions.len(), 2);
    }

    #[test]
    fn test_event_builder_betrayal() {
        let event = EventBuilder::new(EventType::Betrayal, "secret_shared_with_enemy")
            .id("evt_00042371")
            .timestamp(SimTimestamp::new(84729, 3, Season::Winter, 12))
            .primary_actor(ActorSnapshot::new("agent_mira_0042", "Mira of Thornwood", "thornwood", "scout", "eastern_bridge"))
            .secondary_actor(ActorSnapshot::new("agent_voss_0017", "Voss the Quiet", "ironmere", "spymaster", "eastern_bridge"))
            .add_affected(AffectedActor::new("agent_corin_0003", "Corin Thornwood", "thornwood", "faction_leader")
                .with_relationship("trusts_highly"))
            .trigger("mira_reliability_trust_in_corin_below_threshold")
            .add_precondition("mira witnessed corin_broke_promise_to_mira at tick 81204")
            .location_description("neutral territory, no witnesses")
            .drama_score(0.87)
            .add_drama_tag(drama_tags::BETRAYAL)
            .add_drama_tag(drama_tags::FACTION_CRITICAL)
            .add_drama_tag(drama_tags::SECRET_MEETING)
            .add_connected_event("evt_00039102")
            .build();

        assert_eq!(event.event_type, EventType::Betrayal);
        assert!(event.is_high_drama());
        assert_eq!(event.drama_tags.len(), 3);
        assert_eq!(event.actors.affected.len(), 1);
    }
}
