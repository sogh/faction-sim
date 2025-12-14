//! Agent rendering: sprites, movement, and selection.

use bevy::prelude::*;
use std::collections::HashMap;

use crate::camera::{CameraController, MainCamera};
use crate::state_loader::{SimulationState, StateUpdatedEvent};
use crate::world::{FactionColors, LocationPositions};

/// Plugin for agent rendering and interaction.
pub struct AgentPlugin;

impl Plugin for AgentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AgentEntities>()
            .add_event::<AgentSelectedEvent>()
            .add_systems(
                Update,
                (
                    sync_agents_with_state,
                    interpolate_agent_movement,
                    handle_agent_click,
                    handle_agent_hover,
                    update_selection_visuals,
                    update_agent_label_visibility,
                ),
            );
    }
}

/// Event fired when an agent is selected.
#[derive(Event)]
pub struct AgentSelectedEvent {
    /// The selected agent's ID, or None if deselected.
    pub agent_id: Option<String>,
}

/// Marker component for selected agents.
#[derive(Component)]
pub struct Selected;

/// Marker component for hovered agents.
#[derive(Component)]
pub struct Hovered;

/// Component for agent name labels.
#[derive(Component)]
pub struct AgentLabel {
    /// The agent entity this label belongs to.
    pub agent_entity: Entity,
}

/// Component for selection highlight visuals.
#[derive(Component)]
pub struct SelectionHighlight {
    /// The agent entity this highlight belongs to.
    pub agent_entity: Entity,
}

/// Component for visual representation of an agent.
#[derive(Component)]
pub struct VisualAgent {
    /// Unique agent identifier.
    pub agent_id: String,
    /// Agent's faction.
    pub faction: String,
    /// Agent's role (leader, scout, etc.).
    pub role: String,
    /// Target position for movement interpolation.
    pub target_position: Vec2,
    /// Movement speed in world units per second.
    pub move_speed: f32,
}

/// Resource mapping agent IDs to their entities.
#[derive(Resource, Default)]
pub struct AgentEntities {
    /// Map of agent ID to entity.
    pub map: HashMap<String, Entity>,
}

impl AgentEntities {
    /// Get the entity for an agent.
    pub fn get(&self, agent_id: &str) -> Option<Entity> {
        self.map.get(agent_id).copied()
    }

    /// Insert or update an agent entity mapping.
    pub fn insert(&mut self, agent_id: impl Into<String>, entity: Entity) {
        self.map.insert(agent_id.into(), entity);
    }

    /// Remove an agent entity mapping.
    pub fn remove(&mut self, agent_id: &str) -> Option<Entity> {
        self.map.remove(agent_id)
    }
}

/// System to synchronize visual agents with simulation state.
fn sync_agents_with_state(
    mut commands: Commands,
    state: Res<SimulationState>,
    faction_colors: Res<FactionColors>,
    location_positions: Res<LocationPositions>,
    mut agent_entities: ResMut<AgentEntities>,
    mut events: EventReader<StateUpdatedEvent>,
    mut agents: Query<(&mut VisualAgent, &mut Sprite)>,
) {
    // Only process on state update events
    if events.read().next().is_none() {
        return;
    }

    let Some(ref snapshot) = state.snapshot else {
        return;
    };

    // Collect existing agent IDs
    let existing_ids: std::collections::HashSet<_> =
        agent_entities.map.keys().cloned().collect();

    // Collect snapshot agent IDs
    let snapshot_ids: std::collections::HashSet<_> = snapshot
        .agents
        .iter()
        .filter(|a| a.alive)
        .map(|a| a.agent_id.clone())
        .collect();

    // Remove agents that no longer exist
    for agent_id in existing_ids.difference(&snapshot_ids) {
        if let Some(entity) = agent_entities.remove(agent_id) {
            commands.entity(entity).despawn_recursive();
        }
    }

    // Update or spawn agents
    for (index, agent_snapshot) in snapshot.agents.iter().filter(|a| a.alive).enumerate() {
        let location_pos = location_positions.get(&agent_snapshot.location);
        // Add offset based on agent index to prevent stacking
        let offset = agent_offset(index);
        let target_pos = location_pos + offset;

        if let Some(&entity) = agent_entities.map.get(&agent_snapshot.agent_id) {
            // Update existing agent
            if let Ok((mut visual_agent, mut sprite)) = agents.get_mut(entity) {
                visual_agent.target_position = target_pos;
                visual_agent.faction = agent_snapshot.faction.clone();
                visual_agent.role = agent_snapshot.role.clone();

                // Update color if faction changed
                sprite.color = faction_colors.get(&agent_snapshot.faction);
            }
        } else {
            // Spawn new agent
            let entity = spawn_agent(
                &mut commands,
                agent_snapshot,
                &faction_colors,
                target_pos,
            );
            agent_entities.insert(agent_snapshot.agent_id.clone(), entity);
        }
    }
}

