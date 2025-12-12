//! Needs Update System
//!
//! Updates agent needs (food_security, social_belonging) based on world state.

use bevy_ecs::prelude::*;

use crate::components::agent::{AgentId, FoodSecurity, Needs, Role, SocialBelonging};
use crate::components::faction::{FactionMembership, FactionRegistry, RitualSchedule};
use crate::components::social::RelationshipGraph;
use crate::components::world::WorldState;

use super::perception::VisibleAgents;

/// Resource tracking interaction counts for social belonging calculation
#[derive(Resource, Debug, Default)]
pub struct InteractionTracker {
    /// Maps agent_id -> count of recent interactions (decays over time)
    interaction_counts: std::collections::HashMap<String, u32>,
    /// Last tick when decay was applied
    last_decay_tick: u64,
    /// How many ticks between decay applications
    decay_interval: u64,
}

impl InteractionTracker {
    pub fn new() -> Self {
        Self {
            interaction_counts: std::collections::HashMap::new(),
            last_decay_tick: 0,
            decay_interval: 100, // Decay every 100 ticks
        }
    }

    /// Record an interaction for an agent
    pub fn record_interaction(&mut self, agent_id: &str) {
        *self.interaction_counts.entry(agent_id.to_string()).or_default() += 1;
    }

    /// Get interaction count for an agent
    pub fn get_count(&self, agent_id: &str) -> u32 {
        self.interaction_counts.get(agent_id).copied().unwrap_or(0)
    }

    /// Apply decay to all interaction counts
    pub fn apply_decay(&mut self, current_tick: u64) {
        if current_tick >= self.last_decay_tick + self.decay_interval {
            for count in self.interaction_counts.values_mut() {
                *count = (*count).saturating_sub(1);
            }
            // Remove zero entries
            self.interaction_counts.retain(|_, &mut v| v > 0);
            self.last_decay_tick = current_tick;
        }
    }
}

/// Resource tracking ritual attendance for social belonging
#[derive(Resource, Debug, Default)]
pub struct RitualAttendance {
    /// Maps agent_id -> number of rituals attended recently
    attendance: std::collections::HashMap<String, u32>,
    /// Maps agent_id -> number of rituals missed recently
    missed: std::collections::HashMap<String, u32>,
}

impl RitualAttendance {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record ritual attendance
    pub fn record_attended(&mut self, agent_id: &str) {
        *self.attendance.entry(agent_id.to_string()).or_default() += 1;
        // Attending reduces missed count
        if let Some(missed) = self.missed.get_mut(agent_id) {
            *missed = missed.saturating_sub(1);
        }
    }

    /// Record ritual missed
    pub fn record_missed(&mut self, agent_id: &str) {
        *self.missed.entry(agent_id.to_string()).or_default() += 1;
    }

    /// Get attendance score (attended - missed, clamped to reasonable range)
    pub fn get_score(&self, agent_id: &str) -> i32 {
        let attended = self.attendance.get(agent_id).copied().unwrap_or(0) as i32;
        let missed = self.missed.get(agent_id).copied().unwrap_or(0) as i32;
        attended - missed
    }

    /// Apply decay to attendance records (call periodically)
    pub fn decay(&mut self) {
        for count in self.attendance.values_mut() {
            *count = (*count).saturating_sub(1);
        }
        for count in self.missed.values_mut() {
            *count = (*count).saturating_sub(1);
        }
        self.attendance.retain(|_, &mut v| v > 0);
        self.missed.retain(|_, &mut v| v > 0);
    }
}

/// Thresholds for food security state transitions
struct FoodSecurityThresholds {
    /// Resources per member below which stress begins
    stress_threshold: f32,
    /// Resources per member below which desperation begins
    desperate_threshold: f32,
    /// Resources per member above which security is restored
    secure_threshold: f32,
}

impl Default for FoodSecurityThresholds {
    fn default() -> Self {
        Self {
            stress_threshold: 3.0,      // < 3 grain per member = stressed
            desperate_threshold: 1.0,   // < 1 grain per member = desperate
            secure_threshold: 5.0,      // >= 5 grain per member = secure
        }
    }
}

