//! Faction Components
//!
//! Components for faction membership, archives, and status.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::agent::Role;

/// Component: An agent's membership in a faction
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct FactionMembership {
    /// Which faction the agent belongs to
    pub faction_id: String,
    /// Role within the faction
    pub role: Role,
    /// Numeric status level (derived from role but can be modified)
    pub status_level: u8,
}

impl FactionMembership {
    pub fn new(faction_id: impl Into<String>, role: Role) -> Self {
        let status_level = role.status_level() as u8;
        Self {
            faction_id: faction_id.into(),
            role,
            status_level,
        }
    }

    pub fn is_leader(&self) -> bool {
        matches!(self.role, Role::Leader)
    }

    pub fn is_reader(&self) -> bool {
        matches!(self.role, Role::Reader)
    }

    pub fn can_write_archive(&self) -> bool {
        // Leaders, readers, and council members can write to the archive
        matches!(
            self.role,
            Role::Leader | Role::Reader | Role::CouncilMember
        )
    }
}

/// Unique identifier for a faction
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FactionId(pub String);

impl From<&str> for FactionId {
    fn from(s: &str) -> Self {
        FactionId(s.to_string())
    }
}

impl From<String> for FactionId {
    fn from(s: String) -> Self {
        FactionId(s)
    }
}

/// Resources that a faction controls
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FactionResources {
    pub grain: u32,
    pub iron: u32,
    pub salt: u32,
    pub beer: u32,
}

impl FactionResources {
    pub fn new(grain: u32, iron: u32, salt: u32) -> Self {
        Self { grain, iron, salt, beer: 0 }
    }

    /// Create resources with beer included
    pub fn with_beer(grain: u32, iron: u32, salt: u32, beer: u32) -> Self {
        Self { grain, iron, salt, beer }
    }

    pub fn total(&self) -> u32 {
        self.grain + self.iron + self.salt + self.beer
    }

    /// Check if food resources are critically low
    /// Beer counts as 0.5 grain equivalent for this check
    pub fn is_critical(&self) -> bool {
        let effective_food = self.grain as f32 + (self.beer as f32 * 0.5);
        effective_food < 100.0
    }

    /// Get effective food value (grain + beer at 50% value)
    pub fn effective_food(&self) -> f32 {
        self.grain as f32 + (self.beer as f32 * 0.5)
    }
}

/// A single faction in the simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Faction {
    /// Unique identifier
    pub id: FactionId,
    /// Display name
    pub name: String,
    /// List of location IDs in this faction's territory
    pub territory: Vec<String>,
    /// Location ID of the faction's headquarters
    pub hq_location: String,
    /// Current resources
    pub resources: FactionResources,
    /// Agent ID of the current leader
    pub leader: Option<String>,
    /// Agent ID of the current reader (archive keeper)
    pub reader: Option<String>,
    /// Number of members
    pub member_count: u32,
}

impl Faction {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        hq_location: impl Into<String>,
    ) -> Self {
        let id_str = id.into();
        Self {
            id: FactionId(id_str),
            name: name.into(),
            territory: Vec::new(),
            hq_location: hq_location.into(),
            resources: FactionResources::default(),
            leader: None,
            reader: None,
            member_count: 0,
        }
    }

    pub fn with_territory(mut self, locations: Vec<String>) -> Self {
        self.territory = locations;
        self
    }

    pub fn with_resources(mut self, resources: FactionResources) -> Self {
        self.resources = resources;
        self
    }

    pub fn controls_location(&self, location_id: &str) -> bool {
        self.territory.contains(&location_id.to_string())
    }
}

/// An entry in a faction's archive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveEntry {
    /// Unique identifier for this entry
    pub entry_id: String,
    /// Agent ID of the author
    pub author_id: String,
    /// Author's name at time of writing
    pub author_name: String,
    /// Subject of the entry (agent ID, event type, or topic)
    pub subject: String,
    /// The content of the entry
    pub content: String,
    /// Tick when this entry was written
    pub tick_written: u64,
    /// Number of times this entry has been read at rituals
    pub times_read: u32,
    /// Is this entry true or forged?
    pub is_authentic: bool,
}

impl ArchiveEntry {
    pub fn new(
        entry_id: impl Into<String>,
        author_id: impl Into<String>,
        author_name: impl Into<String>,
        subject: impl Into<String>,
        content: impl Into<String>,
        tick: u64,
    ) -> Self {
        Self {
            entry_id: entry_id.into(),
            author_id: author_id.into(),
            author_name: author_name.into(),
            subject: subject.into(),
            content: content.into(),
            tick_written: tick,
            times_read: 0,
            is_authentic: true,
        }
    }

    pub fn forged(
        entry_id: impl Into<String>,
        author_id: impl Into<String>,
        author_name: impl Into<String>,
        subject: impl Into<String>,
        content: impl Into<String>,
        tick: u64,
    ) -> Self {
        Self {
            entry_id: entry_id.into(),
            author_id: author_id.into(),
            author_name: author_name.into(),
            subject: subject.into(),
            content: content.into(),
            tick_written: tick,
            times_read: 0,
            is_authentic: false,
        }
    }

    pub fn increment_reads(&mut self) {
        self.times_read += 1;
    }
}

/// A faction's archive - the institutional memory
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Archive {
    /// All entries in the archive
    entries: Vec<ArchiveEntry>,
    /// Counter for generating entry IDs
    next_entry_id: u64,
}

