//! World Components
//!
//! Components for locations, resources, and seasons.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Component: An agent's current position in the world
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub location_id: String,
}

impl Position {
    pub fn new(location_id: impl Into<String>) -> Self {
        Self {
            location_id: location_id.into(),
        }
    }
}

/// Type of location in the world
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocationType {
    /// A faction's main settlement
    Village,
    /// Agricultural area
    Fields,
    /// Wooded area for lumber/hunting
    Forest,
    /// Strategic crossing point
    Bridge,
    /// Meeting point between territories
    Crossroads,
    /// Faction headquarters building
    Hall,
    /// Trade location
    Market,
    /// Defensive structure
    Watchtower,
    /// Resource extraction
    Mine,
    /// Coastal/fishing area
    Harbor,
}

/// Properties that a location can have
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocationProperty {
    /// Can be used for secret meetings
    HiddenMeetingSpot,
    /// Part of a trade route
    TradeRoute,
    /// Faction headquarters
    FactionHQ,
    /// Neutral territory
    Neutral,
    /// Contested between factions
    Contested,
    /// Produces food
    FoodProduction,
    /// Strategic military value
    Strategic,
    /// Has natural defenses
    Defensible,
}

/// Resources available at a location
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocationResources {
    pub grain_production: u32,
    pub iron_production: u32,
    pub salt_production: u32,
}

impl LocationResources {
    pub fn new(grain: u32, iron: u32, salt: u32) -> Self {
        Self {
            grain_production: grain,
            iron_production: iron,
            salt_production: salt,
        }
    }
}

/// Types of production available at a location
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductionType {
    /// Harvest grain from fields
    Harvest,
    /// Hunt for meat and hides
    Hunt,
    /// Gather wood from forest
    GatherWood,
    /// Gather herbs from forest
    GatherHerbs,
    /// Fetch water
    FetchWater,
    /// Brew beer
    Brew,
    /// Preserve food
    Preserve,
    /// Mine iron or salt
    Mine,
    /// Fish from harbor
    Fish,
}

/// Benefits that a location provides for agent decision-making
///
/// This struct defines what needs can be satisfied at a location and
/// what production activities are available, enabling desire-based
/// action generation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocationBenefits {
    /// Does this location provide shelter (reduces warmth need)?
    pub provides_shelter: bool,
    /// Does this location have access to faction food stores?
    pub has_food_stores: bool,
    /// Does this location have water access?
    pub has_water: bool,
    /// Social hub rating (0.0-1.0) - how good for building belonging
    pub social_hub_rating: f32,
    /// Production types available at this location
    pub production_types: Vec<ProductionType>,
    /// Is this location a faction HQ (ritual attendance possible)?
    pub is_faction_hq: bool,
    /// Safety rating (0.0-1.0) - how defensible
    pub safety_rating: f32,
    /// Rest quality (0.0-1.0) - how well agents can rest here
    pub rest_quality: f32,
}

impl LocationBenefits {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create benefits for a faction hall (HQ)
    pub fn hall() -> Self {
        Self {
            provides_shelter: true,
            has_food_stores: true,
            has_water: true,
            social_hub_rating: 0.8,
            production_types: vec![ProductionType::Brew, ProductionType::Preserve],
            is_faction_hq: true,
            safety_rating: 0.7,
            rest_quality: 0.8,
        }
    }

    /// Create benefits for a village
    pub fn village() -> Self {
        Self {
            provides_shelter: true,
            has_food_stores: true,
            has_water: true,
            social_hub_rating: 0.5,
            production_types: vec![ProductionType::Brew, ProductionType::Preserve],
            is_faction_hq: false,
            safety_rating: 0.5,
            rest_quality: 0.6,
        }
    }

    /// Create benefits for agricultural fields
    pub fn fields() -> Self {
        Self {
            provides_shelter: false,
            has_food_stores: false,
            has_water: false,
            social_hub_rating: 0.2,
            production_types: vec![ProductionType::Harvest],
            is_faction_hq: false,
            safety_rating: 0.2,
            rest_quality: 0.2,
        }
    }

    /// Create benefits for a forest
    pub fn forest() -> Self {
        Self {
            provides_shelter: false,
            has_food_stores: false,
            has_water: false,
            social_hub_rating: 0.1,
            production_types: vec![
                ProductionType::Hunt,
                ProductionType::GatherWood,
                ProductionType::GatherHerbs,
            ],
            is_faction_hq: false,
            safety_rating: 0.3,
            rest_quality: 0.3,
        }
    }