/// Role modifiers for food security
fn food_role_modifier(role: &Role) -> f32 {
    match role {
        // Leaders and specialists have priority access
        Role::Leader => 1.5,
        Role::Reader => 1.3,
        Role::CouncilMember => 1.2,
        Role::ScoutCaptain | Role::Healer | Role::Smith => 1.1,
        Role::SkilledWorker => 1.0,
        Role::Laborer => 0.9,
        Role::Newcomer => 0.8,
    }
}

/// System to update agent food security based on faction resources
pub fn update_food_security(
    faction_registry: Res<FactionRegistry>,
    mut query: Query<(&AgentId, &FactionMembership, &mut Needs)>,
) {
    let thresholds = FoodSecurityThresholds::default();

    for (_agent_id, membership, mut needs) in query.iter_mut() {
        // Get faction resources
        let Some(faction) = faction_registry.get(&membership.faction_id) else {
            continue;
        };

        // Calculate grain per member
        let member_count = faction.member_count.max(1);
        let grain_per_member = faction.resources.grain as f32 / member_count as f32;

        // Apply role modifier
        let effective_grain = grain_per_member * food_role_modifier(&membership.role);

        // State transitions with hysteresis to prevent rapid oscillation
        let new_state = match needs.food_security {
            FoodSecurity::Secure => {
                if effective_grain < thresholds.desperate_threshold {
                    FoodSecurity::Desperate
                } else if effective_grain < thresholds.stress_threshold {
                    FoodSecurity::Stressed
                } else {
                    FoodSecurity::Secure
                }
            }
            FoodSecurity::Stressed => {
                if effective_grain < thresholds.desperate_threshold {
                    FoodSecurity::Desperate
                } else if effective_grain >= thresholds.secure_threshold {
                    FoodSecurity::Secure
                } else {
                    FoodSecurity::Stressed
                }
            }
            FoodSecurity::Desperate => {
                // Harder to recover from desperate
                if effective_grain >= thresholds.secure_threshold * 1.2 {
                    FoodSecurity::Secure
                } else if effective_grain >= thresholds.stress_threshold * 1.1 {
                    FoodSecurity::Stressed
                } else {
                    FoodSecurity::Desperate
                }
            }
        };

        needs.food_security = new_state;
    }
}

/// Thresholds for social belonging state transitions
struct SocialBelongingThresholds {
    /// Trust received score below which agent is peripheral
    peripheral_threshold: f32,
    /// Trust received score below which agent is isolated
    isolated_threshold: f32,
    /// Trust received score above which agent is integrated
    integrated_threshold: f32,
    /// Minimum interactions per 100 ticks to maintain belonging
    interaction_requirement: u32,
}

impl Default for SocialBelongingThresholds {
    fn default() -> Self {
        Self {
            peripheral_threshold: 0.5,
            isolated_threshold: 0.1,
            integrated_threshold: 1.0,
            interaction_requirement: 3,
        }
    }
}