/// Calculate offset for agent based on index to prevent stacking.
fn agent_offset(index: usize) -> Vec2 {
    // Arrange agents in a small grid pattern around the location
    let row = index / 5;
    let col = index % 5;
    let spacing = 25.0;
    Vec2::new(
        (col as f32 - 2.0) * spacing,
        (row as f32 - 1.0) * spacing,
    )
}

/// Spawn a new visual agent entity.
fn spawn_agent(
    commands: &mut Commands,
    agent: &sim_events::AgentSnapshot,
    faction_colors: &FactionColors,
    position: Vec2,
) -> Entity {
    let color = faction_colors.get(&agent.faction);

    commands
        .spawn((
            SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(20.0, 30.0)),
                    ..default()
                },
                transform: Transform::from_xyz(position.x, position.y, 1.0),
                ..default()
            },
            VisualAgent {
                agent_id: agent.agent_id.clone(),
                faction: agent.faction.clone(),
                role: agent.role.clone(),
                target_position: position,
                move_speed: 100.0,
            },
        ))
        .id()
}

/// System to smoothly interpolate agent positions toward their targets.
fn interpolate_agent_movement(
    time: Res<Time>,
    mut agents: Query<(&mut Transform, &VisualAgent)>,
) {
    for (mut transform, agent) in agents.iter_mut() {
        let current = transform.translation.truncate();
        let target = agent.target_position;
        let distance = current.distance(target);

        if distance > 1.0 {
            let direction = (target - current).normalize();
            let movement = direction * agent.move_speed * time.delta_seconds();

            if movement.length() > distance {
                // Snap to target if we'd overshoot
                transform.translation = target.extend(transform.translation.z);
            } else {
                transform.translation += movement.extend(0.0);
            }
        }
    }
}

/// System to handle clicking on agents for selection.
fn handle_agent_click(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    agents: Query<(Entity, &Transform, &VisualAgent)>,
    selected: Query<Entity, With<Selected>>,
    mut events: EventWriter<AgentSelectedEvent>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.get_single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    // Find agent under cursor (within click radius)
    let click_radius = 20.0;
    let mut closest_agent: Option<(Entity, &VisualAgent, f32)> = None;

    for (entity, transform, agent) in agents.iter() {
        let agent_pos = transform.translation.truncate();
        let distance = world_pos.distance(agent_pos);

        if distance < click_radius {
            if closest_agent.is_none() || distance < closest_agent.unwrap().2 {
                closest_agent = Some((entity, agent, distance));
            }
        }
    }

    // Deselect current selection
    for entity in selected.iter() {
        commands.entity(entity).remove::<Selected>();
    }

    // Select new agent if found
    if let Some((entity, agent, _)) = closest_agent {
        commands.entity(entity).insert(Selected);
        events.send(AgentSelectedEvent {
            agent_id: Some(agent.agent_id.clone()),
        });
    } else {
        events.send(AgentSelectedEvent { agent_id: None });
    }
}