impl Archive {
    pub fn new() -> Self {
        Self::default()
    }

    /// Generate a unique entry ID for this archive
    pub fn generate_id(&mut self, faction_id: &str) -> String {
        let id = format!("archive_{}_{:04}", faction_id, self.next_entry_id);
        self.next_entry_id += 1;
        id
    }

    /// Add an entry to the archive
    pub fn add_entry(&mut self, entry: ArchiveEntry) {
        self.entries.push(entry);
    }

    /// Get all entries
    pub fn entries(&self) -> &[ArchiveEntry] {
        &self.entries
    }

    /// Get entry count
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Get entries about a specific subject
    pub fn entries_about(&self, subject: &str) -> Vec<&ArchiveEntry> {
        self.entries
            .iter()
            .filter(|e| e.subject == subject)
            .collect()
    }

    /// Get entries by author
    pub fn entries_by_author(&self, author_id: &str) -> Vec<&ArchiveEntry> {
        self.entries
            .iter()
            .filter(|e| e.author_id == author_id)
            .collect()
    }

    /// Find entry by ID
    pub fn find_entry(&self, entry_id: &str) -> Option<&ArchiveEntry> {
        self.entries.iter().find(|e| e.entry_id == entry_id)
    }

    /// Find mutable entry by ID
    pub fn find_entry_mut(&mut self, entry_id: &str) -> Option<&mut ArchiveEntry> {
        self.entries.iter_mut().find(|e| e.entry_id == entry_id)
    }

    /// Remove an entry by ID
    pub fn remove_entry(&mut self, entry_id: &str) -> bool {
        let initial_len = self.entries.len();
        self.entries.retain(|e| e.entry_id != entry_id);
        self.entries.len() < initial_len
    }

    /// Get entries sorted by times read (descending)
    pub fn most_read_entries(&self, limit: usize) -> Vec<&ArchiveEntry> {
        let mut sorted: Vec<_> = self.entries.iter().collect();
        sorted.sort_by(|a, b| b.times_read.cmp(&a.times_read));
        sorted.into_iter().take(limit).collect()
    }

    /// Get entries that haven't been read recently
    pub fn least_read_entries(&self, limit: usize) -> Vec<&ArchiveEntry> {
        let mut sorted: Vec<_> = self.entries.iter().collect();
        sorted.sort_by(|a, b| a.times_read.cmp(&b.times_read));
        sorted.into_iter().take(limit).collect()
    }
}

/// Resource: Registry of all factions
#[derive(Resource, Debug, Default)]
pub struct FactionRegistry {
    factions: HashMap<String, Faction>,
    archives: HashMap<String, Archive>,
}

impl FactionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new faction
    pub fn register(&mut self, faction: Faction) {
        let id = faction.id.0.clone();
        self.archives.insert(id.clone(), Archive::new());
        self.factions.insert(id, faction);
    }

    /// Get a faction by ID
    pub fn get(&self, faction_id: &str) -> Option<&Faction> {
        self.factions.get(faction_id)
    }

    /// Get mutable faction by ID
    pub fn get_mut(&mut self, faction_id: &str) -> Option<&mut Faction> {
        self.factions.get_mut(faction_id)
    }

    /// Get a faction's archive
    pub fn get_archive(&self, faction_id: &str) -> Option<&Archive> {
        self.archives.get(faction_id)
    }

    /// Get mutable archive
    pub fn get_archive_mut(&mut self, faction_id: &str) -> Option<&mut Archive> {
        self.archives.get_mut(faction_id)
    }

    /// Get all factions
    pub fn all_factions(&self) -> impl Iterator<Item = &Faction> {
        self.factions.values()
    }

    /// Get all factions mutably
    pub fn all_factions_mut(&mut self) -> impl Iterator<Item = &mut Faction> {
        self.factions.values_mut()
    }

    /// Get all faction IDs
    pub fn faction_ids(&self) -> Vec<&String> {
        self.factions.keys().collect()
    }

    /// Find which faction controls a location
    pub fn faction_controlling(&self, location_id: &str) -> Option<&Faction> {
        self.factions
            .values()
            .find(|f| f.controls_location(location_id))
    }
}

/// Tick of next ritual for each faction
#[derive(Resource, Debug, Default)]
pub struct RitualSchedule {
    /// Maps faction_id -> next ritual tick
    next_rituals: HashMap<String, u64>,
    /// Interval between rituals (in ticks)
    pub ritual_interval: u64,
}

impl RitualSchedule {
    pub fn new(ritual_interval: u64) -> Self {
        Self {
            next_rituals: HashMap::new(),
            ritual_interval,
        }
    }

    pub fn schedule_ritual(&mut self, faction_id: impl Into<String>, tick: u64) {
        self.next_rituals.insert(faction_id.into(), tick);
    }

    pub fn next_ritual(&self, faction_id: &str) -> Option<u64> {
        self.next_rituals.get(faction_id).copied()
    }

    pub fn is_ritual_due(&self, faction_id: &str, current_tick: u64) -> bool {
        self.next_rituals
            .get(faction_id)
            .map_or(false, |&tick| current_tick >= tick)
    }

    pub fn advance_ritual(&mut self, faction_id: &str) {
        if let Some(tick) = self.next_rituals.get_mut(faction_id) {
            *tick += self.ritual_interval;
        }
    }
}
