//! Emergent Medieval Simulation Engine
//!
//! A terrarium-style simulation where hundreds of agents with simple behavioral
//! rules produce emergent, dramatically compelling narratives.

use bevy_ecs::prelude::*;
use clap::Parser;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::fs;
use std::path::Path;

mod components;
mod systems;
mod events;
mod actions;
mod output;
mod setup;

pub use components::*;

/// Command line arguments for the simulation
#[derive(Parser, Debug)]
#[command(name = "emergent_sim")]
#[command(about = "An emergent medieval simulation engine")]
struct Args {
    /// Random seed for reproducibility
    #[arg(long, default_value_t = 42)]
    seed: u64,

    /// Number of ticks to simulate
    #[arg(long, default_value_t = 1000)]
    ticks: u64,

    /// Interval between world snapshots (in ticks)
    #[arg(long, default_value_t = 100)]
    snapshot_interval: u64,

    /// Interval between faction rituals (in ticks)
    #[arg(long, default_value_t = 500)]
    ritual_interval: u64,

    /// Output initial world state as JSON
    #[arg(long)]
    output_initial_state: bool,
}

/// Global simulation state resource
#[derive(Resource)]
pub struct SimulationState {
    pub current_tick: u64,
    pub max_ticks: u64,
    pub snapshot_interval: u64,
}

/// Seeded random number generator resource
#[derive(Resource)]
pub struct SimRng(pub SmallRng);

fn main() {
    let args = Args::parse();

    println!("Emergent Simulation Engine");
    println!("==========================");
    println!("Seed: {}", args.seed);
    println!("Ticks: {}", args.ticks);
    println!("Snapshot interval: {}", args.snapshot_interval);
    println!("Ritual interval: {}", args.ritual_interval);
    println!();

    // Ensure output directories exist
    fs::create_dir_all("output/snapshots").unwrap_or_else(|e| {
        eprintln!("Warning: Could not create output directories: {}", e);
    });

    // Initialize the ECS world
    let mut world = World::new();

    // Insert core resources
    world.insert_resource(SimulationState {
        current_tick: 0,
        max_ticks: args.ticks,
        snapshot_interval: args.snapshot_interval,
    });
    world.insert_resource(SimRng(SmallRng::seed_from_u64(args.seed)));

    // Initialize world map
    println!("Creating world map...");
    let location_registry = setup::create_world_map();
    let location_count = location_registry.location_ids().len();
    world.insert_resource(location_registry);
    println!("  Created {} locations", location_count);

    // Initialize factions
    println!("Creating factions...");
    let faction_registry = setup::create_factions();
    let faction_count = faction_registry.faction_ids().len();
    world.insert_resource(faction_registry);
    println!("  Created {} factions", faction_count);

    // Initialize ritual schedule
    let ritual_schedule = setup::create_ritual_schedule(args.ritual_interval);
    world.insert_resource(ritual_schedule);
    println!("  Scheduled faction rituals");

    // Initialize world state
    world.insert_resource(world::WorldState::new());

    // Initialize social resources
    world.insert_resource(social::RelationshipGraph::new());
    world.insert_resource(social::MemoryBank::new());

    // Output initial state if requested
    if args.output_initial_state {
        output_initial_state(&world);
    }

    // Create the schedule
    let mut schedule = Schedule::default();

    // Add systems to the schedule (placeholder for now)
    // Systems will be added in later implementation phases

    println!();
    println!("Starting simulation...");
    println!();

    // Main simulation loop
    for tick in 0..args.ticks {
        // Update current tick
        world.resource_mut::<SimulationState>().current_tick = tick;
        world.resource_mut::<world::WorldState>().advance_tick();

        // Run all systems
        schedule.run(&mut world);

        // Print progress every 100 ticks
        if tick > 0 && tick % 100 == 0 {
            let world_state = world.resource::<world::WorldState>();
            println!(
                "Tick {} / {} ({})",
                tick,
                args.ticks,
                world_state.formatted_date()
            );
        }
    }

    println!();
    let world_state = world.resource::<world::WorldState>();
    println!(
        "Simulation complete. Ran {} ticks (ending on {}).",
        args.ticks,
        world_state.formatted_date()
    );
}

/// Output the initial world state as JSON files
fn output_initial_state(world: &World) {
    println!();
    println!("Outputting initial world state...");

    // Output locations
    let location_registry = world.resource::<world::LocationRegistry>();
    let locations_json = setup::world_to_json(location_registry);
    let locations_path = Path::new("output/initial_locations.json");
    if let Err(e) = fs::write(locations_path, &locations_json) {
        eprintln!("  Warning: Could not write locations: {}", e);
    } else {
        println!("  Wrote {}", locations_path.display());
    }

    // Output factions
    let faction_registry = world.resource::<faction::FactionRegistry>();
    let factions_json = setup::factions_to_json(faction_registry);
    let factions_path = Path::new("output/initial_factions.json");
    if let Err(e) = fs::write(factions_path, &factions_json) {
        eprintln!("  Warning: Could not write factions: {}", e);
    } else {
        println!("  Wrote {}", factions_path.display());
    }
}
