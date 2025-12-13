//! Drama Scoring System
//!
//! Calculates drama scores for events to help the Director AI
//! identify narratively interesting moments.

use crate::events::types::{Event, EventType, EventSubtype, EventOutcome};

/// Drama score calculation result
#[derive(Debug, Clone)]
pub struct DramaAnalysis {
    /// Overall drama score (0.0 - 1.0)
    pub score: f32,
    /// Tags describing the dramatic elements
    pub tags: Vec<String>,
    /// Brief explanation of why this is dramatic
    pub reason: Option<String>,
}

/// Base drama scores by event type
pub mod base_scores {
    // Movement events - generally low drama
    pub const MOVEMENT_WANDER: f32 = 0.05;
    pub const MOVEMENT_PATROL: f32 = 0.08;
    pub const MOVEMENT_RETURN_HOME: f32 = 0.03;
    pub const MOVEMENT_FLEE: f32 = 0.4;

    // Communication events - moderate drama
    pub const COMMUNICATION_CHAT: f32 = 0.1;
    pub const COMMUNICATION_GOSSIP: f32 = 0.25;
    pub const COMMUNICATION_SECRET: f32 = 0.5;
    pub const COMMUNICATION_SHARE_MEMORY: f32 = 0.3;
    pub const COMMUNICATION_LIE: f32 = 0.4;
    pub const COMMUNICATION_CONFESS: f32 = 0.6;

    // Resource events - low to moderate
    pub const RESOURCE_WORK: f32 = 0.05;
    pub const RESOURCE_TRADE: f32 = 0.15;
    pub const RESOURCE_STEAL: f32 = 0.5;
    pub const RESOURCE_HOARD: f32 = 0.3;

    // Social events - moderate
    pub const SOCIAL_BUILD_TRUST: f32 = 0.1;
    pub const SOCIAL_CURRY_FAVOR: f32 = 0.2;
    pub const SOCIAL_GIFT: f32 = 0.25;
    pub const SOCIAL_OSTRACIZE: f32 = 0.4;

    // Faction events - high drama
    pub const FACTION_DEFECT: f32 = 0.8;
    pub const FACTION_EXILE: f32 = 0.7;
    pub const FACTION_CHALLENGE_LEADER: f32 = 0.9;
    pub const FACTION_SUPPORT_LEADER: f32 = 0.3;
    pub const FACTION_SUCCESSION: f32 = 0.95;

    // Conflict events - high drama
    pub const CONFLICT_ARGUMENT: f32 = 0.35;
    pub const CONFLICT_FIGHT: f32 = 0.6;
    pub const CONFLICT_SABOTAGE: f32 = 0.55;
    pub const CONFLICT_ASSASSINATION: f32 = 0.98;

    // Archive events - moderate
    pub const ARCHIVE_WRITE: f32 = 0.2;
    pub const ARCHIVE_READ: f32 = 0.1;
    pub const ARCHIVE_DESTROY: f32 = 0.5;
    pub const ARCHIVE_FORGE: f32 = 0.6;

    // Ritual events - moderate to high
    pub const RITUAL_GATHERING: f32 = 0.25;
    pub const RITUAL_READING: f32 = 0.3;
}

/// Drama multipliers based on context
pub mod multipliers {
    /// High status agent involved (leader, council)
    pub const HIGH_STATUS_ACTOR: f32 = 1.4;
    /// Leader directly involved
    pub const LEADER_INVOLVED: f32 = 1.6;

    /// Cross-faction interaction
    pub const CROSS_FACTION: f32 = 1.3;
    /// Enemy factions
    pub const ENEMY_FACTIONS: f32 = 1.5;

    /// Close relationship affected (high trust broken)
    pub const CLOSE_RELATIONSHIP: f32 = 1.4;
    /// Betrayal of trust
    pub const BETRAYAL: f32 = 1.8;

    /// Winter events (survival drama)
    pub const WINTER_CONTEXT: f32 = 1.2;
    /// Desperate need state
    pub const DESPERATE_STATE: f32 = 1.3;

    /// First occurrence of rare event
    pub const FIRST_OCCURRENCE: f32 = 1.5;
    /// Recurring pattern (ongoing conflict)
    pub const RECURRING_PATTERN: f32 = 1.2;
}

