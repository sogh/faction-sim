//! World rendering: map background, locations, and faction territories.

use bevy::prelude::*;
use std::collections::HashMap;

use crate::camera::CameraController;
use crate::state_loader::{SimulationState, StateUpdatedEvent};

/// Plugin for world/map rendering.
pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VisualWorld>()
            .init_resource::<FactionColors>()
            .init_resource::<LocationPositions>()
            .add_systems(Startup, spawn_map_background)
            .add_systems(
                Update,
                (update_locations, update_location_labels).run_if(on_event::<StateUpdatedEvent>()),
            );
    }
}

/// Core resource tracking the visual representation of the world.
#[derive(Resource)]
pub struct VisualWorld {
    /// Current simulation tick being displayed.
    pub current_tick: u64,
    /// Current season (for visual effects).
    pub season: String,
    /// World boundaries for camera constraints.
    pub bounds: WorldBounds,
}

impl Default for VisualWorld {
    fn default() -> Self {
        Self {
            current_tick: 0,
            season: "spring".into(),
            bounds: WorldBounds::default(),
        }
    }
}

/// World coordinate boundaries.
#[derive(Clone, Debug)]
pub struct WorldBounds {
    /// Minimum corner of the world.
    pub min: Vec2,
    /// Maximum corner of the world.
    pub max: Vec2,
}

impl Default for WorldBounds {
    fn default() -> Self {
        Self {
            min: Vec2::new(-1000.0, -1000.0),
            max: Vec2::new(1000.0, 1000.0),
        }
    }
}

impl WorldBounds {
    /// Create bounds from center and half-size.
    pub fn from_center(center: Vec2, half_size: Vec2) -> Self {
        Self {
            min: center - half_size,
            max: center + half_size,
        }
    }

    /// Get the center of the bounds.
    pub fn center(&self) -> Vec2 {
        (self.min + self.max) / 2.0
    }

    /// Get the size of the bounds.
    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }

    /// Check if a point is within bounds.
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    /// Clamp a point to be within bounds.
    pub fn clamp(&self, point: Vec2) -> Vec2 {
        Vec2::new(
            point.x.clamp(self.min.x, self.max.x),
            point.y.clamp(self.min.y, self.max.y),
        )
    }
}

/// Visual representation of a location on the map.
#[derive(Component)]
pub struct VisualLocation {
    /// Unique identifier for this location.
    pub location_id: String,
    /// Type of location (affects rendering).
    pub location_type: LocationType,
    /// Faction currently controlling this location, if any.
    pub controlling_faction: Option<String>,
}

/// Types of locations that can appear on the map.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LocationType {
    /// A village or town.
    Village,
    /// A temporary camp.
    Camp,
    /// A notable landmark.
    Landmark,
    /// A bridge crossing.
    Bridge,
    /// A road intersection.
    Crossroads,
}

impl LocationType {
    /// Get the base size for rendering this location type.
    pub fn base_size(&self) -> Vec2 {
        match self {
            LocationType::Village => Vec2::new(40.0, 40.0),
            LocationType::Camp => Vec2::new(25.0, 25.0),
            LocationType::Landmark => Vec2::new(30.0, 30.0),
            LocationType::Bridge => Vec2::new(50.0, 20.0),
            LocationType::Crossroads => Vec2::new(30.0, 30.0),
        }
    }

    /// Get the base color for this location type.
    pub fn base_color(&self) -> Color {
        match self {
            LocationType::Village => Color::srgb(0.6, 0.5, 0.4),
            LocationType::Camp => Color::srgb(0.5, 0.4, 0.3),
            LocationType::Landmark => Color::srgb(0.7, 0.7, 0.6),
            LocationType::Bridge => Color::srgb(0.5, 0.4, 0.35),
            LocationType::Crossroads => Color::srgb(0.55, 0.5, 0.45),
        }
    }
}

/// Component marking a location marker sprite.
#[derive(Component)]
pub struct LocationMarker {
    /// The location ID this marker represents.
    pub location_id: String,
}

/// Resource mapping faction IDs to their display colors.
#[derive(Resource)]
pub struct FactionColors {
    /// Map of faction ID to color.
    pub colors: HashMap<String, Color>,
}

