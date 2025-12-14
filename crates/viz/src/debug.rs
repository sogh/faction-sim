//! Debug overlay for development information display.
//!
//! Shows FPS, camera info, agent count, and other debug information.
//! Toggle with F3 key.

use bevy::prelude::*;
use std::collections::VecDeque;

use crate::agents::VisualAgent;
use crate::camera::CameraController;
use crate::state_loader::SimulationState;

/// Plugin for the debug overlay.
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugOverlay>()
            .add_systems(Startup, setup_debug_overlay)
            .add_systems(
                Update,
                (toggle_debug_overlay, update_debug_display, update_debug_text),
            );
    }
}

/// Resource controlling debug overlay settings.
#[derive(Resource)]
pub struct DebugOverlay {
    /// Whether the debug overlay is visible.
    pub enabled: bool,
    /// Show agent IDs above agents.
    pub show_agent_ids: bool,
    /// Show world positions.
    pub show_positions: bool,
    /// Show FPS counter.
    pub show_fps: bool,
    /// Show camera information.
    pub show_camera_info: bool,
}

impl Default for DebugOverlay {
    fn default() -> Self {
        Self {
            enabled: false,
            show_agent_ids: true,
            show_positions: false,
            show_fps: true,
            show_camera_info: true,
        }
    }
}

/// Component marking the debug overlay container.
#[derive(Component)]
pub struct DebugOverlayContainer;

/// Component for the debug text.
#[derive(Component)]
pub struct DebugText;

/// Local resource for FPS history.
#[derive(Default)]
struct FpsHistory {
    history: VecDeque<f32>,
}

impl FpsHistory {
    fn push(&mut self, fps: f32) {
        self.history.push_back(fps);
        if self.history.len() > 60 {
            self.history.pop_front();
        }
    }

    fn average(&self) -> f32 {
        if self.history.is_empty() {
            0.0
        } else {
            self.history.iter().sum::<f32>() / self.history.len() as f32
        }
    }
}

/// System to set up the debug overlay UI.
fn setup_debug_overlay(mut commands: Commands) {
    // Debug overlay container (top-left)
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(50.0), // Below status bar
                    left: Val::Px(10.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: Color::srgba(0.0, 0.0, 0.0, 0.8).into(),
                visibility: Visibility::Hidden,
                ..default()
            },
            DebugOverlayContainer,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn(TextBundle::from_section(
                "DEBUG (F3 to toggle)",
                TextStyle {
                    font_size: 14.0,
                    color: Color::srgb(0.9, 0.9, 0.3),
                    ..default()
                },
            ));

            // Debug info text
            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size: 12.0,
                        color: Color::srgb(0.8, 0.8, 0.8),
                        ..default()
                    },
                ),
                DebugText,
            ));
        });
}

