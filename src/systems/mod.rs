//! ECS Systems
//!
//! All simulation systems for perception, needs, actions, memory, and tension.

pub mod action;
pub mod perception;
pub mod needs;

// Re-export commonly used systems
pub use perception::{build_location_index, update_perception, AgentsByLocation, VisibleAgents};
pub use needs::{
    decay_interaction_counts, update_food_security, update_social_belonging,
    InteractionTracker, RitualAttendance,
};
pub use action::{
    Action, PendingActions, SelectedActions, TickEvents, WeightedAction,
    generate_movement_actions, generate_patrol_actions,
    apply_trait_weights, add_noise_to_weights, select_actions,
    execute_movement_actions,
};

// Placeholder modules - will be implemented in later phases
// pub mod memory;
// pub mod tension;
// pub mod snapshot;
