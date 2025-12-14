//! Tension Types
//!
//! Data structures for representing developing dramatic situations in the simulation.
//! These are higher-level abstractions than raw events, designed for the Director AI.

use serde::{Deserialize, Serialize};

/// Type of tension detected in the simulation.
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

/// Status of a tension's lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TensionStatus {
    /// Tension is just beginning to form
    Emerging,
    /// Tension is getting worse
    Escalating,
    /// Tension has reached a critical point
    Critical,
    /// Tension is at its peak moment
    Climax,
    /// Tension is decreasing
    Resolving,
    /// Tension has been fully resolved
    Resolved,
    /// Tension is temporarily inactive but may resurface
    Dormant,
}

/// Agent's role in a tension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TensionAgent {
    pub agent_id: String,
    pub role_in_tension: String,
    pub trajectory: String,
}

impl TensionAgent {
    /// Creates a new TensionAgent.
    pub fn new(
        agent_id: impl Into<String>,
        role_in_tension: impl Into<String>,
        trajectory: impl Into<String>,
    ) -> Self {
        Self {
            agent_id: agent_id.into(),
            role_in_tension: role_in_tension.into(),
            trajectory: trajectory.into(),
        }
    }
}

/// Predicted outcome of a tension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictedOutcome {
    pub outcome: String,
    pub probability: f32,
    pub impact: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_ticks_until: Option<u64>,
}

impl PredictedOutcome {
    /// Creates a new PredictedOutcome.
    pub fn new(outcome: impl Into<String>, probability: f32, impact: impl Into<String>) -> Self {
        Self {
            outcome: outcome.into(),
            probability,
            impact: impact.into(),
            estimated_ticks_until: None,
        }
    }

    /// Sets the estimated ticks until this outcome.
    pub fn with_estimated_ticks(mut self, ticks: u64) -> Self {
        self.estimated_ticks_until = Some(ticks);
        self
    }
}

/// Camera focus recommendation for the Director AI.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CameraRecommendation {
    /// Primary agent to focus on (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<String>,
    /// Secondary agents of interest
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub secondary: Vec<String>,
    /// Locations worth showing
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub locations_of_interest: Vec<String>,
}

impl CameraRecommendation {
    /// Creates a new camera recommendation with a primary focus.
    pub fn new(primary: impl Into<String>) -> Self {
        Self {
            primary: Some(primary.into()),
            secondary: Vec::new(),
            locations_of_interest: Vec::new(),
        }
    }

    /// Adds secondary agents.
    pub fn with_secondary(mut self, agents: Vec<String>) -> Self {
        self.secondary = agents;
        self
    }

    /// Adds locations of interest.
    pub fn with_locations(mut self, locations: Vec<String>) -> Self {
        self.locations_of_interest = locations;
        self
    }
}

/// Type alias for backwards compatibility.
pub type CameraFocus = CameraRecommendation;

/// A detected tension in the simulation.
///
/// Tensions are higher-level narrative structures that the Director AI
/// uses to decide what's worth showing to the viewer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tension {
    /// Unique identifier (e.g., "tens_00284")
    pub tension_id: String,
    /// Tick when the tension was first detected
    pub detected_at_tick: u64,
    /// Tick of the last update
    pub last_updated_tick: u64,
    /// Current status in the tension lifecycle
    pub status: TensionStatus,
    /// Type of dramatic tension
    pub tension_type: TensionType,
    /// Severity from 0.0 to 1.0
    pub severity: f32,
    /// Confidence in the tension assessment (0.0 to 1.0)
    pub confidence: f32,
    /// Human-readable summary
    pub summary: String,
    /// Key agents involved
    pub key_agents: Vec<TensionAgent>,
    /// Important locations
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub key_locations: Vec<String>,
    /// Events that triggered or contributed to this tension
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trigger_events: Vec<String>,
    /// Possible outcomes with probabilities
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub predicted_outcomes: Vec<PredictedOutcome>,
    /// Narrative hooks for storytelling
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub narrative_hooks: Vec<String>,
    /// Camera focus recommendation for the Director
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended_camera_focus: Option<CameraRecommendation>,
    /// Related tension IDs
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub connected_tensions: Vec<String>,
}

