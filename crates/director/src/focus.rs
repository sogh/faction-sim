//! Tension-based camera focus selection.
//!
//! Decides which narrative thread deserves camera attention based on
//! tension severity, thread fatigue, and dramatic value.

use sim_events::Tension;

use crate::config::FocusConfig;
use crate::output::{
    generate_instruction_id, CameraFocus, CameraInstruction, CameraMode, PacingHint, ZoomLevel,
};
use crate::threads::{NarrativeThread, ScoredEvent};

/// Selects camera focus based on tensions and narrative threads.
#[derive(Debug, Clone)]
pub struct FocusSelector {
    /// Configuration for focus selection
    config: FocusConfig,
    /// Current tick for generating instruction IDs
    current_tick: u64,
    /// Sequence number for instruction IDs
    instruction_sequence: u32,
}

impl FocusSelector {
    /// Creates a new focus selector with the given configuration.
    pub fn new(config: FocusConfig) -> Self {
        Self {
            config,
            current_tick: 0,
            instruction_sequence: 0,
        }
    }

    /// Creates a new focus selector with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(FocusConfig::default())
    }

    /// Sets the current tick for instruction ID generation.
    pub fn set_current_tick(&mut self, tick: u64) {
        if tick != self.current_tick {
            self.current_tick = tick;
            self.instruction_sequence = 0;
        }
    }

    /// Selects camera focus based on tensions, threads, and current state.
    ///
    /// Selection logic:
    /// 1. Filter tensions to those with severity >= min_tension_severity
    /// 2. If no viable tensions, return default_wandering_camera()
    /// 3. Check if current focus is still on an active, non-fatigued thread
    ///    - If yes, continue with that focus
    /// 4. Otherwise, select highest severity tension that isn't fatigued
    /// 5. Generate appropriate CameraInstruction based on tension type
    pub fn select_focus(
        &mut self,
        tensions: &[Tension],
        threads: &[NarrativeThread],
        current_focus: Option<&CameraFocus>,
        _scored_events: &[ScoredEvent],
        timestamp: sim_events::SimTimestamp,
    ) -> CameraInstruction {
        self.set_current_tick(timestamp.tick);

        // Filter to viable tensions (above severity threshold and active)
        let viable_tensions: Vec<_> = tensions
            .iter()
            .filter(|t| t.severity >= self.config.min_tension_severity && t.is_active())
            .collect();

        // No viable tensions -> wandering camera
        if viable_tensions.is_empty() {
            return self.default_wandering_camera(timestamp);
        }

        // Check if current focus should continue (non-fatigued, still active)
        if let Some(focus) = current_focus {
            if let Some(continuing_tension) =
                self.find_continuing_tension(&viable_tensions, focus, threads)
            {
                if !self.is_fatigued(continuing_tension, threads) {
                    return self.continue_focus(continuing_tension, timestamp);
                }
            }
        }

        // Select highest severity non-fatigued tension
        let selected = viable_tensions
            .iter()
            .filter(|t| !self.is_fatigued(t, threads))
            .max_by(|a, b| a.severity.partial_cmp(&b.severity).unwrap());

        match selected {
            Some(tension) => self.focus_on_tension(tension, timestamp),
            None => {
                // All tensions fatigued - fall back to highest severity anyway
                // but mark it as a fatigue-induced choice
                if let Some(fallback) = viable_tensions
                    .iter()
                    .max_by(|a, b| a.severity.partial_cmp(&b.severity).unwrap())
                {
                    self.focus_on_tension_with_fatigue(fallback, timestamp)
                } else {
                    self.default_wandering_camera(timestamp)
                }
            }
        }
    }

    /// Finds a tension that matches the current focus and is still viable.
    fn find_continuing_tension<'a>(
        &self,
        tensions: &[&'a Tension],
        current_focus: &CameraFocus,
        threads: &[NarrativeThread],
    ) -> Option<&'a Tension> {
        let focus_agent_ids = current_focus.agent_ids();

        for tension in tensions {
            // Check if tension's key agents overlap with current focus
            let tension_agent_ids: Vec<_> =
                tension.key_agents.iter().map(|a| a.agent_id.as_str()).collect();

            let has_overlap = tension_agent_ids
                .iter()
                .any(|id| focus_agent_ids.contains(id));

            if has_overlap {
                // Also verify there's an active thread for this tension
                let has_active_thread = threads
                    .iter()
                    .any(|t| t.involves_tension(&tension.tension_id) && t.is_active());

                if has_active_thread {
                    return Some(tension);
                }
            }
        }
        None
    }

    /// Checks if a tension's thread is fatigued (shown too long).
    pub fn is_fatigued(&self, tension: &Tension, threads: &[NarrativeThread]) -> bool {
        // Find the thread for this tension
        let thread = threads
            .iter()
            .find(|t| t.involves_tension(&tension.tension_id));

        match thread {
            Some(thread) => {
                // Thread is fatigued if it's been shown for too long
                thread.screen_time_ticks >= self.config.thread_fatigue_threshold_ticks
            }
            None => false, // No thread = not fatigued
        }
    }

    /// Creates a camera instruction to focus on a tension.
    ///
    /// Uses different camera modes based on tension characteristics:
    /// - FollowAgent for tensions with a clear primary agent
    /// - FrameMultiple for tensions involving multiple key agents
    /// - FrameLocation for location-centric tensions
    pub fn focus_on_tension(
        &mut self,
        tension: &Tension,
        timestamp: sim_events::SimTimestamp,
    ) -> CameraInstruction {
        let (camera_mode, camera_focus) = self.determine_camera_for_tension(tension);
        let pacing = self.severity_to_pacing(tension.severity);

        let instruction_id = self.next_instruction_id();
        CameraInstruction::new(
            instruction_id,
            timestamp,
            camera_mode,
            camera_focus,
            format!(
                "Focus on {} tension: {}",
                format!("{:?}", tension.tension_type).to_lowercase(),
                tension.summary
            ),
        )
        .with_pacing(pacing)
        .with_tension(&tension.tension_id)
    }

    /// Creates a camera instruction to focus on a fatigued tension (fallback).
    fn focus_on_tension_with_fatigue(
        &mut self,
        tension: &Tension,
        timestamp: sim_events::SimTimestamp,
    ) -> CameraInstruction {
        let (camera_mode, camera_focus) = self.determine_camera_for_tension(tension);
        let pacing = self.severity_to_pacing(tension.severity);

        let instruction_id = self.next_instruction_id();
        CameraInstruction::new(
            instruction_id,
            timestamp,
            camera_mode,
            camera_focus,
            format!(
                "Returning to fatigued tension (no alternatives): {}",
                tension.summary
            ),
        )
        .with_pacing(pacing)
        .with_tension(&tension.tension_id)
    }

    /// Creates a camera instruction to continue focusing on the current tension.
    pub fn continue_focus(
        &mut self,
        tension: &Tension,
        timestamp: sim_events::SimTimestamp,
    ) -> CameraInstruction {
        let (camera_mode, camera_focus) = self.determine_camera_for_tension(tension);
        let pacing = self.severity_to_pacing(tension.severity);

        let instruction_id = self.next_instruction_id();
        CameraInstruction::new(
            instruction_id,
            timestamp,
            camera_mode,
            camera_focus,
            format!("Continuing focus on: {}", tension.summary),
        )
        .with_pacing(pacing)
        .with_tension(&tension.tension_id)
    }

    /// Creates a default wandering camera instruction when no tensions warrant focus.
    pub fn default_wandering_camera(
        &mut self,
        timestamp: sim_events::SimTimestamp,
    ) -> CameraInstruction {
        let instruction_id = self.next_instruction_id();
        CameraInstruction::new(
            instruction_id,
            timestamp,
            CameraMode::overview(None),
            CameraFocus::location("world_overview"),
            "No active tensions - default overview",
        )
        .with_pacing(PacingHint::Slow)
    }

    /// Determines the appropriate camera mode and focus for a tension.
    fn determine_camera_for_tension(&self, tension: &Tension) -> (CameraMode, CameraFocus) {
        let agent_count = tension.key_agents.len();
        let has_locations = !tension.key_locations.is_empty();

        // Check for recommended camera focus from tension
        if let Some(ref recommendation) = tension.recommended_camera_focus {
            if let Some(ref primary) = recommendation.primary {
                if !recommendation.secondary.is_empty() {
                    // Multiple agents recommended
                    let mut agent_ids = vec![primary.clone()];
                    agent_ids.extend(recommendation.secondary.clone());
                    return (
                        CameraMode::frame_multiple(agent_ids.clone(), true),
                        CameraFocus::group(agent_ids),
                    );
                } else {
                    // Single agent recommended
                    return (
                        CameraMode::follow_agent(primary, self.severity_to_zoom(tension.severity)),
                        CameraFocus::primary(primary),
                    );
                }
            } else if !recommendation.locations_of_interest.is_empty() {
                // Location-focused
                let location = &recommendation.locations_of_interest[0];
                return (
                    CameraMode::frame_location(location, ZoomLevel::Wide),
                    CameraFocus::location(location),
                );
            }
        }

        // Default behavior based on agent count
        match agent_count {
            0 => {
                // No agents, use location if available
                if has_locations {
                    let location = &tension.key_locations[0];
                    (
                        CameraMode::frame_location(location, ZoomLevel::Wide),
                        CameraFocus::location(location),
                    )
                } else {
                    // Fallback to overview
                    (CameraMode::overview(None), CameraFocus::location("unknown"))
                }
            }
            1 => {
                // Single agent - follow them
                let agent_id = &tension.key_agents[0].agent_id;
                (
                    CameraMode::follow_agent(agent_id, self.severity_to_zoom(tension.severity)),
                    CameraFocus::primary(agent_id),
                )
            }
            2 => {
                // Two agents - could be a conversation or confrontation
                let agent_a = &tension.key_agents[0].agent_id;
                let agent_b = &tension.key_agents[1].agent_id;
                (
                    CameraMode::frame_multiple(vec![agent_a.clone(), agent_b.clone()], true),
                    CameraFocus::conversation(agent_a, agent_b),
                )
            }
            _ => {
                // Multiple agents - frame them all
                let agent_ids: Vec<_> = tension
                    .key_agents
                    .iter()
                    .map(|a| a.agent_id.clone())
                    .collect();
                (
                    CameraMode::frame_multiple(agent_ids.clone(), true),
                    CameraFocus::group(agent_ids),
                )
            }
        }
    }

    /// Converts tension severity to a pacing hint.
    fn severity_to_pacing(&self, severity: f32) -> PacingHint {
        if severity >= 0.9 {
            PacingHint::Climactic
        } else if severity >= 0.7 {
            PacingHint::Urgent
        } else if severity >= 0.4 {
            PacingHint::Normal
        } else {
            PacingHint::Slow
        }
    }

    /// Converts tension severity to a zoom level.
    fn severity_to_zoom(&self, severity: f32) -> ZoomLevel {
        if severity >= 0.8 {
            ZoomLevel::Close
        } else if severity >= 0.5 {
            ZoomLevel::Medium
        } else {
            ZoomLevel::Wide
        }
    }

    /// Generates the next instruction ID.
    fn next_instruction_id(&mut self) -> String {
        self.instruction_sequence += 1;
        generate_instruction_id(self.current_tick, self.instruction_sequence)
    }

    /// Returns a reference to the configuration.
    pub fn config(&self) -> &FocusConfig {
        &self.config
    }
}