    /// Create benefits for a mine
    pub fn mine() -> Self {
        Self {
            provides_shelter: false,
            has_food_stores: false,
            has_water: false,
            social_hub_rating: 0.1,
            production_types: vec![ProductionType::Mine],
            is_faction_hq: false,
            safety_rating: 0.4,
            rest_quality: 0.1,
        }
    }

    /// Create benefits for a harbor
    pub fn harbor() -> Self {
        Self {
            provides_shelter: false,
            has_food_stores: false,
            has_water: true,
            social_hub_rating: 0.4,
            production_types: vec![ProductionType::Fish, ProductionType::Mine],
            is_faction_hq: false,
            safety_rating: 0.3,
            rest_quality: 0.3,
        }
    }

    /// Create benefits for a market
    pub fn market() -> Self {
        Self {
            provides_shelter: false,
            has_food_stores: false,
            has_water: true,
            social_hub_rating: 0.6,
            production_types: vec![],
            is_faction_hq: false,
            safety_rating: 0.4,
            rest_quality: 0.3,
        }
    }

    /// Create benefits for a crossroads
    pub fn crossroads() -> Self {
        Self {
            provides_shelter: false,
            has_food_stores: false,
            has_water: false,
            social_hub_rating: 0.3,
            production_types: vec![],
            is_faction_hq: false,
            safety_rating: 0.2,
            rest_quality: 0.2,
        }
    }

    /// Create benefits for a watchtower
    pub fn watchtower() -> Self {
        Self {
            provides_shelter: true,
            has_food_stores: false,
            has_water: false,
            social_hub_rating: 0.2,
            production_types: vec![],
            is_faction_hq: false,
            safety_rating: 0.8,
            rest_quality: 0.4,
        }
    }

    /// Create benefits for a bridge
    pub fn bridge() -> Self {
        Self {
            provides_shelter: false,
            has_food_stores: false,
            has_water: true,
            social_hub_rating: 0.2,
            production_types: vec![ProductionType::FetchWater, ProductionType::Fish],
            is_faction_hq: false,
            safety_rating: 0.3,
            rest_quality: 0.2,
        }
    }

    /// Check if this location can satisfy a particular need
    pub fn can_satisfy_need(&self, need: &str) -> bool {
        match need {
            "hunger" => self.has_food_stores,
            "thirst" => self.has_water,
            "warmth" => self.provides_shelter,
            "rest" => self.rest_quality > 0.4,
            "safety" => self.safety_rating > 0.5,
            "belonging" => self.social_hub_rating > 0.3 || self.is_faction_hq,
            _ => false,
        }
    }

    /// Get the satisfaction amount for a need at this location
    pub fn need_satisfaction_amount(&self, need: &str) -> f32 {
        match need {
            "hunger" => if self.has_food_stores { 0.4 } else { 0.0 },
            "thirst" => if self.has_water { 0.5 } else { 0.0 },
            "warmth" => if self.provides_shelter { 0.6 } else { 0.0 },
            "rest" => self.rest_quality * 0.5,
            "safety" => self.safety_rating * 0.4,
            "belonging" => {
                let base = self.social_hub_rating * 0.3;
                if self.is_faction_hq { base + 0.2 } else { base }
            },
            _ => 0.0,
        }
    }
}

/// A location in the world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    /// Unique identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Type of location
    pub location_type: LocationType,
    /// Which faction controls this location (if any)
    pub controlling_faction: Option<String>,
    /// Special properties
    pub properties: Vec<LocationProperty>,
    /// Resources produced here
    pub resources: LocationResources,
    /// Adjacent location IDs (for movement)
    pub adjacent: Vec<String>,
    /// Benefits this location provides for agent decision-making
    pub benefits: LocationBenefits,
}

