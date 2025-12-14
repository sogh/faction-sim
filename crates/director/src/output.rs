//! Director output types and structures.
//!
//! Contains camera instructions, commentary items, and the main DirectorOutput
//! that is consumed by the visualization layer. Also provides file I/O for
//! writing output to JSON files.

use serde::{Deserialize, Serialize};
use sim_events::SimTimestamp;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::threads::NarrativeThread;

/// Camera instruction telling visualization what to show.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraInstruction {
    /// Unique identifier for this instruction
    pub instruction_id: String,
    /// When this instruction takes effect
    pub timestamp: SimTimestamp,
    /// When this instruction expires (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<SimTimestamp>,
    /// Camera behavior mode
    pub camera_mode: CameraMode,
    /// What the camera should focus on
    pub focus: CameraFocus,
    /// Suggested pacing for transitions
    pub pacing: PacingHint,
    /// Debug/logging reason for this instruction
    pub reason: String,
    /// Related tension ID if this instruction is tension-driven
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tension_id: Option<String>,
}

impl CameraInstruction {
    /// Creates a new camera instruction.
    pub fn new(
        instruction_id: impl Into<String>,
        timestamp: SimTimestamp,
        camera_mode: CameraMode,
        focus: CameraFocus,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            instruction_id: instruction_id.into(),
            timestamp,
            valid_until: None,
            camera_mode,
            focus,
            pacing: PacingHint::Normal,
            reason: reason.into(),
            tension_id: None,
        }
    }

    /// Sets the valid_until timestamp.
    pub fn with_valid_until(mut self, valid_until: SimTimestamp) -> Self {
        self.valid_until = Some(valid_until);
        self
    }

    /// Sets the pacing hint.
    pub fn with_pacing(mut self, pacing: PacingHint) -> Self {
        self.pacing = pacing;
        self
    }

    /// Sets the related tension ID.
    pub fn with_tension(mut self, tension_id: impl Into<String>) -> Self {
        self.tension_id = Some(tension_id.into());
        self
    }
}

/// Camera behavior mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum CameraMode {
    /// Follow a single agent
    FollowAgent {
        agent_id: String,
        zoom: ZoomLevel,
    },
    /// Frame a location
    FrameLocation {
        location_id: String,
        zoom: ZoomLevel,
    },
    /// Frame multiple agents (auto-adjusts to fit all)
    FrameMultiple {
        agent_ids: Vec<String>,
        auto_zoom: bool,
    },
    /// Cinematic camera path
    Cinematic {
        path: Vec<CameraWaypoint>,
        duration_ticks: u32,
    },
    /// Wide overview shot
    Overview {
        region: Option<String>,
    },
}

impl CameraMode {
    /// Creates a FollowAgent mode.
    pub fn follow_agent(agent_id: impl Into<String>, zoom: ZoomLevel) -> Self {
        Self::FollowAgent {
            agent_id: agent_id.into(),
            zoom,
        }
    }

    /// Creates a FrameLocation mode.
    pub fn frame_location(location_id: impl Into<String>, zoom: ZoomLevel) -> Self {
        Self::FrameLocation {
            location_id: location_id.into(),
            zoom,
        }
    }

    /// Creates a FrameMultiple mode.
    pub fn frame_multiple(agent_ids: Vec<String>, auto_zoom: bool) -> Self {
        Self::FrameMultiple { agent_ids, auto_zoom }
    }

    /// Creates an Overview mode.
    pub fn overview(region: Option<String>) -> Self {
        Self::Overview { region }
    }
}

/// What the camera should focus on.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum CameraFocus {
    /// Focus on a single agent or location
    Primary {
        id: String,
    },
    /// Focus on a conversation between two agents
    Conversation {
        agent_a: String,
        agent_b: String,
    },
    /// Focus on a group of agents
    Group {
        agent_ids: Vec<String>,
    },
    /// Focus on a specific location
    Location {
        location_id: String,
    },
}

impl CameraFocus {
    /// Creates a Primary focus.
    pub fn primary(id: impl Into<String>) -> Self {
        Self::Primary { id: id.into() }
    }

    /// Creates a Conversation focus.
    pub fn conversation(agent_a: impl Into<String>, agent_b: impl Into<String>) -> Self {
        Self::Conversation {
            agent_a: agent_a.into(),
            agent_b: agent_b.into(),
        }
    }

