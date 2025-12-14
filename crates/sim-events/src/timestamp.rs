//! Timestamp Types
//!
//! Simulation timestamp handling for events and snapshots.

use serde::{Deserialize, Serialize};

/// Timestamp for events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTimestamp {
    pub tick: u64,
    pub date: String,
}

impl EventTimestamp {
    pub fn new(tick: u64, date: impl Into<String>) -> Self {
        Self {
            tick,
            date: date.into(),
        }
    }
}

/// Timestamp for snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotTimestamp {
    pub tick: u64,
    pub date: String,
}

impl SnapshotTimestamp {
    pub fn new(tick: u64, date: impl Into<String>) -> Self {
        Self {
            tick,
            date: date.into(),
        }
    }
}
