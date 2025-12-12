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

use systems::{
    AgentsByLocation, InteractionTracker, RitualAttendance, SeasonTracker, TrustEventQueue,
    build_location_index, update_perception,
    update_food_security, update_social_belonging, decay_interaction_counts,
    decay_memories, cleanup_memories,
    process_trust_events, decay_grudges,
    PendingActions, SelectedActions, TickEvents,
    generate_movement_actions, generate_patrol_actions, generate_communication_actions,
    apply_trait_weights, add_noise_to_weights, select_actions,
    execute_movement_actions, execute_communication_actions,
};

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

// Re-export SimRng from lib
pub use emergent_sim::SimRng;

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

    // Initialize perception and needs resources
    world.insert_resource(AgentsByLocation::new());
    world.insert_resource(InteractionTracker::new());
    world.insert_resource(RitualAttendance::new());
    world.insert_resource(SeasonTracker::new());

    // Initialize action resources
    world.insert_resource(PendingActions::new());
    world.insert_resource(SelectedActions::new());
    world.insert_resource(TickEvents::new());

    // Initialize trust resources
    world.insert_resource(TrustEventQueue::new());

    // Spawn agents
    println!("Spawning agents...");
    {
        // Take the RNG out to avoid borrow conflicts
        let mut sim_rng = world.remove_resource::<SimRng>().unwrap();
        setup::spawn_all_agents(&mut world, &mut sim_rng.0);
        world.insert_resource(sim_rng);
    }
    let summary = setup::get_spawn_summary(&mut world);
    println!("  Spawned {} agents", summary.total_agents);
    for (faction, count) in &summary.by_faction {
        println!("    {}: {}", faction, count);
    }

    // Initialize snapshot generator
    world.insert_resource(output::SnapshotGenerator::new(args.snapshot_interval));

    // Output initial state if requested
    if args.output_initial_state {
        output_initial_state(&world);
    }

    // Generate initial snapshot
    println!("Generating initial snapshot...");
    let initial_snapshot = output::generate_snapshot(&mut world, "simulation_start");
    if let Err(e) = output::write_snapshot_to_dir(&initial_snapshot) {
        eprintln!("  Warning: Could not write initial snapshot: {}", e);
    }
    if let Err(e) = output::write_current_state(&initial_snapshot) {
        eprintln!("  Warning: Could not write current state: {}", e);
    } else {
        println!("  Wrote initial snapshot (tick 0)");
    }

    // Create the schedule
    let mut schedule = Schedule::default();

    // Add systems to the schedule
    // Perception systems run first to update awareness
    schedule.add_systems((
        build_location_index,
        update_perception,
    ).chain());

    // Needs systems run after perception
    schedule.add_systems((
        update_food_security,
        update_social_belonging,
        decay_interaction_counts,
    ).after(update_perception));

    // Memory systems run after needs (decay is per-season, cleanup is periodic)
    schedule.add_systems((
        decay_memories,
        cleanup_memories,
    ).after(decay_interaction_counts));

    // Action systems run after memory
    // 1. Generate possible actions
    // 2. Apply trait-based weight modifiers
    // 3. Add noise for variety
    // 4. Select action probabilistically
    // 5. Execute selected actions
    schedule.add_systems((
        generate_movement_actions,
        generate_patrol_actions,
        generate_communication_actions,
    ).after(cleanup_memories));

    schedule.add_systems(
        apply_trait_weights
            .after(generate_movement_actions)
            .after(generate_patrol_actions)
            .after(generate_communication_actions)
    );

    schedule.add_systems(
        add_noise_to_weights.after(apply_trait_weights)
    );

    schedule.add_systems(
        select_actions.after(add_noise_to_weights)
    );

    // Execute all actions after selection
    schedule.add_systems((
        execute_movement_actions,
        execute_communication_actions,
    ).after(select_actions));

    // Trust systems run after action execution
    // Process trust events generated by actions, then decay grudges
    schedule.add_systems((
        process_trust_events,
        decay_grudges,
    ).after(execute_communication_actions).after(execute_movement_actions));

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

        // Report events generated this tick (summary every 10 ticks)
        {
            let tick_events = world.resource::<systems::TickEvents>();
            let event_count = tick_events.events.len();
            if event_count > 0 && tick % 10 == 0 {
                let world_state = world.resource::<world::WorldState>();
                let mut move_count = 0;
                let mut comm_count = 0;
                for event in &tick_events.events {
                    match event.event_type {
                        events::types::EventType::Movement => move_count += 1,
                        events::types::EventType::Communication => comm_count += 1,
                        _ => {}
                    }
                }
                println!(
                    "[Tick {:>4}] {} - {} events (moves: {}, comms: {})",
                    tick, world_state.formatted_date(), event_count, move_count, comm_count
                );
            }
        }

        // Clear events after reporting (they'd be logged to file in full implementation)
        world.resource_mut::<systems::TickEvents>().events.clear();

        // Generate periodic snapshots
        let should_snapshot = {
            let generator = world.resource::<output::SnapshotGenerator>();
            tick > 0 && generator.should_snapshot(tick)
        };
        if should_snapshot {
            let snapshot = output::generate_snapshot(&mut world, "periodic");
            if let Err(e) = output::write_snapshot_to_dir(&snapshot) {
                eprintln!("Warning: Could not write snapshot at tick {}: {}", tick, e);
            }
            if let Err(e) = output::write_current_state(&snapshot) {
                eprintln!("Warning: Could not write current state at tick {}: {}", tick, e);
            }
            world.resource_mut::<output::SnapshotGenerator>().mark_snapshot(tick);
        }

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

    // Generate final snapshot
    let final_snapshot = output::generate_snapshot(&mut world, "simulation_end");
    if let Err(e) = output::write_snapshot_to_dir(&final_snapshot) {
        eprintln!("Warning: Could not write final snapshot: {}", e);
    }
    if let Err(e) = output::write_current_state(&final_snapshot) {
        eprintln!("Warning: Could not write final current state: {}", e);
    }

    println!();
    let world_state = world.resource::<world::WorldState>();
    println!(
        "Simulation complete. Ran {} ticks (ending on {}).",
        args.ticks,
        world_state.formatted_date()
    );

    let generator = world.resource::<output::SnapshotGenerator>();
    println!("Generated {} snapshots.", generator.snapshot_count());
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
