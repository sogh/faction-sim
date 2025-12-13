//! Conflict Actions
//!
//! Actions for interpersonal conflict, from arguments to violence.

use serde::{Deserialize, Serialize};

/// Type of conflict action
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictActionType {
    /// Verbal argument - damages relationship, possible resolution
    Argue,
    /// Physical fight - causes harm, high relationship damage
    Fight,
    /// Sabotage target's resources or reputation
    Sabotage,
    /// Kill target (high risk, requires extreme conditions)
    Assassinate,
}

/// A conflict action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictAction {
    pub actor_id: String,
    pub action_type: ConflictActionType,
    pub target_id: String,
    /// Optional reason/motivation for the conflict
    pub reason: Option<String>,
    /// Related goal that triggered this conflict
    pub related_goal: Option<String>,
}

impl ConflictAction {
    /// Create an argue action
    pub fn argue(
        actor_id: impl Into<String>,
        target_id: impl Into<String>,
        reason: Option<String>,
    ) -> Self {
        Self {
            actor_id: actor_id.into(),
            action_type: ConflictActionType::Argue,
            target_id: target_id.into(),
            reason,
            related_goal: None,
        }
    }

    /// Create a fight action
    pub fn fight(
        actor_id: impl Into<String>,
        target_id: impl Into<String>,
        reason: Option<String>,
    ) -> Self {
        Self {
            actor_id: actor_id.into(),
            action_type: ConflictActionType::Fight,
            target_id: target_id.into(),
            reason,
            related_goal: None,
        }
    }

    /// Create a sabotage action
    pub fn sabotage(
        actor_id: impl Into<String>,
        target_id: impl Into<String>,
        reason: Option<String>,
    ) -> Self {
        Self {
            actor_id: actor_id.into(),
            action_type: ConflictActionType::Sabotage,
            target_id: target_id.into(),
            reason,
            related_goal: None,
        }
    }

    /// Create an assassinate action
    pub fn assassinate(
        actor_id: impl Into<String>,
        target_id: impl Into<String>,
        related_goal: impl Into<String>,
    ) -> Self {
        Self {
            actor_id: actor_id.into(),
            action_type: ConflictActionType::Assassinate,
            target_id: target_id.into(),
            reason: Some("deadly intent".to_string()),
            related_goal: Some(related_goal.into()),
        }
    }

    /// Builder method to add related goal
    pub fn with_goal(mut self, goal: impl Into<String>) -> Self {
        self.related_goal = Some(goal.into());
        self
    }
}

/// Weight constants for conflict actions
pub mod conflict_weights {
    /// Base weight for argue action
    pub const ARGUE_BASE: f32 = 0.1;
    /// Argue bonus for negative relationship
    pub const ARGUE_NEGATIVE_REL_BONUS: f32 = 0.15;
    /// Argue bonus for grudge/revenge goal
    pub const ARGUE_GRUDGE_BONUS: f32 = 0.2;
    /// Argue bonus based on boldness trait
    pub const ARGUE_BOLDNESS_MULT: f32 = 0.1;

    /// Base weight for fight action (low - violent action)
    pub const FIGHT_BASE: f32 = 0.02;
    /// Fight bonus for existing argument/escalation
    pub const FIGHT_ESCALATION_BONUS: f32 = 0.15;
    /// Fight bonus for revenge goal
    pub const FIGHT_REVENGE_BONUS: f32 = 0.2;
    /// Fight bonus based on boldness trait
    pub const FIGHT_BOLDNESS_MULT: f32 = 0.15;
    /// Fight penalty for low boldness
    pub const FIGHT_LOW_BOLDNESS_PENALTY: f32 = 0.1;

    /// Base weight for sabotage action (low - sneaky action)
    pub const SABOTAGE_BASE: f32 = 0.03;
    /// Sabotage bonus for revenge goal
    pub const SABOTAGE_REVENGE_BONUS: f32 = 0.2;
    /// Sabotage penalty for high honesty
    pub const SABOTAGE_HONESTY_PENALTY: f32 = 0.15;
    /// Sabotage bonus for negative relationship
    pub const SABOTAGE_NEGATIVE_REL_BONUS: f32 = 0.1;

    /// Base weight for assassinate action (very low - extreme action)
    pub const ASSASSINATE_BASE: f32 = 0.001;
    /// Assassinate bonus for active revenge goal
    pub const ASSASSINATE_REVENGE_BONUS: f32 = 0.1;
    /// Assassinate bonus for desperate need state
    pub const ASSASSINATE_DESPERATE_BONUS: f32 = 0.05;
    /// Assassinate bonus for isolated social state
    pub const ASSASSINATE_ISOLATED_BONUS: f32 = 0.03;
    /// Assassinate requires critical trust level
    pub const ASSASSINATE_MIN_DISTRUST: f32 = -0.5;

    /// Relationship damage from argument
    pub const ARGUE_RELATIONSHIP_DAMAGE: f32 = 0.05;
    /// Relationship damage from fight
    pub const FIGHT_RELATIONSHIP_DAMAGE: f32 = 0.2;
    /// Relationship damage from sabotage (if discovered)
    pub const SABOTAGE_RELATIONSHIP_DAMAGE: f32 = 0.3;
    /// Detection chance for sabotage
    pub const SABOTAGE_DETECTION_CHANCE: f32 = 0.4;
    /// Fight resolution chance (argument resolves conflict)
    pub const ARGUE_RESOLUTION_CHANCE: f32 = 0.3;
    /// Fight success rate modifier based on capability
    pub const FIGHT_CAPABILITY_MODIFIER: f32 = 0.3;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argue_action() {
        let action = ConflictAction::argue(
            "agent_001",
            "agent_002",
            Some("resource dispute".to_string()),
        );
        assert_eq!(action.action_type, ConflictActionType::Argue);
        assert_eq!(action.target_id, "agent_002");
        assert_eq!(action.reason, Some("resource dispute".to_string()));
    }

    #[test]
    fn test_fight_action() {
        let action = ConflictAction::fight("agent_001", "agent_002", None);
        assert_eq!(action.action_type, ConflictActionType::Fight);
        assert!(action.reason.is_none());
    }

    #[test]
    fn test_sabotage_action() {
        let action = ConflictAction::sabotage(
            "agent_001",
            "agent_002",
            Some("jealousy".to_string()),
        );
        assert_eq!(action.action_type, ConflictActionType::Sabotage);
    }

    #[test]
    fn test_assassinate_action() {
        let action = ConflictAction::assassinate("agent_001", "enemy_001", "revenge_goal_123");
        assert_eq!(action.action_type, ConflictActionType::Assassinate);
        assert_eq!(action.related_goal, Some("revenge_goal_123".to_string()));
    }

    #[test]
    fn test_with_goal_builder() {
        let action = ConflictAction::argue("a", "b", None).with_goal("test_goal");
        assert_eq!(action.related_goal, Some("test_goal".to_string()));
    }
}
