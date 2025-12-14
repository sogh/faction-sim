//! Shared event types and serialization for the emergent simulation.
//!
//! This crate contains pure data structures with no simulation logic.
//! It is a dependency for all other crates in the workspace.

pub mod event;
pub mod timestamp;
pub mod tension;
pub mod snapshot;

// Re-export commonly used types
pub use event::*;
pub use timestamp::*;
pub use tension::{Tension, TensionType, TensionStatus, TensionAgent, PredictedOutcome, CameraFocus};
pub use snapshot::*;