impl Default for FocusSelector {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_events::{Season, SimTimestamp, TensionStatus, TensionType};

    fn test_timestamp(tick: u64) -> SimTimestamp {
        SimTimestamp::new(tick, 1, Season::Spring, 10)
    }

    fn make_tension(id: &str, severity: f32, status: TensionStatus) -> Tension {
        let mut tension = Tension::new(id, TensionType::BrewingBetrayal, 1000, "Test tension");
        tension.severity = severity;
        tension.status = status;
        tension.add_agent_inline("agent_mira", "betrayer", "escalating");
        tension
    }

    fn make_tension_with_agents(
        id: &str,
        severity: f32,
        agent_ids: Vec<&str>,
    ) -> Tension {
        let mut tension = Tension::new(id, TensionType::ResourceConflict, 1000, "Multi-agent tension");
        tension.severity = severity;
        tension.status = TensionStatus::Escalating;
        for agent_id in agent_ids {
            tension.add_agent_inline(agent_id, "participant", "stable");
        }
        tension
    }

    fn make_thread_for_tension(tension: &Tension, screen_time: u64) -> NarrativeThread {
        let mut thread = NarrativeThread::from_tension(tension, "thread_00001");
        thread.screen_time_ticks = screen_time;
        thread
    }

    #[test]
    fn test_focus_selector_creation() {
        let selector = FocusSelector::new(FocusConfig::default());
        assert_eq!(selector.config.min_tension_severity, 0.3);
    }

