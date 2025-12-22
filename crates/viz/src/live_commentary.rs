//! Live Commentary Generation
//!
//! Generates real-time commentary from simulation events without requiring
//! the director crate to be running. Watches events.jsonl and creates
//! interesting commentary items.

use bevy::prelude::*;
use std::collections::{HashSet, VecDeque};
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;

use crate::director_state::DirectorState;
use crate::overlay::PlaybackState;
use sim_events::{SimTimestamp, SimDate, Season};

/// Plugin for live commentary generation.
pub struct LiveCommentaryPlugin;

impl Plugin for LiveCommentaryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EventWatcher>()
            .init_resource::<AgentEventHistory>()
            .add_systems(
                Update,
                (
                    watch_events_file,
                    generate_commentary_from_events,
                    track_agent_events,
                ),
            );
    }
}

/// Resource for watching the events file.
#[derive(Resource)]
pub struct EventWatcher {
    /// Path to events.jsonl
    pub path: PathBuf,
    /// Last position in the file
    pub last_position: u64,
    /// Recent events (last 100)
    pub recent_events: VecDeque<SimEvent>,
    /// Last check time
    pub last_check: std::time::Instant,
    /// Generation counter - increments when file is reset
    pub generation: u64,
}

impl Default for EventWatcher {
    fn default() -> Self {
        Self {
            path: PathBuf::from("output/events.jsonl"),
            last_position: 0,
            recent_events: VecDeque::with_capacity(100),
            last_check: std::time::Instant::now(),
            generation: 0,
        }
    }
}

/// Simplified event structure for commentary
#[derive(Debug, Clone)]
pub struct SimEvent {
    pub event_id: String,
    pub tick: u64,
    pub event_type: String,
    pub subtype: String,
    pub primary_agent_id: String,
    pub primary_agent_name: String,
    pub primary_faction: String,
    pub secondary_agent_id: Option<String>,
    pub secondary_agent_name: Option<String>,
    pub location: String,
    pub drama_score: f32,
    pub drama_tags: Vec<String>,
    pub content: Option<String>,
}

/// Resource tracking event history per agent.
#[derive(Resource)]
pub struct AgentEventHistory {
    /// Map of agent_id to their recent events
    pub history: std::collections::HashMap<String, VecDeque<SimEvent>>,
    /// Max events per agent
    pub max_per_agent: usize,
}

impl Default for AgentEventHistory {
    fn default() -> Self {
        Self {
            history: std::collections::HashMap::new(),
            max_per_agent: 20,
        }
    }
}

impl AgentEventHistory {
    pub fn add_event(&mut self, agent_id: &str, event: SimEvent) {
        let events = self.history.entry(agent_id.to_string()).or_insert_with(|| VecDeque::with_capacity(self.max_per_agent));
        events.push_front(event);
        if events.len() > self.max_per_agent {
            events.pop_back();
        }
    }

    pub fn get_history(&self, agent_id: &str) -> Option<&VecDeque<SimEvent>> {
        self.history.get(agent_id)
    }
}

/// System to watch the events file for new events.
fn watch_events_file(
    mut watcher: ResMut<EventWatcher>,
) {
    // Only check every 500ms
    if watcher.last_check.elapsed().as_millis() < 500 {
        return;
    }
    watcher.last_check = std::time::Instant::now();

    if !watcher.path.exists() {
        return;
    }

    let Ok(file) = File::open(&watcher.path) else {
        return;
    };

    let Ok(metadata) = file.metadata() else {
        return;
    };

    let file_size = metadata.len();

    // If file is smaller than last position, it was reset
    if file_size < watcher.last_position {
        watcher.last_position = 0;
        watcher.recent_events.clear();
        watcher.generation += 1;
        tracing::info!("Events file reset detected, generation now {}", watcher.generation);
    }

    // If no new content, skip
    if file_size <= watcher.last_position {
        return;
    }

    // Read new lines
    let mut reader = BufReader::new(file);
    if reader.seek(SeekFrom::Start(watcher.last_position)).is_err() {
        return;
    }

    let mut new_position = watcher.last_position;
    let mut line = String::new();
    let mut events_read = 0;

    while reader.read_line(&mut line).unwrap_or(0) > 0 {
        new_position = reader.stream_position().unwrap_or(new_position);

        if let Some(event) = parse_event_line(&line) {
            watcher.recent_events.push_back(event);
            events_read += 1;
            if watcher.recent_events.len() > 100 {
                watcher.recent_events.pop_front();
            }
        }
        line.clear();
    }

    if events_read > 0 {
        tracing::debug!("Read {} new events from events.jsonl, total in queue: {}",
            events_read, watcher.recent_events.len());
    }

    watcher.last_position = new_position;
}

