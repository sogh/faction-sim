//! Movement Actions
//!
//! Actions for agent movement between locations.

use serde::{Deserialize, Serialize};

/// A movement action to travel to an adjacent location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveAction {
    /// The agent performing the movement
    pub agent_id: String,
    /// The destination location
    pub destination: String,
    /// The type of movement
    pub movement_type: MovementType,
}

/// Types of movement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MovementType {
    /// Normal travel between adjacent locations
    Travel,
    /// Fleeing from danger
    Flee,
    /// Pursuing a target
    Pursue,
    /// Patrol duty (for scouts)
    Patrol,
    /// Returning to faction HQ
    ReturnHome,
}

impl MoveAction {
    /// Create a new travel action
    pub fn travel(agent_id: impl Into<String>, destination: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            destination: destination.into(),
            movement_type: MovementType::Travel,
        }
    }

    /// Create a return home action
    pub fn return_home(agent_id: impl Into<String>, hq_location: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            destination: hq_location.into(),
            movement_type: MovementType::ReturnHome,
        }
    }

    /// Create a patrol action
    pub fn patrol(agent_id: impl Into<String>, destination: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            destination: destination.into(),
            movement_type: MovementType::Patrol,
        }
    }

    /// Create a flee action
    pub fn flee(agent_id: impl Into<String>, destination: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            destination: destination.into(),
            movement_type: MovementType::Flee,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_action_creation() {
        let action = MoveAction::travel("agent_001", "village");
        assert_eq!(action.agent_id, "agent_001");
        assert_eq!(action.destination, "village");
        assert_eq!(action.movement_type, MovementType::Travel);

        let home_action = MoveAction::return_home("agent_002", "thornwood_hall");
        assert_eq!(home_action.movement_type, MovementType::ReturnHome);
    }
}
