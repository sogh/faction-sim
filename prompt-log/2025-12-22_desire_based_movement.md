# Desire-Based Action System Implementation

**Date:** 2025-12-22
**Model:** Claude Opus 4.5

## User Prompt

> In the action generator we have a consistent movement generator that guides agents back to their HQ if they aren't doing anything else. We don't want to generate movement actions in this way. We want agents to generate a desire to perform an action based on their expectation of the action results. Actions require locations, so in this way the agents will decide to move to a place then we generate their movements based on that. Attending the ritual reading has a high benefit for learning so agents will often choose to go to HQ. HQ has safety and food stockpiles as well. So they can eat there and get warm there if its cold outside.

## Summary

Replaced the explicit "return home" movement generator with a desire-based system where agents generate action desires for all known locations, and movement emerges from wanting to perform actions at specific locations.

## Key Changes

### New Files
- `crates/sim-core/src/systems/action/utility.rs` - ActionUtility struct and multi-factor utility calculation

### Modified Files

1. **`crates/sim-core/src/components/world.rs`**
   - Added `ProductionType` enum (Harvest, Hunt, GatherWood, etc.)
   - Added `LocationBenefits` struct with:
     - `provides_shelter`, `has_food_stores`, `has_water`
     - `social_hub_rating`, `is_faction_hq`
     - `safety_rating`, `rest_quality`
     - `production_types: Vec<ProductionType>`
   - Added factory methods for each location type (hall, village, fields, etc.)
   - Added `benefits` field to `Location` struct

2. **`crates/sim-core/src/systems/action/generate.rs`**
   - Replaced `generate_movement_actions()` with `generate_desire_based_actions()`
   - Removed explicit ReturnHome logic
   - Added helper functions:
     - `get_known_locations()` - faction territory + adjacent locations
     - `generate_consumption_desires()` - eat, drink, rest, warmth
     - `generate_production_desires()` - harvest, hunt, etc.
     - `generate_belonging_desires()` - social hubs, ritual attendance

3. **`crates/sim-core/src/actions/resource.rs`**
   - Added `Consume` variant to `ResourceActionType`
   - Added `ResourceType::from_need()` method
   - Added `ResourceAction::consume()` constructor

4. **`crates/sim-core/src/systems/action/execute.rs`**
   - Added Consume action handler that deducts faction resources

5. **Event type updates**
   - Added `ResourceSubtype::Consume` to both sim-core and sim-events

## Architecture

### Multi-Factor Utility Calculation

```
ActionUtility = (need_satisfaction + social_gain + faction_benefit + goal_advancement) * distance_cost
```

Where:
- `distance_cost = 0.7^distance_steps` (adjusted by boldness)
- Need satisfaction = `satisfaction_amount * urgency_weight * 2.0`

### Movement Emergence

Instead of:
```
if not_at_HQ:
    generate ReturnHome action
```

Now:
```
for each known_location:
    for each need this location can satisfy:
        utility = calculate_utility(need, location)
        if at_location: generate consumption action
        else: generate travel toward location
```

### Example Flow

Agent "Tom" is hungry (urgency=0.7) at fields, HQ is 2 steps away:
1. For HQ (has food stores):
   - Hunger utility: `0.4 * 0.7 * 2.0 = 0.56`
   - Distance penalty: `0.7^2 = 0.49`
   - Total: `0.27` → Travel toward village (next step to HQ)

2. For fields (current, has harvest):
   - Faction benefit: `0.18`
   - Distance: 0, penalty: 1.0
   - Total: `0.46` → Harvest action

Movement toward HQ emerges from the desire to eat, not from hardcoded homing behavior.

## Verification

- `cargo build --workspace` succeeds (54 warnings, 0 errors)
- Old `generate_movement_actions` function kept as alias for backwards compatibility
