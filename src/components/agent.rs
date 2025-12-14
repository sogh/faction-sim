//! Agent Components
//!
//! Components for individual agents: traits, needs, goals.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

/// Marker component identifying an entity as an agent
#[derive(Component, Debug, Clone, Default)]
pub struct Agent;

/// Unique identifier for an agent
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub String);

/// Human-readable name for an agent
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct AgentName(pub String);

/// Agent personality traits - fixed at creation
/// All values are 0.0 to 1.0
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Traits {
    /// Willingness to take risks
    pub boldness: f32,
    /// How much faction loyalty factors into decisions
    pub loyalty_weight: f32,
    /// How slowly negative memories fade
    pub grudge_persistence: f32,
    /// Drive toward higher status/power
    pub ambition: f32,
    /// Tendency toward truth vs. deception
    pub honesty: f32,
    /// Frequency of voluntary interaction
    pub sociability: f32,
    /// Prefers group interaction (1.0) vs 1-on-1 (0.0)
    pub group_preference: f32,
}

impl Default for Traits {
    fn default() -> Self {
        Self {
            boldness: 0.5,
            loyalty_weight: 0.5,
            grudge_persistence: 0.5,
            ambition: 0.5,
            honesty: 0.5,
            sociability: 0.5,
            group_preference: 0.5,
        }
    }
}

/// Food security state - abstracted need
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FoodSecurity {
    #[default]
    Secure,
    Stressed,
    Desperate,
}

/// Social belonging state - abstracted need
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SocialBelonging {
    #[default]
    Integrated,
    Peripheral,
    Isolated,
}

/// Agent needs - abstracted states rather than numeric values
#[derive(Component, Debug, Clone, Serialize, Deserialize, Default)]
pub struct Needs {
    pub food_security: FoodSecurity,
    pub social_belonging: SocialBelonging,
}

/// Types of goals an agent can have
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoalType {
    /// General survival goal
    Survive,
    /// Survive the winter specifically
    SurviveWinter,
    /// Seek revenge against a target
    Revenge,
    /// Rise in status within faction
    RiseInStatus,
    /// Protect a specific agent
    Protect,
    /// Accumulate resources
    AccumulateResources,
    /// Build relationship with target
    BuildRelationship,
    /// Defect to another faction
    Defect,
    /// Support current leadership
    SupportLeader,
    /// Challenge for leadership
    ChallengeLeader,
}

/// A single goal with priority and optional target/expiry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub goal_type: GoalType,
    /// Priority from 0.0 (lowest) to 1.0 (highest)
    pub priority: f32,
    /// Optional target agent for this goal
    pub target: Option<String>,
    /// Optional tick at which this goal expires
    pub expires_at: Option<u64>,
    /// Event that originated this goal (for revenge, etc.)
    pub origin_event: Option<String>,
}

impl Goal {
    pub fn new(goal_type: GoalType, priority: f32) -> Self {
        Self {
            goal_type,
            priority,
            target: None,
            expires_at: None,
            origin_event: None,
        }
    }

    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    pub fn with_expiry(mut self, tick: u64) -> Self {
        self.expires_at = Some(tick);
        self
    }

    pub fn with_origin(mut self, event_id: impl Into<String>) -> Self {
        self.origin_event = Some(event_id.into());
        self
    }
}

/// Collection of active goals for an agent
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct Goals {
    pub goals: Vec<Goal>,
}

impl Goals {
    pub fn new() -> Self {
        Self { goals: Vec::new() }
    }

    pub fn add(&mut self, goal: Goal) {
        self.goals.push(goal);
    }

    pub fn remove_expired(&mut self, current_tick: u64) {
        self.goals
            .retain(|g| g.expires_at.map_or(true, |exp| exp > current_tick));
    }

    pub fn has_goal(&self, goal_type: &GoalType) -> bool {
        self.goals.iter().any(|g| &g.goal_type == goal_type)
    }

    pub fn get_goal(&self, goal_type: &GoalType) -> Option<&Goal> {
        self.goals.iter().find(|g| &g.goal_type == goal_type)
    }

