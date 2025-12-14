//! Action Execution System
//!
//! Executes selected actions and generates events.

use bevy_ecs::prelude::*;
use rand::Rng;

use crate::actions::movement::{MoveAction, MovementType};
use crate::actions::communication::{CommunicationAction, CommunicationType, TargetMode, communication_weights};
use crate::actions::archive::{ArchiveAction, ArchiveActionType};
use crate::actions::resource::{ResourceAction, ResourceActionType};
use crate::actions::social::{SocialAction, SocialActionType, social_weights};
use crate::actions::faction::{FactionAction, FactionActionType};
use crate::actions::conflict::{ConflictAction, ConflictActionType, conflict_weights};
use crate::actions::beer::{BeerAction, BeerActionType, beer_weights};
use crate::components::agent::{AgentId, AgentName, Alive, Goals, GoalType, Intoxication, Needs, Role, SocialBelonging, Traits};
use crate::components::social::{Memory, MemoryBank, MemorySource, MemoryValence, RelationshipGraph};
use crate::components::world::{Position, WorldState};
use crate::events::types::{
    ActorSnapshot, Event, EventActors, EventContext, EventOutcome, EventTimestamp, EventType,
    EventSubtype, MovementSubtype, MovementOutcome, CommunicationSubtype,
    CommunicationOutcome as EventCommunicationOutcome, MemorySharedInfo, RecipientStateChange,
    ArchiveSubtype, ArchiveOutcome, ResourceSubtype, CooperationSubtype, FactionSubtype,
    ConflictSubtype, GeneralOutcome, RelationshipOutcome, RelationshipChange,
};
use crate::components::faction::{FactionMembership, FactionRegistry, ArchiveEntry};
use crate::systems::memory::calculate_secondhand_trust_impact;
use crate::systems::perception::AgentsByLocation;
use crate::SimRng;

use super::generate::Action;
use super::select::SelectedActions;

/// Resource storing events generated this tick
#[derive(Resource, Debug, Default)]
pub struct TickEvents {
    pub events: Vec<Event>,
    next_event_id: u64,
}

impl TickEvents {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn generate_id(&mut self) -> String {
        let id = format!("evt_{:08}", self.next_event_id);
        self.next_event_id += 1;
        id
    }