    /// Creates a Group focus.
    pub fn group(agent_ids: Vec<String>) -> Self {
        Self::Group { agent_ids }
    }

    /// Creates a Location focus.
    pub fn location(location_id: impl Into<String>) -> Self {
        Self::Location {
            location_id: location_id.into(),
        }
    }

    /// Returns all agent IDs involved in this focus.
    pub fn agent_ids(&self) -> Vec<&str> {
        match self {
            CameraFocus::Primary { id } => vec![id.as_str()],
            CameraFocus::Conversation { agent_a, agent_b } => {
                vec![agent_a.as_str(), agent_b.as_str()]
            }
            CameraFocus::Group { agent_ids } => agent_ids.iter().map(|s| s.as_str()).collect(),
            CameraFocus::Location { .. } => vec![],
        }
    }
}

/// Pacing hint for camera transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PacingHint {
    /// Linger, something subtle is happening
    Slow,
    /// Standard pacing
    #[default]
    Normal,
    /// Quick cuts, tension building
    Urgent,
    /// Key moment, hold steady
    Climactic,
}

impl PacingHint {
    /// Returns the suggested hold duration in ticks.
    pub fn suggested_hold_ticks(&self) -> u32 {
        match self {
            PacingHint::Slow => 200,
            PacingHint::Normal => 100,
            PacingHint::Urgent => 50,
            PacingHint::Climactic => 150,
        }
    }
}

/// Camera zoom level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ZoomLevel {
    /// Face details visible
    Extreme,
    /// Single agent + immediate surroundings
    Close,
    /// Small group, single building
    #[default]
    Medium,
    /// Village/camp scale
    Wide,
    /// Multiple locations visible
    Regional,
}

/// A waypoint for cinematic camera paths.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraWaypoint {
    /// Target location or agent ID
    pub target: String,
    /// Zoom level at this waypoint
    pub zoom: ZoomLevel,
    /// Ticks to spend at this waypoint
    pub hold_ticks: u32,
    /// Ease-in/out type for the transition
    #[serde(default)]
    pub easing: CameraEasing,
}

impl CameraWaypoint {
    /// Creates a new camera waypoint.
    pub fn new(target: impl Into<String>, zoom: ZoomLevel, hold_ticks: u32) -> Self {
        Self {
            target: target.into(),
            zoom,
            hold_ticks,
            easing: CameraEasing::default(),
        }
    }

    /// Sets the easing type.
    pub fn with_easing(mut self, easing: CameraEasing) -> Self {
        self.easing = easing;
        self
    }
}

/// Camera easing function for transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CameraEasing {
    /// Linear interpolation
    #[default]
    Linear,
    /// Smooth ease in
    EaseIn,
    /// Smooth ease out
    EaseOut,
    /// Smooth ease in and out
    EaseInOut,
}

/// A commentary item for text overlays.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentaryItem {
    /// Unique identifier
    pub item_id: String,
    /// When to display this commentary
    pub timestamp: SimTimestamp,
    /// How long to show (in ticks)
    pub display_duration_ticks: u32,
    /// Type of commentary
    pub commentary_type: CommentaryType,
    /// The actual text content
    pub content: String,
    /// Priority for queue management (higher = more important)
    pub priority: f32,
    /// Agents mentioned in this commentary
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_agents: Vec<String>,
    /// Related tension if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_tension: Option<String>,
}

impl CommentaryItem {
    /// Creates a new commentary item.
    pub fn new(
        item_id: impl Into<String>,
        timestamp: SimTimestamp,
        commentary_type: CommentaryType,
        content: impl Into<String>,
    ) -> Self {
        Self {
            item_id: item_id.into(),
            timestamp,
            display_duration_ticks: 100,
            commentary_type,
            content: content.into(),
            priority: 0.5,
            related_agents: Vec::new(),
            related_tension: None,
        }
    }

    /// Sets the display duration.
    pub fn with_duration(mut self, ticks: u32) -> Self {
        self.display_duration_ticks = ticks;
        self
    }

    /// Sets the priority.
    pub fn with_priority(mut self, priority: f32) -> Self {
        self.priority = priority;
        self
    }

    /// Sets related agents.
    pub fn with_agents(mut self, agents: Vec<String>) -> Self {
        self.related_agents = agents;
        self
    }

