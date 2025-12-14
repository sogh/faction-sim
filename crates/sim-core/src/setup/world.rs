//! World Setup
//!
//! Creates the medieval world map with locations and connectivity.

use crate::components::world::{
    Location, LocationProperty, LocationRegistry, LocationResources, LocationType,
};

/// Create the world map with all locations
pub fn create_world_map() -> LocationRegistry {
    let mut registry = LocationRegistry::new();

    // === THORNWOOD TERRITORY (Northwest) ===
    registry.register(
        Location::new("thornwood_hall", "Thornwood Hall", LocationType::Hall)
            .with_faction("thornwood")
            .with_properties(vec![LocationProperty::FactionHQ, LocationProperty::Defensible])
            .with_adjacent(vec![
                "thornwood_village".into(),
                "thornwood_fields".into(),
            ]),
    );

    registry.register(
        Location::new("thornwood_village", "Thornwood Village", LocationType::Village)
            .with_faction("thornwood")
            .with_adjacent(vec![
                "thornwood_hall".into(),
                "thornwood_fields".into(),
                "western_forest".into(),
                "northern_crossroads".into(),
            ]),
    );

    registry.register(
        Location::new("thornwood_fields", "Thornwood Fields", LocationType::Fields)
            .with_faction("thornwood")
            .with_properties(vec![LocationProperty::FoodProduction])
            .with_resources(LocationResources::new(20, 0, 0))
            .with_adjacent(vec![
                "thornwood_hall".into(),
                "thornwood_village".into(),
                "northern_crossroads".into(),
            ]),
    );

    registry.register(
        Location::new("western_forest", "Western Forest", LocationType::Forest)
            .with_faction("thornwood")
            .with_properties(vec![LocationProperty::HiddenMeetingSpot])
            .with_adjacent(vec![
                "thornwood_village".into(),
                "central_crossroads".into(),
            ]),
    );

    // === IRONMERE TERRITORY (Northeast) ===
    registry.register(
        Location::new("ironmere_hall", "Ironmere Hall", LocationType::Hall)
            .with_faction("ironmere")
            .with_properties(vec![LocationProperty::FactionHQ, LocationProperty::Defensible])
            .with_adjacent(vec!["ironmere_village".into(), "iron_mine".into()]),
    );

    registry.register(
        Location::new("ironmere_village", "Ironmere Village", LocationType::Village)
            .with_faction("ironmere")
            .with_adjacent(vec![
                "ironmere_hall".into(),
                "iron_mine".into(),
                "ironmere_fields".into(),
                "northern_crossroads".into(),
            ]),
    );

    registry.register(
        Location::new("iron_mine", "Iron Mine", LocationType::Mine)
            .with_faction("ironmere")
            .with_resources(LocationResources::new(0, 25, 0))
            .with_adjacent(vec!["ironmere_hall".into(), "ironmere_village".into()]),
    );

    registry.register(
        Location::new("ironmere_fields", "Ironmere Fields", LocationType::Fields)
            .with_faction("ironmere")
            .with_properties(vec![LocationProperty::FoodProduction])
            .with_resources(LocationResources::new(15, 0, 0))
            .with_adjacent(vec![
                "ironmere_village".into(),
                "eastern_bridge".into(),
                "northern_crossroads".into(),
            ]),
    );

    // === SALTCLIFF TERRITORY (Southeast) ===
    registry.register(
        Location::new("saltcliff_hall", "Saltcliff Hall", LocationType::Hall)
            .with_faction("saltcliff")
            .with_properties(vec![LocationProperty::FactionHQ, LocationProperty::Defensible])
            .with_adjacent(vec!["saltcliff_village".into(), "salt_harbor".into()]),
    );

    registry.register(
        Location::new("saltcliff_village", "Saltcliff Village", LocationType::Village)
            .with_faction("saltcliff")
            .with_adjacent(vec![
                "saltcliff_hall".into(),
                "salt_harbor".into(),
                "saltcliff_fields".into(),
                "eastern_bridge".into(),
            ]),
    );

    registry.register(
        Location::new("salt_harbor", "Salt Harbor", LocationType::Harbor)
            .with_faction("saltcliff")
            .with_properties(vec![LocationProperty::TradeRoute])
            .with_resources(LocationResources::new(5, 0, 30))
            .with_adjacent(vec!["saltcliff_hall".into(), "saltcliff_village".into()]),
    );

    registry.register(
        Location::new("saltcliff_fields", "Saltcliff Fields", LocationType::Fields)
            .with_faction("saltcliff")
            .with_properties(vec![LocationProperty::FoodProduction])
            .with_resources(LocationResources::new(12, 0, 0))
            .with_adjacent(vec![
                "saltcliff_village".into(),
                "southern_forest".into(),
                "central_crossroads".into(),
            ]),
    );

    // === NORTHERN HOLD TERRITORY (Southwest) ===
    registry.register(
        Location::new("northern_hold", "Northern Hold", LocationType::Hall)
            .with_faction("northern_hold")
            .with_properties(vec![
                LocationProperty::FactionHQ,
                LocationProperty::Defensible,
                LocationProperty::Strategic,
            ])
            .with_adjacent(vec!["hold_village".into(), "mountain_pass".into()]),
    );

    registry.register(
        Location::new("hold_village", "Hold Village", LocationType::Village)
            .with_faction("northern_hold")
            .with_adjacent(vec![
                "northern_hold".into(),
                "hold_fields".into(),
                "mountain_pass".into(),
                "central_crossroads".into(),
            ]),
    );

    registry.register(
        Location::new("hold_fields", "Hold Fields", LocationType::Fields)
            .with_faction("northern_hold")
            .with_properties(vec![LocationProperty::FoodProduction])
            .with_resources(LocationResources::new(18, 0, 0))
            .with_adjacent(vec![
                "hold_village".into(),
                "southern_forest".into(),
            ]),
    );

    registry.register(
        Location::new("mountain_pass", "Mountain Pass", LocationType::Watchtower)
            .with_faction("northern_hold")
            .with_properties(vec![LocationProperty::Strategic, LocationProperty::Defensible])
            .with_resources(LocationResources::new(0, 10, 0))
            .with_adjacent(vec!["northern_hold".into(), "hold_village".into()]),
    );

    // === NEUTRAL TERRITORIES ===
    registry.register(
        Location::new("northern_crossroads", "Northern Crossroads", LocationType::Crossroads)
            .with_properties(vec![LocationProperty::Neutral, LocationProperty::TradeRoute])
            .with_adjacent(vec![
                "thornwood_village".into(),
                "thornwood_fields".into(),
                "ironmere_village".into(),
                "ironmere_fields".into(),
                "central_crossroads".into(),
            ]),
    );

    registry.register(
        Location::new("central_crossroads", "Central Crossroads", LocationType::Crossroads)
            .with_properties(vec![
                LocationProperty::Neutral,
                LocationProperty::TradeRoute,
                LocationProperty::HiddenMeetingSpot,
            ])
            .with_adjacent(vec![
                "northern_crossroads".into(),
                "western_forest".into(),
                "southern_forest".into(),
                "hold_village".into(),
                "saltcliff_fields".into(),
            ]),
    );

    registry.register(
        Location::new("eastern_bridge", "Eastern Bridge", LocationType::Bridge)
            .with_properties(vec![
                LocationProperty::Neutral,
                LocationProperty::Strategic,
                LocationProperty::HiddenMeetingSpot,
            ])
            .with_adjacent(vec![
                "ironmere_fields".into(),
                "saltcliff_village".into(),
            ]),
    );

    registry.register(
        Location::new("southern_forest", "Southern Forest", LocationType::Forest)
            .with_properties(vec![LocationProperty::Neutral, LocationProperty::HiddenMeetingSpot])
            .with_adjacent(vec![
                "central_crossroads".into(),
                "saltcliff_fields".into(),
                "hold_fields".into(),
            ]),
    );

    registry.register(
        Location::new("old_market", "Old Market", LocationType::Market)
            .with_properties(vec![LocationProperty::Neutral, LocationProperty::TradeRoute])
            .with_adjacent(vec![
                "central_crossroads".into(),
                "northern_crossroads".into(),
            ]),
    );

    registry
}

