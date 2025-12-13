//! Social Actions
//!
//! Actions for building relationships, seeking favor, and social manipulation.

use serde::{Deserialize, Serialize};

/// Type of social action
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SocialActionType {
    /// Slowly build trust through positive interaction
    BuildTrust,
    /// Target higher status agents for favor
    CurryFavor,
    /// Give gift for faster trust building (costs resources)
    Gift,
    /// Reduce target's social belonging
    Ostracize,
}

/// A social action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialAction {
    pub actor_id: String,
    pub action_type: SocialActionType,
    pub target_id: String,
    /// Optional resource cost for gift actions
    pub resource_cost: Option<u32>,
}

impl SocialAction {
    /// Create a build trust action
    pub fn build_trust(actor_id: impl Into<String>, target_id: impl Into<String>) -> Self {
        Self {
            actor_id: actor_id.into(),
            action_type: SocialActionType::BuildTrust,
            target_id: target_id.into(),
            resource_cost: None,
        }
    }

    /// Create a curry favor action (targeting higher status agent)
    pub fn curry_favor(actor_id: impl Into<String>, target_id: impl Into<String>) -> Self {
        Self {
            actor_id: actor_id.into(),
            action_type: SocialActionType::CurryFavor,
            target_id: target_id.into(),
            resource_cost: None,
        }
    }

    /// Create a gift action
    pub fn gift(
        actor_id: impl Into<String>,
        target_id: impl Into<String>,
        resource_cost: u32,
    ) -> Self {
        Self {
            actor_id: actor_id.into(),
            action_type: SocialActionType::Gift,
            target_id: target_id.into(),
            resource_cost: Some(resource_cost),
        }
    }

    /// Create an ostracize action
    pub fn ostracize(actor_id: impl Into<String>, target_id: impl Into<String>) -> Self {
        Self {
            actor_id: actor_id.into(),
            action_type: SocialActionType::Ostracize,
            target_id: target_id.into(),
            resource_cost: None,
        }
    }
}

/// Weight constants for social actions
pub mod social_weights {
    /// Base weight for build trust action
    pub const BUILD_TRUST_BASE: f32 = 0.25;
    /// Bonus when social belonging is peripheral/isolated
    pub const BUILD_TRUST_BELONGING_BONUS: f32 = 0.15;
    /// Bonus based on sociability trait
    pub const BUILD_TRUST_SOCIABILITY_MULT: f32 = 0.2;
    /// Bonus for existing positive relationship
    pub const BUILD_TRUST_EXISTING_BONUS: f32 = 0.1;

    /// Base weight for curry favor action
    pub const CURRY_FAVOR_BASE: f32 = 0.15;
    /// Bonus based on ambition trait
    pub const CURRY_FAVOR_AMBITION_MULT: f32 = 0.25;
    /// Bonus for targeting leader
    pub const CURRY_FAVOR_LEADER_BONUS: f32 = 0.15;
    /// Bonus for targeting council members
    pub const CURRY_FAVOR_COUNCIL_BONUS: f32 = 0.1;

    /// Base weight for gift action
    pub const GIFT_BASE: f32 = 0.1;
    /// Bonus for trying to repair relationship
    pub const GIFT_REPAIR_BONUS: f32 = 0.2;
    /// Penalty when resources are low
    pub const GIFT_LOW_RESOURCE_PENALTY: f32 = 0.15;
    /// Standard gift cost
    pub const GIFT_STANDARD_COST: u32 = 3;

    /// Base weight for ostracize action
    pub const OSTRACIZE_BASE: f32 = 0.05;
    /// Bonus for negative relationship with target
    pub const OSTRACIZE_GRUDGE_BONUS: f32 = 0.2;
    /// Bonus for higher status than target
    pub const OSTRACIZE_STATUS_BONUS: f32 = 0.1;
    /// Penalty for high honesty
    pub const OSTRACIZE_HONESTY_PENALTY: f32 = 0.1;

    /// Trust gained from build trust action
    pub const BUILD_TRUST_GAIN: f32 = 0.03;
    /// Trust gained from curry favor action
    pub const CURRY_FAVOR_GAIN: f32 = 0.02;
    /// Trust gained from gift action
    pub const GIFT_TRUST_GAIN: f32 = 0.08;
    /// Belonging reduction from ostracize (on target)
    pub const OSTRACIZE_BELONGING_IMPACT: f32 = 0.1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_trust_action() {
        let action = SocialAction::build_trust("agent_001", "agent_002");
        assert_eq!(action.action_type, SocialActionType::BuildTrust);
        assert_eq!(action.target_id, "agent_002");
        assert!(action.resource_cost.is_none());
    }

    #[test]
    fn test_curry_favor_action() {
        let action = SocialAction::curry_favor("agent_001", "leader_001");
        assert_eq!(action.action_type, SocialActionType::CurryFavor);
        assert_eq!(action.target_id, "leader_001");
    }

    #[test]
    fn test_gift_action() {
        let action = SocialAction::gift("agent_001", "agent_002", 5);
        assert_eq!(action.action_type, SocialActionType::Gift);
        assert_eq!(action.resource_cost, Some(5));
    }

    #[test]
    fn test_ostracize_action() {
        let action = SocialAction::ostracize("agent_001", "agent_003");
        assert_eq!(action.action_type, SocialActionType::Ostracize);
        assert_eq!(action.target_id, "agent_003");
    }
}