    /// Sets the related tension.
    pub fn with_tension(mut self, tension_id: impl Into<String>) -> Self {
        self.related_tension = Some(tension_id.into());
        self
    }
}

/// Type of commentary overlay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommentaryType {
    /// Simple event caption ("Mira arrives at the eastern bridge")
    EventCaption,
    /// Dramatic irony ("Corin doesn't know...")
    DramaticIrony,
    /// Context reminder ("Three months ago...")
    ContextReminder,
    /// Tension teaser ("Winter stores are running low...")
    TensionTeaser,
    /// LLM-generated narrator voice (Phase 3)
    NarratorVoice,
}

/// A highlight marker for notable moments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightMarker {
    /// Event that was highlighted
    pub event_id: String,
    /// Type of highlight
    pub highlight_type: HighlightType,
    /// Suggested start tick for clip
    pub suggested_clip_start: u64,
    /// Suggested end tick for clip
    pub suggested_clip_end: u64,
    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl HighlightMarker {
    /// Creates a new highlight marker.
    pub fn new(
        event_id: impl Into<String>,
        highlight_type: HighlightType,
        clip_start: u64,
        clip_end: u64,
    ) -> Self {
        Self {
            event_id: event_id.into(),
            highlight_type,
            suggested_clip_start: clip_start,
            suggested_clip_end: clip_end,
            description: None,
        }
    }

    /// Sets the description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// Type of highlight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HighlightType {
    /// A key dramatic moment
    KeyMoment,
    /// A turning point in a storyline
    TurningPoint,
    /// The climax of a tension
    Climax,
    /// Resolution of a conflict
    Resolution,
    /// Foreshadowing of future events
    Foreshadowing,
}

/// Complete output from the Director for a processing tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectorOutput {
    /// Tick when this output was generated
    pub generated_at_tick: u64,
    /// Camera instructions for this tick
    pub camera_script: Vec<CameraInstruction>,
    /// Commentary items to display
    pub commentary_queue: Vec<CommentaryItem>,
    /// Currently active narrative threads
    pub active_threads: Vec<NarrativeThread>,
    /// Highlighted moments for later summarization
    pub highlights: Vec<HighlightMarker>,
}

impl DirectorOutput {
    /// Creates a new empty DirectorOutput.
    pub fn new(tick: u64) -> Self {
        Self {
            generated_at_tick: tick,
            camera_script: Vec::new(),
            commentary_queue: Vec::new(),
            active_threads: Vec::new(),
            highlights: Vec::new(),
        }
    }

    /// Adds a camera instruction.
    pub fn add_camera_instruction(&mut self, instruction: CameraInstruction) {
        self.camera_script.push(instruction);
    }

    /// Adds a commentary item.
    pub fn add_commentary(&mut self, item: CommentaryItem) {
        self.commentary_queue.push(item);
    }

    /// Adds a highlight marker.
    pub fn add_highlight(&mut self, marker: HighlightMarker) {
        self.highlights.push(marker);
    }

    /// Returns true if there are any camera instructions.
    pub fn has_camera_instructions(&self) -> bool {
        !self.camera_script.is_empty()
    }

    /// Returns true if there is any commentary.
    pub fn has_commentary(&self) -> bool {
        !self.commentary_queue.is_empty()
    }

    /// Writes the camera script to a JSON file.
    pub fn write_camera_script(&self, path: &Path) -> Result<(), OutputError> {
        let file = File::create(path).map_err(OutputError::Io)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self.camera_script).map_err(OutputError::Json)?;
        Ok(())
    }

    /// Writes the commentary queue to a JSON file.
    pub fn write_commentary(&self, path: &Path) -> Result<(), OutputError> {
        let file = File::create(path).map_err(OutputError::Io)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self.commentary_queue).map_err(OutputError::Json)?;
        Ok(())
    }

    /// Writes the highlights to a JSON file.
    pub fn write_highlights(&self, path: &Path) -> Result<(), OutputError> {
        let file = File::create(path).map_err(OutputError::Io)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self.highlights).map_err(OutputError::Json)?;
        Ok(())
    }

    /// Writes all output files to a directory.
    ///
    /// Creates the directory if it doesn't exist. Writes:
    /// - `camera_script.json` - Camera instructions
    /// - `commentary.json` - Commentary queue
    /// - `highlights.json` - Highlight markers
    pub fn write_all(&self, output_dir: &Path) -> Result<(), OutputError> {
        fs::create_dir_all(output_dir).map_err(OutputError::Io)?;

        self.write_camera_script(&output_dir.join("camera_script.json"))?;
        self.write_commentary(&output_dir.join("commentary.json"))?;
        self.write_highlights(&output_dir.join("highlights.json"))?;

        Ok(())
    }

    /// Serializes the entire output to JSON.
    pub fn to_json(&self) -> Result<String, OutputError> {
        serde_json::to_string_pretty(self).map_err(OutputError::Json)
    }

    /// Serializes the entire output to compact JSON (single line).
    pub fn to_json_compact(&self) -> Result<String, OutputError> {
        serde_json::to_string(self).map_err(OutputError::Json)
    }
}

