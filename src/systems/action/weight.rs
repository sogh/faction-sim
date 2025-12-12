//! Action Weighting System
//!
//! Weights actions based on agent traits, needs, and circumstances.

use bevy_ecs::prelude::*;

use crate::actions::movement::MovementType;
use crate::actions::communication::{CommunicationType, TargetMode, communication_weights};
use crate::components::agent::{AgentId, FoodSecurity, Needs, Role, SocialBelonging, Traits};
use crate::components::faction::{FactionMembership, FactionRegistry};
use crate::components::social::{MemoryValence, RelationshipGraph};
use crate::components::world::Position;

use super::generate::{Action, PendingActions, WeightedAction};

/// Apply trait-based weight modifiers to pending actions
pub fn apply_trait_weights(
    mut pending_actions: ResMut<PendingActions>,
    query: Query<(&AgentId, &Traits, &Needs, &FactionMembership, &Position)>,
) {
    for (agent_id, traits, needs, membership, position) in query.iter() {
        let Some(actions) = pending_actions.actions.get_mut(&agent_id.0) else {
            continue;
        };

        for weighted_action in actions.iter_mut() {
            let modifier = calculate_weight_modifier(
                &weighted_action.action,
                traits,
                needs,
                membership,
            );
            weighted_action.weight *= modifier;

            // Clamp weight to reasonable range
            weighted_action.weight = weighted_action.weight.clamp(0.01, 10.0);
        }
    }
}

/// Calculate weight modifier based on agent state
fn calculate_weight_modifier(
    action: &Action,
    traits: &Traits,
    needs: &Needs,
    membership: &FactionMembership,
) -> f32 {
    match action {
        Action::Move(move_action) => {
            calculate_movement_modifier(move_action.movement_type, traits, needs, membership)
        }
        Action::Communicate(comm_action) => {
            calculate_communication_modifier(comm_action, traits, needs, membership)
        }
        Action::Idle => calculate_idle_modifier(traits, needs),
    }
}

/// Calculate communication action weight modifier
fn calculate_communication_modifier(
    action: &crate::actions::communication::CommunicationAction,
    traits: &Traits,
    needs: &Needs,
    _membership: &FactionMembership,
) -> f32 {
    let mut modifier = 1.0;

    match action.communication_type {
        CommunicationType::ShareMemory | CommunicationType::SpreadRumor => {
            // Sociability is the primary driver of gossip
            // At sociability=1.0, bonus is +0.4 (from behavioral rules)
            modifier *= 0.6 + traits.sociability * communication_weights::SOCIABILITY_BONUS;

            // Isolated agents might gossip more to try to connect
            if needs.social_belonging == SocialBelonging::Isolated {
                modifier *= 1.3;
            }

            // Group communication is faster but shallower
            if action.target_mode == TargetMode::Group {
                modifier *= 0.8; // Slightly prefer individual
            }
        }
        CommunicationType::Lie => {
            // Lies are primarily driven by low honesty
            // Weight scales inversely with honesty
            modifier *= 1.5 - traits.honesty;

            // Bold agents lie more
            modifier *= 0.7 + traits.boldness * 0.6;
        }
        CommunicationType::Confess => {
            // Confessions are driven by high honesty
            modifier *= 0.5 + traits.honesty;

            // Integrated agents confess more (trust the group)
            if needs.social_belonging == SocialBelonging::Integrated {
                modifier *= 1.5;
            }
        }
    }

    modifier
}

