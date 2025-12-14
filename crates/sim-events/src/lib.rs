//! Shared event types and serialization for the emergent simulation.
//!
//! This crate contains pure data structures with no simulation logic.
//! It is a dependency for all other crates in the workspace.

pub mod event;
pub mod snapshot;
pub mod tension;
pub mod timestamp;

// Re-export timestamp types
pub use timestamp::{
    ParseDateError, Season, SimDate, SimTimestamp, DAYS_PER_SEASON, TICKS_PER_DAY,
};

// Re-export event types
pub use event::*;

// Re-export tension types
pub use tension::{
    generate_tension_id, CameraFocus, CameraRecommendation, PredictedOutcome, Tension,
    TensionAgent, TensionStatus, TensionType,
};

// Re-export snapshot types
pub use snapshot::{
    generate_snapshot_id, AgentSnapshot, ComputedMetrics, FactionResourcesSnapshot,
    FactionSnapshot, GlobalResources, GoalSnapshot, LocationResourcesSnapshot, LocationSnapshot,
    NeedsSnapshot, RelationshipSnapshot, SocialBridge, SocialHub, SocialIsolate,
    SocialNetworkSnapshot, StatusSnapshot, TraitsSnapshot, WorldSnapshot, WorldStateSnapshot,
};