/// Generates a camera instruction ID.
pub fn generate_instruction_id(tick: u64, sequence: u32) -> String {
    format!("cam_{}_{:04}", tick, sequence)
}

/// Generates a commentary item ID.
pub fn generate_commentary_id(tick: u64, sequence: u32) -> String {
    format!("com_{}_{:04}", tick, sequence)
}

/// Errors that can occur during output operations.
#[derive(Debug)]
pub enum OutputError {
    /// I/O error (file operations)
    Io(std::io::Error),
    /// JSON serialization error
    Json(serde_json::Error),
}

impl std::fmt::Display for OutputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputError::Io(e) => write!(f, "I/O error: {}", e),
            OutputError::Json(e) => write!(f, "JSON error: {}", e),
        }
    }
}

impl std::error::Error for OutputError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            OutputError::Io(e) => Some(e),
            OutputError::Json(e) => Some(e),
        }
    }
}

impl From<std::io::Error> for OutputError {
    fn from(e: std::io::Error) -> Self {
        OutputError::Io(e)
    }
}

impl From<serde_json::Error> for OutputError {
    fn from(e: serde_json::Error) -> Self {
        OutputError::Json(e)
    }
}

/// Streaming output writer for real-time visualization.
///
/// This writer appends to files, allowing the visualization layer to tail
/// them in real-time as the director processes ticks.
///
/// # Output Files
///
/// - `camera_script.jsonl` - One camera instruction set per line
/// - `commentary.jsonl` - One commentary queue per line
/// - `full_output.jsonl` - Complete DirectorOutput per line
///
/// The `.jsonl` format (JSON Lines) allows efficient appending and tailing.
#[derive(Debug)]
pub struct OutputWriter {
    /// Output directory path
    output_dir: std::path::PathBuf,
    /// Buffered writer for camera script
    camera_writer: BufWriter<File>,
    /// Buffered writer for commentary
    commentary_writer: BufWriter<File>,
    /// Buffered writer for full output
    full_writer: BufWriter<File>,
    /// Number of ticks written
    ticks_written: u64,
}

impl OutputWriter {
    /// Creates a new output writer for the given directory.
    ///
    /// Creates the directory if it doesn't exist and opens files for writing.
    pub fn new(output_dir: &Path) -> Result<Self, OutputError> {
        fs::create_dir_all(output_dir)?;

        let camera_file = File::create(output_dir.join("camera_script.jsonl"))?;
        let commentary_file = File::create(output_dir.join("commentary.jsonl"))?;
        let full_file = File::create(output_dir.join("full_output.jsonl"))?;

        Ok(Self {
            output_dir: output_dir.to_path_buf(),
            camera_writer: BufWriter::new(camera_file),
            commentary_writer: BufWriter::new(commentary_file),
            full_writer: BufWriter::new(full_file),
            ticks_written: 0,
        })
    }

    /// Writes a tick's output to all files.
    ///
    /// Each file gets one JSON object per line (JSON Lines format).
    pub fn write_tick(&mut self, output: &DirectorOutput) -> Result<(), OutputError> {
        // Write camera script (array of instructions as single line)
        let camera_json = serde_json::to_string(&output.camera_script)?;
        writeln!(self.camera_writer, "{}", camera_json)?;

        // Write commentary (array of items as single line)
        let commentary_json = serde_json::to_string(&output.commentary_queue)?;
        writeln!(self.commentary_writer, "{}", commentary_json)?;

        // Write full output as single line
        let full_json = serde_json::to_string(output)?;
        writeln!(self.full_writer, "{}", full_json)?;

        self.ticks_written += 1;
        Ok(())
    }

