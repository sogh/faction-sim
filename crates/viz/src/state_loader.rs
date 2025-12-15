//! State loading and file watching.
//!
//! Watches simulation output files and triggers updates when they change.
//! Also handles playback control - advancing through snapshots based on
//! PlaybackState settings.

use bevy::prelude::*;
use notify::{Event as NotifyEvent, RecommendedWatcher, RecursiveMode, Watcher};
use sim_events::WorldSnapshot;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::time::Instant;

use crate::overlay::PlaybackState;

/// Plugin for loading simulation state from files.
pub struct StateLoaderPlugin;

impl Plugin for StateLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimulationState>()
            .init_resource::<SnapshotCache>()
            .add_event::<StateUpdatedEvent>()
            .add_systems(
                Update,
                (
                    check_file_updates,
                    scan_available_snapshots,
                    advance_playback,
                    load_snapshot_for_playback,
                    handle_reload_key,
                )
                    .chain(),
            );
    }
}

/// Current simulation state loaded from file.
#[derive(Resource, Default)]
pub struct SimulationState {
    /// The most recently loaded world snapshot.
    pub snapshot: Option<WorldSnapshot>,
    /// When the state was last updated.
    pub last_update: Option<Instant>,
    /// Path to the state file being watched.
    pub file_path: Option<PathBuf>,
    /// Any error from the last load attempt.
    pub last_error: Option<String>,
}

impl SimulationState {
    /// Get the current tick from the snapshot.
    pub fn current_tick(&self) -> u64 {
        self.snapshot
            .as_ref()
            .map(|s| s.timestamp.tick)
            .unwrap_or(0)
    }

    /// Get the current season from the snapshot.
    pub fn current_season(&self) -> &str {
        self.snapshot
            .as_ref()
            .map(|s| s.world.season.as_str())
            .unwrap_or("unknown")
    }

    /// Check if we have loaded state.
    pub fn has_state(&self) -> bool {
        self.snapshot.is_some()
    }
}

/// Event emitted when simulation state is updated.
#[derive(Event)]
pub struct StateUpdatedEvent {
    /// The tick of the new state.
    pub tick: u64,
}

/// File watching state stored in Local (doesn't need Send+Sync).
#[derive(Default)]
struct FileWatcherState {
    /// The watcher instance.
    watcher: Option<RecommendedWatcher>,
    /// Receiver for file change events.
    rx: Option<Receiver<Result<NotifyEvent, notify::Error>>>,
    /// Directory being watched.
    watch_path: Option<PathBuf>,
    /// Whether we've initialized.
    initialized: bool,
}

impl FileWatcherState {
    /// Initialize the file watcher if not already done.
    /// Returns true if initial state was loaded (caller should send event).
    fn ensure_initialized(&mut self, state: &mut SimulationState) -> bool {
        if self.initialized {
            return false;
        }
        self.initialized = true;

        // Parse command line arguments for state path
        let args: Vec<String> = std::env::args().collect();
        let state_path = parse_state_path_from_args(&args)
            .unwrap_or_else(|| PathBuf::from("output/current_state.json"));

        state.file_path = Some(state_path.clone());

        // Determine the watch directory
        let watch_dir = state_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));

        // Create channel for file events
        let (tx, rx) = channel();

        // Create the watcher
        match RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            notify::Config::default(),
        ) {
            Ok(mut watcher) => {
                // Only watch if directory exists
                if watch_dir.exists() {
                    if let Err(e) = watcher.watch(&watch_dir, RecursiveMode::NonRecursive) {
                        tracing::warn!("Failed to watch directory {:?}: {}", watch_dir, e);
                    } else {
                        tracing::info!("Watching directory: {:?}", watch_dir);
                        self.watch_path = Some(watch_dir);
                    }
                } else {
                    tracing::info!(
                        "Watch directory {:?} does not exist yet, will retry on manual reload",
                        watch_dir
                    );
                }

                self.watcher = Some(watcher);
                self.rx = Some(rx);
            }
            Err(e) => {
                tracing::error!("Failed to create file watcher: {}", e);
            }
        }

        // Try to load initial state
        if state_path.exists() {
            load_state_file(&state_path, state)
        } else {
            false
        }
    }
}