/// System to update agent social belonging based on relationships and interactions
pub fn update_social_belonging(
    world_state: Res<WorldState>,
    relationship_graph: Res<RelationshipGraph>,
    interaction_tracker: Res<InteractionTracker>,
    ritual_attendance: Res<RitualAttendance>,
    visible_agents: Query<&VisibleAgents>,
    mut query: Query<(Entity, &AgentId, &FactionMembership, &mut Needs)>,
    faction_mates: Query<(&AgentId, &FactionMembership)>,
) {
    let thresholds = SocialBelongingThresholds::default();

    for (entity, agent_id, membership, mut needs) in query.iter_mut() {
        // Calculate trust received from faction-mates
        let mut trust_received_sum = 0.0f32;
        let mut faction_mate_count = 0u32;

        for (other_id, other_membership) in faction_mates.iter() {
            if other_id.0 == agent_id.0 {
                continue; // Skip self
            }
            if other_membership.faction_id != membership.faction_id {
                continue; // Skip non-faction-mates
            }

            faction_mate_count += 1;

            // Get relationship from other agent to this agent (trust they have in us)
            if let Some(rel) = relationship_graph.get(&other_id.0, &agent_id.0) {
                trust_received_sum += rel.trust.overall();
            }
        }

        // Calculate average trust received
        let avg_trust_received = if faction_mate_count > 0 {
            trust_received_sum / faction_mate_count as f32
        } else {
            0.0
        };

        // Get interaction count
        let interaction_count = interaction_tracker.get_count(&agent_id.0);

        // Get ritual attendance score
        let ritual_score = ritual_attendance.get_score(&agent_id.0);

        // Check if agent is with others of their faction (visibility bonus)
        let has_visible_faction_mates = if let Ok(visible) = visible_agents.get(entity) {
            faction_mates.iter().any(|(other_id, other_membership)| {
                other_membership.faction_id == membership.faction_id
                    && visible.can_see(&other_id.0)
            })
        } else {
            false
        };

        // Calculate belonging score
        // Base: average trust received from faction-mates
        // Bonus: interactions, ritual attendance, visible faction-mates
        let mut belonging_score = avg_trust_received;

        // Interaction bonus (up to +0.3)
        let interaction_bonus = (interaction_count as f32 / 10.0).min(0.3);
        belonging_score += interaction_bonus;

        // Ritual attendance bonus/penalty
        belonging_score += ritual_score as f32 * 0.1;

        // Visibility bonus for being with faction
        if has_visible_faction_mates {
            belonging_score += 0.1;
        }

        // State transitions with hysteresis
        let new_state = match needs.social_belonging {
            SocialBelonging::Integrated => {
                if belonging_score < thresholds.isolated_threshold {
                    SocialBelonging::Isolated
                } else if belonging_score < thresholds.peripheral_threshold {
                    SocialBelonging::Peripheral
                } else {
                    SocialBelonging::Integrated
                }
            }
            SocialBelonging::Peripheral => {
                if belonging_score < thresholds.isolated_threshold {
                    SocialBelonging::Isolated
                } else if belonging_score >= thresholds.integrated_threshold {
                    SocialBelonging::Integrated
                } else {
                    SocialBelonging::Peripheral
                }
            }
            SocialBelonging::Isolated => {
                // Harder to recover from isolation
                if belonging_score >= thresholds.integrated_threshold * 1.2 {
                    SocialBelonging::Integrated
                } else if belonging_score >= thresholds.peripheral_threshold * 1.1 {
                    SocialBelonging::Peripheral
                } else {
                    SocialBelonging::Isolated
                }
            }
        };

        needs.social_belonging = new_state;
    }
}