    #[test]
    fn test_empty_tensions_returns_wandering_camera() {
        let mut selector = FocusSelector::with_defaults();
        let tensions: Vec<Tension> = vec![];
        let threads: Vec<NarrativeThread> = vec![];

        let instruction = selector.select_focus(
            &tensions,
            &threads,
            None,
            &[],
            test_timestamp(1000),
        );

        assert!(instruction.reason.contains("No active tensions"));
        assert_eq!(instruction.pacing, PacingHint::Slow);
        matches!(instruction.camera_mode, CameraMode::Overview { .. });
    }

    #[test]
    fn test_highest_severity_wins() {
        let mut selector = FocusSelector::with_defaults();

        let low_tension = make_tension("tens_low", 0.4, TensionStatus::Escalating);
        let high_tension = make_tension("tens_high", 0.8, TensionStatus::Escalating);
        let medium_tension = make_tension("tens_med", 0.6, TensionStatus::Escalating);

        let tensions = vec![low_tension, high_tension, medium_tension];
        let threads: Vec<NarrativeThread> = vec![];

        let instruction = selector.select_focus(
            &tensions,
            &threads,
            None,
            &[],
            test_timestamp(1000),
        );

        assert_eq!(instruction.tension_id, Some("tens_high".to_string()));
    }

