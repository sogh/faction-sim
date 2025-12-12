//! ECS Systems
//!
//! All simulation systems for perception, needs, actions, memory, trust, ritual, and tension.

pub mod action;
pub mod perception;
pub mod needs;
pub mod memory;
pub mod trust;
pub mod ritual;

// Re-export commonly used systems
pub use perception::{build_location_index, update_perception, AgentsByLocation, VisibleAgents};
pub use needs::{
    decay_interaction_counts, update_food_security, update_social_belonging,
    InteractionTracker, RitualAttendance,
};
pub use action::{
    Action, PendingActions, SelectedActions, TickEvents, WeightedAction,
    generate_movement_actions, generate_patrol_actions, generate_communication_actions, generate_archive_actions,
    apply_trait_weights, add_noise_to_weights, select_actions,
    execute_movement_actions, execute_communication_actions, execute_archive_actions,
};
pub use memory::{
    decay_memories, cleanup_memories, SeasonTracker,
    calculate_secondhand_trust_impact, get_most_interesting_memory,
};
pub use trust::{
    process_trust_events, decay_grudges, TrustEventQueue, TrustEvent, TrustEventType,
    create_trust_event,
};
pub use ritual::execute_rituals;

// Placeholder modules - will be implemented in later phases
// pub mod tension;
// pub mod snapshot;
