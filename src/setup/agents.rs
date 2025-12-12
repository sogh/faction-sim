//! Agent Spawning
//!
//! Functions to spawn agents with randomized traits, assign roles, and initialize relationships.

use bevy_ecs::prelude::*;
use rand::rngs::SmallRng;
use rand::Rng;

use crate::components::agent::{
    Agent, AgentId, AgentName, Alive, FoodSecurity, Goals, Needs, Role, SocialBelonging, Traits,
};
use crate::components::faction::FactionMembership;
use crate::components::social::{Relationship, RelationshipGraph, Trust};
use crate::components::world::Position;
use crate::systems::perception::VisibleAgents;

/// Name lists for each faction
const THORNWOOD_NAMES: &[&str] = &[
    "Alder", "Ash", "Birch", "Bryn", "Cedar", "Elm", "Fern", "Hazel", "Holly", "Ivy",
    "Laurel", "Linden", "Maple", "Oak", "Olive", "Reed", "Rowan", "Sage", "Thorn", "Willow",
    "Moss", "Glen", "Brook", "Dale", "Heath", "Meadow", "Vale", "Wren", "Finch", "Robin",
    "Sparrow", "Lark", "Hawk", "Raven", "Swift", "Hunter", "Fletcher", "Archer", "Forester", "Woodward",
    "Thicket", "Bramble", "Nettle", "Clover", "Yarrow", "Sorrel", "Basil", "Rue", "Tansy", "Aster",
];

const IRONMERE_NAMES: &[&str] = &[
    "Anvil", "Blade", "Brand", "Crag", "Flint", "Forge", "Hammer", "Iron", "Steel", "Stone",
    "Bolt", "Chain", "Helm", "Pike", "Shield", "Sword", "Axe", "Mace", "Spear", "Ward",
    "Garrett", "Marcus", "Victor", "Conrad", "Roland", "Werner", "Klaus", "Gunther", "Aldric", "Bertram",
    "Cedric", "Dietrich", "Ernst", "Friedrich", "Gustav", "Heinrich", "Ingram", "Johann", "Karl", "Ludwig",
    "Magnus", "Norbert", "Otto", "Rolf", "Sigmund", "Ulrich", "Volker", "Wilhelm", "Gerhard", "Hartmann",
];

const SALTCLIFF_NAMES: &[&str] = &[
    "Brine", "Coral", "Cove", "Drift", "Eddy", "Finn", "Gulf", "Harbor", "Inlet", "Jetty",
    "Kelp", "Marina", "Nautilus", "Oar", "Pearl", "Quay", "Reef", "Shell", "Tide", "Wave",
    "Adriana", "Bianca", "Celeste", "Diana", "Elena", "Flavia", "Giselle", "Helena", "Iris", "Julia",
    "Livia", "Marina", "Nerissa", "Octavia", "Portia", "Serena", "Talia", "Vera", "Cordelia", "Delphine",
    "Anchor", "Captain", "Helmsman", "Mariner", "Navigator", "Sailor", "Skipper", "Boatswain", "Rigger", "Trader",
];

const NORTHERN_HOLD_NAMES: &[&str] = &[
    "Bjorn", "Erik", "Freya", "Gunnar", "Harald", "Ingrid", "Jarl", "Kara", "Leif", "Magnus",
    "Nora", "Olaf", "Ragna", "Sigrid", "Thorin", "Ulf", "Viggo", "Astrid", "Bodil", "Dagny",
    "Eira", "Frey", "Greta", "Hilda", "Ivar", "Jorund", "Knut", "Liv", "Maren", "Njord",
    "Orm", "Rune", "Sven", "Thyra", "Unn", "Valdis", "Ylva", "Ake", "Bera", "Dag",
    "Stone", "Frost", "Winter", "Storm", "Mountain", "Peak", "Ridge", "Cliff", "Boulder", "Crag",
];

/// Configuration for agent spawning
pub struct SpawnConfig {
    pub agents_per_faction: usize,
    pub specialist_count: usize,
    pub skilled_worker_count: usize,
}

impl Default for SpawnConfig {
    fn default() -> Self {
        Self {
            agents_per_faction: 55,
            specialist_count: 3,      // healer, smith, scout_captain
            skilled_worker_count: 8,
        }
    }
}

/// Generate randomized traits for an agent
fn generate_traits(rng: &mut SmallRng) -> Traits {
    // Use normal-ish distribution centered around 0.5
    // We'll use a simple approach: average of two uniform randoms
    let rand_trait = |rng: &mut SmallRng| -> f32 {
        let a: f32 = rng.gen();
        let b: f32 = rng.gen();
        ((a + b) / 2.0).clamp(0.05, 0.95)
    };

    Traits {
        boldness: rand_trait(rng),
        loyalty_weight: rand_trait(rng),
        grudge_persistence: rand_trait(rng),
        ambition: rand_trait(rng),
        honesty: rand_trait(rng),
        sociability: rand_trait(rng),
        group_preference: rand_trait(rng),
    }
}

