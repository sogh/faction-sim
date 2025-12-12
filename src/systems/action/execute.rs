//! Action Execution System
//!
//! Executes selected actions and generates events.

use bevy_ecs::prelude::*;

use crate::actions::movement::{MoveAction, MovementType};
use crate::components::agent::AgentId;
use crate::components::world::{Position, WorldState};
use crate::events::types::{
    ActorSnapshot, Event, EventActors, EventContext, EventOutcome, EventTimestamp, EventType,
    EventSubtype, MovementSubtype, MovementOutcome,
};

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