/// Calculate drama score for an event
pub fn calculate_drama_score(event: &Event) -> DramaAnalysis {
    let base_score = get_base_score(event);
    let mut score = base_score;
    let mut tags = Vec::new();
    let mut reasons = Vec::new();

    // Apply actor-based multipliers
    let actor = &event.actors.primary;

    // Check if high status actor
    if actor.role == "leader" {
        score *= multipliers::LEADER_INVOLVED;
        tags.push("leader_involved".to_string());
        reasons.push("faction leader involved");
    } else if actor.role == "council_member" || actor.role == "reader" {
        score *= multipliers::HIGH_STATUS_ACTOR;
        tags.push("high_status".to_string());
    }

    // Check for cross-faction interactions
    if let Some(ref secondary) = event.actors.secondary {
        if actor.faction != secondary.faction && !secondary.faction.is_empty() && secondary.faction != "unknown" {
            score *= multipliers::CROSS_FACTION;
            tags.push("cross_faction".to_string());
            reasons.push("cross-faction interaction");
        }
    }

    // Check context for winter
    if let Some(ref location_desc) = event.context.location_description {
        if location_desc.contains("winter") {
            score *= multipliers::WINTER_CONTEXT;
            tags.push("winter_crisis".to_string());
        }
    }

    // Add event-type specific tags
    add_event_tags(event, &mut tags);

    // Clamp score to 0.0 - 1.0
    score = score.clamp(0.0, 1.0);

    let reason = if reasons.is_empty() {
        None
    } else {
        Some(reasons.join("; "))
    };

    DramaAnalysis { score, tags, reason }
}

/// Get base drama score for an event
fn get_base_score(event: &Event) -> f32 {
    use crate::events::types::*;

    match &event.subtype {
        EventSubtype::Movement(m) => match m {
            MovementSubtype::Travel => base_scores::MOVEMENT_WANDER,
            MovementSubtype::Patrol => base_scores::MOVEMENT_PATROL,
            MovementSubtype::ReturnHome => base_scores::MOVEMENT_RETURN_HOME,
            MovementSubtype::Flee => base_scores::MOVEMENT_FLEE,
            MovementSubtype::Pursue => 0.3,
        },
        EventSubtype::Communication(c) => match c {
            CommunicationSubtype::ShareMemory => base_scores::COMMUNICATION_SHARE_MEMORY,
            CommunicationSubtype::SpreadRumor => base_scores::COMMUNICATION_GOSSIP,
            CommunicationSubtype::Lie => base_scores::COMMUNICATION_LIE,
            CommunicationSubtype::Confess => base_scores::COMMUNICATION_CONFESS,
            CommunicationSubtype::Recruit => 0.35,
            CommunicationSubtype::Report => 0.2,
        },
        EventSubtype::Resource(r) => match r {
            ResourceSubtype::Work => base_scores::RESOURCE_WORK,
            ResourceSubtype::Trade => base_scores::RESOURCE_TRADE,
            ResourceSubtype::Steal => base_scores::RESOURCE_STEAL,
            ResourceSubtype::Hoard => base_scores::RESOURCE_HOARD,
            ResourceSubtype::Acquire => 0.1,
            ResourceSubtype::Lose => 0.2,
        },
        EventSubtype::Cooperation(c) => match c {
            CooperationSubtype::BuildTrust => base_scores::SOCIAL_BUILD_TRUST,
            CooperationSubtype::Favor => base_scores::SOCIAL_CURRY_FAVOR,
            CooperationSubtype::Gift => base_scores::SOCIAL_GIFT,
            CooperationSubtype::Trade => base_scores::RESOURCE_TRADE,
            CooperationSubtype::AllianceFormed => 0.5,
        },
        EventSubtype::Faction(f) => match f {
            FactionSubtype::Leave => base_scores::FACTION_DEFECT,
            FactionSubtype::Exile => base_scores::FACTION_EXILE,
            FactionSubtype::ChallengeLeader => base_scores::FACTION_CHALLENGE_LEADER,
            FactionSubtype::SupportLeader => base_scores::FACTION_SUPPORT_LEADER,
            FactionSubtype::Join => 0.3,
            FactionSubtype::Promotion => 0.4,
            FactionSubtype::Demotion => 0.35,
        },
        EventSubtype::Conflict(c) => match c {
            ConflictSubtype::Argument => base_scores::CONFLICT_ARGUMENT,
            ConflictSubtype::Fight => base_scores::CONFLICT_FIGHT,
            ConflictSubtype::Raid => base_scores::CONFLICT_SABOTAGE,
            ConflictSubtype::Assassination => base_scores::CONFLICT_ASSASSINATION,
            ConflictSubtype::Duel => 0.7,
        },
        EventSubtype::Archive(a) => match a {
            ArchiveSubtype::WriteEntry => base_scores::ARCHIVE_WRITE,
            ArchiveSubtype::ReadEntry => base_scores::ARCHIVE_READ,
            ArchiveSubtype::DestroyEntry => base_scores::ARCHIVE_DESTROY,
            ArchiveSubtype::ForgeEntry => base_scores::ARCHIVE_FORGE,
        },
        EventSubtype::Ritual(r) => match r {
            RitualSubtype::ReadingHeld => base_scores::RITUAL_GATHERING,
            RitualSubtype::ReadingAttended => base_scores::RITUAL_READING,
            RitualSubtype::ReadingMissed => 0.15,
            RitualSubtype::ReadingDisrupted => 0.45,
        },
        EventSubtype::Betrayal(b) => match b {
            BetrayalSubtype::Defection => base_scores::FACTION_DEFECT,
            BetrayalSubtype::Sabotage => base_scores::CONFLICT_SABOTAGE,
            BetrayalSubtype::SecretSharedWithEnemy => 0.7,
            BetrayalSubtype::FalseTestimony => 0.5,
        },
        EventSubtype::Loyalty(l) => match l {
            LoyaltySubtype::DefendAlly => 0.4,
            LoyaltySubtype::SacrificeForFaction => 0.6,
            LoyaltySubtype::RefuseBribe => 0.35,
            LoyaltySubtype::ReportSuspicion => 0.3,
        },
        EventSubtype::Death(d) => match d {
            DeathSubtype::Natural => 0.5,
            DeathSubtype::Killed => 0.9,
            DeathSubtype::Executed => 0.85,
            DeathSubtype::Sacrifice => 0.95,
        },
        EventSubtype::Birth(b) => match b {
            BirthSubtype::Born => 0.3,
            BirthSubtype::Arrived => 0.2,
            BirthSubtype::Created => 0.15,
        },
    }
}

