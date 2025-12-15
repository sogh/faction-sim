//! Intervention system: allows users to intervene at a specific tick
//! and restart the simulation from that point.

use bevy::prelude::*;

use crate::overlay::PlaybackState;
use crate::sim_runner::{find_snapshot_at_or_before, SimConfig, SimRunner};
use crate::state_loader::SimulationState;

/// Plugin for intervention handling.
pub struct InterventionPlugin;

impl Plugin for InterventionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InterventionState>()
            .add_event::<InterventionEvent>()
            .add_systems(
                Update,
                (
                    handle_intervention_input,
                    process_intervention,
                ),
            );
    }
}

/// Resource tracking intervention state.
#[derive(Resource, Default)]
pub struct InterventionState {
    /// Pending intervention tick (None if no intervention pending).
    pub pending_tick: Option<u64>,
    /// Number of interventions performed this session.
    pub intervention_count: u32,
}

/// Event fired when an intervention is triggered.
#[derive(Event, Debug, Clone)]
pub struct InterventionEvent {
    /// The tick at which the intervention was triggered.
    pub tick: u64,
    /// Path to the snapshot file used for restart.
    pub snapshot_path: std::path::PathBuf,
}

/// System to handle intervention input (I key).
fn handle_intervention_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<SimulationState>,
    config: Res<SimConfig>,
    mut intervention_state: ResMut<InterventionState>,
    mut playback: ResMut<PlaybackState>,
) {
    if keyboard.just_pressed(KeyCode::KeyI) {
        let current_tick = state.current_tick();

        // Check if we have a valid tick to intervene at
        if current_tick == 0 {
            tracing::warn!("Cannot intervene at tick 0");
            return;
        }

        // Find the nearest snapshot
        if let Some(_snapshot_path) = find_snapshot_at_or_before(&config.output_dir, current_tick) {
            intervention_state.pending_tick = Some(current_tick);
            playback.playing = false; // Pause playback during intervention
            tracing::info!("Intervention triggered at tick {}", current_tick);
        } else {
            tracing::warn!("No snapshot available before tick {}", current_tick);
        }
    }
}

/// System to process pending interventions.
fn process_intervention(
    mut intervention_state: ResMut<InterventionState>,
    mut sim_runner: ResMut<SimRunner>,
    config: Res<SimConfig>,
    mut events: EventWriter<InterventionEvent>,
) {
    let Some(tick) = intervention_state.pending_tick.take() else {
        return;
    };

    // Find the snapshot to restart from
    let Some(snapshot_path) = find_snapshot_at_or_before(&config.output_dir, tick) else {
        tracing::error!("Could not find snapshot for intervention at tick {}", tick);
        return;
    };

    // Stop current simulation if running
    if sim_runner.is_running() {
        sim_runner.stop();
    }

    // Start new simulation from snapshot
    match sim_runner.intervene(snapshot_path.clone(), tick, &config) {
        Ok(()) => {
            intervention_state.intervention_count += 1;
            tracing::info!(
                "Intervention #{} started from snapshot at tick {}",
                intervention_state.intervention_count,
                tick
            );
            events.send(InterventionEvent {
                tick,
                snapshot_path,
            });
        }
        Err(e) => {
            tracing::error!("Failed to start intervention: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intervention_state_default() {
        let state = InterventionState::default();
        assert!(state.pending_tick.is_none());
        assert_eq!(state.intervention_count, 0);
    }
}