/// System to toggle debug overlay with F3.
fn toggle_debug_overlay(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut debug_overlay: ResMut<DebugOverlay>,
    mut container: Query<&mut Visibility, With<DebugOverlayContainer>>,
) {
    if keyboard.just_pressed(KeyCode::F3) {
        debug_overlay.enabled = !debug_overlay.enabled;

        for mut visibility in container.iter_mut() {
            *visibility = if debug_overlay.enabled {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }

        let status = if debug_overlay.enabled { "ON" } else { "OFF" };
        tracing::info!("Debug overlay: {}", status);
    }
}

/// System to update debug display data.
fn update_debug_display(
    debug_overlay: Res<DebugOverlay>,
    camera: Res<CameraController>,
    state: Res<SimulationState>,
    agents: Query<&VisualAgent>,
    time: Res<Time>,
    mut fps_history: Local<FpsHistory>,
    mut debug_text: Query<&mut Text, With<DebugText>>,
) {
    if !debug_overlay.enabled {
        return;
    }

    // Calculate FPS
    let dt = time.delta_seconds();
    if dt > 0.0 {
        fps_history.push(1.0 / dt);
    }
    let avg_fps = fps_history.average();

    // Count agents
    let agent_count = agents.iter().count();

    // Get tick
    let tick = state.current_tick();

    // Camera mode string
    let mode_str = match &camera.mode {
        crate::camera::CameraMode::Manual => "Manual",
        crate::camera::CameraMode::Director { .. } => "Director",
        crate::camera::CameraMode::FollowAgent { agent_id } => {
            // Can't return a reference to a temporary, so just note it's following
            return update_debug_text_with_follow(
                &debug_overlay,
                avg_fps,
                &camera,
                agent_count,
                tick,
                agent_id,
                &mut debug_text,
            );
        }
    };

    // Build debug text
    let mut lines = Vec::new();

    if debug_overlay.show_fps {
        let fps_color = if avg_fps < 30.0 { "LOW!" } else { "" };
        lines.push(format!("FPS: {:.0} {}", avg_fps, fps_color));
    }

    if debug_overlay.show_camera_info {
        lines.push(format!(
            "Camera: ({:.0}, {:.0})",
            camera.position.x, camera.position.y
        ));
        lines.push(format!("Zoom: {:.2}x", camera.zoom));
        lines.push(format!("Mode: {}", mode_str));
    }

    lines.push(format!("Agents: {}", agent_count));
    lines.push(format!("Tick: {}", tick));

    if let Some(ref error) = state.last_error {
        lines.push(format!("ERROR: {}", error));
    }

    // Update text
    for mut text in debug_text.iter_mut() {
        text.sections[0].value = lines.join("\n");
    }
}

/// Helper to update debug text when following an agent.
fn update_debug_text_with_follow(
    debug_overlay: &DebugOverlay,
    avg_fps: f32,
    camera: &CameraController,
    agent_count: usize,
    tick: u64,
    agent_id: &str,
    debug_text: &mut Query<&mut Text, With<DebugText>>,
) {
    let mut lines = Vec::new();

    if debug_overlay.show_fps {
        let fps_color = if avg_fps < 30.0 { "LOW!" } else { "" };
        lines.push(format!("FPS: {:.0} {}", avg_fps, fps_color));
    }

    if debug_overlay.show_camera_info {
        lines.push(format!(
            "Camera: ({:.0}, {:.0})",
            camera.position.x, camera.position.y
        ));
        lines.push(format!("Zoom: {:.2}x", camera.zoom));
        lines.push(format!("Mode: Following {}", agent_id));
    }

    lines.push(format!("Agents: {}", agent_count));
    lines.push(format!("Tick: {}", tick));

    for mut text in debug_text.iter_mut() {
        text.sections[0].value = lines.join("\n");
    }
}

/// System to update debug text styling based on performance.
fn update_debug_text(
    debug_overlay: Res<DebugOverlay>,
    time: Res<Time>,
    mut fps_history: Local<FpsHistory>,
    mut debug_text: Query<&mut Text, With<DebugText>>,
) {
    if !debug_overlay.enabled {
        return;
    }

    let dt = time.delta_seconds();
    if dt > 0.0 {
        fps_history.push(1.0 / dt);
    }

    let avg_fps = fps_history.average();

    // Change text color based on FPS
    for mut text in debug_text.iter_mut() {
        if let Some(section) = text.sections.first_mut() {
            section.style.color = if avg_fps < 30.0 {
                Color::srgb(1.0, 0.3, 0.3) // Red for low FPS
            } else if avg_fps < 55.0 {
                Color::srgb(1.0, 0.8, 0.3) // Yellow for medium FPS
            } else {
                Color::srgb(0.8, 0.8, 0.8) // Gray for good FPS
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_overlay_default() {
        let overlay = DebugOverlay::default();
        assert!(!overlay.enabled);
        assert!(overlay.show_fps);
        assert!(overlay.show_camera_info);
    }

    #[test]
    fn test_fps_history() {
        let mut history = FpsHistory::default();
        assert_eq!(history.average(), 0.0);

        history.push(60.0);
        history.push(60.0);
        assert_eq!(history.average(), 60.0);

        history.push(30.0);
        assert!((history.average() - 50.0).abs() < 0.01);
    }
}