/// Add event-type specific tags
fn add_event_tags(event: &Event, tags: &mut Vec<String>) {
    use crate::events::types::*;

    match &event.subtype {
        EventSubtype::Communication(CommunicationSubtype::SpreadRumor) => {
            tags.push("gossip".to_string());
        }
        EventSubtype::Communication(CommunicationSubtype::Lie) => {
            tags.push("deception".to_string());
        }
        EventSubtype::Communication(CommunicationSubtype::Confess) => {
            tags.push("confession".to_string());
        }
        EventSubtype::Resource(ResourceSubtype::Steal) => {
            tags.push("theft".to_string());
            tags.push("crime".to_string());
        }
        EventSubtype::Faction(FactionSubtype::Leave) => {
            tags.push("defection".to_string());
            tags.push("faction_critical".to_string());
        }
        EventSubtype::Faction(FactionSubtype::ChallengeLeader) => {
            tags.push("succession_crisis".to_string());
            tags.push("faction_critical".to_string());
        }
        EventSubtype::Faction(FactionSubtype::Exile) => {
            tags.push("exile".to_string());
            tags.push("faction_critical".to_string());
        }
        EventSubtype::Conflict(ConflictSubtype::Fight) => {
            tags.push("violence".to_string());
        }
        EventSubtype::Conflict(ConflictSubtype::Duel) => {
            tags.push("violence".to_string());
            tags.push("duel".to_string());
        }
        EventSubtype::Conflict(ConflictSubtype::Assassination) => {
            tags.push("death".to_string());
            tags.push("assassination".to_string());
            tags.push("faction_critical".to_string());
        }
        EventSubtype::Archive(ArchiveSubtype::DestroyEntry) => {
            tags.push("history_destroyed".to_string());
        }
        EventSubtype::Archive(ArchiveSubtype::ForgeEntry) => {
            tags.push("forgery".to_string());
            tags.push("deception".to_string());
        }
        EventSubtype::Betrayal(BetrayalSubtype::SecretSharedWithEnemy) => {
            tags.push("secret_shared".to_string());
            tags.push("betrayal".to_string());
        }
        EventSubtype::Betrayal(BetrayalSubtype::Defection) => {
            tags.push("defection".to_string());
            tags.push("betrayal".to_string());
        }
        EventSubtype::Betrayal(BetrayalSubtype::Sabotage) => {
            tags.push("sabotage".to_string());
            tags.push("betrayal".to_string());
        }
        EventSubtype::Death(_) => {
            tags.push("death".to_string());
        }
        _ => {}
    }

    // Add tags from outcome if present
    if let EventOutcome::Relationship(ref rel) = event.outcome {
        for change in &rel.relationship_changes {
            if change.new_value < change.old_value - 0.1 {
                tags.push("trust_damaged".to_string());
            } else if change.new_value > change.old_value + 0.1 {
                tags.push("trust_improved".to_string());
            }
        }
    }
}