    /// Flushes all buffered writers to disk.
    pub fn flush(&mut self) -> Result<(), OutputError> {
        self.camera_writer.flush()?;
        self.commentary_writer.flush()?;
        self.full_writer.flush()?;
        Ok(())
    }

    /// Returns the number of ticks written.
    pub fn ticks_written(&self) -> u64 {
        self.ticks_written
    }

    /// Returns the output directory path.
    pub fn output_dir(&self) -> &Path {
        &self.output_dir
    }

    /// Writes a summary file with metadata about the output.
    pub fn write_summary(&self, total_events: u64, total_tensions: u64) -> Result<(), OutputError> {
        let summary = serde_json::json!({
            "ticks_written": self.ticks_written,
            "total_events_processed": total_events,
            "total_tensions_processed": total_tensions,
            "files": {
                "camera_script": "camera_script.jsonl",
                "commentary": "commentary.jsonl",
                "full_output": "full_output.jsonl"
            }
        });

        let summary_path = self.output_dir.join("summary.json");
        let file = File::create(summary_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &summary)?;
        Ok(())
    }
}

/// Wrapper for reading DirectorOutput from JSON Lines files.
#[derive(Debug)]
pub struct OutputReader {
    /// Path to the full_output.jsonl file
    path: std::path::PathBuf,
}

