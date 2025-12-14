//! Tension Types
//!
//! Data structures for representing developing dramatic situations in the simulation.
//! These are higher-level abstractions than raw events, designed for the Director AI.

use serde::{Deserialize, Serialize};

/// Type of tension detected in the simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TensionType {
    /// Trust eroding toward defection/sabotage
    BrewingBetrayal,
    /// Leadership contested or uncertain
    SuccessionCrisis,
    /// Scarcity driving competition
    ResourceConflict,
    /// Cross-faction relationship forming
    ForbiddenAlliance,
    /// Agent pursuing payback
    RevengeArc,
    /// Agent gaining influence rapidly
    RisingPower,
    /// Internal faction divisions deepening
    FactionFracture,
    /// Outside pressure forcing response
    ExternalThreat,
    /// Hidden information about to surface
    SecretExposed,
    /// Upcoming ritual at risk
    RitualDisruption,
}

/// Status of a tension
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TensionStatus {
    /// Tension is developing
    Developing,
    /// Tension is getting worse
    Escalating,
    /// Tension has reached critical level
    Critical,
    /// Tension is decreasing
    DeEscalating,
    /// Tension has been resolved
    Resolved,
}

/// Agent's role in a tension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TensionAgent {
    pub agent_id: String,
    pub role_in_tension: String,
    pub trajectory: String,
}

/// Predicted outcome of a tension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictedOutcome {
    pub outcome: String,
    pub probability: f32,
    pub impact: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_ticks_until: Option<u64>,
}

/// Camera focus recommendation for the Director AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraFocus {
    pub primary: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub secondary: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub locations_of_interest: Vec<String>,
}

/// A detected tension in the simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tension {
    pub tension_id: String,
    pub detected_at_tick: u64,
    pub last_updated_tick: u64,
    pub status: TensionStatus,
    pub tension_type: TensionType,
    pub severity: f32,
    pub confidence: f32,
    pub summary: String,
    pub key_agents: Vec<TensionAgent>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub key_locations: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub trigger_events: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub predicted_outcomes: Vec<PredictedOutcome>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub narrative_hooks: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended_camera_focus: Option<CameraFocus>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub connected_tensions: Vec<String>,
}

impl Tension {
    /// Create a new tension
    pub fn new(
        tension_id: impl Into<String>,
        tension_type: TensionType,
        detected_at_tick: u64,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            tension_id: tension_id.into(),
            detected_at_tick,
            last_updated_tick: detected_at_tick,
            status: TensionStatus::Developing,
            tension_type,
            severity: 0.3,
            confidence: 0.5,
            summary: summary.into(),
            key_agents: Vec::new(),
            key_locations: Vec::new(),
            trigger_events: Vec::new(),
            predicted_outcomes: Vec::new(),
            narrative_hooks: Vec::new(),
            recommended_camera_focus: None,
            connected_tensions: Vec::new(),
        }
    }

    /// Add a key agent to the tension
    pub fn add_agent(
        &mut self,
        agent_id: impl Into<String>,
        role: impl Into<String>,
        trajectory: impl Into<String>,
    ) {
        self.key_agents.push(TensionAgent {
            agent_id: agent_id.into(),
            role_in_tension: role.into(),
            trajectory: trajectory.into(),
        });
    }

    /// Add a key location
    pub fn add_location(&mut self, location: impl Into<String>) {
        self.key_locations.push(location.into());
    }

    /// Add a trigger event
    pub fn add_trigger_event(&mut self, event_id: impl Into<String>) {
        self.trigger_events.push(event_id.into());
    }

    /// Add a predicted outcome
    pub fn add_predicted_outcome(
        &mut self,
        outcome: impl Into<String>,
        probability: f32,
        impact: impl Into<String>,
    ) {
        self.predicted_outcomes.push(PredictedOutcome {
            outcome: outcome.into(),
            probability,
            impact: impact.into(),
            estimated_ticks_until: None,
        });
    }

    /// Update severity and status
    pub fn update_severity(&mut self, new_severity: f32, current_tick: u64) {
        let old_severity = self.severity;
        self.severity = new_severity.clamp(0.0, 1.0);
        self.last_updated_tick = current_tick;

        // Update status based on severity change
        if self.severity >= 0.8 {
            self.status = TensionStatus::Critical;
        } else if self.severity > old_severity + 0.1 {
            self.status = TensionStatus::Escalating;
        } else if self.severity < old_severity - 0.1 {
            self.status = TensionStatus::DeEscalating;
        }

        if self.severity < 0.1 {
            self.status = TensionStatus::Resolved;
        }
    }

    /// Check if this tension should be removed
    pub fn is_resolved(&self) -> bool {
        self.status == TensionStatus::Resolved
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tension_creation() {
        let tension = Tension::new("tens_00001", TensionType::BrewingBetrayal, 1000, "Test tension");
        assert_eq!(tension.tension_type, TensionType::BrewingBetrayal);
        assert_eq!(tension.status, TensionStatus::Developing);
        assert_eq!(tension.severity, 0.3);
    }

    #[test]
    fn test_tension_severity_update() {
        let mut tension = Tension::new("tens_00001", TensionType::RevengeArc, 1000, "Test");

        tension.update_severity(0.85, 1100);
        assert_eq!(tension.status, TensionStatus::Critical);

        tension.update_severity(0.05, 1200);
        assert_eq!(tension.status, TensionStatus::Resolved);
    }

    #[test]
    fn test_tension_serialization() {
        let mut tension = Tension::new("tens_00001", TensionType::ForbiddenAlliance, 500, "Cross-faction friendship");
        tension.add_agent("agent_001", "ally", "deepening");
        tension.add_agent("agent_002", "ally", "deepening");
        tension.add_location("neutral_ground");
        tension.add_predicted_outcome("alliance_formalized", 0.4, "high");

        let json = serde_json::to_string(&tension).unwrap();
        assert!(json.contains("forbidden_alliance"));
        assert!(json.contains("agent_001"));
    }
}