/// Calculate movement action weight modifier
fn calculate_movement_modifier(
    movement_type: MovementType,
    traits: &Traits,
    needs: &Needs,
    membership: &FactionMembership,
) -> f32 {
    let mut modifier = 1.0;

    match movement_type {
        MovementType::Travel => {
            // Bold agents more likely to wander
            modifier *= 0.5 + traits.boldness * 0.5;
            // Sociable agents like to move around
            modifier *= 0.8 + traits.sociability * 0.4;
        }
        MovementType::Patrol => {
            // Scouts should patrol
            if matches!(membership.role, Role::ScoutCaptain) {
                modifier *= 2.0;
            }
            // Loyal agents more likely to patrol
            modifier *= 0.5 + traits.loyalty_weight;
        }
        MovementType::ReturnHome => {
            // Isolated agents strongly want to return home
            if needs.social_belonging == SocialBelonging::Isolated {
                modifier *= 3.0;
            } else if needs.social_belonging == SocialBelonging::Peripheral {
                modifier *= 1.5;
            }
            // Loyal agents more likely to return
            modifier *= 0.5 + traits.loyalty_weight;
            // Less bold agents prefer home
            modifier *= 1.5 - traits.boldness * 0.5;
        }
        MovementType::Flee => {
            // Less bold agents more likely to flee
            modifier *= 2.0 - traits.boldness;
        }
        MovementType::Pursue => {
            // Bold agents more likely to pursue
            modifier *= 0.5 + traits.boldness * 1.5;
        }
    }

    // Desperate agents are more erratic in movement
    if needs.food_security == FoodSecurity::Desperate {
        modifier *= 1.2;
    }

    modifier
}

/// Calculate idle action weight modifier
fn calculate_idle_modifier(traits: &Traits, needs: &Needs) -> f32 {
    let mut modifier = 1.0;

    // Less sociable agents more likely to stay put
    modifier *= 1.5 - traits.sociability * 0.5;

    // Stressed or desperate agents less likely to idle
    if needs.food_security == FoodSecurity::Desperate {
        modifier *= 0.5;
    } else if needs.food_security == FoodSecurity::Stressed {
        modifier *= 0.8;
    }

    // Isolated agents want to move to find others
    if needs.social_belonging == SocialBelonging::Isolated {
        modifier *= 0.5;
    }

    modifier
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_traits() -> Traits {
        Traits::default()
    }

    fn default_needs() -> Needs {
        Needs::default()
    }

    fn default_membership() -> FactionMembership {
        FactionMembership::new("test", Role::Laborer)
    }

    #[test]
    fn test_patrol_modifier_for_scout() {
        let traits = default_traits();
        let needs = default_needs();
        let scout_membership = FactionMembership::new("test", Role::ScoutCaptain);

        let modifier = calculate_movement_modifier(
            MovementType::Patrol,
            &traits,
            &needs,
            &scout_membership,
        );

        // Scouts should have high patrol weight
        assert!(modifier > 1.5);
    }

    #[test]
    fn test_return_home_for_isolated() {
        let traits = default_traits();
        let mut needs = Needs::default();
        needs.social_belonging = SocialBelonging::Isolated;
        let membership = default_membership();

        let modifier = calculate_movement_modifier(
            MovementType::ReturnHome,
            &traits,
            &needs,
            &membership,
        );

        // Isolated agents should strongly want to return home
        assert!(modifier > 2.0);
    }

    #[test]
    fn test_idle_modifier_for_unsociable() {
        let mut traits = Traits::default();
        traits.sociability = 0.1; // Very unsociable
        let needs = default_needs();

        let modifier = calculate_idle_modifier(&traits, &needs);

        // Unsociable agents should prefer idling
        assert!(modifier > 1.0);
    }

    #[test]
    fn test_bold_wander_more() {
        let mut bold_traits = Traits::default();
        bold_traits.boldness = 0.9;

        let mut timid_traits = Traits::default();
        timid_traits.boldness = 0.1;

        let needs = default_needs();
        let membership = default_membership();

        let bold_modifier = calculate_movement_modifier(
            MovementType::Travel,
            &bold_traits,
            &needs,
            &membership,
        );

        let timid_modifier = calculate_movement_modifier(
            MovementType::Travel,
            &timid_traits,
            &needs,
            &membership,
        );

        // Bold agents should wander more
        assert!(bold_modifier > timid_modifier);
    }
}