impl Tension {
    /// Create a new tension.
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
            status: TensionStatus::Emerging,
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

    /// Add a key agent to the tension.
    pub fn add_agent(&mut self, agent: TensionAgent) {
        self.key_agents.push(agent);
    }

    /// Add a key agent with inline construction.
    pub fn add_agent_inline(
        &mut self,
        agent_id: impl Into<String>,
        role: impl Into<String>,
        trajectory: impl Into<String>,
    ) {
        self.key_agents.push(TensionAgent::new(agent_id, role, trajectory));
    }

    /// Add a key location.
    pub fn add_location(&mut self, location: impl Into<String>) {
        self.key_locations.push(location.into());
    }

    /// Add a trigger event.
    pub fn add_trigger_event(&mut self, event_id: impl Into<String>) {
        self.trigger_events.push(event_id.into());
    }

    /// Add a predicted outcome.
    pub fn add_predicted_outcome(&mut self, outcome: PredictedOutcome) {
        self.predicted_outcomes.push(outcome);
    }

    /// Add a narrative hook.
    pub fn add_narrative_hook(&mut self, hook: impl Into<String>) {
        self.narrative_hooks.push(hook.into());
    }

    /// Set the camera recommendation.
    pub fn set_camera_recommendation(&mut self, recommendation: CameraRecommendation) {
        self.recommended_camera_focus = Some(recommendation);
    }

    /// Add a connected tension.
    pub fn add_connected_tension(&mut self, tension_id: impl Into<String>) {
        self.connected_tensions.push(tension_id.into());
    }

    /// Update severity and status based on new severity value.
    pub fn update_severity(&mut self, new_severity: f32, current_tick: u64) {
        let old_severity = self.severity;
        self.severity = new_severity.clamp(0.0, 1.0);
        self.last_updated_tick = current_tick;

        // Update status based on severity change
        if self.severity >= 0.9 {
            self.status = TensionStatus::Climax;
        } else if self.severity >= 0.8 {
            self.status = TensionStatus::Critical;
        } else if self.severity > old_severity + 0.1 {
            self.status = TensionStatus::Escalating;
        } else if self.severity < old_severity - 0.2 {
            self.status = TensionStatus::Resolving;
        }

        if self.severity < 0.1 {
            self.status = TensionStatus::Resolved;
        }
    }

    /// Mark the tension as dormant.
    pub fn mark_dormant(&mut self, current_tick: u64) {
        self.status = TensionStatus::Dormant;
        self.last_updated_tick = current_tick;
    }

    /// Check if this tension is resolved.
    pub fn is_resolved(&self) -> bool {
        self.status == TensionStatus::Resolved
    }

    /// Check if this tension is active (not resolved or dormant).
    pub fn is_active(&self) -> bool {
        !matches!(self.status, TensionStatus::Resolved | TensionStatus::Dormant)
    }

    /// Check if this tension is high severity (> 0.7).
    pub fn is_high_severity(&self) -> bool {
        self.severity > 0.7
    }
}