    pub fn push(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn drain(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// System to execute movement actions
pub fn execute_movement_actions(
    world_state: Res<WorldState>,
    mut selected_actions: ResMut<SelectedActions>,
    mut tick_events: ResMut<TickEvents>,
    mut query: Query<(Entity, &AgentId, &mut Position, &crate::components::faction::FactionMembership, &crate::components::agent::AgentName)>,
) {
    for (entity, agent_id, mut position, membership, name) in query.iter_mut() {
        let Some(action) = selected_actions.take(&agent_id.0) else {
            continue;
        };

        match action {
            Action::Move(move_action) => {
                let old_location = position.location_id.clone();
                let new_location = move_action.destination.clone();

                // Update position
                position.location_id = new_location.clone();

                // Generate movement event
                let event = create_movement_event(
                    &mut tick_events,
                    &world_state,
                    agent_id,
                    name,
                    membership,
                    &old_location,
                    &new_location,
                    move_action.movement_type,
                );

                tick_events.push(event);
            }
            Action::Communicate(_) => {
                // Communication is handled by execute_communication_actions
            }
            Action::Archive(_) => {
                // Archive is handled by execute_archive_actions
            }
            Action::Resource(_) => {
                // Resource actions are handled by execute_resource_actions
            }
            Action::Social(_) => {
                // Social actions are handled by execute_social_actions
            }
            Action::Faction(_) => {
                // Faction actions are handled by execute_faction_actions
            }
            Action::Conflict(_) => {
                // Conflict actions are handled by execute_conflict_actions
            }
            Action::Beer(_) => {
                // Beer actions are handled by execute_beer_actions
            }
            Action::Idle => {
                // No action needed for idle
            }
        }
    }
}

/// Create a movement event
fn create_movement_event(
    tick_events: &mut TickEvents,
    world_state: &WorldState,
    agent_id: &AgentId,
    name: &crate::components::agent::AgentName,
    membership: &crate::components::faction::FactionMembership,
    from_location: &str,
    to_location: &str,
    movement_type: MovementType,
) -> Event {
    let event_id = tick_events.generate_id();
    let timestamp = EventTimestamp {
        tick: world_state.current_tick,
        date: world_state.formatted_date(),
    };

    let actor = ActorSnapshot {
        agent_id: agent_id.0.clone(),
        name: name.0.clone(),
        faction: membership.faction_id.clone(),
        role: format!("{:?}", membership.role).to_lowercase(),
        location: from_location.to_string(),
    };

    let subtype = match movement_type {
        MovementType::Travel => MovementSubtype::Travel,
        MovementType::Flee => MovementSubtype::Flee,
        MovementType::Pursue => MovementSubtype::Pursue,
        MovementType::Patrol => MovementSubtype::Patrol,
        MovementType::ReturnHome => MovementSubtype::ReturnHome,
    };

    let trigger = match movement_type {
        MovementType::Travel => "random_wandering",
        MovementType::Flee => "fleeing_danger",
        MovementType::Pursue => "pursuing_target",
        MovementType::Patrol => "scheduled_patrol",
        MovementType::ReturnHome => "returning_home",
    };

    Event {
        event_id,
        timestamp,
        event_type: EventType::Movement,
        subtype: EventSubtype::Movement(subtype),
        actors: EventActors {
            primary: actor,
            secondary: None,
            affected: None,
        },
        context: EventContext {
            trigger: trigger.to_string(),
            preconditions: Vec::new(),
            location_description: Some(format!("traveling from {} to {}", from_location, to_location)),
        },
        outcome: EventOutcome::Movement(MovementOutcome {
            new_location: to_location.to_string(),
            travel_duration_ticks: Some(1),
        }),
        drama_tags: Vec::new(),
        drama_score: 0.1, // Movement is low drama
        connected_events: Vec::new(),
    }
}

/// System to execute communication actions
pub fn execute_communication_actions(
    world_state: Res<WorldState>,
    agents_by_location: Res<AgentsByLocation>,
    mut memory_bank: ResMut<MemoryBank>,
    mut relationship_graph: ResMut<RelationshipGraph>,
    mut selected_actions: ResMut<SelectedActions>,
    mut tick_events: ResMut<TickEvents>,
    query: Query<(&AgentId, &AgentName, &Position, &crate::components::faction::FactionMembership)>,
) {
    // Build lookup map for agent info
    let agent_info: std::collections::HashMap<String, (&AgentName, &Position, &crate::components::faction::FactionMembership)> = query
        .iter()
        .map(|(id, name, pos, mem)| (id.0.clone(), (name, pos, mem)))
        .collect();

    // Collect communication actions to process
    let mut comm_actions: Vec<(String, CommunicationAction)> = Vec::new();

    for (agent_id, _, _, _) in query.iter() {
        if let Some(action) = selected_actions.actions.get(&agent_id.0) {
            if let Action::Communicate(comm_action) = action {
                comm_actions.push((agent_id.0.clone(), comm_action.clone()));
            }
        }
    }

    // Process each communication action
    for (actor_id, comm_action) in comm_actions {
        let Some((actor_name, actor_pos, actor_membership)) = agent_info.get(&actor_id) else {
            continue;
        };

        // Get the memory being shared (if any)
        let shared_memory = comm_action.memory_id.as_ref().and_then(|mem_id| {
            memory_bank.get_memories(&actor_id)
                .and_then(|mems| mems.iter().find(|m| &m.memory_id == mem_id).cloned())
        });

        match comm_action.communication_type {
            CommunicationType::ShareMemory => {
                if let Some(memory) = shared_memory {
                    execute_share_memory(
                        &world_state,
                        &agents_by_location,
                        &mut memory_bank,
                        &mut relationship_graph,
                        &mut tick_events,
                        &agent_info,
                        &actor_id,
                        actor_name,
                        actor_pos,
                        actor_membership,
                        &comm_action,
                        &memory,
                    );
                }
            }
            CommunicationType::SpreadRumor => {
                // Similar to share memory but with potential distortion
                // For now, treat same as share memory
                if let Some(memory) = shared_memory {
                    execute_share_memory(
                        &world_state,
                        &agents_by_location,
                        &mut memory_bank,
                        &mut relationship_graph,
                        &mut tick_events,
                        &agent_info,
                        &actor_id,
                        actor_name,
                        actor_pos,
                        actor_membership,
                        &comm_action,
                        &memory,
                    );
                }
            }
            CommunicationType::Lie | CommunicationType::Confess => {
                // These require more complex handling - placeholder for now
            }
        }
    }
}

/// Execute a share memory action
fn execute_share_memory(
    world_state: &WorldState,
    agents_by_location: &AgentsByLocation,
    memory_bank: &mut MemoryBank,
    relationship_graph: &mut RelationshipGraph,
    tick_events: &mut TickEvents,
    agent_info: &std::collections::HashMap<String, (&AgentName, &Position, &crate::components::faction::FactionMembership)>,
    actor_id: &str,
    actor_name: &AgentName,
    actor_pos: &Position,
    actor_membership: &crate::components::faction::FactionMembership,
    comm_action: &CommunicationAction,
    memory: &Memory,
) {
    let mut recipients = Vec::new();
    let mut memories_created = Vec::new();

    // Determine recipients based on target mode
    let target_ids: Vec<String> = if comm_action.target_mode == TargetMode::Group {
        // Target all agents at location except self
        agents_by_location.at_location(&actor_pos.location_id)
            .iter()
            .filter(|id| *id != actor_id)
            .cloned()
            .collect()
    } else {
        // Just the individual target
        vec![comm_action.target_id.clone()]
    };

    // Calculate fidelity for secondhand memory
    let fidelity_multiplier = if comm_action.target_mode == TargetMode::Group {
        communication_weights::GROUP_FIDELITY_MULTIPLIER
    } else {
        1.0
    };

    // Create memory source
    let source = MemorySource {
        agent_id: actor_id.to_string(),
        agent_name: actor_name.0.clone(),
    };

    // Transfer memory to each recipient
    for target_id in &target_ids {
        if target_id == actor_id {
            continue;
        }

        let Some((target_name, _, target_membership)) = agent_info.get(target_id) else {
            continue;
        };

        // Create secondhand memory for recipient
        let new_memory_id = memory_bank.generate_id();
        let mut new_memory = Memory::secondhand(
            &new_memory_id,
            memory,
            source.clone(),
            world_state.current_tick,
        );

        // Apply group fidelity multiplier
        new_memory.fidelity *= fidelity_multiplier;

        memory_bank.add_memory(target_id, new_memory);
        memories_created.push(new_memory_id);
        recipients.push(target_id.clone());

        // Update trust if memory is about a third party
        if memory.subject != actor_id && memory.subject != *target_id {
            // Get trust in source (actor)
            let source_trust = relationship_graph
                .get(target_id, actor_id)
                .map(|r| r.trust.overall())
                .unwrap_or(0.0);

            // Calculate trust impact toward the subject
            let trust_delta = calculate_secondhand_trust_impact(
                memory.valence,
                source_trust,
                memory.fidelity,
            );

            if trust_delta.abs() > 0.001 {
                let rel = relationship_graph.ensure_relationship(target_id, &memory.subject);
                rel.trust.update_alignment(trust_delta);
            }
        }

        // Small trust boost between communicating parties
        let rel = relationship_graph.ensure_relationship(target_id, actor_id);
        let trust_bonus = if comm_action.target_mode == TargetMode::Individual {
            0.02 * communication_weights::INDIVIDUAL_RELATIONSHIP_MULTIPLIER
        } else {
            0.02 * communication_weights::GROUP_RELATIONSHIP_MULTIPLIER
        };
        rel.trust.update_reliability(trust_bonus);
        rel.last_interaction_tick = world_state.current_tick;
    }

    // Generate communication event
    if !recipients.is_empty() {
        let event = create_communication_event(
            tick_events,
            world_state,
            actor_id,
            actor_name,
            actor_membership,
            &actor_pos.location_id,
            &comm_action,
            memory,
            &recipients,
            agent_info,
        );
        tick_events.push(event);
    }
}

/// Create a communication event
fn create_communication_event(
    tick_events: &mut TickEvents,
    world_state: &WorldState,
    actor_id: &str,
    actor_name: &AgentName,
    actor_membership: &crate::components::faction::FactionMembership,
    location: &str,
    comm_action: &CommunicationAction,
    memory: &Memory,
    recipients: &[String],
    agent_info: &std::collections::HashMap<String, (&AgentName, &Position, &crate::components::faction::FactionMembership)>,
) -> Event {
    let event_id = tick_events.generate_id();
    let timestamp = EventTimestamp {
        tick: world_state.current_tick,
        date: world_state.formatted_date(),
    };

    let actor = ActorSnapshot {
        agent_id: actor_id.to_string(),
        name: actor_name.0.clone(),
        faction: actor_membership.faction_id.clone(),
        role: format!("{:?}", actor_membership.role).to_lowercase(),
        location: location.to_string(),
    };

    // Get secondary actor (first recipient for individual, none for group)
    let secondary = if comm_action.target_mode == TargetMode::Individual {
        recipients.first().and_then(|id| {
            agent_info.get(id).map(|(name, pos, mem)| ActorSnapshot {
                agent_id: id.clone(),
                name: name.0.clone(),
                faction: mem.faction_id.clone(),
                role: format!("{:?}", mem.role).to_lowercase(),
                location: pos.location_id.clone(),
            })
        })
    } else {
        None
    };

    let subtype = match comm_action.communication_type {
        CommunicationType::ShareMemory => CommunicationSubtype::ShareMemory,
        CommunicationType::SpreadRumor => CommunicationSubtype::SpreadRumor,
        CommunicationType::Lie => CommunicationSubtype::Lie,
        CommunicationType::Confess => CommunicationSubtype::Confess,
    };

    let trigger = match comm_action.communication_type {
        CommunicationType::ShareMemory => "gossip",
        CommunicationType::SpreadRumor => "spreading_rumor",
        CommunicationType::Lie => "deception",
        CommunicationType::Confess => "confession",
    };

    let source_chain: Vec<String> = memory.source_chain
        .iter()
        .map(|s| s.agent_name.clone())
        .collect();

    // Calculate drama score based on memory content
    let drama_score = calculate_communication_drama(memory, comm_action, recipients.len());

    Event {
        event_id,
        timestamp,
        event_type: EventType::Communication,
        subtype: EventSubtype::Communication(subtype),
        actors: EventActors {
            primary: actor,
            secondary,
            affected: None,
        },
        context: EventContext {
            trigger: trigger.to_string(),
            preconditions: Vec::new(),
            location_description: Some(format!("at {}", location)),
        },
        outcome: EventOutcome::Communication(EventCommunicationOutcome {
            memory_shared: Some(MemorySharedInfo {
                original_event: memory.event_id.clone(),
                content: memory.content.clone(),
                source_chain,
                fidelity: memory.fidelity,
            }),
            recipient_state_change: Some(RecipientStateChange {
                new_memory_added: true,
                trust_impact: None, // Could add detailed trust impact here
            }),
        }),
        drama_tags: get_communication_drama_tags(memory, comm_action),
        drama_score,
        connected_events: memory.event_id.clone().map(|e| vec![e]).unwrap_or_default(),
    }
}

/// Calculate drama score for communication
fn calculate_communication_drama(memory: &Memory, comm_action: &CommunicationAction, recipient_count: usize) -> f32 {
    let mut score = 0.2; // Base communication drama

    // Negative gossip is more dramatic
    if memory.valence == MemoryValence::Negative {
        score += 0.3;
    }

    // High emotional weight memories are more dramatic
    score += memory.emotional_weight * 0.2;

    // Secrets being shared are dramatic
    if memory.is_secret {
        score += 0.5;
    }

    // Group communication spreads drama wider
    if comm_action.target_mode == TargetMode::Group {
        score += 0.1 * (recipient_count as f32).min(5.0);
    }

    // Lies are dramatic
    if comm_action.communication_type == CommunicationType::Lie {
        score += 0.4;
    }

    score.min(1.0)
}

/// Get drama tags for communication event
fn get_communication_drama_tags(memory: &Memory, comm_action: &CommunicationAction) -> Vec<String> {
    let mut tags = Vec::new();

    if memory.valence == MemoryValence::Negative {
        tags.push("negative_gossip".to_string());
    }

    if memory.is_secret {
        tags.push("secret_revealed".to_string());
    }

    if comm_action.target_mode == TargetMode::Group {
        tags.push("public_announcement".to_string());
    }

    if comm_action.communication_type == CommunicationType::Lie {
        tags.push("deception".to_string());
    }

    if memory.source_chain.len() >= 2 {
        tags.push("secondhand_information".to_string());
    }

    tags
}

/// System to execute archive actions
pub fn execute_archive_actions(
    world_state: Res<WorldState>,
    mut faction_registry: ResMut<FactionRegistry>,
    memory_bank: Res<MemoryBank>,
    mut selected_actions: ResMut<SelectedActions>,
    mut tick_events: ResMut<TickEvents>,
    query: Query<(&AgentId, &AgentName, &Position, &crate::components::faction::FactionMembership)>,
) {
    // Collect archive actions to process
    let mut archive_actions: Vec<(String, ArchiveAction, String, String, String)> = Vec::new();

    for (agent_id, name, pos, membership) in query.iter() {
        if let Some(action) = selected_actions.actions.get(&agent_id.0) {
            if let Action::Archive(archive_action) = action {
                archive_actions.push((
                    agent_id.0.clone(),
                    archive_action.clone(),
                    name.0.clone(),
                    pos.location_id.clone(),
                    membership.faction_id.clone(),
                ));
            }
        }
    }

    // Process each archive action
    for (actor_id, archive_action, actor_name, location, actor_faction) in archive_actions {
        match archive_action.action_type {
            ArchiveActionType::WriteEntry => {
                // Get the memory being written
                let memory_content = archive_action.memory_id.as_ref().and_then(|mem_id| {
                    memory_bank.get_memories(&actor_id)
                        .and_then(|mems| mems.iter().find(|m| &m.memory_id == mem_id).cloned())
                });

                if let Some(memory) = memory_content {
                    // Write entry to archive
                    if let Some(archive) = faction_registry.get_archive_mut(&archive_action.faction_id) {
                        let entry_id = archive.generate_id(&archive_action.faction_id);
                        let entry = ArchiveEntry::new(
                            &entry_id,
                            &actor_id,
                            &actor_name,
                            &memory.subject,
                            &memory.content,
                            world_state.current_tick,
                        );
                        archive.add_entry(entry);

                        // Generate event
                        let event = create_archive_event(
                            &mut tick_events,
                            &world_state,
                            &actor_id,
                            &actor_name,
                            &actor_faction,
                            &location,
                            ArchiveSubtype::WriteEntry,
                            Some(&entry_id),
                            Some(&memory.content),
                            Some(&memory.subject),
                            true,
                        );
                        tick_events.push(event);
                    }
                }
            }
            ArchiveActionType::ReadArchive => {
                // Reading just generates an event (could also create a memory later)
                let event = create_archive_event(
                    &mut tick_events,
                    &world_state,
                    &actor_id,
                    &actor_name,
                    &actor_faction,
                    &location,
                    ArchiveSubtype::ReadEntry,
                    None,
                    None,
                    None,
                    true,
                );
                tick_events.push(event);
            }
            ArchiveActionType::DestroyEntry => {
                if let Some(entry_id) = &archive_action.entry_id {
                    if let Some(archive) = faction_registry.get_archive_mut(&archive_action.faction_id) {
                        let entry_content = archive.find_entry(entry_id).map(|e| e.content.clone());
                        let entry_subject = archive.find_entry(entry_id).map(|e| e.subject.clone());

                        if archive.remove_entry(entry_id) {
                            let event = create_archive_event(
                                &mut tick_events,
                                &world_state,
                                &actor_id,
                                &actor_name,
                                &actor_faction,
                                &location,
                                ArchiveSubtype::DestroyEntry,
                                Some(entry_id),
                                entry_content.as_deref(),
                                entry_subject.as_deref(),
                                true,
                            );
                            tick_events.push(event);
                        }
                    }
                }
            }
            ArchiveActionType::ForgeEntry => {
                // Create a forged entry
                if let (Some(subject), Some(content)) = (&archive_action.subject, &archive_action.content) {
                    if let Some(archive) = faction_registry.get_archive_mut(&archive_action.faction_id) {
                        let entry_id = archive.generate_id(&archive_action.faction_id);
                        let entry = ArchiveEntry::forged(
                            &entry_id,
                            &actor_id,
                            &actor_name,
                            subject,
                            content,
                            world_state.current_tick,
                        );
                        archive.add_entry(entry);

                        let event = create_archive_event(
                            &mut tick_events,
                            &world_state,
                            &actor_id,
                            &actor_name,
                            &actor_faction,
                            &location,
                            ArchiveSubtype::ForgeEntry,
                            Some(&entry_id),
                            Some(content),
                            Some(subject),
                            false,
                        );
                        tick_events.push(event);
                    }
                }
            }
        }
    }
}

/// Create an archive event
fn create_archive_event(
    tick_events: &mut TickEvents,
    world_state: &WorldState,
    actor_id: &str,
    actor_name: &str,
    actor_faction: &str,
    location: &str,
    subtype: ArchiveSubtype,
    entry_id: Option<&str>,
    content: Option<&str>,
    subject: Option<&str>,
    is_authentic: bool,
) -> Event {
    let event_id = tick_events.generate_id();
    let timestamp = EventTimestamp {
        tick: world_state.current_tick,
        date: world_state.formatted_date(),
    };

    let actor = ActorSnapshot {
        agent_id: actor_id.to_string(),
        name: actor_name.to_string(),
        faction: actor_faction.to_string(),
        role: "archive_accessor".to_string(),
        location: location.to_string(),
    };

    let trigger = match subtype {
        ArchiveSubtype::WriteEntry => "recording_memory",
        ArchiveSubtype::ReadEntry => "reading_history",
        ArchiveSubtype::DestroyEntry => "destroying_record",
        ArchiveSubtype::ForgeEntry => "forging_record",
    };

    let drama_score = match subtype {
        ArchiveSubtype::WriteEntry => 0.2,
        ArchiveSubtype::ReadEntry => 0.1,
        ArchiveSubtype::DestroyEntry => 0.6,
        ArchiveSubtype::ForgeEntry => 0.7,
    };

    let mut drama_tags = Vec::new();
    if !is_authentic {
        drama_tags.push("forgery".to_string());
    }
    if matches!(subtype, ArchiveSubtype::DestroyEntry) {
        drama_tags.push("history_erased".to_string());
    }

    Event {
        event_id,
        timestamp,
        event_type: EventType::Archive,
        subtype: EventSubtype::Archive(subtype),
        actors: EventActors {
            primary: actor,
            secondary: None,
            affected: None,
        },
        context: EventContext {
            trigger: trigger.to_string(),
            preconditions: Vec::new(),
            location_description: Some(format!("at faction archive in {}", location)),
        },
        outcome: EventOutcome::Archive(ArchiveOutcome {
            entry_id: entry_id.map(|s| s.to_string()),
            content: content.map(|s| s.to_string()),
            subject: subject.map(|s| s.to_string()),
            is_authentic,
        }),
        drama_tags,
        drama_score,
        connected_events: Vec::new(),
    }
}

/// System to execute resource actions
pub fn execute_resource_actions(
    world_state: Res<WorldState>,
    mut faction_registry: ResMut<FactionRegistry>,
    mut selected_actions: ResMut<SelectedActions>,
    mut tick_events: ResMut<TickEvents>,
    query: Query<(&AgentId, &AgentName, &Position, &FactionMembership)>,
) {
    // Collect resource actions
    let mut resource_actions: Vec<(String, ResourceAction, String, String, String)> = Vec::new();

    for (agent_id, name, pos, membership) in query.iter() {
        if let Some(action) = selected_actions.actions.get(&agent_id.0) {
            if let Action::Resource(resource_action) = action {
                resource_actions.push((
                    agent_id.0.clone(),
                    resource_action.clone(),
                    name.0.clone(),
                    pos.location_id.clone(),
                    membership.faction_id.clone(),
                ));
            }
        }
    }

    for (actor_id, action, actor_name, location, actor_faction) in resource_actions {
        match action.action_type {
            ResourceActionType::Work => {
                // Add resources to faction with seasonal modifier
                // Spring: 0.8x, Summer: 1.2x, Autumn: 1.0x, Winter: 0.4x
                let seasonal_modifier = world_state.current_season.production_modifier();
                let actual_yield = (action.amount as f32 * seasonal_modifier).round() as u32;

                if let Some(faction) = faction_registry.get_mut(&actor_faction) {
                    faction.resources.grain += actual_yield;
                }

                let event = create_resource_event(
                    &mut tick_events,
                    &world_state,
                    &actor_id,
                    &actor_name,
                    &actor_faction,
                    &location,
                    ResourceSubtype::Work,
                    actual_yield,
                    None,
                );
                tick_events.push(event);
            }
            ResourceActionType::Trade => {
                // Simple trade event (actual resource exchange would need more logic)
                let event = create_resource_event(
                    &mut tick_events,
                    &world_state,
                    &actor_id,
                    &actor_name,
                    &actor_faction,
                    &location,
                    ResourceSubtype::Trade,
                    action.amount,
                    action.target_id.as_deref(),
                );
                tick_events.push(event);
            }
            ResourceActionType::Steal => {
                let event = create_resource_event(
                    &mut tick_events,
                    &world_state,
                    &actor_id,
                    &actor_name,
                    &actor_faction,
                    &location,
                    ResourceSubtype::Steal,
                    action.amount,
                    action.target_id.as_deref(),
                );
                tick_events.push(event);
            }
            ResourceActionType::Hoard => {
                let event = create_resource_event(
                    &mut tick_events,
                    &world_state,
                    &actor_id,
                    &actor_name,
                    &actor_faction,
                    &location,
                    ResourceSubtype::Hoard,
                    action.amount,
                    None,
                );
                tick_events.push(event);
            }
        }
    }
}

/// Create a resource event
fn create_resource_event(
    tick_events: &mut TickEvents,
    world_state: &WorldState,
    actor_id: &str,
    actor_name: &str,
    actor_faction: &str,
    location: &str,
    subtype: ResourceSubtype,
    amount: u32,
    target: Option<&str>,
) -> Event {
    let event_id = tick_events.generate_id();
    let timestamp = EventTimestamp {
        tick: world_state.current_tick,
        date: world_state.formatted_date(),
    };

    let actor = ActorSnapshot {
        agent_id: actor_id.to_string(),
        name: actor_name.to_string(),
        faction: actor_faction.to_string(),
        role: "worker".to_string(),
        location: location.to_string(),
    };

    let (trigger, drama_score) = match subtype {
        ResourceSubtype::Work => ("productive_labor", 0.1),
        ResourceSubtype::Trade => ("mutual_exchange", 0.2),
        ResourceSubtype::Steal => ("theft", 0.5),
        ResourceSubtype::Hoard => ("self_interest", 0.3),
        _ => ("resource_action", 0.1),
    };

    Event {
        event_id,
        timestamp,
        event_type: EventType::Resource,
        subtype: EventSubtype::Resource(subtype),
        actors: EventActors {
            primary: actor,
            secondary: None,
            affected: None,
        },
        context: EventContext {
            trigger: trigger.to_string(),
            preconditions: Vec::new(),
            location_description: Some(format!("at {}", location)),
        },
        outcome: EventOutcome::General(GeneralOutcome {
            description: Some(format!("Resource action involving {} units", amount)),
            state_changes: Vec::new(),
        }),
        drama_tags: Vec::new(),
        drama_score,
        connected_events: Vec::new(),
    }
}

/// System to execute social actions
pub fn execute_social_actions(
    world_state: Res<WorldState>,
    mut relationship_graph: ResMut<RelationshipGraph>,
    mut selected_actions: ResMut<SelectedActions>,
    mut tick_events: ResMut<TickEvents>,
    query: Query<(&AgentId, &AgentName, &Position, &FactionMembership)>,
) {
    // Build agent info map
    let agent_info: std::collections::HashMap<String, (&AgentName, &Position, &FactionMembership)> =
        query.iter().map(|(id, name, pos, mem)| (id.0.clone(), (name, pos, mem))).collect();

    // Collect social actions
    let mut social_actions: Vec<(String, SocialAction, String, String, String)> = Vec::new();

    for (agent_id, name, pos, membership) in query.iter() {
        if let Some(action) = selected_actions.actions.get(&agent_id.0) {
            if let Action::Social(social_action) = action {
                social_actions.push((
                    agent_id.0.clone(),
                    social_action.clone(),
                    name.0.clone(),
                    pos.location_id.clone(),
                    membership.faction_id.clone(),
                ));
            }
        }
    }

    for (actor_id, action, actor_name, location, actor_faction) in social_actions {
        let target_info = agent_info.get(&action.target_id);

        match action.action_type {
            SocialActionType::BuildTrust => {
                // Update relationship
                let rel = relationship_graph.ensure_relationship(&actor_id, &action.target_id);
                let old_trust = rel.trust.overall();
                rel.trust.update_reliability(social_weights::BUILD_TRUST_GAIN);
                rel.last_interaction_tick = world_state.current_tick;
                let new_trust = rel.trust.overall();

                let event = create_social_event(
                    &mut tick_events,
                    &world_state,
                    &actor_id,
                    &actor_name,
                    &actor_faction,
                    &location,
                    CooperationSubtype::BuildTrust,
                    &action.target_id,
                    target_info.map(|(n, _, _)| n.0.as_str()),
                    old_trust,
                    new_trust,
                );
                tick_events.push(event);
            }
            SocialActionType::CurryFavor => {
                let rel = relationship_graph.ensure_relationship(&actor_id, &action.target_id);
                let old_trust = rel.trust.overall();
                rel.trust.update_alignment(social_weights::CURRY_FAVOR_GAIN);
                rel.last_interaction_tick = world_state.current_tick;
                let new_trust = rel.trust.overall();

                let event = create_social_event(
                    &mut tick_events,
                    &world_state,
                    &actor_id,
                    &actor_name,
                    &actor_faction,
                    &location,
                    CooperationSubtype::Favor,
                    &action.target_id,
                    target_info.map(|(n, _, _)| n.0.as_str()),
                    old_trust,
                    new_trust,
                );
                tick_events.push(event);
            }
            SocialActionType::Gift => {
                let rel = relationship_graph.ensure_relationship(&actor_id, &action.target_id);
                let old_trust = rel.trust.overall();
                rel.trust.update_reliability(social_weights::GIFT_TRUST_GAIN);
                rel.last_interaction_tick = world_state.current_tick;
                let new_trust = rel.trust.overall();

                let event = create_social_event(
                    &mut tick_events,
                    &world_state,
                    &actor_id,
                    &actor_name,
                    &actor_faction,
                    &location,
                    CooperationSubtype::Gift,
                    &action.target_id,
                    target_info.map(|(n, _, _)| n.0.as_str()),
                    old_trust,
                    new_trust,
                );
                tick_events.push(event);
            }
            SocialActionType::Ostracize => {
                // Damage relationship
                let rel = relationship_graph.ensure_relationship(&actor_id, &action.target_id);
                let old_trust = rel.trust.overall();
                rel.trust.update_alignment(-social_weights::OSTRACIZE_BELONGING_IMPACT);
                let new_trust = rel.trust.overall();

                let event = create_social_event(
                    &mut tick_events,
                    &world_state,
                    &actor_id,
                    &actor_name,
                    &actor_faction,
                    &location,
                    CooperationSubtype::BuildTrust, // No ostracize subtype, using generic
                    &action.target_id,
                    target_info.map(|(n, _, _)| n.0.as_str()),
                    old_trust,
                    new_trust,
                );
                tick_events.push(event);
            }
        }
    }
}

/// Create a social/cooperation event
fn create_social_event(
    tick_events: &mut TickEvents,
    world_state: &WorldState,
    actor_id: &str,
    actor_name: &str,
    actor_faction: &str,
    location: &str,
    subtype: CooperationSubtype,
    target_id: &str,
    target_name: Option<&str>,
    old_trust: f32,
    new_trust: f32,
) -> Event {
    let event_id = tick_events.generate_id();
    let timestamp = EventTimestamp {
        tick: world_state.current_tick,
        date: world_state.formatted_date(),
    };

    let actor = ActorSnapshot {
        agent_id: actor_id.to_string(),
        name: actor_name.to_string(),
        faction: actor_faction.to_string(),
        role: "member".to_string(),
        location: location.to_string(),
    };

    let secondary = Some(ActorSnapshot {
        agent_id: target_id.to_string(),
        name: target_name.unwrap_or("unknown").to_string(),
        faction: "unknown".to_string(),
        role: "member".to_string(),
        location: location.to_string(),
    });

    let (trigger, drama_score) = match subtype {
        CooperationSubtype::BuildTrust => ("building_rapport", 0.15),
        CooperationSubtype::Favor => ("seeking_favor", 0.2),
        CooperationSubtype::Gift => ("generous_gift", 0.25),
        _ => ("social_interaction", 0.15),
    };

    Event {
        event_id,
        timestamp,
        event_type: EventType::Cooperation,
        subtype: EventSubtype::Cooperation(subtype),
        actors: EventActors {
            primary: actor,
            secondary,
            affected: None,
        },
        context: EventContext {
            trigger: trigger.to_string(),
            preconditions: Vec::new(),
            location_description: Some(format!("at {}", location)),
        },
        outcome: EventOutcome::Relationship(RelationshipOutcome {
            relationship_changes: vec![RelationshipChange {
                from: actor_id.to_string(),
                to: target_id.to_string(),
                dimension: "overall".to_string(),
                old_value: old_trust,
                new_value: new_trust,
            }],
            state_changes: Vec::new(),
        }),
        drama_tags: Vec::new(),
        drama_score,
        connected_events: Vec::new(),
    }
}

/// System to execute faction political actions
pub fn execute_faction_actions(
    world_state: Res<WorldState>,
    mut faction_registry: ResMut<FactionRegistry>,
    mut selected_actions: ResMut<SelectedActions>,
    mut tick_events: ResMut<TickEvents>,
    mut query: Query<(&AgentId, &AgentName, &Position, &mut FactionMembership)>,
) {
    // Collect faction actions
    let mut faction_actions: Vec<(String, FactionAction, String, String, String)> = Vec::new();

    for (agent_id, name, pos, membership) in query.iter() {
        if let Some(action) = selected_actions.actions.get(&agent_id.0) {
            if let Action::Faction(faction_action) = action {
                faction_actions.push((
                    agent_id.0.clone(),
                    faction_action.clone(),
                    name.0.clone(),
                    pos.location_id.clone(),
                    membership.faction_id.clone(),
                ));
            }
        }
    }

    for (actor_id, action, actor_name, location, actor_faction) in faction_actions {
        match action.action_type {
            FactionActionType::Defect => {
                // Change faction membership (would need mutable query)
                let event = create_faction_event(
                    &mut tick_events,
                    &world_state,
                    &actor_id,
                    &actor_name,
                    &actor_faction,
                    &location,
                    FactionSubtype::Leave,
                    action.new_faction_id.as_deref(),
                );
                tick_events.push(event);
            }
            FactionActionType::Exile => {
                let event = create_faction_event(
                    &mut tick_events,
                    &world_state,
                    &actor_id,
                    &actor_name,
                    &actor_faction,
                    &location,
                    FactionSubtype::Exile,
                    Some(&action.target_id),
                );
                tick_events.push(event);
            }
            FactionActionType::ChallengeLeader => {
                let event = create_faction_event(
                    &mut tick_events,
                    &world_state,
                    &actor_id,
                    &actor_name,
                    &actor_faction,
                    &location,
                    FactionSubtype::ChallengeLeader,
                    None,
                );
                tick_events.push(event);
            }
            FactionActionType::SupportLeader => {
                let event = create_faction_event(
                    &mut tick_events,
                    &world_state,
                    &actor_id,
                    &actor_name,
                    &actor_faction,
                    &location,
                    FactionSubtype::SupportLeader,
                    None,
                );
                tick_events.push(event);
            }
        }
    }
}

/// Create a faction event
fn create_faction_event(
    tick_events: &mut TickEvents,
    world_state: &WorldState,
    actor_id: &str,
    actor_name: &str,
    actor_faction: &str,
    location: &str,
    subtype: FactionSubtype,
    target: Option<&str>,
) -> Event {
    let event_id = tick_events.generate_id();
    let timestamp = EventTimestamp {
        tick: world_state.current_tick,
        date: world_state.formatted_date(),
    };

    let actor = ActorSnapshot {
        agent_id: actor_id.to_string(),
        name: actor_name.to_string(),
        faction: actor_faction.to_string(),
        role: "member".to_string(),
        location: location.to_string(),
    };

    let (trigger, drama_score, mut drama_tags) = match subtype {
        FactionSubtype::Leave => ("defection", 0.7, vec!["defection".to_string()]),
        FactionSubtype::Exile => ("exile_order", 0.6, vec!["exile".to_string()]),
        FactionSubtype::ChallengeLeader => ("leadership_challenge", 0.8, vec!["succession_crisis".to_string()]),
        FactionSubtype::SupportLeader => ("loyalty_display", 0.3, Vec::new()),
        _ => ("faction_action", 0.3, Vec::new()),
    };

    Event {
        event_id,
        timestamp,
        event_type: EventType::Faction,
        subtype: EventSubtype::Faction(subtype),
        actors: EventActors {
            primary: actor,
            secondary: None,
            affected: None,
        },
        context: EventContext {
            trigger: trigger.to_string(),
            preconditions: Vec::new(),
            location_description: Some(format!("at {}", location)),
        },
        outcome: EventOutcome::General(GeneralOutcome {
            description: target.map(|t| format!("Involving {}", t)),
            state_changes: Vec::new(),
        }),
        drama_tags,
        drama_score,
        connected_events: Vec::new(),
    }
}

/// System to execute conflict actions
pub fn execute_conflict_actions(
    mut rng: ResMut<SimRng>,
    world_state: Res<WorldState>,
    mut relationship_graph: ResMut<RelationshipGraph>,
    mut selected_actions: ResMut<SelectedActions>,
    mut tick_events: ResMut<TickEvents>,
    query: Query<(&AgentId, &AgentName, &Position, &FactionMembership, &Traits)>,
) {
    // Build agent info map
    let agent_info: std::collections::HashMap<String, (&AgentName, &FactionMembership, &Traits)> =
        query.iter().map(|(id, name, _, mem, traits)| (id.0.clone(), (name, mem, traits))).collect();

    // Collect conflict actions
    let mut conflict_actions: Vec<(String, ConflictAction, String, String, String, f32)> = Vec::new();

    for (agent_id, name, pos, membership, traits) in query.iter() {
        if let Some(action) = selected_actions.actions.get(&agent_id.0) {
            if let Action::Conflict(conflict_action) = action {
                conflict_actions.push((
                    agent_id.0.clone(),
                    conflict_action.clone(),
                    name.0.clone(),
                    pos.location_id.clone(),
                    membership.faction_id.clone(),
                    traits.boldness,
                ));
            }
        }
    }

    for (actor_id, action, actor_name, location, actor_faction, actor_boldness) in conflict_actions {
        let target_info = agent_info.get(&action.target_id);

        match action.action_type {
            ConflictActionType::Argue => {
                // Damage relationship
                let rel = relationship_graph.ensure_relationship(&actor_id, &action.target_id);
                rel.trust.update_alignment(-conflict_weights::ARGUE_RELATIONSHIP_DAMAGE);

                // Check for resolution
                let resolved = rng.0.gen::<f32>() < conflict_weights::ARGUE_RESOLUTION_CHANCE;

                let event = create_conflict_event(
                    &mut tick_events,
                    &world_state,
                    &actor_id,
                    &actor_name,
                    &actor_faction,
                    &location,
                    ConflictSubtype::Argument,
                    &action.target_id,
                    target_info.map(|(n, _, _)| n.0.as_str()),
                    resolved,
                    false,
                );
                tick_events.push(event);
            }
            ConflictActionType::Fight => {
                // Heavy relationship damage
                let rel = relationship_graph.ensure_relationship(&actor_id, &action.target_id);
                rel.trust.update_reliability(-conflict_weights::FIGHT_RELATIONSHIP_DAMAGE);
                rel.trust.update_alignment(-conflict_weights::FIGHT_RELATIONSHIP_DAMAGE);

                // Determine winner based on capability/boldness
                let target_capability = target_info.map(|(_, _, t)| t.boldness).unwrap_or(0.5);
                let actor_advantage = actor_boldness - target_capability;
                let win_chance = 0.5 + actor_advantage * conflict_weights::FIGHT_CAPABILITY_MODIFIER;
                let actor_wins = rng.0.gen::<f32>() < win_chance;

                let event = create_conflict_event(
                    &mut tick_events,
                    &world_state,
                    &actor_id,
                    &actor_name,
                    &actor_faction,
                    &location,
                    ConflictSubtype::Fight,
                    &action.target_id,
                    target_info.map(|(n, _, _)| n.0.as_str()),
                    false,
                    actor_wins,
                );
                tick_events.push(event);
            }
            ConflictActionType::Sabotage => {
                // Check if detected
                let detected = rng.0.gen::<f32>() < conflict_weights::SABOTAGE_DETECTION_CHANCE;

                if detected {
                    // Heavy relationship damage if caught
                    let rel = relationship_graph.ensure_relationship(&actor_id, &action.target_id);
                    rel.trust.update_reliability(-conflict_weights::SABOTAGE_RELATIONSHIP_DAMAGE);
                }

                let event = create_conflict_event(
                    &mut tick_events,
                    &world_state,
                    &actor_id,
                    &actor_name,
                    &actor_faction,
                    &location,
                    ConflictSubtype::Raid, // Using Raid as closest to sabotage
                    &action.target_id,
                    target_info.map(|(n, _, _)| n.0.as_str()),
                    !detected,
                    !detected,
                );
                tick_events.push(event);
            }
            ConflictActionType::Assassinate => {
                // Extremely dramatic - would set Alive to false on target
                // For now, just generate event
                let event = create_conflict_event(
                    &mut tick_events,
                    &world_state,
                    &actor_id,
                    &actor_name,
                    &actor_faction,
                    &location,
                    ConflictSubtype::Assassination,
                    &action.target_id,
                    target_info.map(|(n, _, _)| n.0.as_str()),
                    false,
                    true, // Assassination attempt
                );
                tick_events.push(event);
            }
        }
    }
}

/// Create a conflict event
fn create_conflict_event(
    tick_events: &mut TickEvents,
    world_state: &WorldState,
    actor_id: &str,
    actor_name: &str,
    actor_faction: &str,
    location: &str,
    subtype: ConflictSubtype,
    target_id: &str,
    target_name: Option<&str>,
    resolved: bool,
    actor_success: bool,
) -> Event {
    let event_id = tick_events.generate_id();
    let timestamp = EventTimestamp {
        tick: world_state.current_tick,
        date: world_state.formatted_date(),
    };

    let actor = ActorSnapshot {
        agent_id: actor_id.to_string(),
        name: actor_name.to_string(),
        faction: actor_faction.to_string(),
        role: "combatant".to_string(),
        location: location.to_string(),
    };

    let secondary = Some(ActorSnapshot {
        agent_id: target_id.to_string(),
        name: target_name.unwrap_or("unknown").to_string(),
        faction: "unknown".to_string(),
        role: "target".to_string(),
        location: location.to_string(),
    });

    let (trigger, base_drama, mut drama_tags) = match subtype {
        ConflictSubtype::Argument => ("heated_dispute", 0.3, vec!["conflict".to_string()]),
        ConflictSubtype::Fight => ("physical_altercation", 0.6, vec!["violence".to_string()]),
        ConflictSubtype::Raid => ("sabotage_attempt", 0.5, vec!["sabotage".to_string()]),
        ConflictSubtype::Assassination => ("murder_attempt", 0.95, vec!["assassination".to_string(), "death".to_string()]),
        _ => ("conflict", 0.4, vec!["conflict".to_string()]),
    };

    let drama_score = if actor_success { base_drama } else { base_drama * 0.8 };

    Event {
        event_id,
        timestamp,
        event_type: EventType::Conflict,
        subtype: EventSubtype::Conflict(subtype),
        actors: EventActors {
            primary: actor,
            secondary,
            affected: None,
        },
        context: EventContext {
            trigger: trigger.to_string(),
            preconditions: Vec::new(),
            location_description: Some(format!("at {}", location)),
        },
        outcome: EventOutcome::General(GeneralOutcome {
            description: Some(if actor_success {
                "Actor prevailed".to_string()
            } else {
                "Conflict unresolved".to_string()
            }),
            state_changes: Vec::new(),
        }),
        drama_tags,
        drama_score,
        connected_events: Vec::new(),
    }
}

/// System to execute beer actions (brew, drink, share)
pub fn execute_beer_actions(
    world_state: Res<WorldState>,
    mut faction_registry: ResMut<FactionRegistry>,
    mut relationship_graph: ResMut<RelationshipGraph>,
    selected_actions: Res<SelectedActions>,
    mut tick_events: ResMut<TickEvents>,
    mut query: Query<(
        &AgentId,
        &AgentName,
        &Position,
        &FactionMembership,
        &mut Intoxication,
        &mut Needs,
    )>,
) {
    // Collect beer actions to process
    let mut beer_actions: Vec<(String, BeerAction, String, String, String)> = Vec::new();

    for (agent_id, name, pos, membership, _, _) in query.iter() {
        if let Some(action) = selected_actions.actions.get(&agent_id.0) {
            if let Action::Beer(beer_action) = action {
                beer_actions.push((
                    agent_id.0.clone(),
                    beer_action.clone(),
                    name.0.clone(),
                    pos.location_id.clone(),
                    membership.faction_id.clone(),
                ));
            }
        }
    }

    for (actor_id, action, actor_name, location, actor_faction) in beer_actions {
        match action.action_type {
            BeerActionType::Brew => {
                if let Some(faction) = faction_registry.get_mut(&actor_faction) {
                    let grain_cost = beer_weights::GRAIN_PER_BEER * action.amount;
                    if faction.resources.grain >= grain_cost {
                        faction.resources.grain -= grain_cost;
                        faction.resources.beer += action.amount;

                        // Generate brew event (using Resource type for now)
                        let event = create_beer_event(
                            &mut tick_events,
                            &world_state,
                            &actor_id,
                            &actor_name,
                            &actor_faction,
                            &location,
                            "brew",
                            action.amount,
                            None,
                        );
                        tick_events.push(event);
                    }
                }
            }
            BeerActionType::Drink => {
                if let Some(faction) = faction_registry.get_mut(&actor_faction) {
                    if faction.resources.beer > 0 {
                        faction.resources.beer -= 1;

                        // Apply intoxication to agent
                        for (id, _, _, _, mut intox, mut needs) in query.iter_mut() {
                            if id.0 == actor_id {
                                intox.apply_drink(world_state.current_tick);

                                // Boost social belonging slightly
                                if needs.social_belonging == SocialBelonging::Isolated {
                                    needs.social_belonging = SocialBelonging::Peripheral;
                                }
                                break;
                            }
                        }

                        // Generate drink event
                        let event = create_beer_event(
                            &mut tick_events,
                            &world_state,
                            &actor_id,
                            &actor_name,
                            &actor_faction,
                            &location,
                            "drink",
                            1,
                            None,
                        );
                        tick_events.push(event);
                    }
                }
            }
            BeerActionType::Share => {
                if let Some(target_id) = &action.target_id {
                    if let Some(faction) = faction_registry.get_mut(&actor_faction) {
                        if faction.resources.beer > 0 {
                            faction.resources.beer -= 1;

                            // Build trust between sharer and recipient
                            let rel = relationship_graph.ensure_relationship(&actor_id, target_id);
                            rel.trust.update_reliability(beer_weights::SHARE_TRUST_GAIN);
                            rel.last_interaction_tick = world_state.current_tick;

                            // Apply intoxication to target
                            for (id, _, _, _, mut intox, _) in query.iter_mut() {
                                if &id.0 == target_id {
                                    intox.apply_drink(world_state.current_tick);
                                    break;
                                }
                            }

                            // Generate share event
                            let event = create_beer_event(
                                &mut tick_events,
                                &world_state,
                                &actor_id,
                                &actor_name,
                                &actor_faction,
                                &location,
                                "share",
                                1,
                                Some(target_id),
                            );
                            tick_events.push(event);
                        }
                    }
                }
            }
        }
    }
}

/// Create a beer-related event
fn create_beer_event(
    tick_events: &mut TickEvents,
    world_state: &WorldState,
    actor_id: &str,
    actor_name: &str,
    actor_faction: &str,
    location: &str,
    action_type: &str,
    amount: u32,
    target: Option<&str>,
) -> Event {
    let event_id = tick_events.generate_id();
    let timestamp = EventTimestamp {
        tick: world_state.current_tick,
        date: world_state.formatted_date(),
    };

    let actor = ActorSnapshot {
        agent_id: actor_id.to_string(),
        name: actor_name.to_string(),
        faction: actor_faction.to_string(),
        role: "worker".to_string(),
        location: location.to_string(),
    };

    let (trigger, drama_score, description) = match action_type {
        "brew" => ("brewing_beer", 0.1, format!("brewed {} beer", amount)),
        "drink" => ("drinking_beer", 0.15, "enjoyed some beer".to_string()),
        "share" => (
            "sharing_beer",
            0.25,
            format!("shared beer with {}", target.unwrap_or("someone")),
        ),
        _ => ("beer_action", 0.1, "beer activity".to_string()),
    };

    Event {
        event_id,
        timestamp,
        event_type: EventType::Resource, // Using Resource type for beer events
        subtype: EventSubtype::Resource(ResourceSubtype::Acquire), // Could add BeerSubtype later
        actors: EventActors {
            primary: actor,
            secondary: target.map(|t| ActorSnapshot {
                agent_id: t.to_string(),
                name: t.to_string(),
                faction: actor_faction.to_string(),
                role: "worker".to_string(),
                location: location.to_string(),
            }),
            affected: None,
        },
        context: EventContext {
            trigger: trigger.to_string(),
            preconditions: Vec::new(),
            location_description: Some(format!("at {}", location)),
        },
        outcome: EventOutcome::General(GeneralOutcome {
            description: Some(description),
            state_changes: Vec::new(),
        }),
        drama_tags: vec!["social".to_string()],
        drama_score,
        connected_events: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_events() {
        let mut events = TickEvents::new();
        assert!(events.is_empty());

        let id1 = events.generate_id();
        let id2 = events.generate_id();
        assert_ne!(id1, id2);
        assert!(id1.starts_with("evt_"));
    }
}
