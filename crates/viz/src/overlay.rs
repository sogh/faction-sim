//! UI overlays: commentary display, status bar, playback controls.

use bevy::prelude::*;
use std::collections::VecDeque;

use crate::agents::{AgentSelectedEvent, VisualAgent};
use crate::camera::CameraController;
use crate::director_state::DirectorState;
use crate::sim_runner::{SimRunner, SimStatus};
use crate::state_loader::SimulationState;
use crate::world::LocationPositions;

/// Plugin for UI overlay rendering.
pub struct OverlayPlugin;

impl Plugin for OverlayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlaybackState>()
            .init_resource::<SelectedAgentInfo>()
            .add_systems(Startup, setup_overlay_ui)
            .add_systems(
                Update,
                (
                    update_status_bar,
                    update_sim_status_display,
                    update_commentary_display,
                    fade_commentary,
                    update_playback_controls,
                    handle_playback_input,
                    update_agent_selection_info,
                    update_agent_info_panel,
                ),
            );
    }
}

/// Resource tracking playback state.
#[derive(Resource)]
pub struct PlaybackState {
    /// Whether playback is running.
    pub playing: bool,
    /// Playback speed multiplier (ticks per second).
    pub speed: f32,
    /// Current playback tick (fractional for smooth interpolation).
    pub current_tick: f64,
    /// Maximum tick available from snapshots.
    pub max_available_tick: u64,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            playing: true,
            speed: 1.0,
            current_tick: 0.0,
            max_available_tick: 0,
        }
    }
}

impl PlaybackState {
    /// Get the current tick as an integer for snapshot selection.
    pub fn tick_for_snapshot(&self) -> u64 {
        self.current_tick as u64
    }
}

/// Component marking the root UI node.
#[derive(Component)]
pub struct OverlayRoot;

/// Component marking the status bar.
#[derive(Component)]
pub struct StatusBar;

/// Component marking the tick display text.
#[derive(Component)]
pub struct TickDisplay;

/// Component marking the camera mode display.
#[derive(Component)]
pub struct CameraModeDisplay;

/// Component marking the simulation status display.
#[derive(Component)]
pub struct SimStatusDisplay;

/// Component marking the commentary container.
#[derive(Component)]
pub struct CommentaryContainer;

/// Component for displayed commentary items.
#[derive(Component)]
pub struct DisplayedCommentary {
    /// Unique ID of this commentary item.
    pub item_id: String,
    /// Remaining time to display (seconds).
    pub time_remaining: f32,
    /// Current opacity (for fade effects).
    pub opacity: f32,
}

/// Component marking the playback controls container.
#[derive(Component)]
pub struct PlaybackControls;

/// Component for play/pause button.
#[derive(Component)]
pub struct PlayPauseButton;

/// Component for speed buttons.
#[derive(Component)]
pub struct SpeedButton(pub f32);

/// Resource tracking selected agent info.
#[derive(Resource, Default)]
pub struct SelectedAgentInfo {
    /// The currently selected agent ID.
    pub agent_id: Option<String>,
    /// Agent's display name.
    pub name: Option<String>,
    /// Agent's faction.
    pub faction: Option<String>,
    /// Agent's role.
    pub role: Option<String>,
    /// Agent's current location ID.
    pub location: Option<String>,
    /// Agent's world position.
    pub position: Option<Vec2>,
}

/// Component marking the agent info panel container.
#[derive(Component)]
pub struct AgentInfoPanel;

/// Component marking the agent info text.
#[derive(Component)]
pub struct AgentInfoText;

/// System to set up the overlay UI structure.
fn setup_overlay_ui(mut commands: Commands) {
    // Root UI node covering the entire screen
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                ..default()
            },
            OverlayRoot,
        ))
        .with_children(|parent| {
            // Top bar (status info)
            spawn_status_bar(parent);

            // Middle spacer (allows clicks through to world)
            parent.spawn(NodeBundle {
                style: Style {
                    flex_grow: 1.0,
                    ..default()
                },
                ..default()
            });

            // Bottom area (commentary and controls)
            spawn_bottom_area(parent);
        });

    // Agent info panel (floating, top-right)
    spawn_agent_info_panel(&mut commands);

    tracing::info!("Overlay UI initialized");
}