impl OutputReader {
    /// Creates a new output reader for the given file.
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
        }
    }

    /// Creates a reader for the full_output.jsonl in a directory.
    pub fn from_dir(output_dir: &Path) -> Self {
        Self::new(&output_dir.join("full_output.jsonl"))
    }

    /// Reads all outputs from the file.
    pub fn read_all(&self) -> Result<Vec<DirectorOutput>, OutputError> {
        let content = fs::read_to_string(&self.path)?;
        let mut outputs = Vec::new();

        for line in content.lines() {
            if !line.trim().is_empty() {
                let output: DirectorOutput = serde_json::from_str(line)?;
                outputs.push(output);
            }
        }

        Ok(outputs)
    }

    /// Reads a specific tick's output (0-indexed).
    pub fn read_tick(&self, tick_index: usize) -> Result<Option<DirectorOutput>, OutputError> {
        let content = fs::read_to_string(&self.path)?;

        for (i, line) in content.lines().enumerate() {
            if i == tick_index && !line.trim().is_empty() {
                let output: DirectorOutput = serde_json::from_str(line)?;
                return Ok(Some(output));
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_events::Season;
    use tempfile::tempdir;

    fn test_timestamp() -> SimTimestamp {
        SimTimestamp::new(1000, 1, Season::Spring, 10)
    }

    fn make_test_output() -> DirectorOutput {
        let ts = test_timestamp();
        let mut output = DirectorOutput::new(1000);

        output.add_camera_instruction(CameraInstruction::new(
            "cam_1000_0001",
            ts.clone(),
            CameraMode::follow_agent("agent_mira", ZoomLevel::Close),
            CameraFocus::primary("agent_mira"),
            "Following protagonist",
        ));

        output.add_commentary(CommentaryItem::new(
            "com_1000_0001",
            ts.clone(),
            CommentaryType::EventCaption,
            "Mira arrives at the eastern bridge",
        ));

        output.add_highlight(
            HighlightMarker::new("evt_00001", HighlightType::KeyMoment, 950, 1050)
                .with_description("Key moment"),
        );

        output
    }

    #[test]
    fn test_camera_instruction_creation() {
        let ts = test_timestamp();
        let instr = CameraInstruction::new(
            "cam_1000_0001",
            ts.clone(),
            CameraMode::follow_agent("agent_mira", ZoomLevel::Close),
            CameraFocus::primary("agent_mira"),
            "Following protagonist",
        )
        .with_pacing(PacingHint::Normal)
        .with_tension("tens_00001");

        assert_eq!(instr.instruction_id, "cam_1000_0001");
        assert_eq!(instr.pacing, PacingHint::Normal);
        assert!(instr.tension_id.is_some());
    }

    #[test]
    fn test_camera_mode_serialization() {
        let mode = CameraMode::follow_agent("agent_001", ZoomLevel::Close);
        let json = serde_json::to_string(&mode).unwrap();
        assert!(json.contains("follow_agent"));
        assert!(json.contains("agent_001"));
        assert!(json.contains("close"));
    }

    #[test]
    fn test_camera_focus_agent_ids() {
        let focus = CameraFocus::conversation("agent_a", "agent_b");
        let ids = focus.agent_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"agent_a"));
        assert!(ids.contains(&"agent_b"));
    }

    #[test]
    fn test_pacing_hint_serialization() {
        assert_eq!(serde_json::to_string(&PacingHint::Slow).unwrap(), r#""slow""#);
        assert_eq!(serde_json::to_string(&PacingHint::Urgent).unwrap(), r#""urgent""#);
        assert_eq!(serde_json::to_string(&PacingHint::Climactic).unwrap(), r#""climactic""#);
    }

    #[test]
    fn test_zoom_level_serialization() {
        assert_eq!(serde_json::to_string(&ZoomLevel::Extreme).unwrap(), r#""extreme""#);
        assert_eq!(serde_json::to_string(&ZoomLevel::Regional).unwrap(), r#""regional""#);
    }

    #[test]
    fn test_camera_waypoint() {
        let wp = CameraWaypoint::new("location_bridge", ZoomLevel::Wide, 50)
            .with_easing(CameraEasing::EaseInOut);

        assert_eq!(wp.target, "location_bridge");
        assert_eq!(wp.zoom, ZoomLevel::Wide);
        assert_eq!(wp.hold_ticks, 50);
        assert_eq!(wp.easing, CameraEasing::EaseInOut);
    }

    #[test]
    fn test_commentary_item_creation() {
        let ts = test_timestamp();
        let item = CommentaryItem::new(
            "com_1000_0001",
            ts,
            CommentaryType::EventCaption,
            "Mira arrives at the eastern bridge",
        )
        .with_duration(150)
        .with_priority(0.8)
        .with_agents(vec!["agent_mira".to_string()]);

        assert_eq!(item.display_duration_ticks, 150);
        assert_eq!(item.priority, 0.8);
        assert_eq!(item.related_agents.len(), 1);
    }

    #[test]
    fn test_commentary_type_serialization() {
        assert_eq!(
            serde_json::to_string(&CommentaryType::EventCaption).unwrap(),
            r#""event_caption""#
        );
        assert_eq!(
            serde_json::to_string(&CommentaryType::DramaticIrony).unwrap(),
            r#""dramatic_irony""#
        );
        assert_eq!(
            serde_json::to_string(&CommentaryType::TensionTeaser).unwrap(),
            r#""tension_teaser""#
        );
    }

    #[test]
    fn test_highlight_marker() {
        let marker = HighlightMarker::new("evt_00007", HighlightType::Climax, 900, 1100)
            .with_description("The betrayal is revealed");

        assert_eq!(marker.event_id, "evt_00007");
        assert_eq!(marker.highlight_type, HighlightType::Climax);
        assert!(marker.description.is_some());
    }

    #[test]
    fn test_highlight_type_serialization() {
        assert_eq!(
            serde_json::to_string(&HighlightType::KeyMoment).unwrap(),
            r#""key_moment""#
        );
        assert_eq!(
            serde_json::to_string(&HighlightType::TurningPoint).unwrap(),
            r#""turning_point""#
        );
    }

    #[test]
    fn test_director_output() {
        let mut output = DirectorOutput::new(1000);

        let ts = test_timestamp();
        output.add_camera_instruction(CameraInstruction::new(
            "cam_1000_0001",
            ts.clone(),
            CameraMode::overview(None),
            CameraFocus::location("village_center"),
            "Default overview",
        ));

        output.add_commentary(CommentaryItem::new(
            "com_1000_0001",
            ts,
            CommentaryType::TensionTeaser,
            "Winter stores are running low",
        ));

        assert!(output.has_camera_instructions());
        assert!(output.has_commentary());
        assert_eq!(output.camera_script.len(), 1);
        assert_eq!(output.commentary_queue.len(), 1);
    }

    #[test]
    fn test_director_output_serialization() {
        let output = DirectorOutput::new(1000);
        let json = serde_json::to_string(&output).unwrap();

        assert!(json.contains("generated_at_tick"));
        assert!(json.contains("camera_script"));
        assert!(json.contains("commentary_queue"));

        // Roundtrip
        let parsed: DirectorOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.generated_at_tick, 1000);
    }

    #[test]
    fn test_generate_instruction_id() {
        assert_eq!(generate_instruction_id(1000, 1), "cam_1000_0001");
        assert_eq!(generate_instruction_id(50000, 42), "cam_50000_0042");
    }

    #[test]
    fn test_generate_commentary_id() {
        assert_eq!(generate_commentary_id(1000, 1), "com_1000_0001");
        assert_eq!(generate_commentary_id(50000, 42), "com_50000_0042");
    }

    #[test]
    fn test_full_camera_instruction_roundtrip() {
        let ts = test_timestamp();
        let instr = CameraInstruction::new(
            "cam_1000_0001",
            ts.clone(),
            CameraMode::Cinematic {
                path: vec![
                    CameraWaypoint::new("loc_a", ZoomLevel::Wide, 50),
                    CameraWaypoint::new("loc_b", ZoomLevel::Close, 100),
                ],
                duration_ticks: 200,
            },
            CameraFocus::group(vec!["agent_a".to_string(), "agent_b".to_string()]),
            "Cinematic reveal",
        )
        .with_valid_until(SimTimestamp::new(1200, 1, Season::Spring, 12))
        .with_pacing(PacingHint::Climactic)
        .with_tension("tens_00042");

        let json = serde_json::to_string_pretty(&instr).unwrap();
        let parsed: CameraInstruction = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.instruction_id, "cam_1000_0001");
        assert_eq!(parsed.pacing, PacingHint::Climactic);
        assert_eq!(parsed.tension_id, Some("tens_00042".to_string()));

        if let CameraMode::Cinematic { path, duration_ticks } = parsed.camera_mode {
            assert_eq!(path.len(), 2);
            assert_eq!(duration_ticks, 200);
        } else {
            panic!("Expected Cinematic camera mode");
        }
    }

    // --- File I/O Tests ---

    #[test]
    fn test_output_to_json() {
        let output = make_test_output();
        let json = output.to_json().unwrap();

        assert!(json.contains("generated_at_tick"));
        assert!(json.contains("1000"));
        assert!(json.contains("agent_mira"));
        assert!(json.contains("Mira arrives"));
    }

    #[test]
    fn test_output_to_json_compact() {
        let output = make_test_output();
        let json = output.to_json_compact().unwrap();

        // Compact JSON should not contain newlines
        assert!(!json.contains('\n'));
        assert!(json.contains("generated_at_tick"));
    }

    #[test]
    fn test_write_camera_script() {
        let dir = tempdir().unwrap();
        let output = make_test_output();
        let path = dir.path().join("camera_script.json");

        output.write_camera_script(&path).unwrap();

        // Verify file exists and contains expected content
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("cam_1000_0001"));
        assert!(content.contains("agent_mira"));
        assert!(content.contains("follow_agent"));
    }

    #[test]
    fn test_write_commentary() {
        let dir = tempdir().unwrap();
        let output = make_test_output();
        let path = dir.path().join("commentary.json");

        output.write_commentary(&path).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("com_1000_0001"));
        assert!(content.contains("Mira arrives"));
        assert!(content.contains("event_caption"));
    }

    #[test]
    fn test_write_highlights() {
        let dir = tempdir().unwrap();
        let output = make_test_output();
        let path = dir.path().join("highlights.json");

        output.write_highlights(&path).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("evt_00001"));
        assert!(content.contains("key_moment"));
    }

    #[test]
    fn test_write_all() {
        let dir = tempdir().unwrap();
        let output = make_test_output();

        output.write_all(dir.path()).unwrap();

        // Verify all files exist
        assert!(dir.path().join("camera_script.json").exists());
        assert!(dir.path().join("commentary.json").exists());
        assert!(dir.path().join("highlights.json").exists());
    }

    #[test]
    fn test_output_writer_creation() {
        let dir = tempdir().unwrap();
        let writer = OutputWriter::new(dir.path()).unwrap();

        assert_eq!(writer.ticks_written(), 0);
        assert!(dir.path().join("camera_script.jsonl").exists());
        assert!(dir.path().join("commentary.jsonl").exists());
        assert!(dir.path().join("full_output.jsonl").exists());
    }

    #[test]
    fn test_output_writer_write_tick() {
        let dir = tempdir().unwrap();
        let mut writer = OutputWriter::new(dir.path()).unwrap();

        let output1 = make_test_output();
        let mut output2 = DirectorOutput::new(1001);
        output2.add_camera_instruction(CameraInstruction::new(
            "cam_1001_0001",
            SimTimestamp::new(1001, 1, Season::Spring, 10),
            CameraMode::overview(None),
            CameraFocus::location("village_center"),
            "Overview shot",
        ));

        writer.write_tick(&output1).unwrap();
        writer.write_tick(&output2).unwrap();
        writer.flush().unwrap();

        assert_eq!(writer.ticks_written(), 2);

        // Read full output file and verify both ticks are there
        let content = fs::read_to_string(dir.path().join("full_output.jsonl")).unwrap();
        let lines: Vec<_> = content.lines().collect();
        assert_eq!(lines.len(), 2);

        // First line should be tick 1000
        assert!(lines[0].contains("1000"));
        // Second line should be tick 1001
        assert!(lines[1].contains("1001"));
    }

    #[test]
    fn test_output_writer_summary() {
        let dir = tempdir().unwrap();
        let mut writer = OutputWriter::new(dir.path()).unwrap();

        writer.write_tick(&make_test_output()).unwrap();
        writer.write_tick(&make_test_output()).unwrap();
        writer.flush().unwrap();

        writer.write_summary(100, 10).unwrap();

        let summary_path = dir.path().join("summary.json");
        assert!(summary_path.exists());

        let content = fs::read_to_string(summary_path).unwrap();
        assert!(content.contains("\"ticks_written\": 2"));
        assert!(content.contains("\"total_events_processed\": 100"));
        assert!(content.contains("camera_script.jsonl"));
    }

    #[test]
    fn test_output_reader_read_all() {
        let dir = tempdir().unwrap();
        let mut writer = OutputWriter::new(dir.path()).unwrap();

        writer.write_tick(&make_test_output()).unwrap();
        let mut output2 = DirectorOutput::new(2000);
        output2.add_camera_instruction(CameraInstruction::new(
            "cam_2000_0001",
            SimTimestamp::new(2000, 1, Season::Summer, 5),
            CameraMode::overview(None),
            CameraFocus::location("market"),
            "Market overview",
        ));
        writer.write_tick(&output2).unwrap();
        writer.flush().unwrap();

        let reader = OutputReader::from_dir(dir.path());
        let outputs = reader.read_all().unwrap();

        assert_eq!(outputs.len(), 2);
        assert_eq!(outputs[0].generated_at_tick, 1000);
        assert_eq!(outputs[1].generated_at_tick, 2000);
    }

    #[test]
    fn test_output_reader_read_tick() {
        let dir = tempdir().unwrap();
        let mut writer = OutputWriter::new(dir.path()).unwrap();

        writer.write_tick(&make_test_output()).unwrap();
        let mut output2 = DirectorOutput::new(2000);
        output2.add_camera_instruction(CameraInstruction::new(
            "cam_2000_0001",
            SimTimestamp::new(2000, 1, Season::Summer, 5),
            CameraMode::overview(None),
            CameraFocus::location("market"),
            "Market overview",
        ));
        writer.write_tick(&output2).unwrap();
        writer.flush().unwrap();

        let reader = OutputReader::from_dir(dir.path());

        let tick0 = reader.read_tick(0).unwrap();
        assert!(tick0.is_some());
        assert_eq!(tick0.unwrap().generated_at_tick, 1000);

        let tick1 = reader.read_tick(1).unwrap();
        assert!(tick1.is_some());
        assert_eq!(tick1.unwrap().generated_at_tick, 2000);

        let tick2 = reader.read_tick(2).unwrap();
        assert!(tick2.is_none());
    }

    #[test]
    fn test_output_error_display() {
        let io_err = OutputError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "test error",
        ));
        assert!(io_err.to_string().contains("I/O error"));

        // JSON error display
        let json_str = "not valid json {{{";
        let json_result: Result<DirectorOutput, _> = serde_json::from_str(json_str);
        if let Err(e) = json_result {
            let output_err = OutputError::Json(e);
            assert!(output_err.to_string().contains("JSON error"));
        }
    }
}
