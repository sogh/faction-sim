//! Communication Actions
//!
//! Actions for sharing information, spreading rumors, and lying.

use serde::{Deserialize, Serialize};

/// Type of communication action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommunicationType {
    /// Share a memory truthfully
    ShareMemory,
    /// Spread a rumor (may distort)
    SpreadRumor,
    /// Tell a lie (create false memory)
    Lie,
    /// Confess a secret
    Confess,
}

/// Whether targeting an individual or a group
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetMode {
    /// One-on-one communication
    Individual,
    /// Addressing a group at the location
    Group,
}

/// A communication action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationAction {
    /// Who is communicating
    pub actor_id: String,
    /// Type of communication
    pub communication_type: CommunicationType,
    /// Target mode
    pub target_mode: TargetMode,
    /// Primary target (for individual) or location (for group)
    pub target_id: String,
    /// Memory being shared (if applicable)
    pub memory_id: Option<String>,
    /// Subject of the communication (for lies/rumors)
    pub subject_id: Option<String>,
    /// Content (for lies - the false claim)
    pub content: Option<String>,
}

impl CommunicationAction {
    /// Create a share memory action
    pub fn share_memory(
        actor_id: impl Into<String>,
        target_id: impl Into<String>,
        memory_id: impl Into<String>,
        target_mode: TargetMode,
    ) -> Self {
        Self {
            actor_id: actor_id.into(),
            communication_type: CommunicationType::ShareMemory,
            target_mode,
            target_id: target_id.into(),
            memory_id: Some(memory_id.into()),
            subject_id: None,
            content: None,
        }
    }

    /// Create a spread rumor action
    pub fn spread_rumor(
        actor_id: impl Into<String>,
        target_id: impl Into<String>,
        memory_id: impl Into<String>,
        target_mode: TargetMode,
    ) -> Self {
        Self {
            actor_id: actor_id.into(),
            communication_type: CommunicationType::SpreadRumor,
            target_mode,
            target_id: target_id.into(),
            memory_id: Some(memory_id.into()),
            subject_id: None,
            content: None,
        }
    }

    /// Create a lie action
    pub fn lie(
        actor_id: impl Into<String>,
        target_id: impl Into<String>,
        subject_id: impl Into<String>,
        false_content: impl Into<String>,
    ) -> Self {
        Self {
            actor_id: actor_id.into(),
            communication_type: CommunicationType::Lie,
            target_mode: TargetMode::Individual, // Lies are always 1-on-1
            target_id: target_id.into(),
            memory_id: None,
            subject_id: Some(subject_id.into()),
            content: Some(false_content.into()),
        }
    }

    /// Create a confession action
    pub fn confess(
        actor_id: impl Into<String>,
        target_id: impl Into<String>,
        memory_id: impl Into<String>,
    ) -> Self {
        Self {
            actor_id: actor_id.into(),
            communication_type: CommunicationType::Confess,
            target_mode: TargetMode::Individual, // Confessions are always 1-on-1
            target_id: target_id.into(),
            memory_id: Some(memory_id.into()),
            subject_id: None,
            content: None,
        }
    }

    /// Check if this is individual communication
    pub fn is_individual(&self) -> bool {
        self.target_mode == TargetMode::Individual
    }

    /// Check if this is group communication
    pub fn is_group(&self) -> bool {
        self.target_mode == TargetMode::Group
    }
}

/// Score for potential interaction targets
#[derive(Debug, Clone)]
pub struct TargetScore {
    pub agent_id: String,
    pub agent_name: String,
    pub score: f32,
    pub reasons: Vec<String>,
}

impl TargetScore {
    pub fn new(agent_id: impl Into<String>, agent_name: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            agent_name: agent_name.into(),
            score: 1.0,
            reasons: Vec::new(),
        }
    }

    pub fn multiply(&mut self, factor: f32, reason: impl Into<String>) {
        self.score *= factor;
        self.reasons.push(reason.into());
    }

    pub fn add(&mut self, delta: f32, reason: impl Into<String>) {
        self.score += delta;
        self.reasons.push(reason.into());
    }
}

/// Target selection modifiers based on behavioral rules
pub mod target_selection {
    /// Same faction bonus
    pub const SAME_FACTION: f32 = 2.0;
    /// Neutral faction
    pub const NEUTRAL_FACTION: f32 = 1.0;
    /// Enemy faction penalty
    pub const ENEMY_FACTION: f32 = 0.3;
    /// Enemy at neutral territory (less penalty)
    pub const ENEMY_AT_NEUTRAL: f32 = 0.8;