    #[test]
    fn test_fatigue_causes_switch() {
        let mut selector = FocusSelector::new(FocusConfig {
            thread_fatigue_threshold_ticks: 100,
            ..FocusConfig::default()
        });

        let high_tension = make_tension("tens_high", 0.9, TensionStatus::Escalating);
        let low_tension = make_tension("tens_low", 0.5, TensionStatus::Escalating);

        // High tension thread is fatigued
        let fatigued_thread = make_thread_for_tension(&high_tension, 200); // Over threshold
        let fresh_thread = make_thread_for_tension(&low_tension, 10);

        let tensions = vec![high_tension, low_tension];
        let threads = vec![fatigued_thread, fresh_thread];

        let instruction = selector.select_focus(
            &tensions,
            &threads,
            None,
            &[],
            test_timestamp(1000),
        );

        // Should select the lower severity but non-fatigued tension
        assert_eq!(instruction.tension_id, Some("tens_low".to_string()));
    }

    #[test]
    fn test_all_fatigued_falls_back_to_highest() {
        let mut selector = FocusSelector::new(FocusConfig {
            thread_fatigue_threshold_ticks: 100,
            ..FocusConfig::default()
        });

        let high_tension = make_tension("tens_high", 0.9, TensionStatus::Escalating);
        let low_tension = make_tension("tens_low", 0.5, TensionStatus::Escalating);

        // Both threads are fatigued
        let fatigued_thread1 = make_thread_for_tension(&high_tension, 200);
        let fatigued_thread2 = make_thread_for_tension(&low_tension, 150);

        let tensions = vec![high_tension, low_tension];
        let threads = vec![fatigued_thread1, fatigued_thread2];

        let instruction = selector.select_focus(
            &tensions,
            &threads,
            None,
            &[],
            test_timestamp(1000),
        );

        // Should fall back to highest severity
        assert_eq!(instruction.tension_id, Some("tens_high".to_string()));
        assert!(instruction.reason.contains("fatigued"));
    }