/// Spawn the agent info panel (floating panel on the right side).
fn spawn_agent_info_panel(commands: &mut Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(50.0),
                    right: Val::Px(10.0),
                    width: Val::Px(250.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                },
                background_color: Color::srgba(0.0, 0.0, 0.0, 0.85).into(),
                visibility: Visibility::Hidden,
                z_index: ZIndex::Global(50),
                ..default()
            },
            AgentInfoPanel,
        ))
        .with_children(|parent| {
            // Panel title
            parent.spawn(TextBundle::from_section(
                "Selected Agent",
                TextStyle {
                    font_size: 16.0,
                    color: Color::srgb(0.9, 0.8, 0.6),
                    ..default()
                },
            ));

            // Separator line
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(4.0)),
                    ..default()
                },
                background_color: Color::srgb(0.4, 0.4, 0.4).into(),
                ..default()
            });

            // Agent info text (will be updated dynamically)
            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size: 14.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                AgentInfoText,
            ));
        });
}

/// Spawn the top status bar.
fn spawn_status_bar(parent: &mut ChildBuilder) {
    parent
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Px(40.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::srgba(0.0, 0.0, 0.0, 0.7).into(),
                ..default()
            },
            StatusBar,
        ))
        .with_children(|parent| {
            // Left side: Tick display
            parent.spawn((
                TextBundle::from_section(
                    "Tick: 0 | Year 1, Spring, Day 1",
                    TextStyle {
                        font_size: 16.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                TickDisplay,
            ));

            // Center: Simulation status display
            parent.spawn((
                TextBundle::from_section(
                    "Sim: Idle | [S] Start",
                    TextStyle {
                        font_size: 14.0,
                        color: Color::srgb(0.8, 0.8, 0.5),
                        ..default()
                    },
                ),
                SimStatusDisplay,
            ));

            // Right side: Camera mode display
            parent.spawn((
                TextBundle::from_section(
                    "Camera: Manual",
                    TextStyle {
                        font_size: 16.0,
                        color: Color::srgb(0.7, 0.7, 0.7),
                        ..default()
                    },
                ),
                CameraModeDisplay,
            ));
        });
}

/// Spawn the bottom area with commentary and controls.
fn spawn_bottom_area(parent: &mut ChildBuilder) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // Commentary area
            spawn_commentary_area(parent);

            // Playback controls
            spawn_playback_controls(parent);
        });
}

/// Spawn the commentary display area.
fn spawn_commentary_area(parent: &mut ChildBuilder) {
    parent.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                min_height: Val::Px(60.0),
                padding: UiRect::all(Val::Px(15.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: Color::srgba(0.0, 0.0, 0.0, 0.6).into(),
            ..default()
        },
        CommentaryContainer,
    ));
}

/// Spawn the playback controls.
fn spawn_playback_controls(parent: &mut ChildBuilder) {
    parent
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Px(50.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    ..default()
                },
                background_color: Color::srgba(0.1, 0.1, 0.1, 0.8).into(),
                ..default()
            },
            PlaybackControls,
        ))
        .with_children(|parent| {
            // Play/Pause button
            spawn_control_button(parent, "||", PlayPauseButton);

            // Separator
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(1.0),
                    height: Val::Px(20.0),
                    ..default()
                },
                background_color: Color::srgb(0.3, 0.3, 0.3).into(),
                ..default()
            });

            // Speed buttons
            spawn_speed_button(parent, "0.5x", 0.5);
            spawn_speed_button(parent, "1x", 1.0);
            spawn_speed_button(parent, "2x", 2.0);

            // Separator
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(1.0),
                    height: Val::Px(20.0),
                    ..default()
                },
                background_color: Color::srgb(0.3, 0.3, 0.3).into(),
                ..default()
            });

            // Director mode indicator
            parent.spawn(TextBundle::from_section(
                "[D] Director",
                TextStyle {
                    font_size: 14.0,
                    color: Color::srgb(0.6, 0.6, 0.6),
                    ..default()
                },
            ));
        });
}