/// Generates a tension ID with the given sequence number.
pub fn generate_tension_id(sequence: u64) -> String {
    format!("tens_{:05}", sequence)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tension_type_serialization() {
        assert_eq!(serde_json::to_string(&TensionType::BrewingBetrayal).unwrap(), r#""brewing_betrayal""#);
        assert_eq!(serde_json::to_string(&TensionType::SuccessionCrisis).unwrap(), r#""succession_crisis""#);
        assert_eq!(serde_json::to_string(&TensionType::ResourceConflict).unwrap(), r#""resource_conflict""#);
    }

    #[test]
    fn test_tension_status_serialization() {
        assert_eq!(serde_json::to_string(&TensionStatus::Emerging).unwrap(), r#""emerging""#);
        assert_eq!(serde_json::to_string(&TensionStatus::Escalating).unwrap(), r#""escalating""#);
        assert_eq!(serde_json::to_string(&TensionStatus::Critical).unwrap(), r#""critical""#);
        assert_eq!(serde_json::to_string(&TensionStatus::Climax).unwrap(), r#""climax""#);
        assert_eq!(serde_json::to_string(&TensionStatus::Resolving).unwrap(), r#""resolving""#);
        assert_eq!(serde_json::to_string(&TensionStatus::Resolved).unwrap(), r#""resolved""#);
        assert_eq!(serde_json::to_string(&TensionStatus::Dormant).unwrap(), r#""dormant""#);
    }

    #[test]
    fn test_tension_creation() {
        let tension = Tension::new("tens_00001", TensionType::BrewingBetrayal, 1000, "Test tension");
        assert_eq!(tension.tension_type, TensionType::BrewingBetrayal);
        assert_eq!(tension.status, TensionStatus::Emerging);
        assert_eq!(tension.severity, 0.3);
    }

    #[test]
    fn test_tension_severity_update() {
        let mut tension = Tension::new("tens_00001", TensionType::RevengeArc, 1000, "Test");

        tension.update_severity(0.85, 1100);
        assert_eq!(tension.status, TensionStatus::Critical);

        tension.update_severity(0.95, 1150);
        assert_eq!(tension.status, TensionStatus::Climax);

        tension.update_severity(0.05, 1200);
        assert_eq!(tension.status, TensionStatus::Resolved);
    }

    #[test]
    fn test_tension_dormant() {
        let mut tension = Tension::new("tens_00001", TensionType::ForbiddenAlliance, 1000, "Test");
        assert!(tension.is_active());

        tension.mark_dormant(1500);
        assert_eq!(tension.status, TensionStatus::Dormant);
        assert!(!tension.is_active());
    }

    #[test]
    fn test_tension_serialization() {
        let mut tension = Tension::new("tens_00001", TensionType::ForbiddenAlliance, 500, "Cross-faction friendship");
        tension.add_agent_inline("agent_001", "ally", "deepening");
        tension.add_agent_inline("agent_002", "ally", "deepening");
        tension.add_location("neutral_ground");
        tension.add_predicted_outcome(
            PredictedOutcome::new("alliance_formalized", 0.4, "high")
                .with_estimated_ticks(2000)
        );
        tension.add_narrative_hook("The two scouts have been meeting in secret");
        tension.set_camera_recommendation(
            CameraRecommendation::new("agent_001")
                .with_secondary(vec!["agent_002".to_string()])
                .with_locations(vec!["neutral_ground".to_string()])
        );

        let json = serde_json::to_string(&tension).unwrap();
        assert!(json.contains("forbidden_alliance"));
        assert!(json.contains("agent_001"));
        assert!(json.contains("estimated_ticks_until"));

        // Verify roundtrip
        let parsed: Tension = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.tension_id, "tens_00001");
        assert_eq!(parsed.key_agents.len(), 2);
    }

    #[test]
    fn test_generate_tension_id() {
        assert_eq!(generate_tension_id(1), "tens_00001");
        assert_eq!(generate_tension_id(284), "tens_00284");
        assert_eq!(generate_tension_id(99999), "tens_99999");
    }

    #[test]
    fn test_camera_recommendation() {
        let rec = CameraRecommendation::new("agent_mira_0042")
            .with_secondary(vec!["agent_voss_0017".to_string(), "agent_corin_0003".to_string()])
            .with_locations(vec!["eastern_bridge".to_string()]);

        assert_eq!(rec.primary, Some("agent_mira_0042".to_string()));
        assert_eq!(rec.secondary.len(), 2);
        assert_eq!(rec.locations_of_interest.len(), 1);
    }

    #[test]
    fn test_predicted_outcome() {
        let outcome = PredictedOutcome::new("mira_defects_to_ironmere", 0.45, "high")
            .with_estimated_ticks(2000);

        assert_eq!(outcome.outcome, "mira_defects_to_ironmere");
        assert_eq!(outcome.probability, 0.45);
        assert_eq!(outcome.estimated_ticks_until, Some(2000));
    }
}
