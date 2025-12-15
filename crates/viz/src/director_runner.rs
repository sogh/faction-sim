//! Director runner: invokes the Director AI to generate commentary and camera instructions.
//!
//! This module bridges the gap between sim-core output (events, tensions, snapshots)
//! and the visualization layer by running the Director AI each time the simulation
//! tick advances.

use bevy::prelude::*;
use std::io::BufRead;
use std::path::Path;

use director::Director;
use sim_events::{Event, Tension};

use crate::director_state::DirectorState;
use crate::state_loader::SimulationState;

/// Plugin for running the Director AI.
pub struct DirectorRunnerPlugin;

impl Plugin for DirectorRunnerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DirectorRunner>()
            .add_systems(Update, run_director_on_tick_change);
    }
}

/// Resource holding the Director instance and tracking state.
#[derive(Resource)]
pub struct DirectorRunner {
    /// The Director AI instance.
    director: Director,
    /// Last tick that was processed by the director.
    last_processed_tick: u64,
}

impl Default for DirectorRunner {
    fn default() -> Self {
        Self {
            director: Director::with_defaults(),
            last_processed_tick: 0,
        }
    }
}

/// System that runs the director when the simulation tick changes.
fn run_director_on_tick_change(
    mut director_runner: ResMut<DirectorRunner>,
    mut director_state: ResMut<DirectorState>,
    sim_state: Res<SimulationState>,
) {
    let Some(snapshot) = &sim_state.snapshot else {
        return;
    };

    let current_tick = snapshot.timestamp.tick;

    // Skip if we've already processed this tick
    if current_tick <= director_runner.last_processed_tick {
        return;
    }

    // Load events for the tick range we haven't processed yet
    let events = load_events_since(director_runner.last_processed_tick, current_tick);
    let tensions = load_tensions();

    // Run the director AI
    let output = director_runner
        .director
        .process_tick(&events, &tensions, snapshot);

    // Feed the output into DirectorState
    let commentary_count = output.commentary_queue.len();
    director_state.camera_script.extend(output.camera_script);
    for item in output.commentary_queue {
        director_state.add_commentary(item);
    }

    // Update the last processed tick
    director_runner.last_processed_tick = current_tick;

    if !events.is_empty() || commentary_count > 0 {
        tracing::debug!(
            "Director processed tick {}: {} events, {} commentary items",
            current_tick,
            events.len(),
            commentary_count
        );
    }
}

/// Load events from events.jsonl that occurred between from_tick (exclusive) and to_tick (inclusive).
fn load_events_since(from_tick: u64, to_tick: u64) -> Vec<Event> {
    let path = Path::new("output/events.jsonl");
    if !path.exists() {
        return Vec::new();
    }

    let Ok(file) = std::fs::File::open(path) else {
        return Vec::new();
    };

    let reader = std::io::BufReader::new(file);

    reader
        .lines()
        .filter_map(|line| line.ok())
        .filter_map(|line| serde_json::from_str::<Event>(&line).ok())
        .filter(|e| e.timestamp.tick > from_tick && e.timestamp.tick <= to_tick)
        .collect()
}

/// Load current tensions from tensions.json.
fn load_tensions() -> Vec<Tension> {
    let path = Path::new("output/tensions.json");

    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_director_runner_default() {
        let runner = DirectorRunner::default();
        assert_eq!(runner.last_processed_tick, 0);
    }
}
