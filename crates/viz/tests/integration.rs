//! Integration tests for the visualization layer.

use sim_events::WorldSnapshot;

/// Test loading and parsing sample state file.
#[test]
fn test_parse_sample_state() {
    let json = include_str!("../../../crates/sim-events/tests/fixtures/sample_state.json");
    let snapshot: WorldSnapshot = serde_json::from_str(json).unwrap();

    assert!(!snapshot.agents.is_empty());
    assert!(!snapshot.factions.is_empty());
    assert!(!snapshot.locations.is_empty());
    assert_eq!(snapshot.timestamp.tick, 91000);
}

/// Test parsing camera script fixture.
#[test]
fn test_parse_camera_script() {
    let json = include_str!("fixtures/sample_camera_script.json");
    let instructions: Vec<director::CameraInstruction> = serde_json::from_str(json).unwrap();

    assert_eq!(instructions.len(), 3);

    // Check first instruction
    assert_eq!(instructions[0].instruction_id, "cam_91000_0001");
    assert!(matches!(
        instructions[0].camera_mode,
        director::CameraMode::FollowAgent { .. }
    ));

    // Check pacing values
    assert_eq!(instructions[0].pacing, director::PacingHint::Normal);
    assert_eq!(instructions[1].pacing, director::PacingHint::Climactic);
    assert_eq!(instructions[2].pacing, director::PacingHint::Slow);
}

/// Test parsing commentary fixture.
#[test]
fn test_parse_commentary() {
    let json = include_str!("fixtures/sample_commentary.json");
    let items: Vec<director::CommentaryItem> = serde_json::from_str(json).unwrap();

    assert_eq!(items.len(), 3);

    // Check first item
    assert_eq!(items[0].item_id, "com_91000_0001");
    assert_eq!(
        items[0].commentary_type,
        director::CommentaryType::EventCaption
    );
    assert_eq!(items[0].related_agents.len(), 1);

    // Check dramatic irony item
    assert_eq!(
        items[1].commentary_type,
        director::CommentaryType::DramaticIrony
    );
    assert!(items[1].related_tension.is_some());

    // Check tension teaser
    assert_eq!(
        items[2].commentary_type,
        director::CommentaryType::TensionTeaser
    );
}

/// Test camera mode variants.
#[test]
fn test_camera_mode_variants() {
    // FollowAgent
    let follow_json = r#"{"type":"follow_agent","agent_id":"agent_001","zoom":"close"}"#;
    let mode: director::CameraMode = serde_json::from_str(follow_json).unwrap();
    assert!(matches!(mode, director::CameraMode::FollowAgent { .. }));

    // FrameLocation
    let frame_json = r#"{"type":"frame_location","location_id":"village_center","zoom":"medium"}"#;
    let mode: director::CameraMode = serde_json::from_str(frame_json).unwrap();
    assert!(matches!(mode, director::CameraMode::FrameLocation { .. }));

    // FrameMultiple
    let multi_json =
        r#"{"type":"frame_multiple","agent_ids":["agent_001","agent_002"],"auto_zoom":true}"#;
    let mode: director::CameraMode = serde_json::from_str(multi_json).unwrap();
    assert!(matches!(mode, director::CameraMode::FrameMultiple { .. }));

    // Overview
    let overview_json = r#"{"type":"overview","region":null}"#;
    let mode: director::CameraMode = serde_json::from_str(overview_json).unwrap();
    assert!(matches!(mode, director::CameraMode::Overview { .. }));
}

/// Test zoom level serialization.
#[test]
fn test_zoom_levels() {
    assert_eq!(
        serde_json::to_string(&director::ZoomLevel::Extreme).unwrap(),
        r#""extreme""#
    );
    assert_eq!(
        serde_json::to_string(&director::ZoomLevel::Close).unwrap(),
        r#""close""#
    );
    assert_eq!(
        serde_json::to_string(&director::ZoomLevel::Medium).unwrap(),
        r#""medium""#
    );
    assert_eq!(
        serde_json::to_string(&director::ZoomLevel::Wide).unwrap(),
        r#""wide""#
    );
    assert_eq!(
        serde_json::to_string(&director::ZoomLevel::Regional).unwrap(),
        r#""regional""#
    );
}

/// Test pacing hint serialization.
#[test]
fn test_pacing_hints() {
    assert_eq!(
        serde_json::to_string(&director::PacingHint::Slow).unwrap(),
        r#""slow""#
    );
    assert_eq!(
        serde_json::to_string(&director::PacingHint::Normal).unwrap(),
        r#""normal""#
    );
    assert_eq!(
        serde_json::to_string(&director::PacingHint::Urgent).unwrap(),
        r#""urgent""#
    );
    assert_eq!(
        serde_json::to_string(&director::PacingHint::Climactic).unwrap(),
        r#""climactic""#
    );
}

/// Test commentary type serialization.
#[test]
fn test_commentary_types() {
    assert_eq!(
        serde_json::to_string(&director::CommentaryType::EventCaption).unwrap(),
        r#""event_caption""#
    );
    assert_eq!(
        serde_json::to_string(&director::CommentaryType::DramaticIrony).unwrap(),
        r#""dramatic_irony""#
    );
    assert_eq!(
        serde_json::to_string(&director::CommentaryType::ContextReminder).unwrap(),
        r#""context_reminder""#
    );
    assert_eq!(
        serde_json::to_string(&director::CommentaryType::TensionTeaser).unwrap(),
        r#""tension_teaser""#
    );
}

/// Test agent snapshot has required fields for visualization.
#[test]
fn test_agent_snapshot_viz_fields() {
    let json = include_str!("../../../crates/sim-events/tests/fixtures/sample_state.json");
    let snapshot: WorldSnapshot = serde_json::from_str(json).unwrap();

    for agent in &snapshot.agents {
        // Required for visualization
        assert!(!agent.agent_id.is_empty());
        assert!(!agent.faction.is_empty());
        assert!(!agent.role.is_empty());
        assert!(!agent.location.is_empty());
    }
}

/// Test location snapshot has required fields for visualization.
#[test]
fn test_location_snapshot_viz_fields() {
    let json = include_str!("../../../crates/sim-events/tests/fixtures/sample_state.json");
    let snapshot: WorldSnapshot = serde_json::from_str(json).unwrap();

    for location in &snapshot.locations {
        // Required for visualization
        assert!(!location.location_id.is_empty());
    }
}

/// Test faction snapshot for color mapping.
#[test]
fn test_faction_snapshot_for_colors() {
    let json = include_str!("../../../crates/sim-events/tests/fixtures/sample_state.json");
    let snapshot: WorldSnapshot = serde_json::from_str(json).unwrap();

    // Each faction should have a unique ID for color mapping
    let faction_ids: std::collections::HashSet<_> = snapshot
        .factions
        .iter()
        .map(|f| f.faction_id.as_str())
        .collect();

    assert_eq!(faction_ids.len(), snapshot.factions.len());
}