/// Spawn a control button.
fn spawn_control_button(parent: &mut ChildBuilder, label: &str, marker: impl Component) {
    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(40.0),
                    height: Val::Px(30.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::srgb(0.2, 0.2, 0.2).into(),
                ..default()
            },
            marker,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                label,
                TextStyle {
                    font_size: 14.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
        });
}

/// Spawn a speed button.
fn spawn_speed_button(parent: &mut ChildBuilder, label: &str, speed: f32) {
    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(50.0),
                    height: Val::Px(30.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::srgb(0.2, 0.2, 0.2).into(),
                ..default()
            },
            SpeedButton(speed),
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                label,
                TextStyle {
                    font_size: 14.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
        });
}

/// System to update the status bar.
fn update_status_bar(
    state: Res<SimulationState>,
    camera: Res<CameraController>,
    director: Res<DirectorState>,
    playback: Res<PlaybackState>,
    mut tick_query: Query<&mut Text, (With<TickDisplay>, Without<CameraModeDisplay>)>,
    mut mode_query: Query<&mut Text, (With<CameraModeDisplay>, Without<TickDisplay>)>,
) {
    // Update tick display - shows playback position and loaded snapshot info
    for mut text in tick_query.iter_mut() {
        if let Some(ref snapshot) = state.snapshot {
            let playback_tick = playback.tick_for_snapshot();
            let max_tick = playback.max_available_tick;
            let play_status = if playback.playing { "▶" } else { "⏸" };

            text.sections[0].value = format!(
                "{} Tick {}/{} | Year {}, {}, Day {}",
                play_status,
                playback_tick,
                max_tick,
                snapshot.timestamp.date.year,
                snapshot.timestamp.date.season,
                snapshot.timestamp.date.day,
            );
        } else {
            text.sections[0].value = "Viewing: No data yet".to_string();
        }
    }

    // Update camera mode display
    for mut text in mode_query.iter_mut() {
        let mode_str = if director.enabled {
            if camera.is_director() {
                "Director"
            } else {
                "Manual (override)"
            }
        } else {
            "Manual"
        };
        text.sections[0].value = format!("Camera: {}", mode_str);
    }
}

/// System to update simulation status display.
fn update_sim_status_display(
    sim_runner: Res<SimRunner>,
    mut status_query: Query<&mut Text, With<SimStatusDisplay>>,
) {
    for mut text in status_query.iter_mut() {
        let (status_text, color) = match &sim_runner.status {
            SimStatus::Idle => ("Simulated: None | [S] Start".to_string(), Color::srgb(0.6, 0.6, 0.6)),
            SimStatus::Starting => ("Simulating...".to_string(), Color::srgb(0.8, 0.8, 0.5)),
            SimStatus::Running { current_tick, max_ticks } => {
                let percent = (*current_tick as f32 / *max_ticks as f32 * 100.0) as u32;
                (
                    format!("Simulated: {}/{} ticks ({}%)", current_tick, max_ticks, percent),
                    Color::srgb(0.5, 0.8, 0.5),
                )
            }
            SimStatus::PausedAhead { paused_at_tick, max_ticks } => {
                (
                    format!("Simulated: {}/{} (paused - ahead of playback)", paused_at_tick, max_ticks),
                    Color::srgb(0.8, 0.7, 0.4),
                )
            }
            SimStatus::Completed { final_tick } => {
                (format!("Simulated: {} ticks (done)", final_tick), Color::srgb(0.5, 0.8, 0.5))
            }
            SimStatus::Failed(error) => {
                let short_error = if error.len() > 30 {
                    format!("{}...", &error[..30])
                } else {
                    error.clone()
                };
                (format!("Sim FAILED: {}", short_error), Color::srgb(0.9, 0.4, 0.4))
            }
        };

        text.sections[0].value = status_text;
        text.sections[0].style.color = color;
    }
}

