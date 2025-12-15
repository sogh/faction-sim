//! Simulation runner: manages sim-core subprocess lifecycle.
//!
//! This plugin spawns and monitors the simulation process, tracking progress
//! and enabling interventions (restart from snapshot).

use bevy::prelude::*;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::io::{BufRead, BufReader};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::Mutex;

/// Plugin for managing simulation subprocess.
pub struct SimRunnerPlugin;

impl Plugin for SimRunnerPlugin {
    fn build(&self, app: &mut App) {
        // SimConfig should be inserted by main.rs before adding this plugin
        // If not present, initialize with defaults
        if !app.world().contains_resource::<SimConfig>() {
            app.init_resource::<SimConfig>();
        }

        app.init_resource::<SimRunner>()
            .add_event::<SimulationEvent>()
            .add_systems(
                Update,
                (
                    poll_simulation,
                    handle_sim_control_input,
                ),
            );
    }
}

/// Configuration for simulation runs.
#[derive(Resource, Clone)]
pub struct SimConfig {
    /// Number of ticks to simulate.
    pub ticks: u64,
    /// Interval between snapshots.
    pub snapshot_interval: u64,
    /// Random seed.
    pub seed: u64,
    /// Path to snapshot file for resuming (intervention workflow).
    pub from_snapshot: Option<PathBuf>,
    /// Starting tick when resuming.
    pub start_tick: Option<u64>,
    /// Output directory.
    pub output_dir: PathBuf,
    /// Whether to auto-start simulation on launch.
    pub auto_start: bool,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            ticks: 2000,
            snapshot_interval: 50,
            seed: 42,
            from_snapshot: None,
            start_tick: None,
            output_dir: PathBuf::from("output"),
            auto_start: false,
        }
    }
}

/// Current status of the simulation.
#[derive(Debug, Clone, PartialEq)]
pub enum SimStatus {
    /// No simulation running, ready to start.
    Idle,
    /// Simulation starting up.
    Starting,
    /// Simulation actively running.
    Running {
        current_tick: u64,
        max_ticks: u64,
    },
    /// Simulation completed successfully.
    Completed {
        final_tick: u64,
    },
    /// Simulation failed with an error.
    Failed(String),
}

impl Default for SimStatus {
    fn default() -> Self {
        Self::Idle
    }
}

/// Events emitted by the simulation runner.
#[derive(Event, Debug, Clone)]
pub enum SimulationEvent {
    /// Simulation started.
    Started,
    /// Simulation tick progressed.
    TickProgress { tick: u64, max: u64 },
    /// Simulation completed.
    Completed { final_tick: u64 },
    /// Simulation failed.
    Failed { error: String },
    /// Simulation stopped by user.
    Stopped,
}

/// Resource managing the simulation subprocess.
#[derive(Resource)]
pub struct SimRunner {
    /// The running process handle (wrapped for thread safety).
    process: Option<Mutex<Child>>,
    /// Current simulation status.
    pub status: SimStatus,
    /// Receiver for process output lines (wrapped for thread safety).
    output_rx: Option<Mutex<Receiver<String>>>,
    /// Last tick observed from output.
    pub last_tick_seen: u64,
    /// Whether we've sent auto-start.
    pub auto_started: bool,
}

impl Default for SimRunner {
    fn default() -> Self {
        Self {
            process: None,
            status: SimStatus::Idle,
            output_rx: None,
            last_tick_seen: 0,
            auto_started: false,
        }
    }
}

impl SimRunner {
    /// Start a new simulation with the given config.
    pub fn start(&mut self, config: &SimConfig) -> Result<(), String> {
        // Stop any existing simulation first
        self.stop();

        let mut cmd = Command::new("cargo");
        cmd.arg("run")
            .arg("-p")
            .arg("sim-core")
            .arg("--")
            .arg("--ticks")
            .arg(config.ticks.to_string())
            .arg("--snapshot-interval")
            .arg(config.snapshot_interval.to_string())
            .arg("--seed")
            .arg(config.seed.to_string());

        // Add from-snapshot if specified (intervention workflow)
        if let Some(ref snapshot_path) = config.from_snapshot {
            cmd.arg("--from-snapshot")
                .arg(snapshot_path);
            if let Some(start_tick) = config.start_tick {
                cmd.arg("--start-tick")
                    .arg(start_tick.to_string());
            }
        }

        cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped());

        tracing::info!("Starting simulation: {:?}", cmd);

