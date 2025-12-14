//! Director integration: loading camera instructions and commentary.
//!
//! This module bridges the Director AI output to the visualization layer,
//! handling camera instruction processing and commentary queue management.

use bevy::prelude::*;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::Instant;

use crate::agents::VisualAgent;
use crate::camera::{CameraController, CameraMode};
use crate::state_loader::SimulationState;
use crate::world::LocationPositions;

/// Plugin for director integration.
pub struct DirectorPlugin;

impl Plugin for DirectorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DirectorState>()
            .init_resource::<ManualOverride>()
            .add_event::<DirectorUpdatedEvent>()
            .add_systems(
                Update,
                (
                    check_director_files,
                    toggle_director_mode,
                    process_director_instructions,
                    update_follow_camera,
                    check_manual_override_timeout,
                )
                    .chain(),
            );
    }
}

/// Director state resource holding camera script and commentary.
#[derive(Resource)]
pub struct DirectorState {
    /// Camera instructions from director.
    pub camera_script: Vec<director::CameraInstruction>,
    /// Commentary items to display.
    pub commentary_queue: VecDeque<director::CommentaryItem>,
    /// Current instruction index.
    pub current_instruction_index: usize,
    /// Whether director mode is enabled.
    pub enabled: bool,
    /// Path to camera script file.
    pub camera_script_path: Option<PathBuf>,
    /// Path to commentary file.
    pub commentary_path: Option<PathBuf>,
    /// Last modification time of camera script.
    last_camera_modified: Option<std::time::SystemTime>,
    /// Last modification time of commentary.
    last_commentary_modified: Option<std::time::SystemTime>,
}

impl Default for DirectorState {
    fn default() -> Self {
        Self {
            camera_script: Vec::new(),
            commentary_queue: VecDeque::new(),
            current_instruction_index: 0,
            enabled: true, // Director mode enabled by default
            camera_script_path: Some(PathBuf::from("output/camera_script.json")),
            commentary_path: Some(PathBuf::from("output/commentary.json")),
            last_camera_modified: None,
            last_commentary_modified: None,
        }
    }
}

impl DirectorState {
    /// Get the current camera instruction for the given tick.
    pub fn current_instruction(&self, tick: u64) -> Option<&director::CameraInstruction> {
        // Find the most recent instruction that's still valid for this tick
        self.camera_script.iter().rev().find(|instr| {
            instr.timestamp.tick <= tick
                && instr
                    .valid_until
                    .as_ref()
                    .map(|v| v.tick >= tick)
                    .unwrap_or(true)
        })
    }

    /// Get the next commentary item to display.
    pub fn next_commentary(&mut self) -> Option<director::CommentaryItem> {
        self.commentary_queue.pop_front()
    }

    /// Add a commentary item to the queue.
    pub fn add_commentary(&mut self, item: director::CommentaryItem) {
        self.commentary_queue.push_back(item);
    }
}

/// Event fired when director files are updated.
#[derive(Event)]
pub struct DirectorUpdatedEvent {
    /// Whether camera script was updated.
    pub camera_updated: bool,
    /// Whether commentary was updated.
    pub commentary_updated: bool,
}

/// Resource tracking manual override state.
#[derive(Resource)]
pub struct ManualOverride {
    /// Whether manual override is active.
    pub active: bool,
    /// When the override started.
    pub started_at: Option<Instant>,
    /// Timeout duration in seconds.
    pub timeout_secs: f32,
}

impl Default for ManualOverride {
    fn default() -> Self {
        Self {
            active: false,
            started_at: None,
            timeout_secs: 5.0, // Return to director after 5 seconds of inactivity
        }
    }
}

impl ManualOverride {
    /// Start a manual override.
    pub fn start(&mut self) {
        self.active = true;
        self.started_at = Some(Instant::now());
    }

    /// Check if the override has timed out.
    pub fn is_timed_out(&self) -> bool {
        if !self.active {
            return false;
        }
        self.started_at
            .map(|t| t.elapsed().as_secs_f32() > self.timeout_secs)
            .unwrap_or(false)
    }

    /// Clear the override.
    pub fn clear(&mut self) {
        self.active = false;
        self.started_at = None;
    }
}

/// System to check for director file updates.
fn check_director_files(
    mut director: ResMut<DirectorState>,
    mut events: EventWriter<DirectorUpdatedEvent>,
) {
    let mut camera_updated = false;
    let mut commentary_updated = false;

    // Clone paths to avoid borrow issues
    let camera_path = director.camera_script_path.clone();
    let commentary_path = director.commentary_path.clone();
    let last_camera_modified = director.last_camera_modified;
    let last_commentary_modified = director.last_commentary_modified;

    // Check camera script file
    if let Some(path) = camera_path {
        if path.exists() {
            if let Ok(metadata) = std::fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    let should_reload = last_camera_modified
                        .map(|last| modified > last)
                        .unwrap_or(true);

                    if should_reload {
                        if load_camera_script(&path, &mut director) {
                            director.last_camera_modified = Some(modified);
                            camera_updated = true;
                        }
                    }
                }
            }
        }
    }

    // Check commentary file
    if let Some(path) = commentary_path {
        if path.exists() {
            if let Ok(metadata) = std::fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    let should_reload = last_commentary_modified
                        .map(|last| modified > last)
                        .unwrap_or(true);

                    if should_reload {
                        if load_commentary(&path, &mut director) {
                            director.last_commentary_modified = Some(modified);
                            commentary_updated = true;
                        }
                    }
                }
            }
        }
    }

    if camera_updated || commentary_updated {
        events.send(DirectorUpdatedEvent {
            camera_updated,
            commentary_updated,
        });
    }
}

