//! Emergent Simulation Visualization
//!
//! Run with: cargo run -p viz
//!
//! Examples:
//!   cargo run -p viz -- --ticks 2000 --auto-start
//!   cargo run -p viz -- --replay output/

use bevy::prelude::*;
use clap::Parser;
use std::path::PathBuf;
use viz::sim_runner::SimConfig;
use viz::SimVizPlugin;

/// Emergent Simulation Visualization
#[derive(Parser, Debug)]
#[command(name = "viz")]
#[command(about = "Visualization for the emergent medieval simulation")]
struct Args {
    /// Number of ticks to simulate
    #[arg(long, default_value_t = 2000)]
    ticks: u64,

    /// Interval between snapshots
    #[arg(long, default_value_t = 50)]
    snapshot_interval: u64,

    /// Random seed for simulation
    #[arg(long, default_value_t = 42)]
    seed: u64,

    /// Auto-start simulation on launch
    #[arg(long)]
    auto_start: bool,

    /// Path to existing output to replay (skip running simulation)
    #[arg(long)]
    replay: Option<PathBuf>,

    /// Output directory for simulation files
    #[arg(long, default_value = "output")]
    output_dir: PathBuf,

    /// Maximum ticks ahead of playback the simulation can run
    #[arg(long, default_value_t = 300)]
    max_ticks_ahead: u64,
}

fn main() {
    let args = Args::parse();

    // Create SimConfig from CLI args
    let sim_config = SimConfig {
        ticks: args.ticks,
        snapshot_interval: args.snapshot_interval,
        seed: args.seed,
        auto_start: args.auto_start,
        output_dir: args.output_dir,
        from_snapshot: None,
        start_tick: None,
        max_ticks_ahead: args.max_ticks_ahead,
    };

    App::new()
        .insert_resource(sim_config)
        .add_plugins(SimVizPlugin)
        .run();
}