impl Location {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        location_type: LocationType,
    ) -> Self {
        // Set default benefits based on location type
        let benefits = match location_type {
            LocationType::Hall => LocationBenefits::hall(),
            LocationType::Village => LocationBenefits::village(),
            LocationType::Fields => LocationBenefits::fields(),
            LocationType::Forest => LocationBenefits::forest(),
            LocationType::Mine => LocationBenefits::mine(),
            LocationType::Harbor => LocationBenefits::harbor(),
            LocationType::Market => LocationBenefits::market(),
            LocationType::Crossroads => LocationBenefits::crossroads(),
            LocationType::Watchtower => LocationBenefits::watchtower(),
            LocationType::Bridge => LocationBenefits::bridge(),
        };

        Self {
            id: id.into(),
            name: name.into(),
            location_type,
            controlling_faction: None,
            properties: Vec::new(),
            resources: LocationResources::default(),
            adjacent: Vec::new(),
            benefits,
        }
    }

    pub fn with_faction(mut self, faction_id: impl Into<String>) -> Self {
        self.controlling_faction = Some(faction_id.into());
        self
    }

    pub fn with_properties(mut self, properties: Vec<LocationProperty>) -> Self {
        self.properties = properties;
        self
    }

    pub fn with_resources(mut self, resources: LocationResources) -> Self {
        self.resources = resources;
        self
    }

    pub fn with_adjacent(mut self, adjacent: Vec<String>) -> Self {
        self.adjacent = adjacent;
        self
    }

    pub fn with_benefits(mut self, benefits: LocationBenefits) -> Self {
        self.benefits = benefits;
        self
    }

    pub fn is_neutral(&self) -> bool {
        self.controlling_faction.is_none()
            || self.properties.contains(&LocationProperty::Neutral)
    }

    pub fn is_hq(&self) -> bool {
        self.properties.contains(&LocationProperty::FactionHQ)
    }

    pub fn has_property(&self, property: &LocationProperty) -> bool {
        self.properties.contains(property)
    }

    pub fn is_adjacent_to(&self, other_id: &str) -> bool {
        self.adjacent.contains(&other_id.to_string())
    }
}

/// Resource: Registry of all locations in the world
#[derive(Resource, Debug, Default)]
pub struct LocationRegistry {
    locations: HashMap<String, Location>,
}

