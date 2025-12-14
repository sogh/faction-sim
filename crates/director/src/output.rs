//! Director output types and structures.
//!
//! Contains camera instructions, commentary items, and the main DirectorOutput
//! that is consumed by the visualization layer.

use serde::{Deserialize, Serialize};
use sim_events::SimTimestamp;

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
}

/// Generates a camera instruction ID.
pub fn generate_instruction_id(tick: u64, sequence: u32) -> String {
    format!("cam_{}_{:04}", tick, sequence)
}

/// Generates a commentary item ID.
pub fn generate_commentary_id(tick: u64, sequence: u32) -> String {
    format!("com_{}_{:04}", tick, sequence)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_events::Season;

    fn test_timestamp() -> SimTimestamp {
        SimTimestamp::new(1000, 1, Season::Spring, 10)
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
}