/// Parse the --state argument from command line.
fn parse_state_path_from_args(args: &[String]) -> Option<PathBuf> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "--state" {
            return iter.next().map(PathBuf::from);
        }
        if let Some(path) = arg.strip_prefix("--state=") {
            return Some(PathBuf::from(path));
        }
    }
    None
}

/// Check for file updates and reload state if necessary.
fn check_file_updates(
    mut watcher_state: Local<FileWatcherState>,
    mut state: ResMut<SimulationState>,
    mut events: EventWriter<StateUpdatedEvent>,
) {
    // Initialize on first run - send event if initial state was loaded
    if watcher_state.ensure_initialized(&mut state) {
        events.send(StateUpdatedEvent {
            tick: state.current_tick(),
        });
    }

    let Some(ref rx) = watcher_state.rx else {
        return;
    };

    // Non-blocking check for file events
    while let Ok(result) = rx.try_recv() {
        match result {
            Ok(event) => {
                // Check if the event affects our state file
                let state_path = match state.file_path.clone() {
                    Some(p) => p,
                    None => continue,
                };

                let state_file_name = state_path.file_name();
                let is_relevant = event.paths.iter().any(|p| {
                    p.file_name() == state_file_name
                        || p.ends_with("current_state.json")
                        || p.ends_with("camera_script.json")
                        || p.ends_with("commentary.json")
                });

                if is_relevant
                    && matches!(
                        event.kind,
                        notify::EventKind::Modify(_) | notify::EventKind::Create(_)
                    )
                {
                    tracing::debug!("Detected file change: {:?}", event.paths);

                    // Reload the state file
                    if load_state_file(&state_path, &mut state) {
                        events.send(StateUpdatedEvent {
                            tick: state.current_tick(),
                        });
                    }
                }
            }
            Err(e) => {
                tracing::warn!("File watcher error: {}", e);
            }
        }
    }
}

/// Handle R key to force reload.
fn handle_reload_key(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<SimulationState>,
    mut events: EventWriter<StateUpdatedEvent>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        if let Some(ref path) = state.file_path.clone() {
            tracing::info!("Manual reload triggered");
            if load_state_file(path, &mut state) {
                events.send(StateUpdatedEvent {
                    tick: state.current_tick(),
                });
            }
        }
    }
}

/// Load state from a file.
fn load_state_file(path: &PathBuf, state: &mut SimulationState) -> bool {
    match std::fs::read_to_string(path) {
        Ok(contents) => match serde_json::from_str::<WorldSnapshot>(&contents) {
            Ok(snapshot) => {
                let tick = snapshot.timestamp.tick;
                state.snapshot = Some(snapshot);
                state.last_update = Some(Instant::now());
                state.last_error = None;
                tracing::info!("Loaded state from {:?} (tick {})", path, tick);
                true
            }
            Err(e) => {
                let error_msg = format!("Failed to parse state file: {}", e);
                tracing::error!("{}", error_msg);
                state.last_error = Some(error_msg);
                false
            }
        },
        Err(e) => {
            let error_msg = format!("Failed to read state file: {}", e);
            tracing::error!("{}", error_msg);
            state.last_error = Some(error_msg);
            false
        }
    }
}

/// Cache of available snapshot files and their ticks.
#[derive(Resource, Default)]
pub struct SnapshotCache {
    /// Available snapshot ticks (sorted).
    pub available_ticks: Vec<u64>,
    /// Last time we scanned for snapshots.
    last_scan: Option<Instant>,
    /// Currently loaded tick.
    loaded_tick: Option<u64>,
}

impl SnapshotCache {
    /// Find the nearest snapshot tick at or before the target tick.
    pub fn find_snapshot_tick(&self, target_tick: u64) -> Option<u64> {
        // Binary search for the largest tick <= target_tick
        match self.available_ticks.binary_search(&target_tick) {
            Ok(idx) => Some(self.available_ticks[idx]),
            Err(idx) => {
                if idx > 0 {
                    Some(self.available_ticks[idx - 1])
                } else {
                    self.available_ticks.first().copied()
                }
            }
        }
    }
}