/// System to update commentary display.
fn update_commentary_display(
    mut commands: Commands,
    director: Res<DirectorState>,
    container_query: Query<Entity, With<CommentaryContainer>>,
    existing: Query<&DisplayedCommentary>,
    mut displayed_ids: Local<VecDeque<String>>,
) {
    let Ok(container) = container_query.get_single() else {
        return;
    };

    // Check for new commentary items
    for item in &director.commentary_queue {
        // Skip if already displayed
        if existing.iter().any(|d| d.item_id == item.item_id) {
            continue;
        }

        if displayed_ids.contains(&item.item_id) {
            continue;
        }

        // Determine style based on commentary type
        let (font_size, color, style_prefix) = match item.commentary_type {
            director::CommentaryType::EventCaption => (18.0, Color::WHITE, ""),
            director::CommentaryType::DramaticIrony => {
                (16.0, Color::srgb(0.9, 0.8, 0.5), "// ")
            }
            director::CommentaryType::ContextReminder => {
                (14.0, Color::srgb(0.7, 0.7, 0.7), "")
            }
            director::CommentaryType::TensionTeaser => {
                (16.0, Color::srgb(0.8, 0.6, 0.6), "")
            }
            director::CommentaryType::NarratorVoice => (18.0, Color::srgb(1.0, 0.95, 0.8), ""),
        };

        // Spawn commentary text
        let text_entity = commands
            .spawn((
                TextBundle::from_section(
                    format!("{}{}", style_prefix, item.content),
                    TextStyle {
                        font_size,
                        color,
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(5.0)),
                    ..default()
                }),
                DisplayedCommentary {
                    item_id: item.item_id.clone(),
                    time_remaining: item.display_duration_ticks as f32 / 60.0, // Convert ticks to seconds
                    opacity: 1.0,
                },
            ))
            .id();

        commands.entity(container).add_child(text_entity);
        displayed_ids.push_back(item.item_id.clone());

        // Limit displayed items
        if displayed_ids.len() > 3 {
            displayed_ids.pop_front();
        }
    }
}