/// Parse a JSON line into a SimEvent.
fn parse_event_line(line: &str) -> Option<SimEvent> {
    let json: serde_json::Value = serde_json::from_str(line.trim()).ok()?;

    let event_type = json.get("event_type")?.as_str()?.to_string();
    let subtype = json.get("subtype")?.as_str()?.to_string();

    let actors = json.get("actors")?;
    let primary = actors.get("primary")?;

    let primary_agent_id = primary.get("agent_id")?.as_str()?.to_string();
    let primary_agent_name = primary.get("name")?.as_str()?.to_string();
    let primary_faction = primary.get("faction")?.as_str()?.to_string();
    let location = primary.get("location")?.as_str()?.to_string();

    let secondary = actors.get("secondary");
    let secondary_agent_id = secondary.and_then(|s| s.get("agent_id")).and_then(|v| v.as_str()).map(String::from);
    let secondary_agent_name = secondary.and_then(|s| s.get("name")).and_then(|v| v.as_str()).map(String::from);

    let timestamp = json.get("timestamp")?;
    let tick = timestamp.get("tick")?.as_u64()?;

    let drama_score = json.get("drama_score").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;

    let drama_tags: Vec<String> = json.get("drama_tags")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    // Extract content from outcome if available
    let content = json.get("outcome")
        .and_then(|o| o.get("memory_shared"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .map(String::from);

    Some(SimEvent {
        event_id: json.get("event_id")?.as_str()?.to_string(),
        tick,
        event_type,
        subtype,
        primary_agent_id,
        primary_agent_name,
        primary_faction,
        secondary_agent_id,
        secondary_agent_name,
        location,
        drama_score,
        drama_tags,
        content,
    })
}

/// System to generate commentary from high-drama events.
fn generate_commentary_from_events(
    watcher: Res<EventWatcher>,
    playback: Res<PlaybackState>,
    mut director: ResMut<DirectorState>,
    mut last_commentary_tick: Local<u64>,
) {
    if !director.enabled {
        return;
    }

    let current_tick = playback.tick_for_snapshot();

    // Only generate commentary for events near playback position
    for event in watcher.recent_events.iter() {
        // Skip if we've already generated commentary past this tick
        if event.tick <= *last_commentary_tick {
            continue;
        }

        // Only process events within playback range
        if event.tick > current_tick + 10 || event.tick + 50 < current_tick {
            continue;
        }

        // Generate commentary for high-drama events
        if event.drama_score >= 0.4 {
            let commentary = generate_commentary_for_event(event);
            if let Some(item) = commentary {
                director.add_commentary(item);
                *last_commentary_tick = event.tick;
            }
        }
    }
}

/// Generate a commentary item for an event.
fn generate_commentary_for_event(event: &SimEvent) -> Option<director::CommentaryItem> {
    let (content, commentary_type) = match (event.event_type.as_str(), event.subtype.as_str()) {
        ("movement", "travel") => {
            let content = format!("{} travels to a new location", event.primary_agent_name);
            (content, director::CommentaryType::EventCaption)
        }
        ("communication", "share_memory") => {
            if event.drama_tags.contains(&"negative_gossip".to_string()) {
                let content = if let Some(ref shared_content) = event.content {
                    format!("Rumors spread: \"{}\"", shared_content)
                } else {
                    format!("{} spreads gossip", event.primary_agent_name)
                };
                (content, director::CommentaryType::DramaticIrony)
            } else {
                let content = format!("{} shares news with {}",
                    event.primary_agent_name,
                    event.secondary_agent_name.as_deref().unwrap_or("another"));
                (content, director::CommentaryType::EventCaption)
            }
        }
        ("social", "express_loyalty") => {
            let content = format!("{} pledges loyalty to their faction", event.primary_agent_name);
            (content, director::CommentaryType::EventCaption)
        }
        ("social", "express_doubt") => {
            let content = format!("{} expresses doubts about the faction's leadership...", event.primary_agent_name);
            (content, director::CommentaryType::TensionTeaser)
        }
        ("conflict", _) => {
            let content = format!("Conflict erupts involving {}", event.primary_agent_name);
            (content, director::CommentaryType::NarratorVoice)
        }
        ("faction", "defection") => {
            let content = format!("{} abandons their faction!", event.primary_agent_name);
            (content, director::CommentaryType::NarratorVoice)
        }
        ("resource", "work") => {
            // Only mention work if drama score is high
            if event.drama_score >= 0.5 {
                let content = format!("{} toils for the faction", event.primary_agent_name);
                (content, director::CommentaryType::ContextReminder)
            } else {
                return None;
            }
        }
        ("ritual", _) => {
            let content = format!("A ritual gathering begins at {}", format_location(&event.location));
            (content, director::CommentaryType::NarratorVoice)
        }
        _ => return None,
    };

    // Create a basic timestamp from the tick (approximate date)
    let ticks_per_day = 10u64;
    let days_per_season = 30u32;
    let seasons_per_year = 4u32;
    let ticks_per_year = (ticks_per_day * days_per_season as u64 * seasons_per_year as u64) as u64;

    let year = (event.tick / ticks_per_year) as u32 + 1;
    let tick_in_year = event.tick % ticks_per_year;
    let ticks_per_season = ticks_per_day * days_per_season as u64;
    let season_num = (tick_in_year / ticks_per_season) as usize;
    let seasons = [Season::Spring, Season::Summer, Season::Autumn, Season::Winter];
    let season = seasons[season_num % 4];
    let tick_in_season = tick_in_year % ticks_per_season;
    let day = ((tick_in_season / ticks_per_day) as u8).clamp(1, 30);

    let timestamp = SimTimestamp::from_date(event.tick, SimDate::new(year, season, day));

    Some(director::CommentaryItem {
        item_id: format!("live_{}", event.event_id),
        timestamp,
        commentary_type,
        content,
        display_duration_ticks: 50,
        priority: event.drama_score,
        related_agents: vec![event.primary_agent_id.clone()],
        related_tension: None,
    })
}

/// Format a location ID for display.
fn format_location(location_id: &str) -> String {
    location_id
        .split('_')
        .map(|word| {
            let mut chars: Vec<char> = word.chars().collect();
            if let Some(first) = chars.first_mut() {
                *first = first.to_ascii_uppercase();
            }
            chars.into_iter().collect::<String>()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Local state for tracking events.
#[derive(Default)]
struct TrackingState {
    seen_events: HashSet<String>,
    last_generation: u64,
}

/// System to track events per agent.
fn track_agent_events(
    watcher: Res<EventWatcher>,
    mut history: ResMut<AgentEventHistory>,
    mut state: Local<TrackingState>,
) {
    // If generation changed, clear seen events and history (new simulation run)
    if state.last_generation != watcher.generation {
        state.seen_events.clear();
        history.history.clear();
        state.last_generation = watcher.generation;
        tracing::info!("Cleared event tracking state for new generation {}", watcher.generation);
    }

    let mut new_events_tracked = 0;
    for event in watcher.recent_events.iter() {
        // Skip if we've already tracked this event
        if state.seen_events.contains(&event.event_id) {
            continue;
        }

        // Mark as seen
        state.seen_events.insert(event.event_id.clone());

        // Add to primary agent's history
        history.add_event(&event.primary_agent_id, event.clone());

        // Add to secondary agent's history if present
        if let Some(ref secondary_id) = event.secondary_agent_id {
            history.add_event(secondary_id, event.clone());
        }

        new_events_tracked += 1;
    }

    if new_events_tracked > 0 {
        tracing::debug!("Tracked {} new events, {} agents have history",
            new_events_tracked, history.history.len());
    }

    // Limit the size of seen_events to prevent unbounded growth
    if state.seen_events.len() > 5000 {
        state.seen_events.clear();
    }
}
