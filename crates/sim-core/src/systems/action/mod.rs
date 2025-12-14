//! Action Systems
//!
//! Systems for action generation, weighting, selection, and execution.

pub mod generate;
pub mod weight;
pub mod select;
pub mod execute;

pub use generate::{
    Action, PendingActions, WeightedAction,
    generate_movement_actions, generate_patrol_actions, generate_communication_actions,
    generate_archive_actions, generate_resource_actions, generate_social_actions,
    generate_faction_actions, generate_conflict_actions, generate_beer_actions,
};
pub use weight::apply_trait_weights;
pub use select::{SelectedActions, select_actions, add_noise_to_weights};
pub use execute::{
    TickEvents, execute_movement_actions, execute_communication_actions,
    execute_archive_actions, execute_resource_actions, execute_social_actions,
    execute_faction_actions, execute_conflict_actions, execute_beer_actions,
};
