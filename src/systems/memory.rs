//! Memory System
//!
//! Systems for memory decay, cleanup, and propagation effects.

use bevy_ecs::prelude::*;

use crate::components::agent::AgentId;
use crate::components::social::{MemoryBank, MemoryValence, RelationshipGraph};
use crate::components::world::WorldState;

/// Resource to track the last season for decay purposes
#[derive(Resource, Debug, Default)]
pub struct SeasonTracker {
    last_decay_tick: u64,
    /// Ticks per season (default 300: 10 ticks/day * 30 days)
    ticks_per_season: u64,
}

impl SeasonTracker {
    pub fn new() -> Self {
        Self {
            last_decay_tick: 0,
            ticks_per_season: 300, // 10 ticks/day * 30 days/season
        }
    }

    /// Check if a season has passed since last decay
    pub fn should_decay(&self, current_tick: u64) -> bool {
        current_tick >= self.last_decay_tick + self.ticks_per_season
    }

    /// Mark that decay has been applied
    pub fn mark_decayed(&mut self, tick: u64) {
        self.last_decay_tick = tick;
    }
}

/// System: Apply memory decay each season
///
/// Memories gradually fade over time:
/// - Firsthand memories decay slowly (0.95 per season)
/// - Secondhand memories decay faster (0.85 per season)
/// - Memories below significance threshold are removed
pub fn decay_memories(
    world_state: Res<WorldState>,
    mut memory_bank: ResMut<MemoryBank>,
    mut season_tracker: ResMut<SeasonTracker>,
    query: Query<&AgentId>,
) {
    // Only decay once per season
    if !season_tracker.should_decay(world_state.current_tick) {
        return;
    }

    // Apply decay to all agent memories
    for agent_id in query.iter() {
        memory_bank.decay_memories(&agent_id.0, 1);
    }

    season_tracker.mark_decayed(world_state.current_tick);
}

/// System: Clean up insignificant memories periodically
///
/// Removes memories that have decayed below the significance threshold
/// to prevent unbounded memory growth.
pub fn cleanup_memories(
    world_state: Res<WorldState>,
    mut memory_bank: ResMut<MemoryBank>,
    query: Query<&AgentId>,
) {
    // Only cleanup every 100 ticks to reduce overhead
    if world_state.current_tick % 100 != 0 {
        return;
    }

    for agent_id in query.iter() {
        memory_bank.cleanup(&agent_id.0);
    }
}

/// System: Process memory propagation effects
///
/// When agents receive negative information about a third party,
/// their trust in that third party should decrease based on:
/// - Trust in the information source
/// - Fidelity of the memory
/// - Existing trust in the subject
pub fn process_memory_propagation(
    mut relationship_graph: ResMut<RelationshipGraph>,
    memory_bank: Res<MemoryBank>,
    query: Query<&AgentId>,
) {
    // This system processes newly received secondhand memories
    // and updates trust accordingly

    // For now, we'll process this during communication action execution
    // This placeholder ensures the system hook exists for future refinement
    let _ = (&relationship_graph, &memory_bank, &query);
}

/// Calculate trust impact from receiving secondhand information
///
/// Returns the trust delta to apply based on:
/// - valence: positive memories increase trust, negative decrease
/// - source_trust: how much the receiver trusts the source
/// - fidelity: how reliable the memory is
pub fn calculate_secondhand_trust_impact(
    valence: MemoryValence,
    source_trust: f32,
    fidelity: f32,
) -> f32 {
    // Base impact: 30% of direct effect (from behavioral rules)
    const SECONDHAND_MULTIPLIER: f32 = 0.3;

    let base_impact = match valence {
        MemoryValence::Positive => 0.1,  // Small positive impact
        MemoryValence::Neutral => 0.0,   // No impact
        MemoryValence::Negative => -0.15, // Larger negative impact (asymmetric)
    };

    // Scale by source trust (only believe if we trust the source)
    // and memory fidelity
    let trust_factor = (source_trust + 1.0) / 2.0; // Normalize to 0-1

    base_impact * SECONDHAND_MULTIPLIER * trust_factor * fidelity
}