/// System to scan for available snapshot files periodically.
fn scan_available_snapshots(
    mut cache: ResMut<SnapshotCache>,
    mut playback: ResMut<PlaybackState>,
) {
    // Only scan every 500ms to avoid excessive disk access
    let should_scan = cache
        .last_scan
        .map(|t| t.elapsed().as_millis() > 500)
        .unwrap_or(true);

    if !should_scan {
        return;
    }

    cache.last_scan = Some(Instant::now());

    let snapshots_dir = PathBuf::from("output/snapshots");
    if !snapshots_dir.exists() {
        return;
    }

    let Ok(entries) = std::fs::read_dir(&snapshots_dir) else {
        return;
    };

    let mut ticks: Vec<u64> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "json") {
                // Parse tick from filename like "snap_000500.json"
                let stem = path.file_stem()?.to_str()?;
                let tick_str = stem.strip_prefix("snap_")?;
                tick_str.parse().ok()
            } else {
                None
            }
        })
        .collect();

    ticks.sort_unstable();

    // Update max available tick
    if let Some(&max_tick) = ticks.last() {
        playback.max_available_tick = max_tick;
    }

    cache.available_ticks = ticks;
}

/// System to advance playback tick based on time and speed.
fn advance_playback(time: Res<Time>, mut playback: ResMut<PlaybackState>) {
    if !playback.playing {
        return;
    }

    if playback.max_available_tick == 0 {
        return;
    }

    // Advance by speed * delta_time (speed is ticks per second)
    // Base rate: 60 ticks per second at 1x speed
    let ticks_per_second = 60.0 * playback.speed as f64;
    let delta_ticks = ticks_per_second * time.delta_seconds_f64();

    playback.current_tick += delta_ticks;

    // Clamp to available range
    if playback.current_tick > playback.max_available_tick as f64 {
        playback.current_tick = playback.max_available_tick as f64;
        // Auto-pause at end
        playback.playing = false;
    }
}

/// System to load the appropriate snapshot for the current playback tick.
fn load_snapshot_for_playback(
    cache: Res<SnapshotCache>,
    playback: Res<PlaybackState>,
    mut state: ResMut<SimulationState>,
    mut snapshot_cache: ResMut<SnapshotCache>,
    mut events: EventWriter<StateUpdatedEvent>,
) {
    let target_tick = playback.tick_for_snapshot();

    // Find the best snapshot for this tick
    let Some(snapshot_tick) = cache.find_snapshot_tick(target_tick) else {
        return;
    };

    // Skip if we already have this snapshot loaded
    if snapshot_cache.loaded_tick == Some(snapshot_tick) {
        return;
    }

    // Load the snapshot
    let path = PathBuf::from(format!("output/snapshots/snap_{:06}.json", snapshot_tick));
    if load_state_file(&path, &mut state) {
        snapshot_cache.loaded_tick = Some(snapshot_tick);
        events.send(StateUpdatedEvent {
            tick: state.current_tick(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_state_path_from_args_with_equals() {
        let args = vec!["viz".to_string(), "--state=./output/test.json".to_string()];
        let path = parse_state_path_from_args(&args);
        assert_eq!(path, Some(PathBuf::from("./output/test.json")));
    }

    #[test]
    fn test_parse_state_path_from_args_with_space() {
        let args = vec![
            "viz".to_string(),
            "--state".to_string(),
            "./output/test.json".to_string(),
        ];
        let path = parse_state_path_from_args(&args);
        assert_eq!(path, Some(PathBuf::from("./output/test.json")));
    }

    #[test]
    fn test_parse_state_path_from_args_missing() {
        let args = vec!["viz".to_string()];
        let path = parse_state_path_from_args(&args);
        assert_eq!(path, None);
    }

    #[test]
    fn test_simulation_state_default() {
        let state = SimulationState::default();
        assert!(!state.has_state());
        assert_eq!(state.current_tick(), 0);
        assert_eq!(state.current_season(), "unknown");
    }
}
