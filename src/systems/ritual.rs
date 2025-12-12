//! Ritual System
//!
//! Handles periodic faction rituals where archive entries are read to attendees.

use bevy_ecs::prelude::*;
use std::collections::HashMap;

use crate::components::agent::{AgentId, AgentName};
use crate::components::faction::{FactionMembership, FactionRegistry, RitualSchedule};
use crate::components::social::{Memory, MemoryBank, MemorySource, MemoryValence};
use crate::components::world::{Position, WorldState};
use crate::events::types::{
    ActorSnapshot, AffectedActor, Event, EventActors, EventContext, EventOutcome,
    EventSubtype, EventTimestamp, EventType, GeneralOutcome, RitualSubtype,
};
use crate::systems::action::TickEvents;
use crate::systems::needs::RitualAttendance;
use crate::systems::perception::AgentsByLocation;

/// Number of entries to read per ritual
const ENTRIES_PER_RITUAL: usize = 3;

/// System to execute faction rituals when due
pub fn execute_rituals(
    world_state: Res<WorldState>,
    mut ritual_schedule: ResMut<RitualSchedule>,
    mut faction_registry: ResMut<FactionRegistry>,
    agents_by_location: Res<AgentsByLocation>,
    mut memory_bank: ResMut<MemoryBank>,
    mut ritual_attendance: ResMut<RitualAttendance>,
    mut tick_events: ResMut<TickEvents>,
    query: Query<(&AgentId, &AgentName, &Position, &FactionMembership)>,
) {
    // Build agent info map
    let agent_info: HashMap<String, (&AgentName, &Position, &FactionMembership)> = query
        .iter()
        .map(|(id, name, pos, mem)| (id.0.clone(), (name, pos, mem)))
        .collect();

    // Get all faction IDs
    let faction_ids: Vec<String> = faction_registry.faction_ids().iter().map(|s| (*s).clone()).collect();

    for faction_id in faction_ids {
        // Check if ritual is due
        if !ritual_schedule.is_ritual_due(&faction_id, world_state.current_tick) {
            continue;
        }


        // Get faction info
        let (hq_location, reader_id, faction_name) = {
            let Some(faction) = faction_registry.get(&faction_id) else {
                continue;
            };
            (faction.hq_location.clone(), faction.reader.clone(), faction.name.clone())
        };

        // Get agents at HQ
        let agents_at_hq: Vec<String> = agents_by_location
            .at_location(&hq_location)
            .iter()
            .filter(|id| {
                agent_info.get(*id)
                    .map(|(_, _, mem)| mem.faction_id == faction_id)
                    .unwrap_or(false)
            })
            .cloned()
            .collect();

        // Need at least 2 attendees for a ritual
        if agents_at_hq.len() < 2 {
            // Ritual skipped due to low attendance
            ritual_schedule.advance_ritual(&faction_id);
            continue;
        }

        // Get entries to read (ritual can still happen with empty archive)
        let entries_to_read = {
            let Some(archive) = faction_registry.get_archive(&faction_id) else {
                ritual_schedule.advance_ritual(&faction_id);
                continue;
            };

            if archive.entry_count() == 0 {
                Vec::new() // Empty archive, but ritual still happens
            } else {
                // Get least-read entries (to spread the reading)
                archive.least_read_entries(ENTRIES_PER_RITUAL)
                    .into_iter()
                    .map(|e| (e.entry_id.clone(), e.subject.clone(), e.content.clone(), e.author_name.clone()))
                    .collect::<Vec<_>>()
            }
        };

        // Find the reader (faction reader or leader as fallback)
        let reader_agent_id = reader_id.clone().or_else(|| {
            agents_at_hq.iter().find(|id| {
                agent_info.get(*id)
                    .map(|(_, _, mem)| mem.is_leader())
                    .unwrap_or(false)
            }).cloned()
        });

        // Create memories for each attendee from the entries read
        for attendee_id in &agents_at_hq {
            // Record attendance
            ritual_attendance.record_attended(attendee_id);

            // Create memories from entries
            for (entry_id, subject, content, author_name) in &entries_to_read {
                let memory_id = memory_bank.generate_id();

                // Determine valence from content (simple heuristic)
                let valence = if content.contains("helped") || content.contains("reliable") || content.contains("good") {
                    MemoryValence::Positive
                } else if content.contains("complained") || content.contains("failed") || content.contains("negative") {
                    MemoryValence::Negative
                } else {
                    MemoryValence::Neutral
                };

                let memory = Memory {
                    memory_id: memory_id.clone(),
                    event_id: Some(entry_id.clone()),
                    subject: subject.clone(),
                    content: format!("Heard at ritual: {}", content),
                    fidelity: 0.9, // Archive memories are high fidelity
                    source_chain: vec![MemorySource {
                        agent_id: "archive".to_string(),
                        agent_name: author_name.clone(),
                    }],
                    emotional_weight: 0.4, // Moderate emotional impact from rituals
                    tick_created: world_state.current_tick,
                    valence,
                    is_secret: false,
                };

                memory_bank.add_memory(attendee_id, memory);
            }
        }

        // Mark entries as read
        if let Some(archive) = faction_registry.get_archive_mut(&faction_id) {
            for (entry_id, _, _, _) in &entries_to_read {
                if let Some(entry) = archive.find_entry_mut(entry_id) {
                    entry.increment_reads();
                }
            }
        }

        // Record missed attendance for faction members not at HQ
        for (agent_id, (_, _, membership)) in &agent_info {
            if membership.faction_id == faction_id && !agents_at_hq.contains(agent_id) {
                ritual_attendance.record_missed(agent_id);
            }
        }

        // Generate ritual event
        let event = create_ritual_event(
            &mut tick_events,
            &world_state,
            &faction_id,
            &faction_name,
            &hq_location,
            &reader_agent_id,
            &agents_at_hq,
            entries_to_read.len(),
            &agent_info,
        );
        tick_events.push(event);

        // Advance to next ritual
        ritual_schedule.advance_ritual(&faction_id);
    }
}