/// System to fade and remove old commentary.
fn fade_commentary(
    mut commands: Commands,
    time: Res<Time>,
    mut commentary: Query<(Entity, &mut DisplayedCommentary, &mut Text)>,
) {
    let dt = time.delta_seconds();

    for (entity, mut displayed, mut text) in commentary.iter_mut() {
        displayed.time_remaining -= dt;

        // Fade out during last second
        if displayed.time_remaining < 1.0 && displayed.time_remaining > 0.0 {
            displayed.opacity = displayed.time_remaining;
            if let Some(section) = text.sections.first_mut() {
                let base_color = section.style.color;
                section.style.color = base_color.with_alpha(displayed.opacity);
            }
        }

        // Remove when expired
        if displayed.time_remaining <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// System to update playback control visuals.
fn update_playback_controls(
    playback: Res<PlaybackState>,
    mut play_button_query: Query<&Children, With<PlayPauseButton>>,
    mut play_button_text: Query<&mut Text>,
    mut speed_buttons: Query<(&SpeedButton, &mut BackgroundColor)>,
) {
    // Update play/pause button text
    for children in play_button_query.iter_mut() {
        for &child in children.iter() {
            if let Ok(mut text) = play_button_text.get_mut(child) {
                text.sections[0].value = if playback.playing {
                    "||".to_string()
                } else {
                    ">".to_string()
                };
            }
        }
    }

    // Highlight active speed button
    for (speed_button, mut bg_color) in speed_buttons.iter_mut() {
        let is_active = (speed_button.0 - playback.speed).abs() < 0.01;
        *bg_color = if is_active {
            Color::srgb(0.3, 0.5, 0.3).into()
        } else {
            Color::srgb(0.2, 0.2, 0.2).into()
        };
    }
}

/// System to handle playback control input.
fn handle_playback_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut playback: ResMut<PlaybackState>,
    play_button_query: Query<&Interaction, (Changed<Interaction>, With<PlayPauseButton>)>,
    speed_button_query: Query<(&Interaction, &SpeedButton), Changed<Interaction>>,
) {
    // Keyboard shortcuts
    if keyboard.just_pressed(KeyCode::Space) {
        playback.playing = !playback.playing;
    }
    if keyboard.just_pressed(KeyCode::Digit1) {
        playback.speed = 0.5;
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        playback.speed = 1.0;
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        playback.speed = 2.0;
    }

    // Button clicks
    for interaction in play_button_query.iter() {
        if *interaction == Interaction::Pressed {
            playback.playing = !playback.playing;
        }
    }

    for (interaction, speed_button) in speed_button_query.iter() {
        if *interaction == Interaction::Pressed {
            playback.speed = speed_button.0;
        }
    }
}

/// System to update selected agent info when selection changes.
fn update_agent_selection_info(
    mut events: EventReader<AgentSelectedEvent>,
    mut selected_info: ResMut<SelectedAgentInfo>,
    state: Res<SimulationState>,
    agents: Query<(&Transform, &VisualAgent)>,
    location_positions: Res<LocationPositions>,
) {
    for event in events.read() {
        match &event.agent_id {
            Some(agent_id) => {
                // Find the agent in the snapshot
                if let Some(ref snapshot) = state.snapshot {
                    if let Some(agent_snap) = snapshot.agents.iter().find(|a| &a.agent_id == agent_id) {
                        // Get position from visual agent if available
                        let position = agents
                            .iter()
                            .find(|(_, va)| &va.agent_id == agent_id)
                            .map(|(t, _)| t.translation.truncate())
                            .or_else(|| Some(location_positions.get(&agent_snap.location)));

                        selected_info.agent_id = Some(agent_id.clone());
                        selected_info.name = Some(agent_snap.name.clone());
                        selected_info.faction = Some(agent_snap.faction.clone());
                        selected_info.role = Some(agent_snap.role.clone());
                        selected_info.location = Some(agent_snap.location.clone());
                        selected_info.position = position;
                    }
                }
            }
            None => {
                // Clear selection
                *selected_info = SelectedAgentInfo::default();
            }
        }
    }
}

/// System to update the agent info panel display.
fn update_agent_info_panel(
    selected_info: Res<SelectedAgentInfo>,
    mut panel_query: Query<&mut Visibility, With<AgentInfoPanel>>,
    mut text_query: Query<&mut Text, With<AgentInfoText>>,
) {
    let Ok(mut panel_visibility) = panel_query.get_single_mut() else {
        return;
    };

    if selected_info.agent_id.is_some() {
        *panel_visibility = Visibility::Visible;

        // Update text content
        if let Ok(mut text) = text_query.get_single_mut() {
            let name = selected_info.name.as_deref().unwrap_or("Unknown");
            let agent_id = selected_info.agent_id.as_deref().unwrap_or("???");
            let faction = selected_info.faction.as_deref().unwrap_or("None");
            let role = selected_info.role.as_deref().unwrap_or("Unknown");
            let location = selected_info
                .location
                .as_ref()
                .map(|l| format_location_name(l))
                .unwrap_or_else(|| "Unknown".to_string());
            let coords = selected_info
                .position
                .map(|p| format!("({:.0}, {:.0})", p.x, p.y))
                .unwrap_or_else(|| "?".to_string());

            text.sections[0].value = format!(
                "Name: {}\nID: {}\nFaction: {}\nRole: {}\nLocation: {}\nCoords: {}",
                name, agent_id, faction, role, location, coords
            );
        }
    } else {
        *panel_visibility = Visibility::Hidden;
    }
}

/// Format a location ID into a human-readable display name.
fn format_location_name(location_id: &str) -> String {
    location_id
        .split('_')
        .map(|word| {
            let mut chars: Vec<char> = word.chars().collect();
            if let Some(first) = chars.first_mut() {
                *first = first.to_ascii_uppercase();
            }
            chars.into_iter().collect::<String>()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playback_state_default() {
        let state = PlaybackState::default();
        assert!(state.playing);
        assert_eq!(state.speed, 1.0);
    }
}