/// Query: Get the most interesting shareable memory for an agent
///
/// Returns a memory suitable for sharing based on:
/// - Not a secret
/// - Sufficient emotional weight
/// - Sorted by interestingness (emotional weight * recency)
pub fn get_most_interesting_memory<'a>(
    memory_bank: &'a MemoryBank,
    agent_id: &str,
    current_tick: u64,
) -> Option<&'a crate::components::social::Memory> {
    let shareable = memory_bank.shareable_memories(agent_id);

    if shareable.is_empty() {
        return None;
    }

    // Score by interestingness: emotional weight boosted by recency
    shareable.into_iter()
        .max_by(|a, b| {
            let score_a = calculate_interestingness(a, current_tick);
            let score_b = calculate_interestingness(b, current_tick);
            score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
        })
}

/// Calculate how "interesting" a memory is for sharing
fn calculate_interestingness(memory: &crate::components::social::Memory, current_tick: u64) -> f32 {
    let age = current_tick.saturating_sub(memory.tick_created) as f32;
    let recency_boost = 1.0 / (1.0 + age / 100.0); // Recent memories are more interesting

    let valence_boost = match memory.valence {
        MemoryValence::Negative => 1.2, // Negative gossip is more "interesting"
        MemoryValence::Positive => 1.0,
        MemoryValence::Neutral => 0.8,
    };

    memory.emotional_weight * recency_boost * valence_boost * memory.fidelity
}

/// Query: Get memories about a specific subject that could damage their reputation
pub fn get_damaging_memories<'a>(
    memory_bank: &'a MemoryBank,
    agent_id: &str,
    subject_id: &str,
) -> Vec<&'a crate::components::social::Memory> {
    memory_bank.memories_about(agent_id, subject_id)
        .into_iter()
        .filter(|m| m.valence == MemoryValence::Negative && !m.is_secret)
        .collect()
}

/// Query: Get all agents who have shared information with this agent
pub fn get_information_sources(
    memory_bank: &MemoryBank,
    agent_id: &str,
) -> Vec<String> {
    let mut sources = std::collections::HashSet::new();

    if let Some(memories) = memory_bank.get_memories(agent_id) {
        for memory in memories {
            for source in &memory.source_chain {
                sources.insert(source.agent_id.clone());
            }
        }
    }

    sources.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::social::Memory;

    #[test]
    fn test_season_tracker() {
        let mut tracker = SeasonTracker::new();

        assert!(tracker.should_decay(300));
        assert!(!tracker.should_decay(100));

        tracker.mark_decayed(300);
        assert!(!tracker.should_decay(400));
        assert!(tracker.should_decay(600));
    }

    #[test]
    fn test_secondhand_trust_impact() {
        // Negative memory from trusted source
        let impact = calculate_secondhand_trust_impact(
            MemoryValence::Negative,
            0.6, // High trust in source
            1.0, // Full fidelity
        );
        assert!(impact < 0.0, "Negative memory should decrease trust");

        // Negative memory from distrusted source
        let impact_low = calculate_secondhand_trust_impact(
            MemoryValence::Negative,
            -0.5, // Low trust in source
            1.0,
        );
        assert!(impact_low.abs() < impact.abs(), "Distrusted source should have less impact");

        // Positive memory
        let positive = calculate_secondhand_trust_impact(
            MemoryValence::Positive,
            0.5,
            1.0,
        );
        assert!(positive > 0.0, "Positive memory should increase trust");
    }

    #[test]
    fn test_interestingness() {
        let recent_negative = Memory::firsthand(
            "mem1", "evt1", "subject1", "bad thing happened",
            0.8, 100, MemoryValence::Negative,
        );

        let old_positive = Memory::firsthand(
            "mem2", "evt2", "subject2", "good thing happened",
            0.8, 0, MemoryValence::Positive,
        );

        let score_recent = calculate_interestingness(&recent_negative, 110);
        let score_old = calculate_interestingness(&old_positive, 110);

        assert!(score_recent > score_old, "Recent negative memory should be more interesting");
    }
}
