//! Beer Actions
//!
//! Actions related to brewing and consuming beer.

use serde::{Deserialize, Serialize};

/// Type of beer action
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BeerActionType {
    /// Brew beer from grain (2 grain -> 1 beer)
    Brew,
    /// Drink beer from faction stockpile
    Drink,
    /// Share beer with another agent (social action)
    Share,
}

/// A beer-related action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeerAction {
    pub actor_id: String,
    pub action_type: BeerActionType,
    /// Target agent for Share action
    pub target_id: Option<String>,
    /// Amount to brew/drink
    pub amount: u32,
}

impl BeerAction {
    /// Create a brew action
    pub fn brew(actor_id: impl Into<String>, amount: u32) -> Self {
        Self {
            actor_id: actor_id.into(),
            action_type: BeerActionType::Brew,
            target_id: None,
            amount,
        }
    }

    /// Create a drink action
    pub fn drink(actor_id: impl Into<String>) -> Self {
        Self {
            actor_id: actor_id.into(),
            action_type: BeerActionType::Drink,
            target_id: None,
            amount: 1,
        }
    }

    /// Create a share action
    pub fn share(actor_id: impl Into<String>, target_id: impl Into<String>) -> Self {
        Self {
            actor_id: actor_id.into(),
            action_type: BeerActionType::Share,
            target_id: Some(target_id.into()),
            amount: 1,
        }
    }
}

/// Weight constants for beer actions
pub mod beer_weights {
    /// Base weight for brewing
    pub const BREW_BASE: f32 = 0.15;
    /// Bonus when faction has excess grain (> 200 per territory)
    pub const BREW_EXCESS_GRAIN_BONUS: f32 = 0.2;
    /// Penalty when grain is scarce (faction is_critical)
    pub const BREW_SCARCE_GRAIN_PENALTY: f32 = 0.3;
    /// Minimum grain to consider brewing
    pub const BREW_GRAIN_THRESHOLD: u32 = 4;

    /// Base weight for drinking
    pub const DRINK_BASE: f32 = 0.1;
    /// Bonus for sociable agents
    pub const DRINK_SOCIABILITY_BONUS: f32 = 0.15;
    /// Bonus when peripheral/isolated (seeking belonging)
    pub const DRINK_BELONGING_BONUS: f32 = 0.2;
    /// Max intoxication level before drinking less likely
    pub const DRINK_MAX_INTOX: f32 = 0.5;

    /// Base weight for sharing
    pub const SHARE_BASE: f32 = 0.2;
    /// Bonus for high sociability
    pub const SHARE_SOCIABILITY_BONUS: f32 = 0.25;
    /// Trust building gain from sharing
    pub const SHARE_TRUST_GAIN: f32 = 0.08;

    /// Grain cost to brew 1 beer
    pub const GRAIN_PER_BEER: u32 = 2;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brew_action() {
        let action = BeerAction::brew("agent_001", 2);
        assert_eq!(action.action_type, BeerActionType::Brew);
        assert!(action.target_id.is_none());
        assert_eq!(action.amount, 2);
    }

    #[test]
    fn test_drink_action() {
        let action = BeerAction::drink("agent_001");
        assert_eq!(action.action_type, BeerActionType::Drink);
        assert!(action.target_id.is_none());
        assert_eq!(action.amount, 1);
    }

    #[test]
    fn test_share_action() {
        let action = BeerAction::share("agent_001", "agent_002");
        assert_eq!(action.action_type, BeerActionType::Share);
        assert_eq!(action.target_id, Some("agent_002".to_string()));
        assert_eq!(action.amount, 1);
    }
}
