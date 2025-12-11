//! Social Components
//!
//! Components for relationships, memories, and trust.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Multi-dimensional trust model
/// Each dimension is -1.0 to 1.0 (negative = distrust, positive = trust)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trust {
    /// Do they do what they say they will?
    pub reliability: f32,
    /// Do they want what I want?
    pub alignment: f32,
    /// Can they actually deliver on promises?
    pub capability: f32,
}

impl Default for Trust {
    fn default() -> Self {
        Self {
            reliability: 0.0,
            alignment: 0.0,
            capability: 0.0,
        }
    }
}

impl Trust {
    pub fn new(reliability: f32, alignment: f32, capability: f32) -> Self {
        Self {
            reliability: reliability.clamp(-1.0, 1.0),
            alignment: alignment.clamp(-1.0, 1.0),
            capability: capability.clamp(-1.0, 1.0),
        }
    }

    /// Create initial trust for faction-mates
    pub fn faction_mate() -> Self {
        Self {
            reliability: 0.3,
            alignment: 0.4,
            capability: 0.2,
        }
    }

    /// Create initial trust for faction leader
    pub fn faction_leader() -> Self {
        Self {
            reliability: 0.5,
            alignment: 0.5,
            capability: 0.6,
        }
    }

    /// Create neutral trust for strangers
    pub fn neutral() -> Self {
        Self::default()
    }

    /// Overall trust score (weighted average)
    pub fn overall(&self) -> f32 {
        self.reliability * 0.4 + self.alignment * 0.35 + self.capability * 0.25
    }

    /// Update reliability with clamping
    pub fn update_reliability(&mut self, delta: f32) {
        self.reliability = (self.reliability + delta).clamp(-1.0, 1.0);
    }

    /// Update alignment with clamping
    pub fn update_alignment(&mut self, delta: f32) {
        self.alignment = (self.alignment + delta).clamp(-1.0, 1.0);
    }

    /// Update capability with clamping
    pub fn update_capability(&mut self, delta: f32) {
        self.capability = (self.capability + delta).clamp(-1.0, 1.0);
    }

    /// Apply betrayal penalty (catastrophic trust collapse)
    pub fn apply_betrayal(&mut self) {
        self.reliability -= 0.5;
        self.alignment -= 0.4;
        self.reliability = self.reliability.clamp(-1.0, 1.0);
        self.alignment = self.alignment.clamp(-1.0, 1.0);
    }
}

/// A relationship between two agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    /// Who this relationship is with
    pub target_id: String,
    /// Multi-dimensional trust
    pub trust: Trust,
    /// Tick of last interaction
    pub last_interaction_tick: u64,
    /// Count of significant memories about this agent
    pub memory_count: u32,
}

impl Relationship {
    pub fn new(target_id: impl Into<String>) -> Self {
        Self {
            target_id: target_id.into(),
            trust: Trust::default(),
            last_interaction_tick: 0,
            memory_count: 0,
        }
    }

    pub fn with_trust(mut self, trust: Trust) -> Self {
        self.trust = trust;
        self
    }
}

/// Source attribution for memory propagation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySource {
    /// Agent ID who provided this information
    pub agent_id: String,
    /// Agent name for display
    pub agent_name: String,
}

/// A memory stored by an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    /// Unique identifier for this memory
    pub memory_id: String,
    /// Reference to the original event (if firsthand)
    pub event_id: Option<String>,
    /// Subject of the memory (agent ID or topic)
    pub subject: String,
    /// Brief content summary
    pub content: String,
    /// Fidelity: 1.0 for firsthand, degrades with each hop
    pub fidelity: f32,
    /// Source chain: who told whom
    pub source_chain: Vec<MemorySource>,
    /// Emotional weight: how significant this memory feels
    pub emotional_weight: f32,
    /// Tick when this memory was formed
    pub tick_created: u64,
    /// Is this memory positive, negative, or neutral about the subject?
    pub valence: MemoryValence,
    /// Is this a secret (should not be shared openly)?
    pub is_secret: bool,
}

/// Emotional valence of a memory
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryValence {
    Positive,
    Neutral,
    Negative,
}

impl Memory {
    /// Create a firsthand memory
    pub fn firsthand(
        memory_id: impl Into<String>,
        event_id: impl Into<String>,
        subject: impl Into<String>,
        content: impl Into<String>,
        emotional_weight: f32,
        tick: u64,
        valence: MemoryValence,
    ) -> Self {
        Self {
            memory_id: memory_id.into(),
            event_id: Some(event_id.into()),
            subject: subject.into(),
            content: content.into(),
            fidelity: 1.0,
            source_chain: Vec::new(),
            emotional_weight,
            tick_created: tick,
            valence,
            is_secret: false,
        }
    }

    /// Create a secondhand memory from another agent's sharing
    pub fn secondhand(
        memory_id: impl Into<String>,
        original: &Memory,
        source: MemorySource,
        tick: u64,
    ) -> Self {
        let mut source_chain = original.source_chain.clone();
        source_chain.push(source);

        Self {
            memory_id: memory_id.into(),
            event_id: original.event_id.clone(),
            subject: original.subject.clone(),
            content: original.content.clone(),
            fidelity: original.fidelity * 0.7, // Degrades with each hop
            source_chain,
            emotional_weight: original.emotional_weight * 0.5, // Diminished impact
            tick_created: tick,
            valence: original.valence,
            is_secret: original.is_secret,
        }
    }

