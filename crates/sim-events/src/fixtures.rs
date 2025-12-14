//! Sample data fixtures for testing.
//!
//! This module provides ready-made test data for other crates to use.
//! Enable the `test-fixtures` feature to access these helpers.
//!
//! # Example
//!
//! ```ignore
//! // In your Cargo.toml:
//! // [dev-dependencies]
//! // sim-events = { path = "../sim-events", features = ["test-fixtures"] }
//!
//! use sim_events::fixtures;
//!
//! let events = fixtures::sample_events();
//! let tensions = fixtures::sample_tensions();
//! let snapshot = fixtures::sample_snapshot();
//! ```

use crate::{Event, Tension, WorldSnapshot};

/// Returns sample events from the fixtures file.
///
/// Contains 10 diverse events:
/// - 3 movement events (low drama)
/// - 2 communication events (gossip, rumors)
/// - 1 resource trade event
/// - 1 betrayal event (high drama)
/// - 1 ritual reading event
/// - 1 death event
/// - 1 faction promotion event
pub fn sample_events() -> Vec<Event> {
    let jsonl = include_str!("../tests/fixtures/sample_events.jsonl");
    jsonl
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            Event::from_jsonl(l).unwrap_or_else(|e| {
                panic!("Failed to parse event line: {}\nError: {}", l, e)
            })
        })
        .collect()
}

/// Returns sample tensions from the fixtures file.
///
/// Contains 2 tensions:
/// - 1 brewing_betrayal tension (high severity, involving Mira)
/// - 1 resource_conflict tension (medium severity, winter shortage)
pub fn sample_tensions() -> Vec<Tension> {
    let json = include_str!("../tests/fixtures/sample_tensions.json");
    serde_json::from_str(json).expect("Failed to parse sample_tensions.json")
}

/// Returns a sample world snapshot from the fixtures file.
///
/// Contains:
/// - 2 factions (Thornwood, Ironmere)
/// - 6 agents with relationships
/// - 5 locations
/// - Computed metrics including social network analysis
pub fn sample_snapshot() -> WorldSnapshot {
    let json = include_str!("../tests/fixtures/sample_state.json");
    serde_json::from_str(json).expect("Failed to parse sample_state.json")
}

/// Returns a specific event by ID from the sample events.
pub fn get_event(event_id: &str) -> Option<Event> {
    sample_events().into_iter().find(|e| e.event_id == event_id)
}

/// Returns a specific tension by ID from the sample tensions.
pub fn get_tension(tension_id: &str) -> Option<Tension> {
    sample_tensions()
        .into_iter()
        .find(|t| t.tension_id == tension_id)
}

/// Returns the high-drama betrayal event from samples.
pub fn betrayal_event() -> Event {
    get_event("evt_00000007").expect("Betrayal event should exist in fixtures")
}

/// Returns the brewing betrayal tension from samples.
pub fn brewing_betrayal_tension() -> Tension {
    get_tension("tens_00001").expect("Brewing betrayal tension should exist in fixtures")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_events_load() {
        let events = sample_events();
        assert_eq!(events.len(), 10, "Should have 10 sample events");

        // Verify event types are diverse
        assert!(events.iter().any(|e| e.event_type == crate::EventType::Movement));
        assert!(events.iter().any(|e| e.event_type == crate::EventType::Betrayal));
        assert!(events.iter().any(|e| e.event_type == crate::EventType::Death));
    }

    #[test]
    fn test_sample_tensions_load() {
        let tensions = sample_tensions();
        assert_eq!(tensions.len(), 2, "Should have 2 sample tensions");

        let betrayal = &tensions[0];
        assert_eq!(betrayal.tension_type, crate::TensionType::BrewingBetrayal);
        assert!(betrayal.severity > 0.8);
    }

    #[test]
    fn test_sample_snapshot_load() {
        let snapshot = sample_snapshot();

        assert_eq!(snapshot.factions.len(), 2);
        assert_eq!(snapshot.agents.len(), 6);
        assert_eq!(snapshot.locations.len(), 5);

        // Verify we can find agents
        let mira = snapshot.find_agent("agent_mira_0042");
        assert!(mira.is_some());
        assert_eq!(mira.unwrap().faction, "thornwood");
    }

    #[test]
    fn test_get_specific_event() {
        let event = get_event("evt_00000007");
        assert!(event.is_some());
        assert_eq!(event.unwrap().event_type, crate::EventType::Betrayal);
    }

    #[test]
    fn test_betrayal_event_helper() {
        let event = betrayal_event();
        assert_eq!(event.event_id, "evt_00000007");
        assert!(event.drama_score > 0.8);
    }

    #[test]
    fn test_brewing_betrayal_tension_helper() {
        let tension = brewing_betrayal_tension();
        assert_eq!(tension.tension_id, "tens_00001");
        assert_eq!(tension.tension_type, crate::TensionType::BrewingBetrayal);
    }

    #[test]
    fn test_events_reference_consistent_ids() {
        let events = sample_events();
        let snapshot = sample_snapshot();

        // All agent IDs in events should exist in snapshot
        for event in &events {
            let primary_id = &event.actors.primary.agent_id;
            // Most agents should be in our snapshot (some events may reference agents not in minimal snapshot)
            if primary_id.starts_with("agent_mira") || primary_id.starts_with("agent_corin") {
                assert!(
                    snapshot.find_agent(primary_id).is_some(),
                    "Agent {} should exist in snapshot",
                    primary_id
                );
            }
        }
    }
}