    #[test]
    fn test_tension_below_threshold_ignored() {
        let mut selector = FocusSelector::new(FocusConfig {
            min_tension_severity: 0.5,
            ..FocusConfig::default()
        });

        let low_tension = make_tension("tens_low", 0.3, TensionStatus::Escalating);
        let tensions = vec![low_tension];
        let threads: Vec<NarrativeThread> = vec![];

        let instruction = selector.select_focus(
            &tensions,
            &threads,
            None,
            &[],
            test_timestamp(1000),
        );

        // Should return wandering camera (tension below threshold)
        assert!(instruction.reason.contains("No active tensions"));
    }

    #[test]
    fn test_resolved_tensions_ignored() {
        let mut selector = FocusSelector::with_defaults();

        let resolved = make_tension("tens_resolved", 0.8, TensionStatus::Resolved);
        let active = make_tension("tens_active", 0.5, TensionStatus::Escalating);

        let tensions = vec![resolved, active];
        let threads: Vec<NarrativeThread> = vec![];

        let instruction = selector.select_focus(
            &tensions,
            &threads,
            None,
            &[],
            test_timestamp(1000),
        );

        // Should select active tension, ignoring resolved one
        assert_eq!(instruction.tension_id, Some("tens_active".to_string()));
    }

    #[test]
    fn test_single_agent_uses_follow() {
        let mut selector = FocusSelector::with_defaults();

        let tension = make_tension("tens_001", 0.6, TensionStatus::Escalating);
        let tensions = vec![tension];
        let threads: Vec<NarrativeThread> = vec![];

        let instruction = selector.select_focus(
            &tensions,
            &threads,
            None,
            &[],
            test_timestamp(1000),
        );

        matches!(instruction.camera_mode, CameraMode::FollowAgent { .. });
    }

    #[test]
    fn test_multiple_agents_uses_frame_multiple() {
        let mut selector = FocusSelector::with_defaults();

        let tension = make_tension_with_agents(
            "tens_001",
            0.6,
            vec!["agent_a", "agent_b", "agent_c"],
        );
        let tensions = vec![tension];
        let threads: Vec<NarrativeThread> = vec![];

        let instruction = selector.select_focus(
            &tensions,
            &threads,
            None,
            &[],
            test_timestamp(1000),
        );

        matches!(instruction.camera_mode, CameraMode::FrameMultiple { .. });
        matches!(instruction.focus, CameraFocus::Group { .. });
    }

    #[test]
    fn test_two_agents_creates_conversation_focus() {
        let mut selector = FocusSelector::with_defaults();

        let tension = make_tension_with_agents("tens_001", 0.6, vec!["agent_a", "agent_b"]);
        let tensions = vec![tension];
        let threads: Vec<NarrativeThread> = vec![];

        let instruction = selector.select_focus(
            &tensions,
            &threads,
            None,
            &[],
            test_timestamp(1000),
        );

        matches!(instruction.focus, CameraFocus::Conversation { .. });
    }

    #[test]
    fn test_severity_to_pacing() {
        let selector = FocusSelector::with_defaults();

        assert_eq!(selector.severity_to_pacing(0.95), PacingHint::Climactic);
        assert_eq!(selector.severity_to_pacing(0.75), PacingHint::Urgent);
        assert_eq!(selector.severity_to_pacing(0.5), PacingHint::Normal);
        assert_eq!(selector.severity_to_pacing(0.2), PacingHint::Slow);
    }