/// Get the name list for a faction
fn get_name_list(faction_id: &str) -> &'static [&'static str] {
    match faction_id {
        "thornwood" => THORNWOOD_NAMES,
        "ironmere" => IRONMERE_NAMES,
        "saltcliff" => SALTCLIFF_NAMES,
        "northern_hold" => NORTHERN_HOLD_NAMES,
        _ => THORNWOOD_NAMES, // fallback
    }
}

/// Generate a unique agent name for a faction
fn generate_name(faction_id: &str, index: usize, rng: &mut SmallRng) -> String {
    let names = get_name_list(faction_id);
    let name_index = (index + rng.gen_range(0..names.len())) % names.len();
    let base_name = names[name_index];

    // Add a suffix for uniqueness if needed
    let faction_suffix = match faction_id {
        "thornwood" => "of Thornwood",
        "ironmere" => "of Ironmere",
        "saltcliff" => "of Saltcliff",
        "northern_hold" => "of the Hold",
        _ => "",
    };

    format!("{} {}", base_name, faction_suffix)
}

/// Generate agent ID
fn generate_agent_id(faction_id: &str, index: usize) -> String {
    format!("agent_{}_{:04}", faction_id, index)
}

/// Determine role based on index and config
fn determine_role(index: usize, config: &SpawnConfig) -> Role {
    match index {
        0 => Role::Leader,
        1 => Role::Reader,
        2 => Role::ScoutCaptain,
        3 => Role::Healer,
        4 => Role::Smith,
        i if i < 5 + config.skilled_worker_count => Role::SkilledWorker,
        _ => Role::Laborer,
    }
}

/// Spawn all agents for a single faction
pub fn spawn_faction_agents(
    world: &mut World,
    faction_id: &str,
    hq_location: &str,
    config: &SpawnConfig,
    rng: &mut SmallRng,
) -> Vec<Entity> {
    let mut spawned_entities = Vec::with_capacity(config.agents_per_faction);

    for i in 0..config.agents_per_faction {
        let agent_id = generate_agent_id(faction_id, i);
        let name = generate_name(faction_id, i, rng);
        let role = determine_role(i, config);
        let traits = generate_traits(rng);

        let entity = world.spawn((
            Agent,
            AgentId(agent_id),
            AgentName(name),
            traits,
            Needs {
                food_security: FoodSecurity::Secure,
                social_belonging: SocialBelonging::Integrated,
            },
            Goals::new(),
            FactionMembership::new(faction_id, role),
            Position::new(hq_location),
            Alive::new(),
            VisibleAgents::new(),
        )).id();

        spawned_entities.push(entity);
    }

    spawned_entities
}

/// Initialize relationships between agents in a faction
pub fn initialize_faction_relationships(
    relationship_graph: &mut RelationshipGraph,
    faction_agents: &[(String, String, Role)], // (agent_id, name, role)
) {
    for (agent_id, _name, _role) in faction_agents {
        for (other_id, _other_name, other_role) in faction_agents {
            if agent_id == other_id {
                continue;
            }

            // Determine trust level based on target's role
            let trust = match other_role {
                Role::Leader => Trust::new(0.5, 0.5, 0.6),
                Role::Reader => Trust::new(0.5, 0.4, 0.5),
                Role::CouncilMember => Trust::new(0.4, 0.4, 0.4),
                Role::ScoutCaptain | Role::Healer | Role::Smith => Trust::new(0.3, 0.3, 0.5),
                Role::SkilledWorker => Trust::new(0.3, 0.3, 0.3),
                Role::Laborer | Role::Newcomer => Trust::new(0.2, 0.3, 0.2),
            };

            let relationship = Relationship::new(other_id.clone()).with_trust(trust);
            relationship_graph.set(agent_id.clone(), relationship);
        }
    }
}

