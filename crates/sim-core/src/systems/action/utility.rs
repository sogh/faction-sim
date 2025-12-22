//! Action Utility Calculation
//!
//! Multi-factor utility calculation for desire-based action selection.
//! Agents generate desires for actions at all known locations, weighted by:
//! - Need satisfaction (how much does this reduce urgent needs?)
//! - Social gain (faction standing, relationships)
//! - Faction benefit (resource production, defense)
//! - Goal advancement (personal objectives)
//! - Distance cost (penalty for remote locations)

use std::collections::{HashMap, HashSet, VecDeque};

use crate::components::agent::Traits;
use crate::components::needs::{NeedStatus, PhysicalNeeds};
use crate::components::world::LocationRegistry;

/// Utility weight constants for multi-factor calculation
pub mod weights {
    /// Weight multiplier for need satisfaction (scaled by urgency)
    pub const NEED: f32 = 2.0;
    /// Weight for social/relationship benefits
    pub const SOCIAL: f32 = 0.8;
    /// Weight for faction resource contribution
    pub const FACTION: f32 = 0.6;
    /// Weight for personal goal advancement
    pub const GOAL: f32 = 1.2;
    /// Base distance penalty per step (0.7 means 30% reduction per step)
    pub const DISTANCE_PENALTY_BASE: f32 = 0.7;
    /// How much boldness mitigates distance penalty (bold agents travel more)
    pub const BOLDNESS_DISTANCE_MITIGATION: f32 = 0.15;
}

/// Multi-factor utility breakdown for an action
#[derive(Debug, Clone, Default)]
pub struct ActionUtility {
    /// Utility from satisfying physical needs (hunger, thirst, warmth, etc.)
    pub need_satisfaction: f32,
    /// Utility from social interactions and belonging
    pub social_gain: f32,
    /// Utility from contributing to faction resources/goals
    pub faction_benefit: f32,
    /// Utility from advancing personal goals
    pub goal_advancement: f32,
    /// Distance cost multiplier (1.0 = at location, lower = farther away)
    pub distance_cost: f32,
}

impl ActionUtility {
    /// Create a new utility with distance cost of 1.0 (at location)
    pub fn new() -> Self {
        Self {
            distance_cost: 1.0,
            ..Default::default()
        }
    }

    /// Compute the total utility, applying distance cost as a multiplier
    pub fn total(&self) -> f32 {
        let raw = self.need_satisfaction
            + self.social_gain
            + self.faction_benefit
            + self.goal_advancement;
        (raw * self.distance_cost).max(0.001)
    }

    /// Add need satisfaction utility
    pub fn with_need(mut self, satisfaction: f32) -> Self {
        self.need_satisfaction = satisfaction;
        self
    }

    /// Add social gain utility
    pub fn with_social(mut self, gain: f32) -> Self {
        self.social_gain = gain;
        self
    }

    /// Add faction benefit utility
    pub fn with_faction(mut self, benefit: f32) -> Self {
        self.faction_benefit = benefit;
        self
    }

    /// Add goal advancement utility
    pub fn with_goal(mut self, advancement: f32) -> Self {
        self.goal_advancement = advancement;
        self
    }

    /// Set distance cost multiplier
    pub fn with_distance_cost(mut self, cost: f32) -> Self {
        self.distance_cost = cost;
        self
    }
}

/// Calculate the utility of satisfying a need
///
/// Returns: satisfaction_amount * urgency_weight * NEED_WEIGHT
pub fn calculate_need_utility(
    need_name: &str,
    satisfaction_amount: f32,
    physical_needs: &PhysicalNeeds,
) -> f32 {
    let status = match need_name {
        "hunger" => physical_needs.hunger.status(),
        "thirst" => physical_needs.thirst.status(),
        "warmth" => physical_needs.warmth.status(),
        "rest" => physical_needs.rest.status(),
        "safety" => physical_needs.safety.status(),
        "belonging" => physical_needs.belonging.status(),
        _ => NeedStatus::Satisfied,
    };

    satisfaction_amount * status.urgency_weight() * weights::NEED
}