impl Default for FactionColors {
    fn default() -> Self {
        let mut colors = HashMap::new();
        // Default faction colors - these match expected faction names
        colors.insert("thornwood".into(), Color::srgb(0.2, 0.6, 0.2)); // Forest green
        colors.insert("ironmere".into(), Color::srgb(0.4, 0.4, 0.6)); // Steel blue
        colors.insert("saltcliff".into(), Color::srgb(0.8, 0.7, 0.5)); // Sandy beige
        colors.insert("northern_hold".into(), Color::srgb(0.5, 0.5, 0.7)); // Icy blue
        colors.insert("ravens_rest".into(), Color::srgb(0.3, 0.3, 0.35)); // Dark gray
        colors.insert("goldvale".into(), Color::srgb(0.8, 0.7, 0.2)); // Gold
        Self { colors }
    }
}

impl FactionColors {
    /// Get the color for a faction, with a fallback for unknown factions.
    pub fn get(&self, faction_id: &str) -> Color {
        self.colors
            .get(faction_id)
            .copied()
            .unwrap_or(Color::srgb(0.5, 0.5, 0.5)) // Gray fallback
    }

    /// Add or update a faction color.
    pub fn set(&mut self, faction_id: impl Into<String>, color: Color) {
        self.colors.insert(faction_id.into(), color);
    }
}

/// Resource mapping location IDs to world positions.
#[derive(Resource)]
pub struct LocationPositions {
    /// Map of location ID to world position.
    pub positions: HashMap<String, Vec2>,
}

impl Default for LocationPositions {
    fn default() -> Self {
        let mut positions = HashMap::new();

        // Thornwood faction (northwest quadrant)
        positions.insert("thornwood_hall".into(), Vec2::new(-300.0, 250.0));
        positions.insert("thornwood_village".into(), Vec2::new(-350.0, 150.0));
        positions.insert("thornwood_fields".into(), Vec2::new(-250.0, 150.0));
        positions.insert("western_forest".into(), Vec2::new(-400.0, 50.0));

        // Ironmere faction (northeast quadrant)
        positions.insert("ironmere_hall".into(), Vec2::new(300.0, 250.0));
        positions.insert("ironmere_village".into(), Vec2::new(350.0, 150.0));
        positions.insert("ironmere_fields".into(), Vec2::new(250.0, 150.0));
        positions.insert("iron_mine".into(), Vec2::new(400.0, 300.0));

        // Saltcliff faction (southeast quadrant)
        positions.insert("saltcliff_hall".into(), Vec2::new(300.0, -250.0));
        positions.insert("saltcliff_village".into(), Vec2::new(350.0, -150.0));
        positions.insert("saltcliff_fields".into(), Vec2::new(250.0, -150.0));
        positions.insert("salt_harbor".into(), Vec2::new(400.0, -300.0));

        // Northern Hold faction (southwest quadrant)
        positions.insert("northern_hold".into(), Vec2::new(-300.0, -250.0));
        positions.insert("hold_village".into(), Vec2::new(-350.0, -150.0));
        positions.insert("hold_fields".into(), Vec2::new(-250.0, -150.0));

        // Central/shared locations
        positions.insert("central_crossroads".into(), Vec2::ZERO);
        positions.insert("northern_crossroads".into(), Vec2::new(0.0, 200.0));
        positions.insert("old_market".into(), Vec2::new(0.0, -100.0));
        positions.insert("eastern_bridge".into(), Vec2::new(150.0, 0.0));
        positions.insert("southern_forest".into(), Vec2::new(-100.0, -300.0));
        positions.insert("mountain_pass".into(), Vec2::new(0.0, 350.0));

        Self { positions }
    }
}

impl LocationPositions {
    /// Get position for a location, returning world origin if not found.
    pub fn get(&self, location_id: &str) -> Vec2 {
        self.positions.get(location_id).copied().unwrap_or(Vec2::ZERO)
    }
}

/// Component for location label text.
#[derive(Component)]
pub struct LocationLabel {
    /// The location this label belongs to.
    pub location_id: String,
}

/// Marker component for the map background.
#[derive(Component)]
pub struct MapBackground;

/// System to spawn the map background.
fn spawn_map_background(mut commands: Commands) {
    // Simple colored rectangle for the map background
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.3, 0.4, 0.3), // Grass-ish green
                custom_size: Some(Vec2::new(2000.0, 2000.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, -10.0),
            ..default()
        },
        MapBackground,
    ));

    tracing::info!("Spawned map background");
}