    /// Create a memory from archive reading
    pub fn from_archive(
        memory_id: impl Into<String>,
        content: impl Into<String>,
        subject: impl Into<String>,
        tick: u64,
        valence: MemoryValence,
    ) -> Self {
        Self {
            memory_id: memory_id.into(),
            event_id: None,
            subject: subject.into(),
            content: content.into(),
            fidelity: 0.9, // Written records have high but not perfect fidelity
            source_chain: Vec::new(), // "the archive records..."
            emotional_weight: 0.3, // Lower emotional weight unless reinforced
            tick_created: tick,
            valence,
            is_secret: false,
        }
    }

    /// Apply memory decay over time
    pub fn decay(&mut self, seasons_elapsed: u32) {
        let decay_rate = if self.source_chain.is_empty() {
            0.95 // Firsthand: slow decay
        } else {
            0.85 // Secondhand: faster decay
        };

        for _ in 0..seasons_elapsed {
            self.fidelity *= decay_rate;
            self.emotional_weight *= decay_rate;
        }
    }

    /// Check if memory is still significant enough to keep
    pub fn is_significant(&self) -> bool {
        self.fidelity > 0.1 || self.emotional_weight > 0.1
    }
}

/// Resource: Graph of all relationships between agents
#[derive(Resource, Debug, Default)]
pub struct RelationshipGraph {
    /// Maps (from_agent_id, to_agent_id) -> Relationship
    relationships: HashMap<(String, String), Relationship>,
}

impl RelationshipGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get relationship from one agent to another
    pub fn get(&self, from: &str, to: &str) -> Option<&Relationship> {
        self.relationships.get(&(from.to_string(), to.to_string()))
    }

    /// Get mutable relationship
    pub fn get_mut(&mut self, from: &str, to: &str) -> Option<&mut Relationship> {
        self.relationships
            .get_mut(&(from.to_string(), to.to_string()))
    }

    /// Set or update a relationship
    pub fn set(&mut self, from: impl Into<String>, relationship: Relationship) {
        let from = from.into();
        let to = relationship.target_id.clone();
        self.relationships.insert((from, to), relationship);
    }

    /// Get all relationships for an agent
    pub fn relationships_for(&self, agent_id: &str) -> Vec<&Relationship> {
        self.relationships
            .iter()
            .filter(|((from, _), _)| from == agent_id)
            .map(|(_, rel)| rel)
            .collect()
    }

    /// Get agents who trust this agent
    pub fn trusted_by(&self, agent_id: &str) -> Vec<(&String, &Relationship)> {
        self.relationships
            .iter()
            .filter(|((_, to), _)| to == agent_id)
            .map(|((from, _), rel)| (from, rel))
            .collect()
    }

    /// Check if relationship exists
    pub fn has_relationship(&self, from: &str, to: &str) -> bool {
        self.relationships
            .contains_key(&(from.to_string(), to.to_string()))
    }

    /// Create or get relationship (ensures it exists)
    pub fn ensure_relationship(&mut self, from: &str, to: &str) -> &mut Relationship {
        let key = (from.to_string(), to.to_string());
        self.relationships
            .entry(key)
            .or_insert_with(|| Relationship::new(to))
    }
}

/// Resource: Bank of all memories for all agents
#[derive(Resource, Debug, Default)]
pub struct MemoryBank {
    /// Maps agent_id -> list of memories
    memories: HashMap<String, Vec<Memory>>,
    /// Counter for generating unique memory IDs
    next_memory_id: u64,
}

impl MemoryBank {
    pub fn new() -> Self {
        Self::default()
    }

    /// Generate a unique memory ID
    pub fn generate_id(&mut self) -> String {
        let id = format!("mem_{:08}", self.next_memory_id);
        self.next_memory_id += 1;
        id
    }

    /// Add a memory for an agent
    pub fn add_memory(&mut self, agent_id: impl Into<String>, memory: Memory) {
        self.memories
            .entry(agent_id.into())
            .or_default()
            .push(memory);
    }

    /// Get all memories for an agent
    pub fn get_memories(&self, agent_id: &str) -> Option<&Vec<Memory>> {
        self.memories.get(agent_id)
    }

    /// Get mutable memories for an agent
    pub fn get_memories_mut(&mut self, agent_id: &str) -> Option<&mut Vec<Memory>> {
        self.memories.get_mut(agent_id)
    }

    /// Get memories about a specific subject
    pub fn memories_about(&self, agent_id: &str, subject: &str) -> Vec<&Memory> {
        self.memories
            .get(agent_id)
            .map(|mems| mems.iter().filter(|m| m.subject == subject).collect())
            .unwrap_or_default()
    }

    /// Get memories by valence
    pub fn memories_by_valence(&self, agent_id: &str, valence: MemoryValence) -> Vec<&Memory> {
        self.memories
            .get(agent_id)
            .map(|mems| mems.iter().filter(|m| m.valence == valence).collect())
            .unwrap_or_default()
    }

    /// Get interesting memories suitable for sharing
    pub fn shareable_memories(&self, agent_id: &str) -> Vec<&Memory> {
        self.memories
            .get(agent_id)
            .map(|mems| {
                mems.iter()
                    .filter(|m| !m.is_secret && m.emotional_weight > 0.2)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Remove insignificant memories (cleanup)
    pub fn cleanup(&mut self, agent_id: &str) {
        if let Some(memories) = self.memories.get_mut(agent_id) {
            memories.retain(|m| m.is_significant());
        }
    }

    /// Apply decay to all memories for an agent
    pub fn decay_memories(&mut self, agent_id: &str, seasons_elapsed: u32) {
        if let Some(memories) = self.memories.get_mut(agent_id) {
            for memory in memories.iter_mut() {
                memory.decay(seasons_elapsed);
            }
        }
    }
}