/// System to decay interaction counts periodically
pub fn decay_interaction_counts(
    world_state: Res<WorldState>,
    mut interaction_tracker: ResMut<InteractionTracker>,
) {
    interaction_tracker.apply_decay(world_state.current_tick);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::agent::{Alive, Role};
    use crate::components::faction::Faction;

    fn create_test_faction_registry() -> FactionRegistry {
        let mut registry = FactionRegistry::new();
        let mut faction = Faction::new("test_faction", "Test Faction", "hq");
        faction.resources.grain = 500; // Healthy grain supply
        faction.member_count = 50;     // 10 grain per member
        registry.register(faction);
        registry
    }

    #[test]
    fn test_food_security_secure() {
        let mut world = World::new();
        world.insert_resource(create_test_faction_registry());

        // Spawn an agent with initial secure state
        world.spawn((
            AgentId("agent_001".to_string()),
            FactionMembership::new("test_faction", Role::Laborer),
            Needs::default(),
            Alive::new(),
        ));

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(update_food_security);
        schedule.run(&mut world);

        // Check result
        let mut query = world.query::<(&AgentId, &Needs)>();
        for (_id, needs) in query.iter(&world) {
            assert_eq!(needs.food_security, FoodSecurity::Secure);
        }
    }

    #[test]
    fn test_food_security_stressed() {
        let mut world = World::new();

        // Create faction with low resources
        let mut registry = FactionRegistry::new();
        let mut faction = Faction::new("poor_faction", "Poor Faction", "hq");
        faction.resources.grain = 100; // Low grain
        faction.member_count = 50;     // 2 grain per member - stressed
        registry.register(faction);
        world.insert_resource(registry);

        world.spawn((
            AgentId("agent_001".to_string()),
            FactionMembership::new("poor_faction", Role::Laborer),
            Needs::default(),
            Alive::new(),
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_food_security);
        schedule.run(&mut world);

        let mut query = world.query::<(&AgentId, &Needs)>();
        for (_id, needs) in query.iter(&world) {
            assert_eq!(needs.food_security, FoodSecurity::Stressed);
        }
    }

    #[test]
    fn test_food_security_desperate() {
        let mut world = World::new();

        // Create faction with critical resources
        let mut registry = FactionRegistry::new();
        let mut faction = Faction::new("starving_faction", "Starving Faction", "hq");
        faction.resources.grain = 25; // Very low grain
        faction.member_count = 50;    // 0.5 grain per member - desperate
        registry.register(faction);
        world.insert_resource(registry);

        world.spawn((
            AgentId("agent_001".to_string()),
            FactionMembership::new("starving_faction", Role::Laborer),
            Needs::default(),
            Alive::new(),
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_food_security);
        schedule.run(&mut world);

        let mut query = world.query::<(&AgentId, &Needs)>();
        for (_id, needs) in query.iter(&world) {
            assert_eq!(needs.food_security, FoodSecurity::Desperate);
        }
    }

    #[test]
    fn test_role_modifier_affects_security() {
        let mut world = World::new();

        // Create faction with borderline resources
        let mut registry = FactionRegistry::new();
        let mut faction = Faction::new("test_faction", "Test Faction", "hq");
        faction.resources.grain = 125; // 2.5 grain per member - borderline stressed
        faction.member_count = 50;
        registry.register(faction);
        world.insert_resource(registry);

        // Laborer should be stressed (2.5 * 0.9 = 2.25 < 3.0)
        world.spawn((
            AgentId("laborer".to_string()),
            FactionMembership::new("test_faction", Role::Laborer),
            Needs::default(),
            Alive::new(),
        ));

        // Leader should be secure (2.5 * 1.5 = 3.75 > 3.0)
        world.spawn((
            AgentId("leader".to_string()),
            FactionMembership::new("test_faction", Role::Leader),
            Needs::default(),
            Alive::new(),
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_food_security);
        schedule.run(&mut world);

        let mut query = world.query::<(&AgentId, &Needs)>();
        for (id, needs) in query.iter(&world) {
            match id.0.as_str() {
                "laborer" => assert_eq!(needs.food_security, FoodSecurity::Stressed),
                "leader" => assert_eq!(needs.food_security, FoodSecurity::Secure),
                _ => {}
            }
        }
    }

    #[test]
    fn test_interaction_tracker() {
        let mut tracker = InteractionTracker::new();

        tracker.record_interaction("agent_001");
        tracker.record_interaction("agent_001");
        tracker.record_interaction("agent_002");

        assert_eq!(tracker.get_count("agent_001"), 2);
        assert_eq!(tracker.get_count("agent_002"), 1);
        assert_eq!(tracker.get_count("agent_003"), 0);

        // Test decay
        tracker.apply_decay(100);
        assert_eq!(tracker.get_count("agent_001"), 1);
        assert_eq!(tracker.get_count("agent_002"), 0);
    }

    #[test]
    fn test_ritual_attendance() {
        let mut attendance = RitualAttendance::new();

        attendance.record_attended("agent_001");
        attendance.record_attended("agent_001");
        attendance.record_missed("agent_002");
        attendance.record_missed("agent_002");

        assert_eq!(attendance.get_score("agent_001"), 2);
        assert_eq!(attendance.get_score("agent_002"), -2);
        assert_eq!(attendance.get_score("agent_003"), 0);
    }
}