    /// Target has higher status
    pub fn higher_status_modifier(status_diff: i32) -> f32 {
        1.0 + 0.5 * status_diff as f32
    }
    /// Same status
    pub const SAME_STATUS: f32 = 1.0;
    /// Lower status
    pub const LOWER_STATUS: f32 = 0.7;

    /// Existing positive relationship
    pub const POSITIVE_RELATIONSHIP: f32 = 1.3;
    /// Existing negative relationship
    pub const NEGATIVE_RELATIONSHIP: f32 = 0.4;
    /// No existing relationship
    pub const NO_RELATIONSHIP: f32 = 1.0;

    /// Target relevant to active goal
    pub const GOAL_RELEVANT: f32 = 1.5;

    /// Recently spoke (avoid repetition)
    pub const SPOKE_THIS_TICK: f32 = 0.1;
    /// Spoke recently
    pub const SPOKE_RECENTLY: f32 = 0.7;
    /// Haven't spoken in a long time
    pub const LONG_TIME_NO_SPEAK: f32 = 1.2;
}

/// Communication weight modifiers
pub mod communication_weights {
    /// Base weight for gossip/sharing
    pub const GOSSIP_BASE: f32 = 0.4;

    /// Sociability bonus at max sociability
    pub const SOCIABILITY_BONUS: f32 = 0.4;

    /// Bonus for negative memory about third party
    pub const NEGATIVE_GOSSIP_BONUS: f32 = 0.2;

    /// Bonus for same faction listener
    pub const SAME_FACTION_BONUS: f32 = 0.2;

    /// Relationship impact for individual communication
    pub const INDIVIDUAL_RELATIONSHIP_MULTIPLIER: f32 = 1.5;

    /// Relationship impact for group communication
    pub const GROUP_RELATIONSHIP_MULTIPLIER: f32 = 0.5;

    /// Fidelity reduction for group communication (noisier)
    pub const GROUP_FIDELITY_MULTIPLIER: f32 = 0.9;

    /// Fidelity reduction per hop in source chain
    pub const SECONDHAND_FIDELITY_MULTIPLIER: f32 = 0.7;

    /// Emotional weight reduction for secondhand
    pub const SECONDHAND_EMOTIONAL_MULTIPLIER: f32 = 0.5;
}

/// Result of a communication action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationOutcome {
    /// Was the communication successful?
    pub success: bool,
    /// Agents who received the information
    pub recipients: Vec<String>,
    /// Memories created in recipients
    pub memories_created: Vec<String>,
    /// Trust changes (agent_id, dimension, delta)
    pub trust_changes: Vec<(String, String, f32)>,
    /// Any distortion that occurred (for rumors)
    pub distortion: Option<String>,
}

impl CommunicationOutcome {
    pub fn new(success: bool) -> Self {
        Self {
            success,
            recipients: Vec::new(),
            memories_created: Vec::new(),
            trust_changes: Vec::new(),
            distortion: None,
        }
    }

    pub fn add_recipient(&mut self, agent_id: impl Into<String>) {
        self.recipients.push(agent_id.into());
    }

    pub fn add_memory(&mut self, memory_id: impl Into<String>) {
        self.memories_created.push(memory_id.into());
    }

    pub fn add_trust_change(&mut self, agent_id: impl Into<String>, dimension: impl Into<String>, delta: f32) {
        self.trust_changes.push((agent_id.into(), dimension.into(), delta));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_share_memory_action() {
        let action = CommunicationAction::share_memory(
            "agent_1",
            "agent_2",
            "mem_001",
            TargetMode::Individual,
        );

        assert_eq!(action.communication_type, CommunicationType::ShareMemory);
        assert!(action.is_individual());
        assert_eq!(action.memory_id, Some("mem_001".to_string()));
    }

    #[test]
    fn test_lie_action() {
        let action = CommunicationAction::lie(
            "agent_1",
            "agent_2",
            "agent_3",
            "Agent 3 stole from the stores",
        );

        assert_eq!(action.communication_type, CommunicationType::Lie);
        assert!(action.is_individual()); // Lies are always 1-on-1
        assert_eq!(action.subject_id, Some("agent_3".to_string()));
    }

    #[test]
    fn test_target_score() {
        let mut score = TargetScore::new("agent_1", "Alice");
        score.multiply(target_selection::SAME_FACTION, "same faction");
        score.add(0.3, "high status");

        assert!(score.score > 1.0);
        assert_eq!(score.reasons.len(), 2);
    }
}