    #[test]
    fn test_severity_to_zoom() {
        let selector = FocusSelector::with_defaults();

        assert_eq!(selector.severity_to_zoom(0.9), ZoomLevel::Close);
        assert_eq!(selector.severity_to_zoom(0.6), ZoomLevel::Medium);
        assert_eq!(selector.severity_to_zoom(0.3), ZoomLevel::Wide);
    }

    #[test]
    fn test_is_fatigued() {
        let selector = FocusSelector::new(FocusConfig {
            thread_fatigue_threshold_ticks: 1000,
            ..FocusConfig::default()
        });

        let tension = make_tension("tens_001", 0.6, TensionStatus::Escalating);

        // Thread under threshold
        let fresh_thread = make_thread_for_tension(&tension, 500);
        assert!(!selector.is_fatigued(&tension, &[fresh_thread]));

        // Thread over threshold
        let fatigued_thread = make_thread_for_tension(&tension, 1500);
        assert!(selector.is_fatigued(&tension, &[fatigued_thread]));

        // No thread
        assert!(!selector.is_fatigued(&tension, &[]));
    }

    #[test]
    fn test_instruction_ids_are_unique() {
        let mut selector = FocusSelector::with_defaults();

        let tension = make_tension("tens_001", 0.6, TensionStatus::Escalating);
        let tensions = vec![tension];
        let threads: Vec<NarrativeThread> = vec![];

        let inst1 = selector.select_focus(&tensions, &threads, None, &[], test_timestamp(1000));
        let inst2 = selector.select_focus(&tensions, &threads, None, &[], test_timestamp(1000));

        assert_ne!(inst1.instruction_id, inst2.instruction_id);
    }

    #[test]
    fn test_instruction_id_resets_on_new_tick() {
        let mut selector = FocusSelector::with_defaults();

        let tension = make_tension("tens_001", 0.6, TensionStatus::Escalating);
        let tensions = vec![tension];
        let threads: Vec<NarrativeThread> = vec![];

        let inst1 = selector.select_focus(&tensions, &threads, None, &[], test_timestamp(1000));
        let inst2 = selector.select_focus(&tensions, &threads, None, &[], test_timestamp(2000));

        // Different ticks, IDs should have different tick prefixes
        assert!(inst1.instruction_id.contains("1000"));
        assert!(inst2.instruction_id.contains("2000"));
    }

    #[test]
    fn test_location_centric_tension() {
        let mut selector = FocusSelector::with_defaults();

        let mut tension = Tension::new(
            "tens_001",
            TensionType::RitualDisruption,
            1000,
            "Ritual at risk",
        );
        tension.severity = 0.6;
        tension.status = TensionStatus::Escalating;
        tension.add_location("sacred_grove");
        // No agents

        let tensions = vec![tension];
        let threads: Vec<NarrativeThread> = vec![];

        let instruction = selector.select_focus(
            &tensions,
            &threads,
            None,
            &[],
            test_timestamp(1000),
        );

        matches!(instruction.camera_mode, CameraMode::FrameLocation { .. });
        matches!(instruction.focus, CameraFocus::Location { .. });
    }

    #[test]
    fn test_continue_focus_when_not_fatigued() {
        let mut selector = FocusSelector::new(FocusConfig {
            thread_fatigue_threshold_ticks: 10000,
            ..FocusConfig::default()
        });

        let tension = make_tension("tens_001", 0.6, TensionStatus::Escalating);
        let thread = make_thread_for_tension(&tension, 500); // Not fatigued

        let tensions = vec![tension];
        let threads = vec![thread];

        // Current focus is on the agent from this tension
        let current_focus = CameraFocus::primary("agent_mira");

        let instruction = selector.select_focus(
            &tensions,
            &threads,
            Some(&current_focus),
            &[],
            test_timestamp(1000),
        );

        // Should continue focus
        assert!(instruction.reason.contains("Continuing focus"));
        assert_eq!(instruction.tension_id, Some("tens_001".to_string()));
    }
}