        match cmd.spawn() {
            Ok(mut child) => {
                // Set up output reading in a separate thread
                let (tx, rx) = mpsc::channel();

                if let Some(stdout) = child.stdout.take() {
                    let tx_clone = tx.clone();
                    std::thread::spawn(move || {
                        let reader = BufReader::new(stdout);
                        for line in reader.lines().map_while(Result::ok) {
                            if tx_clone.send(line).is_err() {
                                break;
                            }
                        }
                    });
                }

                if let Some(stderr) = child.stderr.take() {
                    std::thread::spawn(move || {
                        let reader = BufReader::new(stderr);
                        for line in reader.lines().map_while(Result::ok) {
                            if tx.send(format!("[STDERR] {}", line)).is_err() {
                                break;
                            }
                        }
                    });
                }

                self.process = Some(Mutex::new(child));
                self.output_rx = Some(Mutex::new(rx));
                self.status = SimStatus::Starting;
                self.last_tick_seen = config.start_tick.unwrap_or(0);

                tracing::info!("Simulation process started");
                Ok(())
            }
            Err(e) => {
                let error = format!("Failed to spawn simulation: {}", e);
                self.status = SimStatus::Failed(error.clone());
                Err(error)
            }
        }
    }

    /// Stop the running simulation.
    pub fn stop(&mut self) {
        if let Some(process_mutex) = self.process.take() {
            if let Ok(mut process) = process_mutex.lock() {
                let _ = process.kill();
                let _ = process.wait();
                tracing::info!("Simulation process stopped");
            }
        }
        self.output_rx = None;
        self.status = SimStatus::Idle;
        self.last_tick_seen = 0;
    }

    /// Check if simulation is currently running.
    pub fn is_running(&self) -> bool {
        matches!(self.status, SimStatus::Starting | SimStatus::Running { .. })
    }

    /// Poll for process output and status updates.
    fn poll(&mut self) -> Vec<SimulationEvent> {
        let mut events = Vec::new();

        // Check for output lines
        if let Some(ref rx_mutex) = self.output_rx {
            if let Ok(rx) = rx_mutex.lock() {
                loop {
                    match rx.try_recv() {
                        Ok(line) => {
                            // Parse tick progress from output
                            // Expected format: "Tick 100/2000" or similar
                            if let Some(tick_info) = parse_tick_progress(&line) {
                                self.last_tick_seen = tick_info.0;
                                let max_ticks = tick_info.1;

                                if matches!(self.status, SimStatus::Starting) {
                                    events.push(SimulationEvent::Started);
                                }

                                self.status = SimStatus::Running {
                                    current_tick: tick_info.0,
                                    max_ticks,
                                };

                                events.push(SimulationEvent::TickProgress {
                                    tick: tick_info.0,
                                    max: max_ticks,
                                });
                            }

                            // Log output for debugging
                            tracing::debug!("Sim output: {}", line);
                        }
                        Err(TryRecvError::Empty) => break,
                        Err(TryRecvError::Disconnected) => {
                            // Process ended
                            break;
                        }
                    }
                }
            }
        }

        // Check if process has exited
        let mut process_ended = false;
        if let Some(ref process_mutex) = self.process {
            if let Ok(mut process) = process_mutex.lock() {
                match process.try_wait() {
                    Ok(Some(exit_status)) => {
                        if exit_status.success() {
                            self.status = SimStatus::Completed {
                                final_tick: self.last_tick_seen,
                            };
                            events.push(SimulationEvent::Completed {
                                final_tick: self.last_tick_seen,
                            });
                            tracing::info!("Simulation completed at tick {}", self.last_tick_seen);
                        } else {
                            let error = format!("Simulation exited with status: {:?}", exit_status);
                            self.status = SimStatus::Failed(error.clone());
                            events.push(SimulationEvent::Failed { error });
                        }
                        process_ended = true;
                    }
                    Ok(None) => {
                        // Still running
                    }
                    Err(e) => {
                        let error = format!("Failed to check process status: {}", e);
                        self.status = SimStatus::Failed(error.clone());
                        events.push(SimulationEvent::Failed { error });
                        process_ended = true;
                    }
                }
            }
        }

        if process_ended {
            self.process = None;
            self.output_rx = None;
        }

        events
    }

    /// Start an intervention by restarting simulation from a snapshot.
    pub fn intervene(&mut self, snapshot_path: PathBuf, start_tick: u64, config: &SimConfig) -> Result<(), String> {
        let intervention_config = SimConfig {
            from_snapshot: Some(snapshot_path),
            start_tick: Some(start_tick),
            ticks: config.ticks.saturating_sub(start_tick),
            ..config.clone()
        };

        tracing::info!("Starting intervention from tick {}", start_tick);
        self.start(&intervention_config)
    }
}