/// System to update location markers based on simulation state.
fn update_locations(
    mut commands: Commands,
    state: Res<SimulationState>,
    faction_colors: Res<FactionColors>,
    location_positions: Res<LocationPositions>,
    existing_locations: Query<(Entity, &VisualLocation)>,
) {
    let Some(ref snapshot) = state.snapshot else {
        return;
    };

    // Track which locations exist in the snapshot
    let snapshot_location_ids: std::collections::HashSet<_> = snapshot
        .locations
        .iter()
        .map(|l| l.location_id.as_str())
        .collect();

    // Remove locations that no longer exist
    for (entity, visual_loc) in existing_locations.iter() {
        if !snapshot_location_ids.contains(visual_loc.location_id.as_str()) {
            commands.entity(entity).despawn_recursive();
        }
    }

    // Existing location IDs
    let existing_ids: std::collections::HashSet<_> = existing_locations
        .iter()
        .map(|(_, loc)| loc.location_id.as_str())
        .collect();

    // Spawn new locations
    for location in &snapshot.locations {
        if existing_ids.contains(location.location_id.as_str()) {
            continue;
        }

        let position = location_positions.get(&location.location_id);
        let location_type = parse_location_type(&location.location_id);

        // Determine color based on controlling faction
        let color = location
            .controlling_faction
            .as_ref()
            .map(|f| faction_colors.get(f))
            .unwrap_or(location_type.base_color());

        let size = location_type.base_size();

        // Spawn the location marker
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(size),
                    ..default()
                },
                transform: Transform::from_xyz(position.x, position.y, 0.0),
                ..default()
            },
            VisualLocation {
                location_id: location.location_id.clone(),
                location_type: location_type.clone(),
                controlling_faction: location.controlling_faction.clone(),
            },
            LocationMarker {
                location_id: location.location_id.clone(),
            },
        ));
    }
}

/// Parse location type from location ID.
fn parse_location_type(location_id: &str) -> LocationType {
    if location_id.contains("village") {
        LocationType::Village
    } else if location_id.contains("camp") {
        LocationType::Camp
    } else if location_id.contains("bridge") {
        LocationType::Bridge
    } else if location_id.contains("crossroads") {
        LocationType::Crossroads
    } else if location_id.contains("landmark") {
        LocationType::Landmark
    } else {
        LocationType::Village // Default
    }
}

/// System to update location label visibility based on zoom.
fn update_location_labels(
    camera: Res<CameraController>,
    mut labels: Query<&mut Visibility, With<LocationLabel>>,
) {
    let show_labels = camera.zoom > 0.8;

    for mut visibility in labels.iter_mut() {
        *visibility = if show_labels {
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
    fn test_world_bounds_contains() {
        let bounds = WorldBounds {
            min: Vec2::new(-100.0, -100.0),
            max: Vec2::new(100.0, 100.0),
        };

        assert!(bounds.contains(Vec2::ZERO));
        assert!(bounds.contains(Vec2::new(50.0, 50.0)));
        assert!(!bounds.contains(Vec2::new(150.0, 0.0)));
    }

    #[test]
    fn test_world_bounds_clamp() {
        let bounds = WorldBounds {
            min: Vec2::new(-100.0, -100.0),
            max: Vec2::new(100.0, 100.0),
        };

        assert_eq!(bounds.clamp(Vec2::ZERO), Vec2::ZERO);
        assert_eq!(bounds.clamp(Vec2::new(200.0, 0.0)), Vec2::new(100.0, 0.0));
        assert_eq!(
            bounds.clamp(Vec2::new(-200.0, -200.0)),
            Vec2::new(-100.0, -100.0)
        );
    }

    #[test]
    fn test_faction_colors_default() {
        let colors = FactionColors::default();
        assert!(colors.colors.contains_key("thornwood"));
        assert!(colors.colors.contains_key("ironmere"));
    }

    #[test]
    fn test_faction_colors_get_fallback() {
        let colors = FactionColors::default();
        // Unknown faction should return gray
        let unknown_color = colors.get("unknown_faction");
        assert_eq!(unknown_color, Color::srgb(0.5, 0.5, 0.5));
    }
}
