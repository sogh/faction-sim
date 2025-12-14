//! Event System
//!
//! Event types, logging, and the append-only event stream.

pub mod types;
pub mod logger;
pub mod drama;

pub use types::*;
pub use logger::*;
pub use drama::{DramaAnalysis, calculate_drama_score, is_highly_dramatic, filter_dramatic_events};