    pub fn highest_priority(&self) -> Option<&Goal> {
        self.goals.iter().max_by(|a, b| {
            a.priority
                .partial_cmp(&b.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}

/// Status level within a faction hierarchy (0-5 scale)
/// Note: Some levels are equivalent (e.g., Newcomer=Laborer, Reader=CouncilMember)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusLevel {
    Exile,
    Newcomer,
    Laborer,
    SkilledWorker,
    Specialist,
    CouncilMember,
    Reader,
    FactionLeader,
}

impl StatusLevel {
    /// Get numeric status value (0-5)
    pub fn value(&self) -> u8 {
        match self {
            StatusLevel::Exile => 0,
            StatusLevel::Newcomer | StatusLevel::Laborer => 1,
            StatusLevel::SkilledWorker => 2,
            StatusLevel::Specialist => 3,
            StatusLevel::CouncilMember | StatusLevel::Reader => 4,
            StatusLevel::FactionLeader => 5,
        }
    }
}

impl PartialOrd for StatusLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StatusLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value().cmp(&other.value())
    }
}

impl Default for StatusLevel {
    fn default() -> Self {
        StatusLevel::Laborer
    }
}

/// Role within a faction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Leader,
    Reader,
    CouncilMember,
    ScoutCaptain,
    Healer,
    Smith,
    SkilledWorker,
    Laborer,
    Newcomer,
}

impl Default for Role {
    fn default() -> Self {
        Role::Laborer
    }
}

impl Role {
    pub fn status_level(&self) -> StatusLevel {
        match self {
            Role::Leader => StatusLevel::FactionLeader,
            Role::Reader => StatusLevel::Reader,
            Role::CouncilMember => StatusLevel::CouncilMember,
            Role::ScoutCaptain | Role::Healer | Role::Smith => StatusLevel::Specialist,
            Role::SkilledWorker => StatusLevel::SkilledWorker,
            Role::Laborer => StatusLevel::Laborer,
            Role::Newcomer => StatusLevel::Newcomer,
        }
    }
}

/// Whether the agent is alive
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct Alive(pub bool);

impl Alive {
    pub fn new() -> Self {
        Self(true)
    }

    pub fn is_alive(&self) -> bool {
        self.0
    }
}

/// Temporary intoxication state from beer consumption
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct Intoxication {
    /// Current intoxication level (0.0 to 1.0)
    pub level: f32,
    /// Tick when last drink was consumed
    pub last_drink_tick: u64,
    /// Boldness modifier while intoxicated (+0.2 at full intoxication)
    pub boldness_modifier: f32,
    /// Honesty modifier while intoxicated (-0.1 at full intoxication)
    pub honesty_modifier: f32,
}

impl Intoxication {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply the effect of drinking beer
    pub fn apply_drink(&mut self, current_tick: u64) {
        // Each drink adds 0.3 to intoxication, max 1.0
        self.level = (self.level + 0.3).min(1.0);
        self.last_drink_tick = current_tick;
        // Modifiers scale with intoxication level
        self.update_modifiers();
    }

    /// Update trait modifiers based on current level
    fn update_modifiers(&mut self) {
        // At full intoxication: +0.2 boldness, -0.1 honesty
        self.boldness_modifier = self.level * 0.2;
        self.honesty_modifier = self.level * -0.1;
    }

    /// Check if agent is currently intoxicated
    pub fn is_intoxicated(&self) -> bool {
        self.level > 0.1
    }

    /// Decay intoxication over time
    /// Full decay takes about 20 ticks (2 days)
    pub fn decay(&mut self, ticks_elapsed: u64) {
        if self.level > 0.0 {
            let decay_amount = 0.05 * ticks_elapsed as f32;
            self.level = (self.level - decay_amount).max(0.0);
            self.update_modifiers();
        }
    }

    /// Get effective boldness with intoxication modifier
    pub fn effective_boldness(&self, base_boldness: f32) -> f32 {
        (base_boldness + self.boldness_modifier).clamp(0.0, 1.0)
    }

    /// Get effective honesty with intoxication modifier
    pub fn effective_honesty(&self, base_honesty: f32) -> f32 {
        (base_honesty + self.honesty_modifier).clamp(0.0, 1.0)
    }
}