/// Calculate the distance penalty for reaching a location
///
/// Returns a multiplier between 0 and 1:
/// - 1.0 if at the location
/// - 0.7^distance for remote locations (adjusted by boldness)
pub fn calculate_distance_penalty(
    from_location: &str,
    to_location: &str,
    location_registry: &LocationRegistry,
    boldness: f32,
) -> f32 {
    if from_location == to_location {
        return 1.0;
    }

    let distance = path_distance(from_location, to_location, location_registry);
    if distance == u32::MAX {
        return 0.001; // Unreachable location
    }

    // Bold agents are less deterred by distance
    let adjusted_penalty = weights::DISTANCE_PENALTY_BASE
        + (boldness * weights::BOLDNESS_DISTANCE_MITIGATION);

    adjusted_penalty.powf(distance as f32)
}

/// Calculate the path distance between two locations using BFS
///
/// Returns u32::MAX if unreachable
pub fn path_distance(from: &str, to: &str, registry: &LocationRegistry) -> u32 {
    if from == to {
        return 0;
    }

    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    visited.insert(from.to_string());
    queue.push_back((from.to_string(), 0u32));

    while let Some((current, distance)) = queue.pop_front() {
        for adjacent in registry.get_adjacent(&current) {
            if adjacent == to {
                return distance + 1;
            }
            if !visited.contains(&adjacent) {
                visited.insert(adjacent.clone());
                queue.push_back((adjacent, distance + 1));
            }
        }
    }

    // Unreachable
    u32::MAX
}

/// Calculate idle action weight based on current needs
///
/// Agents with pressing needs should be less likely to idle
pub fn calculate_idle_weight(physical_needs: &PhysicalNeeds) -> f32 {
    let base_idle = 0.2;

    // Reduce idle weight based on most urgent need
    let max_urgency = [
        physical_needs.hunger.status().urgency_weight(),
        physical_needs.thirst.status().urgency_weight(),
        physical_needs.warmth.status().urgency_weight(),
        physical_needs.rest.status().urgency_weight(),
        physical_needs.safety.status().urgency_weight(),
        physical_needs.belonging.status().urgency_weight(),
    ]
    .into_iter()
    .fold(0.0f32, f32::max);

    // Higher urgency = lower idle weight
    (base_idle * (1.0 - max_urgency * 0.8)).max(0.02)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_utility_total() {
        let utility = ActionUtility::new()
            .with_need(0.5)
            .with_social(0.2)
            .with_faction(0.1)
            .with_goal(0.1)
            .with_distance_cost(0.5);

        // (0.5 + 0.2 + 0.1 + 0.1) * 0.5 = 0.45
        assert!((utility.total() - 0.45).abs() < 0.001);
    }

    #[test]
    fn test_distance_penalty_at_location() {
        let registry = LocationRegistry::new();
        let penalty = calculate_distance_penalty("loc_a", "loc_a", &registry, 0.5);
        assert_eq!(penalty, 1.0);
    }

    #[test]
    fn test_idle_weight_with_urgent_needs() {
        let mut needs = PhysicalNeeds::new();

        // All satisfied - high idle weight
        let idle1 = calculate_idle_weight(&needs);
        assert!(idle1 > 0.15);

        // Urgent hunger - low idle weight
        needs.hunger.set_level(0.7);
        let idle2 = calculate_idle_weight(&needs);
        assert!(idle2 < idle1);
    }

    #[test]
    fn test_need_utility_scales_with_urgency() {
        let mut needs = PhysicalNeeds::new();

        // Satisfied need = 0 utility
        let util1 = calculate_need_utility("hunger", 0.3, &needs);
        assert_eq!(util1, 0.0);

        // Urgent need = high utility
        needs.hunger.set_level(0.7);
        let util2 = calculate_need_utility("hunger", 0.3, &needs);
        // 0.3 * 0.7 * 2.0 = 0.42
        assert!((util2 - 0.42).abs() < 0.001);
    }
}
