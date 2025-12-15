//! Main visualization plugin that ties all systems together.

use bevy::prelude::*;

use crate::agents::AgentPlugin;
use crate::camera::CameraPlugin;
use crate::debug::DebugPlugin;
use crate::director_runner::DirectorRunnerPlugin;
use crate::director_state::DirectorPlugin;
use crate::intervention::InterventionPlugin;
use crate::overlay::OverlayPlugin;
use crate::sim_runner::SimRunnerPlugin;
use crate::state_loader::StateLoaderPlugin;
use crate::world::WorldPlugin;

/// Main plugin for the simulation visualization.
///
/// This plugin sets up the window, adds all sub-plugins, and configures
/// the Bevy app for visualization.
pub struct SimVizPlugin;

impl Plugin for SimVizPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Emergent Simulation".into(),
                        resolution: (1280., 720.).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()), // Pixel-perfect rendering
        )
        .add_plugins((
            SimRunnerPlugin,
            CameraPlugin,
            StateLoaderPlugin,
            WorldPlugin,
            AgentPlugin,
            DirectorPlugin,
            DirectorRunnerPlugin,
            InterventionPlugin,
            OverlayPlugin,
            DebugPlugin,
        ));
    }
}