/// Spawn all agents for all factions and set up relationships
pub fn spawn_all_agents(
    world: &mut World,
    rng: &mut SmallRng,
) {
    let config = SpawnConfig::default();

    // Faction data: (faction_id, hq_location)
    let factions = [
        ("thornwood", "thornwood_hall"),
        ("ironmere", "ironmere_hall"),
        ("saltcliff", "saltcliff_hall"),
        ("northern_hold", "northern_hold"),
    ];

    let mut all_faction_agents: Vec<(String, Vec<(String, String, Role)>)> = Vec::new();

    // Spawn agents for each faction
    for (faction_id, hq_location) in &factions {
        let entities = spawn_faction_agents(world, faction_id, hq_location, &config, rng);

        // Collect agent info for relationship initialization
        let mut faction_agent_info = Vec::new();
        for entity in &entities {
            let agent_id = world.get::<AgentId>(*entity).unwrap().0.clone();
            let agent_name = world.get::<AgentName>(*entity).unwrap().0.clone();
            let role = world.get::<FactionMembership>(*entity).unwrap().role.clone();
            faction_agent_info.push((agent_id, agent_name, role));
        }

        all_faction_agents.push((faction_id.to_string(), faction_agent_info));
    }

    // Initialize relationships
    let mut relationship_graph = world.remove_resource::<RelationshipGraph>()
        .unwrap_or_else(RelationshipGraph::new);

    for (_faction_id, agents) in &all_faction_agents {
        initialize_faction_relationships(&mut relationship_graph, agents);
    }

    world.insert_resource(relationship_graph);

    // Update faction member counts and leader/reader assignments
    {
        let mut faction_registry = world.resource_mut::<crate::components::faction::FactionRegistry>();

        for (faction_id, agents) in &all_faction_agents {
            if let Some(faction) = faction_registry.get_mut(faction_id) {
                faction.member_count = agents.len() as u32;

                // Find and assign leader and reader
                for (agent_id, _name, role) in agents {
                    match role {
                        Role::Leader => faction.leader = Some(agent_id.clone()),
                        Role::Reader => faction.reader = Some(agent_id.clone()),
                        _ => {}
                    }
                }
            }
        }
    }
}

/// Get summary stats for spawned agents
pub fn get_spawn_summary(world: &mut World) -> SpawnSummary {
    let mut total_agents = 0;
    let mut by_faction: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    let mut by_role: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

    let mut query = world.query::<(&AgentId, &FactionMembership)>();

    for (_agent_id, membership) in query.iter(world) {
        total_agents += 1;
        *by_faction.entry(membership.faction_id.clone()).or_insert(0) += 1;
        *by_role.entry(format!("{:?}", membership.role)).or_insert(0) += 1;
    }

    SpawnSummary {
        total_agents,
        by_faction,
        by_role,
    }
}

/// Summary of spawned agents
#[derive(Debug)]
pub struct SpawnSummary {
    pub total_agents: u32,
    pub by_faction: std::collections::HashMap<String, u32>,
    pub by_role: std::collections::HashMap<String, u32>,
}

impl std::fmt::Display for SpawnSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Total agents: {}", self.total_agents)?;
        writeln!(f, "By faction:")?;
        for (faction, count) in &self.by_faction {
            writeln!(f, "  {}: {}", faction, count)?;
        }
        writeln!(f, "By role:")?;
        for (role, count) in &self.by_role {
            writeln!(f, "  {}: {}", role, count)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_trait_generation() {
        let mut rng = SmallRng::seed_from_u64(12345);
        let traits = generate_traits(&mut rng);

        // All traits should be in valid range
        assert!(traits.boldness >= 0.0 && traits.boldness <= 1.0);
        assert!(traits.loyalty_weight >= 0.0 && traits.loyalty_weight <= 1.0);
        assert!(traits.ambition >= 0.0 && traits.ambition <= 1.0);
    }

    #[test]
    fn test_name_generation() {
        let mut rng = SmallRng::seed_from_u64(12345);
        let name = generate_name("thornwood", 0, &mut rng);
        assert!(name.contains("of Thornwood"));
    }

    #[test]
    fn test_role_assignment() {
        let config = SpawnConfig::default();

        assert!(matches!(determine_role(0, &config), Role::Leader));
        assert!(matches!(determine_role(1, &config), Role::Reader));
        assert!(matches!(determine_role(2, &config), Role::ScoutCaptain));
        assert!(matches!(determine_role(20, &config), Role::Laborer));
    }

    #[test]
    fn test_spawn_faction() {
        let mut world = World::new();
        world.insert_resource(RelationshipGraph::new());

        let mut rng = SmallRng::seed_from_u64(12345);
        let config = SpawnConfig {
            agents_per_faction: 10,
            specialist_count: 2,
            skilled_worker_count: 3,
        };

        let entities = spawn_faction_agents(
            &mut world,
            "thornwood",
            "thornwood_hall",
            &config,
            &mut rng,
        );

        assert_eq!(entities.len(), 10);

        // Check first agent is leader
        let first_membership = world.get::<FactionMembership>(entities[0]).unwrap();
        assert!(matches!(first_membership.role, Role::Leader));
    }
}
