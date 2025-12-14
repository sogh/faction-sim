//! Consumption System
//!
//! Handles automatic grain consumption, storage caps, spoilage, and intoxication decay.

use bevy_ecs::prelude::*;

use crate::components::agent::{AgentId, Intoxication};
use crate::components::faction::FactionRegistry;
use crate::components::world::WorldState;

/// Constants for consumption (can be overridden by config in future)
pub mod consumption_constants {
    /// Grain consumed per agent per day
    pub const GRAIN_PER_AGENT_PER_DAY: f32 = 1.0;
    /// Grain storage capacity per territory
    pub const GRAIN_CAP_PER_TERRITORY: u32 = 500;
    /// Beer storage capacity per territory
    pub const BEER_CAP_PER_TERRITORY: u32 = 200;
    /// Fraction of resources that spoil each season
    pub const SPOILAGE_RATE: f32 = 0.10;
    /// Nutritional value of beer relative to grain
    pub const BEER_NUTRITION_RATIO: f32 = 0.5;
    /// Ticks per day
    pub const TICKS_PER_DAY: u64 = 10;
    /// Ticks per season
    pub const TICKS_PER_SEASON: u64 = 300;
}

use consumption_constants::*;

/// Resource to track consumption and spoilage timing
#[derive(Resource, Debug)]
pub struct ConsumptionTracker {
    /// Last tick when daily consumption was applied
    pub last_consumption_tick: u64,
    /// Last tick when seasonal spoilage was applied
    pub last_spoilage_tick: u64,
    /// Ticks between consumption applications
    pub consumption_interval: u64,
    /// Ticks between spoilage applications
    pub spoilage_interval: u64,
}

impl Default for ConsumptionTracker {
    fn default() -> Self {
        Self {
            last_consumption_tick: 0,
            last_spoilage_tick: 0,
            consumption_interval: TICKS_PER_DAY,
            spoilage_interval: TICKS_PER_SEASON,
        }
    }
}

impl ConsumptionTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if daily consumption should be applied
    pub fn should_consume(&self, current_tick: u64) -> bool {
        current_tick >= self.last_consumption_tick + self.consumption_interval
    }

    /// Check if seasonal spoilage should be applied
    pub fn should_spoil(&self, current_tick: u64) -> bool {
        current_tick >= self.last_spoilage_tick + self.spoilage_interval
    }

    /// Mark that consumption was applied
    pub fn mark_consumed(&mut self, tick: u64) {
        self.last_consumption_tick = tick;
    }

    /// Mark that spoilage was applied
    pub fn mark_spoiled(&mut self, tick: u64) {
        self.last_spoilage_tick = tick;
    }
}

/// System: Apply daily grain consumption for each faction
///
/// Each day, factions consume grain based on member count.
/// Beer can supplement grain at 50% efficiency when grain runs low.
pub fn apply_daily_consumption(
    world_state: Res<WorldState>,
    mut consumption_tracker: ResMut<ConsumptionTracker>,
    mut faction_registry: ResMut<FactionRegistry>,
) {
    if !consumption_tracker.should_consume(world_state.current_tick) {
        return;
    }

    for faction in faction_registry.all_factions_mut() {
        let member_count = faction.member_count.max(1);
        let grain_needed = (member_count as f32 * GRAIN_PER_AGENT_PER_DAY).ceil() as u32;

        // Calculate how much grain we can consume
        let grain_consumed = grain_needed.min(faction.resources.grain);
        faction.resources.grain -= grain_consumed;

        // Calculate remaining need and try to supplement with beer
        let remaining_need = grain_needed.saturating_sub(grain_consumed);

        if remaining_need > 0 && faction.resources.beer > 0 {
            // Beer provides 50% nutrition, so we need 2 beer for 1 grain equivalent
            let beer_needed = ((remaining_need as f32) / BEER_NUTRITION_RATIO).ceil() as u32;
            let beer_consumed = beer_needed.min(faction.resources.beer);
            faction.resources.beer -= beer_consumed;
        }
    }

    consumption_tracker.mark_consumed(world_state.current_tick);
}

/// System: Enforce storage caps based on territory count
///
/// Excess resources above the cap are lost (spoiled/overflow).
pub fn enforce_storage_caps(
    world_state: Res<WorldState>,
    mut faction_registry: ResMut<FactionRegistry>,
) {
    // Only check every 10 ticks to reduce overhead
    if world_state.current_tick % 10 != 0 {
        return;
    }

    for faction in faction_registry.all_factions_mut() {
        let territory_count = faction.territory.len().max(1) as u32;
        let grain_cap = territory_count * GRAIN_CAP_PER_TERRITORY;
        let beer_cap = territory_count * BEER_CAP_PER_TERRITORY;

        // Cap resources - excess is lost
        faction.resources.grain = faction.resources.grain.min(grain_cap);
        faction.resources.beer = faction.resources.beer.min(beer_cap);
    }
}

/// System: Apply seasonal spoilage
///
/// Each season, a percentage of stored resources spoils.
/// Beer spoils at half the rate of grain (better preserved).
pub fn apply_seasonal_spoilage(
    world_state: Res<WorldState>,
    mut consumption_tracker: ResMut<ConsumptionTracker>,
    mut faction_registry: ResMut<FactionRegistry>,
) {
    if !consumption_tracker.should_spoil(world_state.current_tick) {
        return;
    }

    for faction in faction_registry.all_factions_mut() {
        let grain_spoiled = (faction.resources.grain as f32 * SPOILAGE_RATE).floor() as u32;
        // Beer spoils at half the rate (fermented = better preserved)
        let beer_spoiled = (faction.resources.beer as f32 * SPOILAGE_RATE * 0.5).floor() as u32;

        faction.resources.grain = faction.resources.grain.saturating_sub(grain_spoiled);
        faction.resources.beer = faction.resources.beer.saturating_sub(beer_spoiled);
    }

    consumption_tracker.mark_spoiled(world_state.current_tick);
}

/// System: Decay intoxication effects over time
///
/// Agents gradually sober up after drinking.
pub fn decay_intoxication(mut query: Query<(&AgentId, &mut Intoxication)>) {
    for (_agent_id, mut intox) in query.iter_mut() {
        if intox.level > 0.0 {
            // Decay by 1 tick worth
            intox.decay(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consumption_tracker_timing() {
        let tracker = ConsumptionTracker::new();

        // Should consume at tick 10 (first day)
        assert!(!tracker.should_consume(5));
        assert!(tracker.should_consume(10));
        assert!(tracker.should_consume(15));
    }

    #[test]
    fn test_spoilage_tracker_timing() {
        let tracker = ConsumptionTracker::new();

        // Should spoil at tick 300 (first season)
        assert!(!tracker.should_spoil(100));
        assert!(!tracker.should_spoil(299));
        assert!(tracker.should_spoil(300));
    }

    #[test]
    fn test_consumption_tracker_marking() {
        let mut tracker = ConsumptionTracker::new();
        tracker.mark_consumed(10);
        assert_eq!(tracker.last_consumption_tick, 10);

        // Should not consume again until tick 20
        assert!(!tracker.should_consume(15));
        assert!(tracker.should_consume(20));
    }
}
