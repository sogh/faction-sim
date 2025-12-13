//! Resource Actions
//!
//! Actions related to resource acquisition, trading, and management.

use serde::{Deserialize, Serialize};

/// Type of resource action
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceActionType {
    /// Work to increase faction resources
    Work,
    /// Trade resources with another agent
    Trade,
    /// Steal resources with detection risk
    Steal,
    /// Hoard personal reserves instead of contributing to faction
    Hoard,
}

/// Resource type for trading/stealing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    Grain,
    Iron,
    Salt,
}

/// A resource action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAction {
    pub actor_id: String,
    pub action_type: ResourceActionType,
    /// Target agent for trade/steal actions
    pub target_id: Option<String>,
    /// Resource type being acted upon
    pub resource_type: Option<ResourceType>,
    /// Amount of resource involved
    pub amount: u32,
}

impl ResourceAction {
    /// Create a work action
    pub fn work(actor_id: impl Into<String>) -> Self {
        Self {
            actor_id: actor_id.into(),
            action_type: ResourceActionType::Work,
            target_id: None,
            resource_type: None,
            amount: resource_weights::WORK_BASE_YIELD,
        }
    }

    /// Create a trade action
    pub fn trade(
        actor_id: impl Into<String>,
        target_id: impl Into<String>,
        resource_type: ResourceType,
        amount: u32,
    ) -> Self {
        Self {
            actor_id: actor_id.into(),
            action_type: ResourceActionType::Trade,
            target_id: Some(target_id.into()),
            resource_type: Some(resource_type),
            amount,
        }
    }

    /// Create a steal action
    pub fn steal(
        actor_id: impl Into<String>,
        target_id: impl Into<String>,
        resource_type: ResourceType,
        amount: u32,
    ) -> Self {
        Self {
            actor_id: actor_id.into(),
            action_type: ResourceActionType::Steal,
            target_id: Some(target_id.into()),
            resource_type: Some(resource_type),
            amount,
        }
    }

    /// Create a hoard action
    pub fn hoard(actor_id: impl Into<String>, amount: u32) -> Self {
        Self {
            actor_id: actor_id.into(),
            action_type: ResourceActionType::Hoard,
            target_id: None,
            resource_type: None,
            amount,
        }
    }
}

/// Weight constants for resource actions
pub mod resource_weights {
    /// Base weight for work action
    pub const WORK_BASE: f32 = 0.3;
    /// Weight bonus when food security is stressed
    pub const WORK_STRESSED_BONUS: f32 = 0.2;
    /// Weight bonus when food security is desperate
    pub const WORK_DESPERATE_BONUS: f32 = 0.4;

    /// Base weight for trade action
    pub const TRADE_BASE: f32 = 0.15;
    /// Trade bonus for high trust relationship
    pub const TRADE_TRUST_BONUS: f32 = 0.1;
    /// Trade bonus for cross-faction relations
    pub const TRADE_CROSS_FACTION_BONUS: f32 = 0.05;

    /// Base weight for steal action
    pub const STEAL_BASE: f32 = 0.05;
    /// Steal bonus when desperate
    pub const STEAL_DESPERATE_BONUS: f32 = 0.3;
    /// Steal penalty for high honesty
    pub const STEAL_HONESTY_PENALTY: f32 = 0.2;
    /// Steal bonus for low loyalty
    pub const STEAL_LOW_LOYALTY_BONUS: f32 = 0.1;

    /// Base weight for hoard action
    pub const HOARD_BASE: f32 = 0.05;
    /// Hoard bonus for low loyalty
    pub const HOARD_LOW_LOYALTY_BONUS: f32 = 0.15;
    /// Hoard bonus when stressed
    pub const HOARD_STRESSED_BONUS: f32 = 0.1;
    /// Hoard penalty for high loyalty
    pub const HOARD_HIGH_LOYALTY_PENALTY: f32 = 0.1;

    /// Base yield from work (resource units)
    pub const WORK_BASE_YIELD: u32 = 5;
    /// Detection chance base for stealing
    pub const STEAL_DETECTION_BASE: f32 = 0.3;
    /// Detection bonus from victim's perception
    pub const STEAL_DETECTION_VIGILANCE: f32 = 0.2;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_work_action() {
        let action = ResourceAction::work("agent_001");
        assert_eq!(action.action_type, ResourceActionType::Work);
        assert!(action.target_id.is_none());
        assert_eq!(action.amount, resource_weights::WORK_BASE_YIELD);
    }

    #[test]
    fn test_trade_action() {
        let action = ResourceAction::trade("agent_001", "agent_002", ResourceType::Grain, 10);
        assert_eq!(action.action_type, ResourceActionType::Trade);
        assert_eq!(action.target_id, Some("agent_002".to_string()));
        assert_eq!(action.resource_type, Some(ResourceType::Grain));
        assert_eq!(action.amount, 10);
    }

    #[test]
    fn test_steal_action() {
        let action = ResourceAction::steal("agent_001", "agent_002", ResourceType::Iron, 5);
        assert_eq!(action.action_type, ResourceActionType::Steal);
        assert_eq!(action.target_id, Some("agent_002".to_string()));
    }

    #[test]
    fn test_hoard_action() {
        let action = ResourceAction::hoard("agent_001", 3);
        assert_eq!(action.action_type, ResourceActionType::Hoard);
        assert_eq!(action.amount, 3);
    }
}
