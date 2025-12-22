//! Action Systems
//!
//! Systems for action generation, weighting, selection, and execution.
//!
//! The desire-based action system generates action desires for all known locations,
//! weighted by expected utility. Movement emerges from agents wanting to perform
//! actions at specific locations, rather than explicit "return home" behavior.

pub mod generate;
pub mod weight;
pub mod select;
pub mod execute;
pub mod utility;

pub use generate::{
    Action, PendingActions, WeightedAction,
    generate_movement_actions, generate_desire_based_actions,
    generate_patrol_actions, generate_communication_actions,
    generate_archive_actions, generate_resource_actions, generate_social_actions,
    generate_faction_actions, generate_conflict_actions, generate_beer_actions,
};
pub use utility::{ActionUtility, calculate_distance_penalty, calculate_idle_weight, calculate_need_utility};
pub use weight::apply_trait_weights;
pub use select::{SelectedActions, select_actions, add_noise_to_weights};
pub use execute::{
    TickEvents, execute_movement_actions, execute_communication_actions,
    execute_archive_actions, execute_resource_actions, execute_social_actions,
    execute_faction_actions, execute_conflict_actions, execute_beer_actions,
};
