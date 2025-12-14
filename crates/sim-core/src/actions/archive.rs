//! Archive Actions
//!
//! Actions for writing, reading, and manipulating faction archive entries.

use serde::{Deserialize, Serialize};

/// Type of archive action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchiveActionType {
    /// Write a memory to the archive
    WriteEntry,
    /// Read entries from the archive
    ReadArchive,
    /// Destroy an embarrassing entry
    DestroyEntry,
    /// Create a forged (false) entry
    ForgeEntry,
}

/// An archive action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveAction {
    /// Who is performing the action
    pub actor_id: String,
    /// Type of archive action
    pub action_type: ArchiveActionType,
    /// Faction whose archive is being accessed
    pub faction_id: String,
    /// Memory ID being written (for WriteEntry)
    pub memory_id: Option<String>,
    /// Entry ID being targeted (for Destroy/Read specific)
    pub entry_id: Option<String>,
    /// Subject for forged entries
    pub subject: Option<String>,
    /// Content for forged entries
    pub content: Option<String>,
}

impl ArchiveAction {
    /// Create a write entry action
    pub fn write_entry(
        actor_id: impl Into<String>,
        faction_id: impl Into<String>,
        memory_id: impl Into<String>,
    ) -> Self {
        Self {
            actor_id: actor_id.into(),
            action_type: ArchiveActionType::WriteEntry,
            faction_id: faction_id.into(),
            memory_id: Some(memory_id.into()),
            entry_id: None,
            subject: None,
            content: None,
        }
    }

    /// Create a read archive action
    pub fn read_archive(
        actor_id: impl Into<String>,
        faction_id: impl Into<String>,
    ) -> Self {
        Self {
            actor_id: actor_id.into(),
            action_type: ArchiveActionType::ReadArchive,
            faction_id: faction_id.into(),
            memory_id: None,
            entry_id: None,
            subject: None,
            content: None,
        }
    }

    /// Create a destroy entry action
    pub fn destroy_entry(
        actor_id: impl Into<String>,
        faction_id: impl Into<String>,
        entry_id: impl Into<String>,
    ) -> Self {
        Self {
            actor_id: actor_id.into(),
            action_type: ArchiveActionType::DestroyEntry,
            faction_id: faction_id.into(),
            memory_id: None,
            entry_id: Some(entry_id.into()),
            subject: None,
            content: None,
        }
    }

    /// Create a forge entry action
    pub fn forge_entry(
        actor_id: impl Into<String>,
        faction_id: impl Into<String>,
        subject: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            actor_id: actor_id.into(),
            action_type: ArchiveActionType::ForgeEntry,
            faction_id: faction_id.into(),
            memory_id: None,
            entry_id: None,
            subject: Some(subject.into()),
            content: Some(content.into()),
        }
    }
}

/// Weight modifiers for archive actions
pub mod archive_weights {
    /// Base weight for writing an entry
    pub const WRITE_BASE: f32 = 0.3;
    /// Bonus for writing about significant events
    pub const SIGNIFICANT_EVENT_BONUS: f32 = 0.2;
    /// Bonus for writing entries that reflect well on self
    pub const SELF_FAVORABLE_BONUS: f32 = 0.15;

    /// Base weight for reading the archive
    pub const READ_BASE: f32 = 0.2;

    /// Base weight for destroying entries (very low)
    pub const DESTROY_BASE: f32 = 0.05;
    /// Bonus for destroying embarrassing entries about self
    pub const DESTROY_EMBARRASSING_BONUS: f32 = 0.2;
    /// Honesty reduces destroy weight
    pub const HONESTY_DESTROY_MODIFIER: f32 = -0.15;

    /// Base weight for forging (very low)
    pub const FORGE_BASE: f32 = 0.02;
    /// Requires low honesty
    pub const HONESTY_FORGE_THRESHOLD: f32 = 0.3;
    /// Ambition increases forge weight
    pub const AMBITION_FORGE_BONUS: f32 = 0.1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_entry_action() {
        let action = ArchiveAction::write_entry("agent_1", "thornwood", "mem_001");
        assert_eq!(action.action_type, ArchiveActionType::WriteEntry);
        assert_eq!(action.memory_id, Some("mem_001".to_string()));
    }

    #[test]
    fn test_forge_entry_action() {
        let action = ArchiveAction::forge_entry(
            "agent_1",
            "thornwood",
            "agent_2",
            "A lie about agent_2",
        );
        assert_eq!(action.action_type, ArchiveActionType::ForgeEntry);
        assert!(action.subject.is_some());
        assert!(action.content.is_some());
    }
}
