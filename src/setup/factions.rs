//! Faction Setup
//!
//! Creates the four factions with territory, resources, and empty archives.

use crate::components::faction::{Faction, FactionRegistry, FactionResources, RitualSchedule};

/// Create all factions and register them
pub fn create_factions() -> FactionRegistry {
    let mut registry = FactionRegistry::new();

    // === THORNWOOD ===
    // A woodland faction known for their skilled scouts and strong traditions
    let thornwood = Faction::new("thornwood", "Thornwood", "thornwood_hall")
        .with_territory(vec![
            "thornwood_hall".into(),
            "thornwood_village".into(),
            "thornwood_fields".into(),
            "western_forest".into(),
        ])
        .with_resources(FactionResources::new(800, 150, 40));

    registry.register(thornwood);

    // === IRONMERE ===
    // A militant faction with rich iron deposits and strong warriors
    let ironmere = Faction::new("ironmere", "Ironmere", "ironmere_hall")
        .with_territory(vec![
            "ironmere_hall".into(),
            "ironmere_village".into(),
            "iron_mine".into(),
            "ironmere_fields".into(),
        ])
        .with_resources(FactionResources::new(600, 400, 30));

    registry.register(ironmere);

    // === SALTCLIFF ===
    // A trading faction controlling the salt harbor and coastal routes
    let saltcliff = Faction::new("saltcliff", "Saltcliff", "saltcliff_hall")
        .with_territory(vec![
            "saltcliff_hall".into(),
            "saltcliff_village".into(),
            "salt_harbor".into(),
            "saltcliff_fields".into(),
        ])
        .with_resources(FactionResources::new(500, 100, 300));

    registry.register(saltcliff);

    // === NORTHERN HOLD ===
    // A defensive faction in the mountains, hardy and self-sufficient
    let northern_hold = Faction::new("northern_hold", "Northern Hold", "northern_hold")
        .with_territory(vec![
            "northern_hold".into(),
            "hold_village".into(),
            "hold_fields".into(),
            "mountain_pass".into(),
        ])
        .with_resources(FactionResources::new(700, 200, 50));

    registry.register(northern_hold);

    registry
}

/// Create the ritual schedule for all factions
pub fn create_ritual_schedule(ritual_interval: u64) -> RitualSchedule {
    let mut schedule = RitualSchedule::new(ritual_interval);

    // Stagger rituals so they don't all happen at once
    schedule.schedule_ritual("thornwood", ritual_interval);
    schedule.schedule_ritual("ironmere", ritual_interval + (ritual_interval / 4));
    schedule.schedule_ritual("saltcliff", ritual_interval + (ritual_interval / 2));
    schedule.schedule_ritual("northern_hold", ritual_interval + (3 * ritual_interval / 4));

    schedule
}

/// Output factions as JSON for verification
pub fn factions_to_json(registry: &FactionRegistry) -> String {
    let factions: Vec<_> = registry.all_factions().collect();
    serde_json::to_string_pretty(&factions).unwrap_or_else(|_| "[]".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_faction_creation() {
        let registry = create_factions();

        // Should have 4 factions
        assert_eq!(registry.faction_ids().len(), 4);

        // Each faction should exist
        assert!(registry.get("thornwood").is_some());
        assert!(registry.get("ironmere").is_some());
        assert!(registry.get("saltcliff").is_some());
        assert!(registry.get("northern_hold").is_some());
    }

    #[test]
    fn test_faction_resources() {
        let registry = create_factions();

        // Ironmere should have the most iron
        let ironmere = registry.get("ironmere").unwrap();
        let thornwood = registry.get("thornwood").unwrap();
        assert!(ironmere.resources.iron > thornwood.resources.iron);

        // Saltcliff should have the most salt
        let saltcliff = registry.get("saltcliff").unwrap();
        assert!(saltcliff.resources.salt > ironmere.resources.salt);
    }

    #[test]
    fn test_faction_territory() {
        let registry = create_factions();

        let thornwood = registry.get("thornwood").unwrap();
        assert!(thornwood.controls_location("thornwood_hall"));
        assert!(thornwood.controls_location("thornwood_village"));
        assert!(!thornwood.controls_location("ironmere_hall"));
    }

    #[test]
    fn test_faction_archives_empty() {
        let registry = create_factions();

        // All archives should start empty
        for faction_id in registry.faction_ids() {
            let archive = registry.get_archive(faction_id).unwrap();
            assert_eq!(archive.entry_count(), 0);
        }
    }

    #[test]
    fn test_ritual_schedule() {
        let schedule = create_ritual_schedule(500);

        // All factions should have scheduled rituals
        assert!(schedule.next_ritual("thornwood").is_some());
        assert!(schedule.next_ritual("ironmere").is_some());
        assert!(schedule.next_ritual("saltcliff").is_some());
        assert!(schedule.next_ritual("northern_hold").is_some());

        // Rituals should be staggered
        let tw = schedule.next_ritual("thornwood").unwrap();
        let im = schedule.next_ritual("ironmere").unwrap();
        assert!(im > tw);
    }
}
