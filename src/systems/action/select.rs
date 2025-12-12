//! Action Selection System
//!
//! Probabilistically selects actions based on weights.

use bevy_ecs::prelude::*;
use rand::Rng;
use std::collections::HashMap;

use crate::SimRng;

use super::generate::{Action, PendingActions, WeightedAction};

/// Resource storing selected actions for each agent
#[derive(Resource, Debug, Default)]
pub struct SelectedActions {
    /// Maps agent_id -> selected action
    pub actions: HashMap<String, Action>,
}

impl SelectedActions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.actions.clear();
    }

    pub fn set(&mut self, agent_id: impl Into<String>, action: Action) {
        self.actions.insert(agent_id.into(), action);
    }

    pub fn get(&self, agent_id: &str) -> Option<&Action> {
        self.actions.get(agent_id)
    }

    pub fn take(&mut self, agent_id: &str) -> Option<Action> {
        self.actions.remove(agent_id)
    }
}

/// System to select actions using weighted random choice
pub fn select_actions(
    mut rng: ResMut<SimRng>,
    mut pending_actions: ResMut<PendingActions>,
    mut selected_actions: ResMut<SelectedActions>,
) {
    selected_actions.clear();

    for (agent_id, candidates) in pending_actions.actions.drain() {
        if candidates.is_empty() {
            continue;
        }

        let selected = weighted_random_choice(&mut rng.0, &candidates);
        selected_actions.set(agent_id, selected.action.clone());
    }
}

/// Perform weighted random selection from a list of actions
fn weighted_random_choice<'a, R: Rng>(rng: &mut R, candidates: &'a [WeightedAction]) -> &'a WeightedAction {
    // Calculate total weight
    let total_weight: f32 = candidates.iter().map(|c| c.weight).sum();

    if total_weight <= 0.0 {
        // Fallback to first action if weights are invalid
        return &candidates[0];
    }

    // Generate random value in [0, total_weight)
    let mut roll: f32 = rng.gen::<f32>() * total_weight;

    // Find the selected action
    for candidate in candidates {
        roll -= candidate.weight;
        if roll <= 0.0 {
            return candidate;
        }
    }

    // Fallback to last action (shouldn't happen with valid weights)
    candidates.last().unwrap()
}

/// Add noise to action weights for variety
pub fn add_noise_to_weights(
    mut rng: ResMut<SimRng>,
    mut pending_actions: ResMut<PendingActions>,
) {
    const NOISE_FACTOR: f32 = 0.2; // +/- 20% noise

    for (_agent_id, candidates) in pending_actions.actions.iter_mut() {
        for candidate in candidates.iter_mut() {
            // Add multiplicative noise
            let noise: f32 = 1.0 + (rng.0.gen::<f32>() - 0.5) * 2.0 * NOISE_FACTOR;
            candidate.weight *= noise;
            candidate.weight = candidate.weight.max(0.01); // Ensure positive weight
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::movement::MoveAction;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    #[test]
    fn test_weighted_random_choice() {
        let mut rng = SmallRng::seed_from_u64(12345);

        let candidates = vec![
            WeightedAction::new(Action::Idle, 0.1, "low"),
            WeightedAction::new(
                Action::Move(MoveAction::travel("agent", "village")),
                0.9,
                "high",
            ),
        ];

        // Run many trials
        let mut idle_count = 0;
        let mut move_count = 0;

        for _ in 0..1000 {
            let selected = weighted_random_choice(&mut rng, &candidates);
            match &selected.action {
                Action::Idle => idle_count += 1,
                Action::Move(_) => move_count += 1,
                Action::Communicate(_) => {} // Not used in this test
            }
        }

        // Move should be selected ~90% of the time
        assert!(move_count > idle_count * 5);
    }

    #[test]
    fn test_selected_actions() {
        let mut selected = SelectedActions::new();

        selected.set("agent_001", Action::Idle);
        assert!(matches!(selected.get("agent_001"), Some(Action::Idle)));

        let taken = selected.take("agent_001");
        assert!(matches!(taken, Some(Action::Idle)));
        assert!(selected.get("agent_001").is_none());
    }

    #[test]
    fn test_noise_keeps_positive_weights() {
        let mut world = World::new();
        world.insert_resource(SimRng(SmallRng::seed_from_u64(42)));

        let mut pending = PendingActions::new();
        pending.add(
            "agent",
            WeightedAction::new(Action::Idle, 0.001, "tiny weight"),
        );
        world.insert_resource(pending);

        // Run noise system
        let mut schedule = Schedule::default();
        schedule.add_systems(add_noise_to_weights);
        schedule.run(&mut world);

        // Weight should still be positive
        let pending = world.resource::<PendingActions>();
        let actions = pending.get("agent").unwrap();
        assert!(actions[0].weight > 0.0);
    }
}