/// Parse tick progress from simulation output.
fn parse_tick_progress(line: &str) -> Option<(u64, u64)> {
    // Try to match patterns like:
    // "Tick 100/2000"
    // "Processing tick 100 of 2000"
    // "tick=100 max=2000"

    // Pattern 1: "Tick X/Y"
    if let Some(rest) = line.strip_prefix("Tick ") {
        if let Some((current, max)) = rest.split_once('/') {
            if let (Ok(c), Ok(m)) = (current.trim().parse(), max.trim().parse()) {
                return Some((c, m));
            }
        }
    }

    // Pattern 2: Look for tick= pattern
    if line.contains("tick=") {
        let mut current = None;
        let mut max = None;
        for part in line.split_whitespace() {
            if let Some(v) = part.strip_prefix("tick=") {
                current = v.parse().ok();
            }
            if let Some(v) = part.strip_prefix("max=") {
                max = v.parse().ok();
            }
        }
        if let (Some(c), Some(m)) = (current, max) {
            return Some((c, m));
        }
    }

    // Pattern 3: Look for "tick X of Y" or "tick X / Y"
    let lower = line.to_lowercase();
    if lower.contains("tick") {
        // Simple number extraction
        let numbers: Vec<u64> = line
            .split(|c: char| !c.is_ascii_digit())
            .filter_map(|s| s.parse().ok())
            .collect();
        if numbers.len() >= 2 {
            return Some((numbers[0], numbers[1]));
        }
    }

    None
}

/// System to poll simulation status.
fn poll_simulation(
    mut sim_runner: ResMut<SimRunner>,
    mut events: EventWriter<SimulationEvent>,
    config: Res<SimConfig>,
) {
    // Handle auto-start on first run
    if config.auto_start && !sim_runner.auto_started && matches!(sim_runner.status, SimStatus::Idle) {
        sim_runner.auto_started = true;
        if let Err(e) = sim_runner.start(&config) {
            tracing::error!("Auto-start failed: {}", e);
        }
    }

    // Poll for updates
    let sim_events = sim_runner.poll();
    for event in sim_events {
        events.send(event);
    }
}

/// System to handle simulation control input.
fn handle_sim_control_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut sim_runner: ResMut<SimRunner>,
    config: Res<SimConfig>,
) {
    // S key: Start/restart simulation
    if keyboard.just_pressed(KeyCode::KeyS) {
        if sim_runner.is_running() {
            tracing::info!("Stopping simulation...");
            sim_runner.stop();
        } else {
            tracing::info!("Starting simulation...");
            if let Err(e) = sim_runner.start(&config) {
                tracing::error!("Failed to start simulation: {}", e);
            }
        }
    }
}

/// Find the nearest snapshot file at or before the given tick.
pub fn find_snapshot_at_or_before(output_dir: &PathBuf, tick: u64) -> Option<PathBuf> {
    let snapshots_dir = output_dir.join("snapshots");

    if !snapshots_dir.exists() {
        return None;
    }

    let mut best_match: Option<(u64, PathBuf)> = None;

    if let Ok(entries) = std::fs::read_dir(&snapshots_dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "json") {
                // Parse tick from filename like "snap_000500.json"
                if let Some(file_tick) = parse_snapshot_tick(&path) {
                    if file_tick <= tick {
                        if best_match.as_ref().map_or(true, |(best, _)| file_tick > *best) {
                            best_match = Some((file_tick, path));
                        }
                    }
                }
            }
        }
    }

    best_match.map(|(_, path)| path)
}

/// Parse tick number from snapshot filename.
fn parse_snapshot_tick(path: &PathBuf) -> Option<u64> {
    let stem = path.file_stem()?.to_str()?;
    // Expected format: "snap_000500"
    let tick_str = stem.strip_prefix("snap_")?;
    tick_str.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tick_progress() {
        assert_eq!(parse_tick_progress("Tick 100/2000"), Some((100, 2000)));
        assert_eq!(parse_tick_progress("tick=50 max=1000"), Some((50, 1000)));
        assert_eq!(parse_tick_progress("Processing tick 200 of 500"), Some((200, 500)));
    }

    #[test]
    fn test_parse_snapshot_tick() {
        assert_eq!(
            parse_snapshot_tick(&PathBuf::from("output/snapshots/snap_000500.json")),
            Some(500)
        );
        assert_eq!(
            parse_snapshot_tick(&PathBuf::from("snap_001000.json")),
            Some(1000)
        );
    }

    #[test]
    fn test_sim_config_default() {
        let config = SimConfig::default();
        assert_eq!(config.ticks, 2000);
        assert_eq!(config.snapshot_interval, 50);
        assert_eq!(config.seed, 42);
        assert!(!config.auto_start);
    }

    #[test]
    fn test_sim_status_default() {
        let status = SimStatus::default();
        assert!(matches!(status, SimStatus::Idle));
    }
}
