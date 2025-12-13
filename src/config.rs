//! Configuration System
//!
//! Loads tuning parameters from tuning.toml for easy adjustment without recompiling.

use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Default tuning file path
pub const DEFAULT_TUNING_PATH: &str = "tuning.toml";

/// Top-level configuration structure
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub simulation: SimulationConfig,
    pub agents: AgentConfig,
    pub movement: MovementConfig,
    pub communication: CommunicationConfig,
    pub resource: ResourceConfig,
    pub social: SocialConfig,
    pub faction: FactionConfig,
    pub conflict: ConflictConfig,
    pub archive: ArchiveConfig,
    pub memory: MemoryConfig,
    pub trust: TrustConfig,
    pub drama: DramaConfig,
}

/// Simulation parameters
#[derive(Debug, Clone, Deserialize)]
pub struct SimulationConfig {
    pub default_ticks: u64,
    pub snapshot_interval: u64,
    pub ritual_interval: u64,
}

/// Agent spawning configuration
#[derive(Debug, Clone, Deserialize)]
pub struct AgentConfig {
    pub agents_per_faction: usize,
    pub specialist_count: usize,
}

/// Movement action weights
#[derive(Debug, Clone, Deserialize)]
pub struct MovementConfig {
    pub travel_base: f32,
    pub patrol_base: f32,
    pub wander_base: f32,
    pub return_home_base: f32,
    pub flee_base: f32,
    pub boldness_travel_bonus: f32,
    pub loyalty_patrol_bonus: f32,
    pub sociability_wander_bonus: f32,
}

/// Communication action weights
#[derive(Debug, Clone, Deserialize)]
pub struct CommunicationConfig {
    pub share_memory_base: f32,
    pub spread_rumor_base: f32,
    pub lie_base: f32,
    pub confess_base: f32,
    pub sociability_chat_bonus: f32,
    pub honesty_lie_penalty: f32,
    pub grudge_gossip_bonus: f32,
    pub same_faction_bonus: f32,
    pub positive_relationship_bonus: f32,
    pub negative_relationship_penalty: f32,
}

/// Resource action weights
#[derive(Debug, Clone, Deserialize)]
pub struct ResourceConfig {
    pub work_base: f32,
    pub trade_base: f32,
    pub steal_base: f32,
    pub hoard_base: f32,
    pub desperate_work_bonus: f32,
    pub stressed_steal_bonus: f32,
    pub honesty_steal_penalty: f32,
    pub loyalty_steal_faction_penalty: f32,
}

/// Social action weights
#[derive(Debug, Clone, Deserialize)]
pub struct SocialConfig {
    pub build_trust_base: f32,
    pub curry_favor_base: f32,
    pub gift_base: f32,
    pub ostracize_base: f32,
    pub sociability_build_trust_bonus: f32,
    pub ambition_curry_favor_bonus: f32,
    pub loyalty_ostracize_traitor_bonus: f32,
}

/// Faction action weights
#[derive(Debug, Clone, Deserialize)]
pub struct FactionConfig {
    pub defect_base: f32,
    pub exile_base: f32,
    pub challenge_leader_base: f32,
    pub support_leader_base: f32,
    pub loyalty_defect_penalty: f32,
    pub ambition_challenge_bonus: f32,
    pub loyalty_support_bonus: f32,
}

/// Conflict action weights
#[derive(Debug, Clone, Deserialize)]
pub struct ConflictConfig {
    pub argue_base: f32,
    pub fight_base: f32,
    pub sabotage_base: f32,
    pub assassinate_base: f32,
    pub boldness_fight_bonus: f32,
    pub grudge_revenge_bonus: f32,
    pub honesty_sabotage_penalty: f32,
}

/// Archive action weights
#[derive(Debug, Clone, Deserialize)]
pub struct ArchiveConfig {
    pub write_base: f32,
    pub read_base: f32,
    pub destroy_base: f32,
    pub forge_base: f32,
}

/// Memory system parameters
#[derive(Debug, Clone, Deserialize)]
pub struct MemoryConfig {
    pub memory_decay_rate: f32,
    pub memory_cleanup_threshold: f32,
    pub secondhand_fidelity_multiplier: f32,
    pub emotional_multiplier: f32,
}

/// Trust system parameters
#[derive(Debug, Clone, Deserialize)]
pub struct TrustConfig {
    pub trust_decay_rate: f32,
    pub grudge_decay_rate: f32,
    pub grudge_threshold: f32,
}

