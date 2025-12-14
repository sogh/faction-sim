//! Event Logger
//!
//! Append-only JSONL event logging.

use bevy_ecs::prelude::*;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;

use super::types::Event;

/// Resource for logging events to a JSONL file
#[derive(Resource)]
pub struct EventLogger {
    writer: Option<BufWriter<File>>,
    event_count: u64,
    next_event_id: u64,
}

impl EventLogger {
    /// Create a new event logger writing to the specified path
    pub fn new(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;

        Ok(Self {
            writer: Some(BufWriter::new(file)),
            event_count: 0,
            next_event_id: 1,
        })
    }

    /// Create a logger that discards events (for testing)
    pub fn null() -> Self {
        Self {
            writer: None,
            event_count: 0,
            next_event_id: 1,
        }
    }

    /// Generate the next event ID
    pub fn next_id(&mut self) -> String {
        let id = format!("evt_{:08}", self.next_event_id);
        self.next_event_id += 1;
        id
    }

    /// Get the current event count
    pub fn event_count(&self) -> u64 {
        self.event_count
    }

    /// Log an event to the file
    pub fn log(&mut self, event: &Event) -> std::io::Result<()> {
        self.event_count += 1;
        if let Some(ref mut writer) = self.writer {
            let json = serde_json::to_string(event)?;
            writeln!(writer, "{}", json)?;
        }
        Ok(())
    }

    /// Log multiple events
    pub fn log_batch(&mut self, events: &[Event]) -> std::io::Result<()> {
        for event in events {
            self.log(event)?;
        }
        Ok(())
    }

    /// Flush the buffer to disk
    pub fn flush(&mut self) -> std::io::Result<()> {
        if let Some(ref mut writer) = self.writer {
            writer.flush()?;
        }
        Ok(())
    }
}

impl Drop for EventLogger {
    fn drop(&mut self) {
        if let Err(e) = self.flush() {
            eprintln!("Warning: Failed to flush event logger: {}", e);
        }
    }
}

/// Pending events queue for batch processing
#[derive(Resource, Default)]
pub struct PendingEvents {
    events: Vec<Event>,
}

impl PendingEvents {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn drain(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::types::*;
    use std::fs;
    use std::io::BufRead;

    #[test]
    fn test_event_logging() {
        let test_path = "output/test_events.jsonl";

        // Create logger
        let mut logger = EventLogger::new(test_path).unwrap();

        // Create and log a test event
        let actor = ActorSnapshot::new(
            "agent_test_0001",
            "Test Agent",
            "thornwood",
            "laborer",
            "thornwood_village",
        );

        let event = create_movement_event(
            logger.next_id(),
            100,
            "year_1.spring.day_10",
            actor,
            "scheduled_patrol",
            "thornwood_fields",
        );

        logger.log(&event).unwrap();
        logger.flush().unwrap();

        // Verify the file contents
        let file = File::open(test_path).unwrap();
        let reader = std::io::BufReader::new(file);
        let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

        assert_eq!(lines.len(), 1);

        // Parse the logged event
        let parsed: Event = serde_json::from_str(&lines[0]).unwrap();
        assert_eq!(parsed.event_id, "evt_00000001");
        assert_eq!(parsed.actors.primary.agent_id, "agent_test_0001");

        // Cleanup
        fs::remove_file(test_path).ok();
    }

    #[test]
    fn test_null_logger() {
        let mut logger = EventLogger::null();

        let actor = ActorSnapshot::new("agent_1", "Test", "faction", "role", "loc");
        let event = create_movement_event("evt_1", 1, "day_1", actor, "test", "loc2");

        // Should succeed without actually writing
        logger.log(&event).unwrap();
        assert_eq!(logger.event_count(), 1);
    }

    #[test]
    fn test_event_id_generation() {
        let mut logger = EventLogger::null();

        assert_eq!(logger.next_id(), "evt_00000001");
        assert_eq!(logger.next_id(), "evt_00000002");
        assert_eq!(logger.next_id(), "evt_00000003");
    }

    #[test]
    fn test_pending_events() {
        let mut pending = PendingEvents::new();
        assert!(pending.is_empty());

        let actor = ActorSnapshot::new("agent_1", "Test", "faction", "role", "loc");
        let event = create_movement_event("evt_1", 1, "day_1", actor, "test", "loc2");

        pending.push(event);
        assert_eq!(pending.len(), 1);

        let drained = pending.drain();
        assert_eq!(drained.len(), 1);
        assert!(pending.is_empty());
    }
}
