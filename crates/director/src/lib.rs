//! Director AI: drama detection and camera control.
//!
//! The Director AI sits between simulation and visualization. It watches raw
//! events and active tensions, then decides what's worth showing and how to
//! show it. Think of it as an invisible documentary filmmaker—choosing when
//! to cut, where to point the camera, and what story threads to follow.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐     events.jsonl      ┌──────────┐    camera_script.json
//! │  sim-core   │ ───────────────────▶  │ director │ ──────────────────────▶
//! └─────────────┘                       └──────────┘
//! ```
//!
//! # Modules
//!
//! - [`output`]: Camera instructions, commentary items, and DirectorOutput
//! - [`threads`]: Narrative thread tracking
//! - [`scorer`]: Event prioritization with configurable weights
//! - [`focus`]: Tension-based camera focus selection
//! - [`commentary`]: Template-based text generation

pub mod commentary;
pub mod config;
pub mod focus;
pub mod output;
pub mod scorer;
pub mod threads;

// Re-export output types
pub use output::{
    CameraEasing, CameraFocus, CameraInstruction, CameraMode, CameraWaypoint, CommentaryItem,
    CommentaryType, DirectorOutput, HighlightMarker, HighlightType, PacingHint, ZoomLevel,
    generate_commentary_id, generate_instruction_id,
};

// Re-export thread types
pub use threads::{
    generate_thread_id, NarrativeThread, ScoredEvent, ThreadStatus, ThreadTracker,
    ThreadTrackerConfig,
};

// Re-export scorer types
pub use scorer::{DirectorContext, EventScorer, EventWeights, ScorerError};

// Re-export config types
pub use config::{
    default_config_toml, CommentaryConfig, ConfigError, DefaultCameraMode, DirectorConfig,
    FocusConfig, GeneralConfig, TomlSerializeError,
};

// Re-export focus types
pub use focus::FocusSelector;

// Re-export commentary types
pub use commentary::{
    default_templates, default_templates_toml, CommentaryGenerator, CommentaryTemplates,
    IronySituation, IronyTemplate, ReminderTemplate, TeaserTemplate, TemplateError,
};