/// Drama scoring parameters
#[derive(Debug, Clone, Deserialize)]
pub struct DramaConfig {
    pub high_drama_threshold: f32,
    pub medium_drama_threshold: f32,
    pub enemy_faction_multiplier: f32,
    pub close_relationship_multiplier: f32,
    pub betrayal_multiplier: f32,
    pub desperate_state_multiplier: f32,
}

impl Config {
    /// Load configuration from a TOML file
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Load configuration from default path, or use defaults if not found
    pub fn load_or_default() -> Self {
        Self::load(DEFAULT_TUNING_PATH).unwrap_or_else(|e| {
            eprintln!("Warning: Could not load tuning.toml: {}. Using defaults.", e);
            Self::default()
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            simulation: SimulationConfig {
                default_ticks: 1000,
                snapshot_interval: 100,
                ritual_interval: 500,
            },
            agents: AgentConfig {
                agents_per_faction: 55,
                specialist_count: 5,
            },
            movement: MovementConfig {
                travel_base: 0.4,
                patrol_base: 0.2,
                wander_base: 0.3,
                return_home_base: 0.2,
                flee_base: 0.8,
                boldness_travel_bonus: 0.3,
                loyalty_patrol_bonus: 0.4,
                sociability_wander_bonus: 0.2,
            },
            communication: CommunicationConfig {
                share_memory_base: 0.5,
                spread_rumor_base: 0.4,
                lie_base: 0.15,
                confess_base: 0.2,
                sociability_chat_bonus: 0.4,
                honesty_lie_penalty: 0.15,
                grudge_gossip_bonus: 0.3,
                same_faction_bonus: 0.2,
                positive_relationship_bonus: 0.3,
                negative_relationship_penalty: 0.4,
            },
            resource: ResourceConfig {
                work_base: 0.6,
                trade_base: 0.25,
                steal_base: 0.05,
                hoard_base: 0.2,
                desperate_work_bonus: 0.4,
                stressed_steal_bonus: 0.15,
                honesty_steal_penalty: 0.15,
                loyalty_steal_faction_penalty: 0.2,
            },
            social: SocialConfig {
                build_trust_base: 0.5,
                curry_favor_base: 0.3,
                gift_base: 0.15,
                ostracize_base: 0.1,
                sociability_build_trust_bonus: 0.3,
                ambition_curry_favor_bonus: 0.3,
                loyalty_ostracize_traitor_bonus: 0.3,
            },
            faction: FactionConfig {
                defect_base: 0.02,
                exile_base: 0.05,
                challenge_leader_base: 0.02,
                support_leader_base: 0.3,
                loyalty_defect_penalty: 0.4,
                ambition_challenge_bonus: 0.3,
                loyalty_support_bonus: 0.3,
            },
            conflict: ConflictConfig {
                argue_base: 0.15,
                fight_base: 0.05,
                sabotage_base: 0.02,
                assassinate_base: 0.005,
                boldness_fight_bonus: 0.3,
                grudge_revenge_bonus: 0.4,
                honesty_sabotage_penalty: 0.15,
            },
            archive: ArchiveConfig {
                write_base: 0.3,
                read_base: 0.5,
                destroy_base: 0.05,
                forge_base: 0.02,
            },
            memory: MemoryConfig {
                memory_decay_rate: 0.1,
                memory_cleanup_threshold: 0.1,
                secondhand_fidelity_multiplier: 0.7,
                emotional_multiplier: 0.5,
            },
            trust: TrustConfig {
                trust_decay_rate: 0.01,
                grudge_decay_rate: 0.005,
                grudge_threshold: 0.3,
            },
            drama: DramaConfig {
                high_drama_threshold: 0.7,
                medium_drama_threshold: 0.4,
                enemy_faction_multiplier: 1.5,
                close_relationship_multiplier: 1.4,
                betrayal_multiplier: 1.8,
                desperate_state_multiplier: 1.3,
            },
        }
    }
}

/// Configuration error type
#[derive(Debug)]
pub enum ConfigError {
    IoError(String),
    ParseError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(e) => write!(f, "IO error: {}", e),
            ConfigError::ParseError(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.simulation.default_ticks, 1000);
        assert_eq!(config.agents.agents_per_faction, 55);
        assert!(config.movement.travel_base > 0.0);
    }

    #[test]
    fn test_load_config_file() {
        // This test requires the tuning.toml file to exist
        if Path::new(DEFAULT_TUNING_PATH).exists() {
            let config = Config::load(DEFAULT_TUNING_PATH).unwrap();
            assert!(config.simulation.default_ticks > 0);
        }
    }
}