/// Output the world setup as JSON for verification
pub fn world_to_json(registry: &LocationRegistry) -> String {
    let locations: Vec<_> = registry.all_locations().collect();
    serde_json::to_string_pretty(&locations).unwrap_or_else(|_| "[]".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_creation() {
        let registry = create_world_map();

        // Should have ~19 locations
        assert!(registry.location_ids().len() >= 15);
        assert!(registry.location_ids().len() <= 25);

        // Each faction should have a hall
        assert!(registry.get("thornwood_hall").is_some());
        assert!(registry.get("ironmere_hall").is_some());
        assert!(registry.get("saltcliff_hall").is_some());
        assert!(registry.get("northern_hold").is_some());

        // Halls should be HQs
        let thornwood_hall = registry.get("thornwood_hall").unwrap();
        assert!(thornwood_hall.is_hq());
        assert!(!thornwood_hall.is_neutral());

        // Crossroads should be neutral
        let crossroads = registry.get("central_crossroads").unwrap();
        assert!(crossroads.is_neutral());
    }

    #[test]
    fn test_adjacency() {
        let registry = create_world_map();

        // Thornwood hall should connect to village and fields
        let hall = registry.get("thornwood_hall").unwrap();
        assert!(hall.is_adjacent_to("thornwood_village"));
        assert!(hall.is_adjacent_to("thornwood_fields"));

        // Test bidirectional adjacency
        assert!(registry.are_adjacent("thornwood_hall", "thornwood_village"));
    }
}