/// Create a ritual reading event
fn create_ritual_event(
    tick_events: &mut TickEvents,
    world_state: &WorldState,
    faction_id: &str,
    faction_name: &str,
    location: &str,
    reader_id: &Option<String>,
    attendees: &[String],
    entries_read: usize,
    agent_info: &HashMap<String, (&AgentName, &Position, &FactionMembership)>,
) -> Event {
    let event_id = tick_events.generate_id();
    let timestamp = EventTimestamp {
        tick: world_state.current_tick,
        date: world_state.formatted_date(),
    };

    // Primary actor is the reader
    let primary = if let Some(rid) = reader_id {
        agent_info.get(rid).map(|(name, pos, mem)| ActorSnapshot {
            agent_id: rid.clone(),
            name: name.0.clone(),
            faction: mem.faction_id.clone(),
            role: "reader".to_string(),
            location: pos.location_id.clone(),
        })
    } else {
        None
    }.unwrap_or(ActorSnapshot {
        agent_id: "archive".to_string(),
        name: format!("{} Archive", faction_name),
        faction: faction_id.to_string(),
        role: "reader".to_string(),
        location: location.to_string(),
    });

    // Affected actors are the attendees
    let affected: Vec<AffectedActor> = attendees.iter()
        .filter_map(|id| {
            agent_info.get(id).map(|(name, _, mem)| AffectedActor {
                agent_id: id.clone(),
                name: name.0.clone(),
                faction: mem.faction_id.clone(),
                role: format!("{:?}", mem.role).to_lowercase(),
                relationship_to_primary: None,
                attended: Some(true),
                reason: None,
            })
        })
        .collect();

    let drama_score = 0.3 + (entries_read as f32 * 0.1).min(0.3) + (attendees.len() as f32 * 0.02).min(0.2);

    Event {
        event_id,
        timestamp,
        event_type: EventType::Ritual,
        subtype: EventSubtype::Ritual(RitualSubtype::ReadingHeld),
        actors: EventActors {
            primary,
            secondary: None,
            affected: Some(affected),
        },
        context: EventContext {
            trigger: "scheduled_ritual".to_string(),
            preconditions: Vec::new(),
            location_description: Some(format!("at {} faction hall", faction_name)),
        },
        outcome: EventOutcome::General(GeneralOutcome {
            description: Some(format!(
                "{} archive reading held with {} attendees, {} entries read",
                faction_name, attendees.len(), entries_read
            )),
            state_changes: vec![
                format!("{} agents gained memories from archive", attendees.len()),
            ],
        }),
        drama_tags: vec!["faction_ritual".to_string(), "archive_reading".to_string()],
        drama_score,
        connected_events: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entries_per_ritual() {
        assert_eq!(ENTRIES_PER_RITUAL, 3);
    }
}
