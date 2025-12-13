//! Faction Actions
//!
//! Actions related to faction membership, leadership, and political maneuvering.

use serde::{Deserialize, Serialize};

/// Type of faction action
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FactionActionType {
    /// Leave current faction and join another
    Defect,
    /// Remove another agent from the faction (requires authority)
    Exile,
    /// Challenge the current leader for leadership
    ChallengeLeader,
    /// Support the current leader against challengers
    SupportLeader,
}

/// A faction action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionAction {
    pub actor_id: String,
    pub action_type: FactionActionType,
    /// Current faction for defect/challenge/support, target agent for exile
    pub target_id: String,
    /// New faction for defect action
    pub new_faction_id: Option<String>,
}

impl FactionAction {
    /// Create a defect action
    pub fn defect(
        actor_id: impl Into<String>,
        current_faction: impl Into<String>,
        new_faction: impl Into<String>,
    ) -> Self {
        Self {
            actor_id: actor_id.into(),
            action_type: FactionActionType::Defect,
            target_id: current_faction.into(),
            new_faction_id: Some(new_faction.into()),
        }
    }

    /// Create an exile action
    pub fn exile(
        actor_id: impl Into<String>,
        target_agent: impl Into<String>,
    ) -> Self {
        Self {
            actor_id: actor_id.into(),
            action_type: FactionActionType::Exile,
            target_id: target_agent.into(),
            new_faction_id: None,
        }
    }

    /// Create a challenge leader action
    pub fn challenge_leader(
        actor_id: impl Into<String>,
        faction_id: impl Into<String>,
    ) -> Self {
        Self {
            actor_id: actor_id.into(),
            action_type: FactionActionType::ChallengeLeader,
            target_id: faction_id.into(),
            new_faction_id: None,
        }
    }

    /// Create a support leader action
    pub fn support_leader(
        actor_id: impl Into<String>,
        faction_id: impl Into<String>,
    ) -> Self {
        Self {
            actor_id: actor_id.into(),
            action_type: FactionActionType::SupportLeader,
            target_id: faction_id.into(),
            new_faction_id: None,
        }
    }
}

/// Weight constants for faction actions
pub mod faction_weights {
    /// Base weight for defect action (very low - rare action)
    pub const DEFECT_BASE: f32 = 0.02;
    /// Defect bonus for low loyalty trait
    pub const DEFECT_LOW_LOYALTY_BONUS: f32 = 0.15;
    /// Defect bonus for isolated social belonging
    pub const DEFECT_ISOLATED_BONUS: f32 = 0.2;
    /// Defect bonus for negative trust toward leader
    pub const DEFECT_LEADER_DISTRUST_BONUS: f32 = 0.15;
    /// Defect bonus for having cross-faction contacts
    pub const DEFECT_EXTERNAL_CONTACT_BONUS: f32 = 0.1;
    /// Defect penalty for high loyalty
    pub const DEFECT_HIGH_LOYALTY_PENALTY: f32 = 0.2;

    /// Base weight for exile action (requires leader/council role)
    pub const EXILE_BASE: f32 = 0.05;
    /// Exile bonus for negative trust toward target
    pub const EXILE_DISTRUST_BONUS: f32 = 0.2;
    /// Exile bonus if target has betrayed faction
    pub const EXILE_BETRAYAL_BONUS: f32 = 0.3;

    /// Base weight for challenge leader action (very low - dramatic action)
    pub const CHALLENGE_BASE: f32 = 0.01;
    /// Challenge bonus for high ambition
    pub const CHALLENGE_AMBITION_MULT: f32 = 0.3;
    /// Challenge bonus for having supporter count
    pub const CHALLENGE_SUPPORT_MULT: f32 = 0.1;
    /// Challenge bonus for weak leader (low trust from faction)
    pub const CHALLENGE_WEAK_LEADER_BONUS: f32 = 0.2;
    /// Challenge penalty for high loyalty
    pub const CHALLENGE_HIGH_LOYALTY_PENALTY: f32 = 0.15;

    /// Base weight for support leader action
    pub const SUPPORT_BASE: f32 = 0.15;
    /// Support bonus for high loyalty
    pub const SUPPORT_HIGH_LOYALTY_BONUS: f32 = 0.15;
    /// Support bonus for positive trust toward leader
    pub const SUPPORT_LEADER_TRUST_BONUS: f32 = 0.1;
    /// Support bonus when leader is being challenged
    pub const SUPPORT_CHALLENGE_ACTIVE_BONUS: f32 = 0.25;

    /// Minimum supporters needed to challenge
    pub const CHALLENGE_MIN_SUPPORTERS: usize = 2;
    /// Trust threshold considered "weak" leadership
    pub const WEAK_LEADER_TRUST_THRESHOLD: f32 = 0.2;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defect_action() {
        let action = FactionAction::defect("agent_001", "thornwood", "ironmere");
        assert_eq!(action.action_type, FactionActionType::Defect);
        assert_eq!(action.target_id, "thornwood");
        assert_eq!(action.new_faction_id, Some("ironmere".to_string()));
    }

    #[test]
    fn test_exile_action() {
        let action = FactionAction::exile("leader_001", "agent_traitor");
        assert_eq!(action.action_type, FactionActionType::Exile);
        assert_eq!(action.target_id, "agent_traitor");
        assert!(action.new_faction_id.is_none());
    }

    #[test]
    fn test_challenge_leader_action() {
        let action = FactionAction::challenge_leader("ambitious_001", "thornwood");
        assert_eq!(action.action_type, FactionActionType::ChallengeLeader);
        assert_eq!(action.target_id, "thornwood");
    }

    #[test]
    fn test_support_leader_action() {
        let action = FactionAction::support_leader("loyal_001", "thornwood");
        assert_eq!(action.action_type, FactionActionType::SupportLeader);
        assert_eq!(action.target_id, "thornwood");
    }
}
