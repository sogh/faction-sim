//! Configuration loading for the Director.
//!
//! All director settings are loaded from a TOML configuration file.

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::scorer::EventWeights;
use crate::threads::ThreadTrackerConfig;

/// Complete Director configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectorConfig {
    /// Event scoring weights
    #[serde(default)]
    pub event_weights: EventWeights,
    /// Focus selection settings
    #[serde(default)]
    pub focus: FocusConfig,
    /// Commentary generation settings
    #[serde(default)]
    pub commentary: CommentaryConfig,
    /// Thread tracking settings
    #[serde(default)]
    pub threads: ThreadTrackerConfig,
    /// General director settings
    #[serde(default)]
    pub director: GeneralConfig,
}

impl Default for DirectorConfig {
    fn default() -> Self {
        Self {
            event_weights: EventWeights::default(),
            focus: FocusConfig::default(),
            commentary: CommentaryConfig::default(),
            threads: ThreadTrackerConfig::default(),
            director: GeneralConfig::default(),
        }
    }
}

impl DirectorConfig {
    /// Loads configuration from a TOML file.
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path).map_err(ConfigError::IoError)?;
        Self::from_str(&content)
    }

    /// Parses configuration from a TOML string.
    pub fn from_str(content: &str) -> Result<Self, ConfigError> {
        toml::from_str(content).map_err(ConfigError::TomlError)
    }

    /// Returns a default configuration as a TOML string.
    pub fn to_toml(&self) -> Result<String, TomlSerializeError> {
        toml::to_string_pretty(self).map_err(TomlSerializeError)
    }
}

/// Focus selection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FocusConfig {
    /// Minimum tension severity to consider for focus
    pub min_tension_severity: f32,
    /// Maximum concurrent narrative threads to track
    pub max_concurrent_threads: usize,
    /// Ticks before thread fatigue kicks in
    pub thread_fatigue_threshold_ticks: u64,
    /// Multiplier reduction when fatigued
    pub fatigue_multiplier: f32,
    /// Minimum score for an event to be notable
    pub min_event_score: f32,
    /// Boost for current focus continuity
    pub focus_continuity_boost: f32,
}

impl Default for FocusConfig {
    fn default() -> Self {
        Self {
            min_tension_severity: 0.3,
            max_concurrent_threads: 3,
            thread_fatigue_threshold_ticks: 5000,
            fatigue_multiplier: 0.5,
            min_event_score: 0.2,
            focus_continuity_boost: 1.2,
        }
    }
}

/// Commentary generation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CommentaryConfig {
    /// Maximum items in the commentary queue
    pub max_queue_size: usize,
    /// Minimum drama score to generate a caption
    pub min_drama_for_caption: f32,
    /// Base display duration in ticks
    pub base_display_duration_ticks: u32,
    /// Extra ticks per character for longer captions
    pub ticks_per_character: f32,
    /// Cooldown between similar commentary types
    pub commentary_cooldown_ticks: u64,
    /// Enable dramatic irony commentary
    pub enable_dramatic_irony: bool,
    /// Enable tension teaser commentary
    pub enable_tension_teasers: bool,
    /// Enable context reminder commentary
    pub enable_context_reminders: bool,
}

impl Default for CommentaryConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 5,
            min_drama_for_caption: 0.3,
            base_display_duration_ticks: 100,
            ticks_per_character: 1.0,
            commentary_cooldown_ticks: 500,
            enable_dramatic_irony: true,
            enable_tension_teasers: true,
            enable_context_reminders: true,
        }
    }
}

/// General director settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    /// How many ticks ahead to look for foresight
    pub foresight_ticks: u64,
    /// Enable highlight marking
    pub enable_highlights: bool,
    /// Minimum score for highlight
    pub min_highlight_score: f32,
    /// Default camera mode when no focus is selected
    #[serde(default)]
    pub default_camera_mode: DefaultCameraMode,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            foresight_ticks: 1000,
            enable_highlights: true,
            min_highlight_score: 0.7,
            default_camera_mode: DefaultCameraMode::Overview,
        }
    }
}

/// Default camera mode when no specific focus is selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DefaultCameraMode {
    /// Wide overview of the world
    #[default]
    Overview,
    /// Cycle between active locations
    LocationCycle,
    /// Follow the highest-activity agent
    HighActivity,
}

/// Errors that can occur during configuration loading.
#[derive(Debug)]
pub enum ConfigError {
    /// IO error reading config file
    IoError(std::io::Error),
    /// Error parsing TOML config
    TomlError(toml::de::Error),
}

/// Error that can occur during TOML serialization.
#[derive(Debug)]
pub struct TomlSerializeError(pub toml::ser::Error);

impl std::fmt::Display for TomlSerializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TOML serialize error: {}", self.0)
    }
}

impl std::error::Error for TomlSerializeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(e) => write!(f, "IO error: {}", e),
            ConfigError::TomlError(e) => write!(f, "TOML parse error: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::IoError(e) => Some(e),
            ConfigError::TomlError(e) => Some(e),
        }
    }
}

