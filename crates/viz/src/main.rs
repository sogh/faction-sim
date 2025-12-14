//! Emergent Simulation Visualization
//!
//! Run with: cargo run -p viz

use bevy::prelude::*;
use viz::SimVizPlugin;

fn main() {
    App::new()
        .add_plugins(SimVizPlugin)
        .run();
}