impl LocationRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a location
    pub fn register(&mut self, location: Location) {
        self.locations.insert(location.id.clone(), location);
    }

    /// Get a location by ID
    pub fn get(&self, location_id: &str) -> Option<&Location> {
        self.locations.get(location_id)
    }

    /// Get mutable location
    pub fn get_mut(&mut self, location_id: &str) -> Option<&mut Location> {
        self.locations.get_mut(location_id)
    }

    /// Get all locations
    pub fn all_locations(&self) -> impl Iterator<Item = &Location> {
        self.locations.values()
    }

    /// Get locations of a specific type
    pub fn locations_of_type(&self, location_type: &LocationType) -> Vec<&Location> {
        self.locations
            .values()
            .filter(|l| &l.location_type == location_type)
            .collect()
    }

    /// Get locations controlled by a faction
    pub fn locations_controlled_by(&self, faction_id: &str) -> Vec<&Location> {
        self.locations
            .values()
            .filter(|l| l.controlling_faction.as_deref() == Some(faction_id))
            .collect()
    }

    /// Get neutral locations
    pub fn neutral_locations(&self) -> Vec<&Location> {
        self.locations.values().filter(|l| l.is_neutral()).collect()
    }

    /// Get adjacent locations
    pub fn adjacent_to(&self, location_id: &str) -> Vec<&Location> {
        self.get(location_id)
            .map(|loc| {
                loc.adjacent
                    .iter()
                    .filter_map(|id| self.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if two locations are adjacent
    pub fn are_adjacent(&self, loc1: &str, loc2: &str) -> bool {
        self.get(loc1)
            .map(|l| l.is_adjacent_to(loc2))
            .unwrap_or(false)
    }

    /// Get all location IDs
    pub fn location_ids(&self) -> Vec<&String> {
        self.locations.keys().collect()
    }

    /// Get adjacent location IDs
    pub fn get_adjacent(&self, location_id: &str) -> Vec<String> {
        self.get(location_id)
            .map(|loc| loc.adjacent.clone())
            .unwrap_or_default()
    }

    /// Check if a path exists between two locations (simple BFS)
    pub fn path_exists(&self, from: &str, to: &str) -> bool {
        if from == to {
            return true;
        }

        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();

        visited.insert(from.to_string());
        queue.push_back(from.to_string());

        while let Some(current) = queue.pop_front() {
            let adjacent = self.get_adjacent(&current);
            for next in adjacent {
                if next == to {
                    return true;
                }
                if !visited.contains(&next) {
                    visited.insert(next.clone());
                    queue.push_back(next);
                }
            }
        }

        false
    }

    /// Get the next step on the shortest path to a destination
    pub fn next_step_toward(&self, from: &str, to: &str) -> Option<String> {
        if from == to {
            return None;
        }

        // BFS to find shortest path
        let mut visited = std::collections::HashMap::new();
        let mut queue = std::collections::VecDeque::new();

        visited.insert(from.to_string(), None::<String>);
        queue.push_back(from.to_string());

        while let Some(current) = queue.pop_front() {
            if current == to {
                // Backtrack to find first step
                let mut step = to.to_string();
                while let Some(Some(prev)) = visited.get(&step) {
                    if prev == from {
                        return Some(step);
                    }
                    step = prev.clone();
                }
                return Some(step);
            }

            let adjacent = self.get_adjacent(&current);
            for next in adjacent {
                if !visited.contains_key(&next) {
                    visited.insert(next.clone(), Some(current.clone()));
                    queue.push_back(next);
                }
            }
        }

        None
    }
}

/// Season in the simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Season {
    #[default]
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    /// Get the next season
    pub fn next(&self) -> Self {
        match self {
            Season::Spring => Season::Summer,
            Season::Summer => Season::Autumn,
            Season::Autumn => Season::Winter,
            Season::Winter => Season::Spring,
        }
    }

    /// Resource production modifier for this season
    pub fn production_modifier(&self) -> f32 {
        match self {
            Season::Spring => 0.8,
            Season::Summer => 1.2,
            Season::Autumn => 1.0,
            Season::Winter => 0.4,
        }
    }

    /// Travel difficulty modifier
    pub fn travel_modifier(&self) -> f32 {
        match self {
            Season::Spring => 1.0,
            Season::Summer => 1.2,
            Season::Autumn => 0.9,
            Season::Winter => 0.6,
        }
    }

    /// Is this a harsh season?
    pub fn is_harsh(&self) -> bool {
        matches!(self, Season::Winter)
    }
}

/// Formatted date in the simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationDate {
    pub year: u32,
    pub season: Season,
    pub day: u32,
}

impl SimulationDate {
    pub fn new(year: u32, season: Season, day: u32) -> Self {
        Self { year, season, day }
    }

    pub fn format(&self) -> String {
        let season_str = match self.season {
            Season::Spring => "spring",
            Season::Summer => "summer",
            Season::Autumn => "autumn",
            Season::Winter => "winter",
        };
        format!("year_{}.{}.day_{}", self.year, season_str, self.day)
    }
}

impl Default for SimulationDate {
    fn default() -> Self {
        Self {
            year: 1,
            season: Season::Spring,
            day: 1,
        }
    }
}

/// Configuration for time conversion
#[derive(Debug, Clone)]
pub struct TimeConfig {
    /// Ticks per day
    pub ticks_per_day: u64,
    /// Days per season
    pub days_per_season: u32,
}

impl Default for TimeConfig {
    fn default() -> Self {
        Self {
            ticks_per_day: 10,     // 10 ticks = 1 day
            days_per_season: 30,   // 30 days per season
        }
    }
}

/// Resource: Current state of the world
#[derive(Resource, Debug)]
pub struct WorldState {
    /// Current simulation tick
    pub current_tick: u64,
    /// Current season
    pub current_season: Season,
    /// Current date
    pub current_date: SimulationDate,
    /// Time configuration
    pub time_config: TimeConfig,
    /// Active world-level threats
    pub active_threats: Vec<String>,
}

impl Default for WorldState {
    fn default() -> Self {
        Self {
            current_tick: 0,
            current_season: Season::Spring,
            current_date: SimulationDate::default(),
            time_config: TimeConfig::default(),
            active_threats: Vec::new(),
        }
    }
}

impl WorldState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update world state for a new tick
    pub fn advance_tick(&mut self) {
        self.current_tick += 1;
        self.update_date();
    }

    /// Set the tick to a specific value
    pub fn set_tick(&mut self, tick: u64) {
        self.current_tick = tick;
        self.update_date();
    }

    /// Update the date based on current tick
    fn update_date(&mut self) {
        let total_days = self.current_tick / self.time_config.ticks_per_day;
        let days_per_year = self.time_config.days_per_season * 4;

        let year = (total_days / days_per_year as u64) as u32 + 1;
        let day_of_year = (total_days % days_per_year as u64) as u32;
        let season_index = day_of_year / self.time_config.days_per_season;
        let day_of_season = (day_of_year % self.time_config.days_per_season) + 1;

        self.current_season = match season_index {
            0 => Season::Spring,
            1 => Season::Summer,
            2 => Season::Autumn,
            _ => Season::Winter,
        };

        self.current_date = SimulationDate {
            year,
            season: self.current_season,
            day: day_of_season,
        };
    }

    /// Get formatted date string
    pub fn formatted_date(&self) -> String {
        self.current_date.format()
    }

    /// Add a world threat
    pub fn add_threat(&mut self, threat: impl Into<String>) {
        self.active_threats.push(threat.into());
    }

    /// Remove a world threat
    pub fn remove_threat(&mut self, threat: &str) {
        self.active_threats.retain(|t| t != threat);
    }

    /// Check if it's winter
    pub fn is_winter(&self) -> bool {
        self.current_season == Season::Winter
    }
}