/// Load camera script from file.
fn load_camera_script(path: &PathBuf, director: &mut DirectorState) -> bool {
    match std::fs::read_to_string(path) {
        Ok(contents) => {
            match serde_json::from_str::<Vec<director::CameraInstruction>>(&contents) {
                Ok(instructions) => {
                    director.camera_script = instructions;
                    director.current_instruction_index = 0;
                    tracing::info!(
                        "Loaded {} camera instructions from {:?}",
                        director.camera_script.len(),
                        path
                    );
                    true
                }
                Err(e) => {
                    tracing::warn!("Failed to parse camera script: {}", e);
                    false
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to read camera script: {}", e);
            false
        }
    }
}

/// Load commentary from file.
fn load_commentary(path: &PathBuf, director: &mut DirectorState) -> bool {
    match std::fs::read_to_string(path) {
        Ok(contents) => {
            match serde_json::from_str::<Vec<director::CommentaryItem>>(&contents) {
                Ok(items) => {
                    director.commentary_queue = items.into_iter().collect();
                    tracing::info!(
                        "Loaded {} commentary items from {:?}",
                        director.commentary_queue.len(),
                        path
                    );
                    true
                }
                Err(e) => {
                    tracing::warn!("Failed to parse commentary: {}", e);
                    false
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to read commentary: {}", e);
            false
        }
    }
}

/// System to toggle director mode with D key.
fn toggle_director_mode(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut director: ResMut<DirectorState>,
    mut camera: ResMut<CameraController>,
    mut manual_override: ResMut<ManualOverride>,
) {
    if keyboard.just_pressed(KeyCode::KeyD) {
        director.enabled = !director.enabled;

        if director.enabled {
            camera.set_director();
            manual_override.clear();
            tracing::info!("Director mode enabled");
        } else {
            camera.set_manual();
            tracing::info!("Director mode disabled");
        }
    }
}

/// System to process director camera instructions.
fn process_director_instructions(
    director: Res<DirectorState>,
    state: Res<SimulationState>,
    mut camera: ResMut<CameraController>,
    manual_override: Res<ManualOverride>,
    location_positions: Res<LocationPositions>,
    agents: Query<(&VisualAgent, &Transform)>,
) {
    // Skip if director is disabled or manual override is active
    if !director.enabled || manual_override.active {
        return;
    }

    // Skip if not in director mode
    if !camera.is_director() {
        return;
    }

    let current_tick = state.current_tick();

    // Find current instruction
    let Some(instruction) = director.current_instruction(current_tick) else {
        return;
    };

    // Apply the camera instruction
    apply_camera_instruction(&mut camera, instruction, &location_positions, &agents);
}

/// Apply a camera instruction to the controller.
fn apply_camera_instruction(
    camera: &mut CameraController,
    instruction: &director::CameraInstruction,
    location_positions: &LocationPositions,
    agents: &Query<(&VisualAgent, &Transform)>,
) {
    match &instruction.camera_mode {
        director::CameraMode::FollowAgent { agent_id, zoom } => {
            // Switch to follow mode
            camera.mode = CameraMode::FollowAgent {
                agent_id: agent_id.clone(),
            };
            camera.target_zoom = zoom_level_to_f32(zoom);
        }
        director::CameraMode::FrameLocation { location_id, zoom } => {
            // Get location position and transition to it
            let position = location_positions.get(location_id);
            let zoom_value = zoom_level_to_f32(zoom);
            let duration = pacing_to_duration(&instruction.pacing);
            camera.begin_transition(position, zoom_value, duration);
            camera.mode = CameraMode::Director {
                instruction: Some(crate::camera::CameraInstruction {
                    target: crate::camera::CameraTarget::Position(position),
                    zoom: zoom_value,
                    duration,
                }),
            };
        }
        director::CameraMode::FrameMultiple {
            agent_ids,
            auto_zoom,
        } => {
            let (center, zoom) = calculate_framing(agent_ids, agents, 100.0);
            let zoom_value = if *auto_zoom { zoom } else { 1.0 };
            let duration = pacing_to_duration(&instruction.pacing);
            camera.begin_transition(center, zoom_value, duration);
            camera.mode = CameraMode::Director {
                instruction: Some(crate::camera::CameraInstruction {
                    target: crate::camera::CameraTarget::MultipleAgents(agent_ids.clone()),
                    zoom: zoom_value,
                    duration,
                }),
            };
        }
        director::CameraMode::Overview { region } => {
            // Zoom out to overview
            let position = region
                .as_ref()
                .map(|r| location_positions.get(r))
                .unwrap_or(Vec2::ZERO);
            let duration = pacing_to_duration(&instruction.pacing);
            camera.begin_transition(position, 0.5, duration);
            camera.mode = CameraMode::Director {
                instruction: Some(crate::camera::CameraInstruction {
                    target: crate::camera::CameraTarget::Region {
                        center: position,
                        size: Vec2::new(2000.0, 2000.0),
                    },
                    zoom: 0.5,
                    duration,
                }),
            };
        }
        director::CameraMode::Cinematic {
            path,
            duration_ticks: _,
        } => {
            // For cinematic mode, start with first waypoint
            if let Some(first) = path.first() {
                let position = location_positions.get(&first.target);
                let zoom_value = zoom_level_to_f32(&first.zoom);
                let duration = pacing_to_duration(&instruction.pacing);
                camera.begin_transition(position, zoom_value, duration);
            }
        }
    }
}

/// Convert director ZoomLevel to f32.
fn zoom_level_to_f32(level: &director::ZoomLevel) -> f32 {
    match level {
        director::ZoomLevel::Extreme => 3.0,
        director::ZoomLevel::Close => 2.0,
        director::ZoomLevel::Medium => 1.0,
        director::ZoomLevel::Wide => 0.6,
        director::ZoomLevel::Regional => 0.3,
    }
}

/// Convert PacingHint to transition duration in seconds.
fn pacing_to_duration(pacing: &director::PacingHint) -> f32 {
    match pacing {
        director::PacingHint::Slow => 2.0,
        director::PacingHint::Normal => 1.0,
        director::PacingHint::Urgent => 0.3,
        director::PacingHint::Climactic => 1.5,
    }
}

/// Calculate framing for multiple agents.
fn calculate_framing(
    agent_ids: &[String],
    agents: &Query<(&VisualAgent, &Transform)>,
    padding: f32,
) -> (Vec2, f32) {
    let positions: Vec<Vec2> = agent_ids
        .iter()
        .filter_map(|id| {
            agents
                .iter()
                .find(|(a, _)| a.agent_id == *id)
                .map(|(_, t)| t.translation.truncate())
        })
        .collect();

    if positions.is_empty() {
        return (Vec2::ZERO, 1.0);
    }

    if positions.len() == 1 {
        return (positions[0], 1.5);
    }

    let min = positions.iter().fold(Vec2::MAX, |a, &b| a.min(b));
    let max = positions.iter().fold(Vec2::MIN, |a, &b| a.max(b));
    let center = (min + max) / 2.0;
    let size = (max - min) + Vec2::splat(padding * 2.0);
    // Calculate zoom to fit all agents (assuming ~800 pixel viewport width)
    let zoom = (800.0 / size.max_element()).clamp(0.5, 2.0);

    (center, zoom)
}

/// System to update camera when following an agent.
fn update_follow_camera(
    mut camera: ResMut<CameraController>,
    agents: Query<(&VisualAgent, &Transform)>,
    time: Res<Time>,
) {
    if let CameraMode::FollowAgent { ref agent_id } = camera.mode {
        // Find the agent
        for (agent, transform) in agents.iter() {
            if agent.agent_id == *agent_id {
                let target = transform.translation.truncate();

                // Smooth follow with slight lag
                let follow_speed = 3.0;
                let lerp_factor = (follow_speed * time.delta_seconds()).min(1.0);
                camera.target_position = target;
                camera.position = camera.position.lerp(target, lerp_factor);
                break;
            }
        }
    }
}

/// System to check manual override timeout and return to director.
fn check_manual_override_timeout(
    mut camera: ResMut<CameraController>,
    director: Res<DirectorState>,
    mut manual_override: ResMut<ManualOverride>,
) {
    if !director.enabled {
        return;
    }

    if manual_override.is_timed_out() {
        tracing::info!("Manual override timeout, returning to director mode");
        camera.set_director();
        manual_override.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manual_override_default() {
        let override_state = ManualOverride::default();
        assert!(!override_state.active);
        assert!(override_state.started_at.is_none());
    }

    #[test]
    fn test_zoom_level_conversion() {
        assert_eq!(zoom_level_to_f32(&director::ZoomLevel::Close), 2.0);
        assert_eq!(zoom_level_to_f32(&director::ZoomLevel::Medium), 1.0);
        assert_eq!(zoom_level_to_f32(&director::ZoomLevel::Wide), 0.6);
    }

    #[test]
    fn test_pacing_to_duration() {
        assert_eq!(pacing_to_duration(&director::PacingHint::Slow), 2.0);
        assert_eq!(pacing_to_duration(&director::PacingHint::Normal), 1.0);
        assert_eq!(pacing_to_duration(&director::PacingHint::Urgent), 0.3);
        assert_eq!(pacing_to_duration(&director::PacingHint::Climactic), 1.5);
    }
}
