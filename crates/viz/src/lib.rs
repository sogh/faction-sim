//! Visualization layer: Bevy-based renderer for the emergent simulation.
//!
//! This crate provides the visual interface for observing the simulation.
//! It renders agents, locations, and faction territories, responds to
//! Director AI camera instructions, and provides user interaction.

pub mod agents;
pub mod camera;
pub mod debug;
pub mod director_runner;
pub mod director_state;
pub mod intervention;
pub mod overlay;
pub mod sim_runner;
pub mod state_loader;
pub mod world;

mod plugin;

pub use plugin::SimVizPlugin;