/// Calculate drama score with connected events context
pub fn calculate_drama_with_context(
    event: &Event,
    _connected_events: &[&Event],
) -> DramaAnalysis {
    let mut analysis = calculate_drama_score(event);

    // If this event is part of a chain, boost the score
    if !event.connected_events.is_empty() {
        analysis.score *= multipliers::RECURRING_PATTERN;
        analysis.tags.push("narrative_chain".to_string());
    }

    analysis.score = analysis.score.clamp(0.0, 1.0);
    analysis
}

/// Check if an event meets a drama threshold for logging/highlighting
pub fn is_highly_dramatic(event: &Event, threshold: f32) -> bool {
    calculate_drama_score(event).score >= threshold
}

/// Get all events above a drama threshold
pub fn filter_dramatic_events(events: &[Event], threshold: f32) -> Vec<&Event> {
    events
        .iter()
        .filter(|e| is_highly_dramatic(e, threshold))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::types::*;

    fn create_test_event(event_type: EventType, subtype: EventSubtype) -> Event {
        Event {
            event_id: "test_001".to_string(),
            timestamp: EventTimestamp {
                tick: 100,
                date: "year_1.spring.day_1".to_string(),
            },
            event_type,
            subtype,
            actors: EventActors {
                primary: ActorSnapshot {
                    agent_id: "agent_001".to_string(),
                    name: "Test Agent".to_string(),
                    faction: "thornwood".to_string(),
                    role: "member".to_string(),
                    location: "forest_clearing".to_string(),
                },
                secondary: None,
                affected: None,
            },
            context: EventContext {
                trigger: "test".to_string(),
                preconditions: Vec::new(),
                location_description: None,
            },
            outcome: EventOutcome::General(GeneralOutcome {
                description: None,
                state_changes: Vec::new(),
            }),
            drama_tags: Vec::new(),
            drama_score: 0.0,
            connected_events: Vec::new(),
        }
    }

    #[test]
    fn test_movement_low_drama() {
        let event = create_test_event(
            EventType::Movement,
            EventSubtype::Movement(MovementSubtype::Travel),
        );
        let analysis = calculate_drama_score(&event);
        assert!(analysis.score < 0.2, "Travel should be low drama");
    }

    #[test]
    fn test_assassination_high_drama() {
        let event = create_test_event(
            EventType::Conflict,
            EventSubtype::Conflict(ConflictSubtype::Assassination),
        );
        let analysis = calculate_drama_score(&event);
        assert!(analysis.score > 0.9, "Assassination should be very high drama");
        assert!(analysis.tags.contains(&"assassination".to_string()));
        assert!(analysis.tags.contains(&"death".to_string()));
    }

    #[test]
    fn test_leader_involvement_multiplier() {
        let mut event = create_test_event(
            EventType::Communication,
            EventSubtype::Communication(CommunicationSubtype::ShareMemory),
        );

        let base_analysis = calculate_drama_score(&event);

        event.actors.primary.role = "leader".to_string();
        let leader_analysis = calculate_drama_score(&event);

        assert!(
            leader_analysis.score > base_analysis.score,
            "Leader involvement should increase drama"
        );
        assert!(leader_analysis.tags.contains(&"leader_involved".to_string()));
    }

    #[test]
    fn test_cross_faction_multiplier() {
        let mut event = create_test_event(
            EventType::Communication,
            EventSubtype::Communication(CommunicationSubtype::ShareMemory),
        );

        event.actors.secondary = Some(ActorSnapshot {
            agent_id: "agent_002".to_string(),
            name: "Other Agent".to_string(),
            faction: "ironmere".to_string(), // Different faction
            role: "member".to_string(),
            location: "forest_clearing".to_string(),
        });

        let analysis = calculate_drama_score(&event);
        assert!(analysis.tags.contains(&"cross_faction".to_string()));
    }

    #[test]
    fn test_defection_drama() {
        let event = create_test_event(
            EventType::Faction,
            EventSubtype::Faction(FactionSubtype::Leave),
        );
        let analysis = calculate_drama_score(&event);
        assert!(analysis.score >= 0.7, "Defection should be high drama");
        assert!(analysis.tags.contains(&"defection".to_string()));
        assert!(analysis.tags.contains(&"faction_critical".to_string()));
    }

    #[test]
    fn test_filter_dramatic_events() {
        let low_drama = create_test_event(
            EventType::Movement,
            EventSubtype::Movement(MovementSubtype::Travel),
        );
        let high_drama = create_test_event(
            EventType::Faction,
            EventSubtype::Faction(FactionSubtype::ChallengeLeader),
        );

        let events = vec![low_drama, high_drama];
        let dramatic = filter_dramatic_events(&events, 0.5);

        assert_eq!(dramatic.len(), 1);
    }
}
