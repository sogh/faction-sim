//! Statistics Output
//!
//! Collects and outputs simulation statistics for analysis.

use bevy_ecs::prelude::*;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::components::faction::FactionRegistry;
use crate::events::types::EventType;
use crate::systems::action::TickEvents;

/// Statistics output path
pub const STATS_OUTPUT_PATH: &str = "output/stats.json";

/// Statistics for a single tick
#[derive(Debug, Clone, Default, Serialize)]
pub struct TickStats {
    pub tick: u64,
    pub total_events: usize,
    pub events_by_type: HashMap<String, usize>,
    pub faction_stats: HashMap<String, FactionStats>,
}

/// Statistics for a faction
#[derive(Debug, Clone, Default, Serialize)]
pub struct FactionStats {
    pub member_count: usize,
    pub total_grain: u32,
    pub total_iron: u32,
    pub total_salt: u32,
    pub tensions_involved: usize,
}

/// Overall simulation statistics
#[derive(Debug, Clone, Serialize)]
pub struct SimulationStats {
    pub total_ticks: u64,
    pub total_events: usize,
    pub events_by_type: HashMap<String, usize>,
    pub average_events_per_tick: f64,
    pub faction_summaries: HashMap<String, FactionSummary>,
    pub drama_distribution: DramaDistribution,
    pub tick_history: Vec<TickSummary>,
}

/// Summary of faction over simulation
#[derive(Debug, Clone, Default, Serialize)]
pub struct FactionSummary {
    pub final_member_count: usize,
    pub peak_member_count: usize,
    pub total_defections: usize,
    pub total_exiles: usize,
    pub leadership_changes: usize,
    pub final_resources: ResourceSummary,
}

/// Resource summary
#[derive(Debug, Clone, Default, Serialize)]
pub struct ResourceSummary {
    pub grain: u32,
    pub iron: u32,
    pub salt: u32,
}

/// Distribution of drama scores
#[derive(Debug, Clone, Default, Serialize)]
pub struct DramaDistribution {
    pub low: usize,      // < 0.3
    pub medium: usize,   // 0.3-0.7
    pub high: usize,     // > 0.7
    pub highest_score: f32,
    pub average_score: f64,
}

/// Summary of a tick for history
#[derive(Debug, Clone, Serialize)]
pub struct TickSummary {
    pub tick: u64,
    pub event_count: usize,
    pub drama_sum: f32,
}

/// Resource to accumulate statistics during simulation
#[derive(Resource, Default)]
pub struct StatsCollector {
    pub total_events: usize,
    pub events_by_type: HashMap<String, usize>,
    pub tick_history: Vec<TickSummary>,
    pub faction_stats: HashMap<String, FactionTracker>,
    pub drama_scores: Vec<f32>,
}

/// Tracks faction changes over time
#[derive(Debug, Clone, Default)]
pub struct FactionTracker {
    pub peak_members: usize,
    pub current_members: usize,
    pub defections: usize,
    pub exiles: usize,
    pub leadership_changes: usize,
}

impl StatsCollector {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record events from a tick
    pub fn record_tick(&mut self, tick: u64, events: &TickEvents) {
        let mut drama_sum = 0.0f32;

        for event in &events.events {
            self.total_events += 1;

            let type_name = format!("{:?}", event.event_type);
            *self.events_by_type.entry(type_name).or_insert(0) += 1;

            self.drama_scores.push(event.drama_score);
            drama_sum += event.drama_score;
        }

        self.tick_history.push(TickSummary {
            tick,
            event_count: events.events.len(),
            drama_sum,
        });
    }

    /// Update faction statistics
    pub fn update_faction_stats(&mut self, faction_id: &str, member_count: usize) {
        let tracker = self.faction_stats.entry(faction_id.to_string()).or_default();
        tracker.current_members = member_count;
        if member_count > tracker.peak_members {
            tracker.peak_members = member_count;
        }
    }

    /// Record a defection
    pub fn record_defection(&mut self, from_faction: &str) {
        self.faction_stats
            .entry(from_faction.to_string())
            .or_default()
            .defections += 1;
    }

    /// Record an exile
    pub fn record_exile(&mut self, faction_id: &str) {
        self.faction_stats
            .entry(faction_id.to_string())
            .or_default()
            .exiles += 1;
    }

    /// Record leadership change
    pub fn record_leadership_change(&mut self, faction_id: &str) {
        self.faction_stats
            .entry(faction_id.to_string())
            .or_default()
            .leadership_changes += 1;
    }

    /// Generate final statistics
    pub fn generate_stats(&self, total_ticks: u64, faction_registry: &FactionRegistry) -> SimulationStats {
        let average_events_per_tick = if total_ticks > 0 {
            self.total_events as f64 / total_ticks as f64
        } else {
            0.0
        };

        // Calculate drama distribution
        let mut drama_distribution = DramaDistribution::default();
        let mut drama_sum = 0.0f64;
        for &score in &self.drama_scores {
            drama_sum += score as f64;
            if score > drama_distribution.highest_score {
                drama_distribution.highest_score = score;
            }
            if score < 0.3 {
                drama_distribution.low += 1;
            } else if score < 0.7 {
                drama_distribution.medium += 1;
            } else {
                drama_distribution.high += 1;
            }
        }
        if !self.drama_scores.is_empty() {
            drama_distribution.average_score = drama_sum / self.drama_scores.len() as f64;
        }

        // Build faction summaries
        let mut faction_summaries = HashMap::new();
        for faction_id in faction_registry.faction_ids() {
            let tracker = self.faction_stats.get(faction_id).cloned().unwrap_or_default();
            let faction = faction_registry.get(faction_id);

            let final_resources = faction.map(|f| ResourceSummary {
                grain: f.resources.grain,
                iron: f.resources.iron,
                salt: f.resources.salt,
            }).unwrap_or_default();

            faction_summaries.insert(faction_id.clone(), FactionSummary {
                final_member_count: tracker.current_members,
                peak_member_count: tracker.peak_members,
                total_defections: tracker.defections,
                total_exiles: tracker.exiles,
                leadership_changes: tracker.leadership_changes,
                final_resources,
            });
        }

        SimulationStats {
            total_ticks,
            total_events: self.total_events,
            events_by_type: self.events_by_type.clone(),
            average_events_per_tick,
            faction_summaries,
            drama_distribution,
            tick_history: self.tick_history.clone(),
        }
    }
}

/// Write statistics to output file
pub fn write_stats(stats: &SimulationStats) -> std::io::Result<()> {
    let output_dir = Path::new("output");
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    let json = serde_json::to_string_pretty(stats)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    fs::write(STATS_OUTPUT_PATH, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_collector_new() {
        let collector = StatsCollector::new();
        assert_eq!(collector.total_events, 0);
        assert!(collector.events_by_type.is_empty());
    }

    #[test]
    fn test_faction_tracker() {
        let mut collector = StatsCollector::new();

        collector.update_faction_stats("test_faction", 10);
        assert_eq!(collector.faction_stats["test_faction"].current_members, 10);
        assert_eq!(collector.faction_stats["test_faction"].peak_members, 10);

        collector.update_faction_stats("test_faction", 5);
        assert_eq!(collector.faction_stats["test_faction"].current_members, 5);
        assert_eq!(collector.faction_stats["test_faction"].peak_members, 10);

        collector.record_defection("test_faction");
        assert_eq!(collector.faction_stats["test_faction"].defections, 1);
    }
}
