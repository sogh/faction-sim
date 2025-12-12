//! Emergent Medieval Simulation Engine Library
//!
//! Public API for the simulation engine.

use bevy_ecs::prelude::*;
use rand::rngs::SmallRng;

pub mod components;
pub mod systems;
pub mod events;
pub mod actions;
pub mod output;
pub mod setup;

pub use components::*;

// Re-export setup functions explicitly to avoid module name conflicts
pub use setup::{create_world_map, create_factions, create_ritual_schedule};
pub use setup::{world_to_json, factions_to_json};

/// Seeded random number generator resource
#[derive(Resource)]
pub struct SimRng(pub SmallRng);