/// Generates a default configuration file content.
pub fn default_config_toml() -> String {
    r#"# Director Configuration

[event_weights.base_scores]
betrayal = 0.9
death = 0.85
conflict = 0.7
faction = 0.6
ritual = 0.5
cooperation = 0.4
loyalty = 0.35
communication = 0.3
birth = 0.3
resource = 0.25
archive = 0.2
movement = 0.1

[event_weights.drama_tag_scores]
faction_critical = 0.3
secret_meeting = 0.25
leader_involved = 0.2
cross_faction = 0.15
betrayal = 0.15
revenge = 0.15
power_struggle = 0.15
winter_crisis = 0.1
death = 0.1

[focus]
min_tension_severity = 0.3
max_concurrent_threads = 3
thread_fatigue_threshold_ticks = 5000
fatigue_multiplier = 0.5
min_event_score = 0.2
focus_continuity_boost = 1.2

[commentary]
max_queue_size = 5
min_drama_for_caption = 0.3
base_display_duration_ticks = 100
ticks_per_character = 1.0
commentary_cooldown_ticks = 500
enable_dramatic_irony = true
enable_tension_teasers = true
enable_context_reminders = true

[threads]
min_severity_for_thread = 0.3
dormant_threshold_ticks = 5000
max_threads = 20

[director]
foresight_ticks = 1000
enable_highlights = true
min_highlight_score = 0.7
default_camera_mode = "overview"
"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DirectorConfig::default();

        assert_eq!(config.focus.min_tension_severity, 0.3);
        assert_eq!(config.focus.max_concurrent_threads, 3);
        assert_eq!(config.commentary.max_queue_size, 5);
        assert!(config.commentary.enable_dramatic_irony);
    }

    #[test]
    fn test_focus_config_default() {
        let focus = FocusConfig::default();

        assert_eq!(focus.min_tension_severity, 0.3);
        assert_eq!(focus.thread_fatigue_threshold_ticks, 5000);
        assert_eq!(focus.fatigue_multiplier, 0.5);
    }

    #[test]
    fn test_commentary_config_default() {
        let commentary = CommentaryConfig::default();

        assert_eq!(commentary.max_queue_size, 5);
        assert_eq!(commentary.base_display_duration_ticks, 100);
        assert!(commentary.enable_dramatic_irony);
    }

    #[test]
    fn test_parse_config_from_toml() {
        let toml = r#"
            [focus]
            min_tension_severity = 0.5
            max_concurrent_threads = 5

            [commentary]
            max_queue_size = 10
            enable_dramatic_irony = false
        "#;

        let config = DirectorConfig::from_str(toml).unwrap();

        assert_eq!(config.focus.min_tension_severity, 0.5);
        assert_eq!(config.focus.max_concurrent_threads, 5);
        assert_eq!(config.commentary.max_queue_size, 10);
        assert!(!config.commentary.enable_dramatic_irony);
    }

    #[test]
    fn test_partial_config_uses_defaults() {
        let toml = r#"
            [focus]
            min_tension_severity = 0.5
        "#;

        let config = DirectorConfig::from_str(toml).unwrap();

        // Specified value
        assert_eq!(config.focus.min_tension_severity, 0.5);
        // Default values
        assert_eq!(config.focus.max_concurrent_threads, 3);
        assert_eq!(config.commentary.max_queue_size, 5);
    }

    #[test]
    fn test_config_with_event_weights() {
        let toml = r#"
            [event_weights.base_scores]
            betrayal = 1.0
            death = 0.9

            [event_weights.drama_tag_scores]
            custom_tag = 0.5
        "#;

        let config = DirectorConfig::from_str(toml).unwrap();

        assert_eq!(
            config.event_weights.base_scores.get("betrayal"),
            Some(&1.0)
        );
        assert_eq!(
            config.event_weights.drama_tag_scores.get("custom_tag"),
            Some(&0.5)
        );
    }

    #[test]
    fn test_config_to_toml() {
        let config = DirectorConfig::default();
        let toml = config.to_toml().unwrap();

        // TOML uses nested tables like [event_weights.base_scores]
        assert!(toml.contains("base_scores") || toml.contains("[event_weights]"));
        assert!(toml.contains("[focus]"));
        assert!(toml.contains("[commentary]"));
    }

    #[test]
    fn test_default_config_toml_parses() {
        let toml = default_config_toml();
        let config = DirectorConfig::from_str(&toml).unwrap();

        assert_eq!(config.focus.min_tension_severity, 0.3);
        assert_eq!(config.commentary.max_queue_size, 5);
    }

    #[test]
    fn test_general_config_default() {
        let general = GeneralConfig::default();

        assert_eq!(general.foresight_ticks, 1000);
        assert!(general.enable_highlights);
        assert_eq!(general.min_highlight_score, 0.7);
        assert_eq!(general.default_camera_mode, DefaultCameraMode::Overview);
    }

    #[test]
    fn test_default_camera_mode_serialization() {
        assert_eq!(
            serde_json::to_string(&DefaultCameraMode::Overview).unwrap(),
            r#""overview""#
        );
        assert_eq!(
            serde_json::to_string(&DefaultCameraMode::LocationCycle).unwrap(),
            r#""location_cycle""#
        );
        assert_eq!(
            serde_json::to_string(&DefaultCameraMode::HighActivity).unwrap(),
            r#""high_activity""#
        );
    }

    #[test]
    fn test_config_with_general_settings() {
        let toml = r#"
            [director]
            foresight_ticks = 2000
            enable_highlights = false
            min_highlight_score = 0.8
            default_camera_mode = "high_activity"
        "#;

        let config = DirectorConfig::from_str(toml).unwrap();

        assert_eq!(config.director.foresight_ticks, 2000);
        assert!(!config.director.enable_highlights);
        assert_eq!(config.director.min_highlight_score, 0.8);
        assert_eq!(config.director.default_camera_mode, DefaultCameraMode::HighActivity);
    }
}