/// System to handle hovering over agents.
fn handle_agent_hover(
    mut commands: Commands,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    agents: Query<(Entity, &Transform), With<VisualAgent>>,
    hovered: Query<Entity, With<Hovered>>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        // Clear hover if no cursor
        for entity in hovered.iter() {
            commands.entity(entity).remove::<Hovered>();
        }
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    // Clear previous hover
    for entity in hovered.iter() {
        commands.entity(entity).remove::<Hovered>();
    }

    // Find agent under cursor
    let hover_radius = 20.0;
    let mut closest: Option<(Entity, f32)> = None;

    for (entity, transform) in agents.iter() {
        let agent_pos = transform.translation.truncate();
        let distance = world_pos.distance(agent_pos);

        if distance < hover_radius {
            if closest.is_none() || distance < closest.unwrap().1 {
                closest = Some((entity, distance));
            }
        }
    }

    if let Some((entity, _)) = closest {
        commands.entity(entity).insert(Hovered);
    }
}

/// System to update selection highlight visuals.
fn update_selection_visuals(
    mut commands: Commands,
    selected_agents: Query<(Entity, &Transform), (With<Selected>, With<VisualAgent>)>,
    mut highlights: Query<(Entity, &SelectionHighlight, &mut Transform), Without<VisualAgent>>,
) {
    // Track which agents have highlights
    let selected_entities: std::collections::HashSet<_> =
        selected_agents.iter().map(|(e, _)| e).collect();

    // Remove highlights for deselected agents
    for (highlight_entity, highlight, _) in highlights.iter() {
        if !selected_entities.contains(&highlight.agent_entity) {
            commands.entity(highlight_entity).despawn();
        }
    }

    // Track which agents already have highlights
    let highlighted_agents: std::collections::HashSet<_> =
        highlights.iter().map(|(_, h, _)| h.agent_entity).collect();

    // Add highlights for newly selected agents
    for (agent_entity, agent_transform) in selected_agents.iter() {
        if !highlighted_agents.contains(&agent_entity) {
            // Spawn selection highlight as a slightly larger outline
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgba(1.0, 1.0, 0.0, 0.5), // Yellow semi-transparent
                        custom_size: Some(Vec2::new(28.0, 38.0)),
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        agent_transform.translation.x,
                        agent_transform.translation.y,
                        0.9, // Just behind the agent
                    ),
                    ..default()
                },
                SelectionHighlight {
                    agent_entity,
                },
            ));
        }
    }

    // Update highlight positions to follow agents
    for (_, highlight, mut transform) in highlights.iter_mut() {
        if let Ok((_, agent_transform)) = selected_agents.get(highlight.agent_entity) {
            transform.translation.x = agent_transform.translation.x;
            transform.translation.y = agent_transform.translation.y;
        }
    }
}

/// System to update agent label visibility based on zoom and selection.
fn update_agent_label_visibility(
    camera: Res<CameraController>,
    mut labels: Query<(&mut Visibility, &AgentLabel)>,
    selected: Query<Entity, With<Selected>>,
    hovered: Query<Entity, With<Hovered>>,
) {
    let show_all_labels = camera.zoom > 1.5;
    let selected_entities: std::collections::HashSet<_> = selected.iter().collect();
    let hovered_entities: std::collections::HashSet<_> = hovered.iter().collect();

    for (mut visibility, label) in labels.iter_mut() {
        let is_selected = selected_entities.contains(&label.agent_entity);
        let is_hovered = hovered_entities.contains(&label.agent_entity);

        *visibility = if show_all_labels || is_selected || is_hovered {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_entities_default() {
        let entities = AgentEntities::default();
        assert!(entities.map.is_empty());
    }

    #[test]
    fn test_agent_offset() {
        // First agent should be at top-left
        let offset0 = agent_offset(0);
        assert_eq!(offset0.x, -50.0);
        assert_eq!(offset0.y, -25.0);

        // Center agent (index 7) should be near center
        let offset7 = agent_offset(7);
        assert_eq!(offset7.x, 0.0); // col 2 -> (2-2)*25 = 0
        assert_eq!(offset7.y, 0.0); // row 1 -> (1-1)*25 = 0
    }
}
