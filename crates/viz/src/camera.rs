//! Camera system: controller, transitions, and user input handling.

use bevy::prelude::*;

/// Plugin for camera control and movement.
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraController>()
            .init_resource::<CameraConstraints>()
            .add_event::<PlayPauseEvent>()
            .add_systems(Startup, setup_camera)
            .add_systems(
                Update,
                (
                    handle_camera_input,
                    handle_keyboard_input,
                    update_camera_transition,
                    apply_camera_to_transform,
                )
                    .chain(),
            );
    }
}

/// Event emitted when play/pause is toggled.
#[derive(Event)]
pub struct PlayPauseEvent;

/// Main camera controller resource.
#[derive(Resource)]
pub struct CameraController {
    /// Current camera position in world coordinates.
    pub position: Vec2,
    /// Current zoom level (1.0 = normal, higher = zoomed in).
    pub zoom: f32,
    /// Target position for smooth movement.
    pub target_position: Vec2,
    /// Target zoom for smooth zooming.
    pub target_zoom: f32,
    /// Current camera control mode.
    pub mode: CameraMode,
    /// Active camera transition, if any.
    pub transition: Option<CameraTransition>,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
            target_position: Vec2::ZERO,
            target_zoom: 1.0,
            mode: CameraMode::Manual,
            transition: None,
        }
    }
}

impl CameraController {
    /// Begin a smooth transition to a new position and zoom.
    pub fn begin_transition(&mut self, to_pos: Vec2, to_zoom: f32, duration: f32) {
        self.transition = Some(CameraTransition {
            from_pos: self.position,
            to_pos,
            from_zoom: self.zoom,
            to_zoom,
            duration,
            elapsed: 0.0,
        });
    }

    /// Check if camera is currently in manual control mode.
    pub fn is_manual(&self) -> bool {
        matches!(self.mode, CameraMode::Manual)
    }

    /// Check if camera is currently following director instructions.
    pub fn is_director(&self) -> bool {
        matches!(self.mode, CameraMode::Director { .. })
    }

    /// Switch to manual mode.
    pub fn set_manual(&mut self) {
        self.mode = CameraMode::Manual;
    }

    /// Switch to director mode.
    pub fn set_director(&mut self) {
        self.mode = CameraMode::Director { instruction: None };
    }
}

/// Camera control mode.
#[derive(Clone, Debug)]
pub enum CameraMode {
    /// User has manual control.
    Manual,
    /// Director AI is controlling the camera.
    Director {
        /// Current instruction being executed.
        instruction: Option<CameraInstruction>,
    },
    /// Camera is following a specific agent.
    FollowAgent {
        /// Agent ID to follow.
        agent_id: String,
    },
}

impl Default for CameraMode {
    fn default() -> Self {
        Self::Manual
    }
}

/// A camera instruction from the Director.
#[derive(Clone, Debug)]
pub struct CameraInstruction {
    /// Target position or agent to focus on.
    pub target: CameraTarget,
    /// Target zoom level.
    pub zoom: f32,
    /// Duration for the transition.
    pub duration: f32,
}

/// What the camera should focus on.
#[derive(Clone, Debug)]
pub enum CameraTarget {
    /// A specific world position.
    Position(Vec2),
    /// Follow a specific agent.
    Agent(String),
    /// Frame multiple agents.
    MultipleAgents(Vec<String>),
    /// A specific location.
    Location(String),
    /// Overview of a region.
    Region { center: Vec2, size: Vec2 },
}

/// Active camera transition state.
#[derive(Clone, Debug)]
pub struct CameraTransition {
    /// Starting position.
    pub from_pos: Vec2,
    /// Target position.
    pub to_pos: Vec2,
    /// Starting zoom.
    pub from_zoom: f32,
    /// Target zoom.
    pub to_zoom: f32,
    /// Total duration in seconds.
    pub duration: f32,
    /// Time elapsed so far.
    pub elapsed: f32,
}

impl CameraTransition {
    /// Get the progress of this transition (0.0 to 1.0).
    pub fn progress(&self) -> f32 {
        if self.duration <= 0.0 {
            1.0
        } else {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        }
    }

    /// Check if this transition is complete.
    pub fn is_complete(&self) -> bool {
        self.elapsed >= self.duration
    }

    /// Get the current position based on progress.
    pub fn current_position(&self) -> Vec2 {
        let t = ease_in_out(self.progress());
        self.from_pos.lerp(self.to_pos, t)
    }

    /// Get the current zoom based on progress.
    pub fn current_zoom(&self) -> f32 {
        let t = ease_in_out(self.progress());
        self.from_zoom + (self.to_zoom - self.from_zoom) * t
    }
}

/// Smooth ease-in-out function for transitions.
pub fn ease_in_out(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
    }
}

/// Convert screen coordinates to world coordinates.
pub fn screen_to_world(
    screen_pos: Vec2,
    window_size: Vec2,
    camera_pos: Vec2,
    camera_zoom: f32,
) -> Vec2 {
    // Convert screen position to normalized device coordinates (-1 to 1)
    let ndc = (screen_pos - window_size / 2.0) / window_size * 2.0;
    // Convert NDC to world offset, accounting for zoom
    let world_offset = ndc * (window_size / 2.0) / camera_zoom;
    // Y is inverted in screen coordinates
    camera_pos + Vec2::new(world_offset.x, -world_offset.y)
}

/// Zoom toward a specific world point.
///
/// This adjusts the camera position so that the point under the cursor
/// remains at the same screen position after zooming.
pub fn zoom_toward_point(
    controller: &mut CameraController,
    cursor_world_pos: Vec2,
    zoom_delta: f32,
    constraints: &CameraConstraints,
) {
    let old_zoom = controller.zoom;
    let new_zoom = constraints.clamp_zoom(old_zoom * (1.0 + zoom_delta));

    // Adjust position so cursor stays at same world point
    // The screen offset of the cursor from center is: (cursor - camera_pos) * zoom
    // For this to remain constant: (cursor - old_pos) * old_zoom = (cursor - new_pos) * new_zoom
    // Solving for new_pos: new_pos = cursor - (cursor - old_pos) * old_zoom / new_zoom
    let zoom_ratio = new_zoom / old_zoom;
    let offset = cursor_world_pos - controller.position;
    controller.position = cursor_world_pos - offset / zoom_ratio;
    controller.zoom = new_zoom;
    controller.target_zoom = new_zoom;
    controller.position = constraints.clamp_position(controller.position);
    controller.target_position = controller.position;
}

/// Camera constraints for zooming and panning.
#[derive(Resource)]
pub struct CameraConstraints {
    /// Minimum zoom level (zoomed out).
    pub min_zoom: f32,
    /// Maximum zoom level (zoomed in).
    pub max_zoom: f32,
    /// Optional bounds to constrain camera position.
    pub bounds: Option<Rect>,
}

impl Default for CameraConstraints {
    fn default() -> Self {
        Self {
            min_zoom: 0.25,
            max_zoom: 4.0,
            bounds: None,
        }
    }
}

impl CameraConstraints {
    /// Clamp a zoom value to valid range.
    pub fn clamp_zoom(&self, zoom: f32) -> f32 {
        zoom.clamp(self.min_zoom, self.max_zoom)
    }

    /// Clamp a position to valid bounds if bounds are set.
    pub fn clamp_position(&self, pos: Vec2) -> Vec2 {
        match &self.bounds {
            Some(bounds) => Vec2::new(
                pos.x.clamp(bounds.min.x, bounds.max.x),
                pos.y.clamp(bounds.min.y, bounds.max.y),
            ),
            None => pos,
        }
    }
}

/// Marker component for the main camera.
#[derive(Component)]
pub struct MainCamera;

/// System to set up the camera on startup.
fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
}

/// System to handle camera input (pan and zoom).
fn handle_camera_input(
    mut controller: ResMut<CameraController>,
    constraints: Res<CameraConstraints>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<bevy::input::mouse::MouseMotion>,
    mut scroll: EventReader<bevy::input::mouse::MouseWheel>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };

    // Handle panning with right or middle mouse button
    let is_panning =
        mouse_button.pressed(MouseButton::Right) || mouse_button.pressed(MouseButton::Middle);

    if is_panning {
        // Switch to manual mode if in director mode
        if controller.is_director() {
            controller.set_manual();
        }

        let mut delta = Vec2::ZERO;
        for motion in mouse_motion.read() {
            delta += motion.delta;
        }

        if delta != Vec2::ZERO {
            // Pan speed is inversely proportional to zoom (faster when zoomed out)
            let pan_speed = 1.0 / controller.zoom;
            controller.position -= Vec2::new(delta.x, -delta.y) * pan_speed;
            controller.position = constraints.clamp_position(controller.position);
            controller.target_position = controller.position;
        }
    } else {
        // Clear motion events if not panning
        mouse_motion.clear();
    }

    // Handle zooming with scroll wheel
    for ev in scroll.read() {
        // Switch to manual mode if in director mode
        if controller.is_director() {
            controller.set_manual();
        }

        let zoom_delta = ev.y * 0.1;
        let old_zoom = controller.zoom;
        let new_zoom = constraints.clamp_zoom(old_zoom * (1.0 + zoom_delta));

        // Zoom toward cursor position
        if let Some(cursor_pos) = window.cursor_position() {
            if let Ok((camera, camera_transform)) = camera_query.get_single() {
                if let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos)
                {
                    // Adjust position so the point under cursor stays fixed
                    let zoom_ratio = new_zoom / old_zoom;
                    let offset = world_pos - controller.position;
                    controller.position = world_pos - offset / zoom_ratio;
                }
            }
        }

        controller.zoom = new_zoom;
        controller.target_zoom = new_zoom;
        controller.position = constraints.clamp_position(controller.position);
        controller.target_position = controller.position;
    }
}

/// System to handle keyboard input for camera controls.
fn handle_keyboard_input(
    mut controller: ResMut<CameraController>,
    constraints: Res<CameraConstraints>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut play_pause_events: EventWriter<PlayPauseEvent>,
) {
    let delta = time.delta_seconds();

    // Check if any camera control key is pressed
    let has_camera_input = keyboard.pressed(KeyCode::ArrowLeft)
        || keyboard.pressed(KeyCode::ArrowRight)
        || keyboard.pressed(KeyCode::ArrowUp)
        || keyboard.pressed(KeyCode::ArrowDown)
        || keyboard.just_pressed(KeyCode::Home)
        || keyboard.just_pressed(KeyCode::Equal)
        || keyboard.just_pressed(KeyCode::Minus);

    if has_camera_input && controller.is_director() {
        controller.set_manual();
    }

    // Arrow key panning
    let shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let base_pan_speed = if shift_held { 800.0 } else { 400.0 };
    let pan_speed = base_pan_speed / controller.zoom * delta;

    let mut pan_delta = Vec2::ZERO;
    if keyboard.pressed(KeyCode::ArrowLeft) {
        pan_delta.x -= pan_speed;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        pan_delta.x += pan_speed;
    }
    if keyboard.pressed(KeyCode::ArrowUp) {
        pan_delta.y += pan_speed;
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        pan_delta.y -= pan_speed;
    }

    if pan_delta != Vec2::ZERO {
        controller.position += pan_delta;
        controller.position = constraints.clamp_position(controller.position);
        controller.target_position = controller.position;
    }

    // Home key - return to world center
    if keyboard.just_pressed(KeyCode::Home) {
        controller.begin_transition(Vec2::ZERO, 1.0, 0.5);
    }

    // +/- keys for discrete zoom steps
    if keyboard.just_pressed(KeyCode::Equal) {
        // = key (often used as + without shift)
        let new_zoom = constraints.clamp_zoom(controller.zoom * 1.25);
        controller.zoom = new_zoom;
        controller.target_zoom = new_zoom;
    }
    if keyboard.just_pressed(KeyCode::Minus) {
        let new_zoom = constraints.clamp_zoom(controller.zoom / 1.25);
        controller.zoom = new_zoom;
        controller.target_zoom = new_zoom;
    }

    // Space to toggle play/pause
    if keyboard.just_pressed(KeyCode::Space) {
        play_pause_events.send(PlayPauseEvent);
    }
}

/// System to update camera transitions.
fn update_camera_transition(mut controller: ResMut<CameraController>, time: Res<Time>) {
    let delta = time.delta_seconds();

    // Check if we have an active transition
    let transition_result = if let Some(ref mut transition) = controller.transition {
        transition.elapsed += delta;

        let position = transition.current_position();
        let zoom = transition.current_zoom();
        let is_complete = transition.is_complete();
        let final_pos = transition.to_pos;
        let final_zoom = transition.to_zoom;

        Some((position, zoom, is_complete, final_pos, final_zoom))
    } else {
        None
    };

    // Apply transition results outside of the borrow
    if let Some((position, zoom, is_complete, final_pos, final_zoom)) = transition_result {
        if is_complete {
            controller.position = final_pos;
            controller.zoom = final_zoom;
            controller.transition = None;
        } else {
            controller.position = position;
            controller.zoom = zoom;
        }
    } else {
        // Smooth interpolation toward target (for follow mode)
        let lerp_speed = 5.0 * delta;
        let target_pos = controller.target_position;
        let target_zoom = controller.target_zoom;
        controller.position = controller.position.lerp(target_pos, lerp_speed);
        controller.zoom = controller.zoom + (target_zoom - controller.zoom) * lerp_speed;
    }
}

/// System to apply camera controller state to the actual camera transform.
fn apply_camera_to_transform(
    controller: Res<CameraController>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    for mut transform in camera_query.iter_mut() {
        transform.translation.x = controller.position.x;
        transform.translation.y = controller.position.y;
        // Zoom is applied via projection scale (inverse relationship)
        let scale = 1.0 / controller.zoom;
        transform.scale = Vec3::new(scale, scale, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ease_in_out() {
        assert_eq!(ease_in_out(0.0), 0.0);
        assert_eq!(ease_in_out(1.0), 1.0);
        // Middle should be around 0.5
        let mid = ease_in_out(0.5);
        assert!((mid - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_camera_transition_progress() {
        let transition = CameraTransition {
            from_pos: Vec2::ZERO,
            to_pos: Vec2::new(100.0, 100.0),
            from_zoom: 1.0,
            to_zoom: 2.0,
            duration: 1.0,
            elapsed: 0.5,
        };

        assert_eq!(transition.progress(), 0.5);
        assert!(!transition.is_complete());
    }

    #[test]
    fn test_camera_transition_complete() {
        let transition = CameraTransition {
            from_pos: Vec2::ZERO,
            to_pos: Vec2::new(100.0, 100.0),
            from_zoom: 1.0,
            to_zoom: 2.0,
            duration: 1.0,
            elapsed: 1.5,
        };

        assert!(transition.is_complete());
        assert_eq!(transition.progress(), 1.0);
    }

    #[test]
    fn test_camera_constraints_clamp_zoom() {
        let constraints = CameraConstraints {
            min_zoom: 0.5,
            max_zoom: 2.0,
            bounds: None,
        };

        assert_eq!(constraints.clamp_zoom(0.25), 0.5);
        assert_eq!(constraints.clamp_zoom(1.0), 1.0);
        assert_eq!(constraints.clamp_zoom(3.0), 2.0);
    }

    #[test]
    fn test_camera_mode_checks() {
        let mut controller = CameraController::default();
        assert!(controller.is_manual());
        assert!(!controller.is_director());

        controller.set_director();
        assert!(!controller.is_manual());
        assert!(controller.is_director());
    }

    #[test]
    fn test_screen_to_world_center() {
        // Screen center should map to camera position
        let window_size = Vec2::new(800.0, 600.0);
        let camera_pos = Vec2::new(100.0, 50.0);
        let screen_center = Vec2::new(400.0, 300.0);

        let world_pos = screen_to_world(screen_center, window_size, camera_pos, 1.0);
        assert!((world_pos - camera_pos).length() < 0.1);
    }

    #[test]
    fn test_zoom_toward_point() {
        let mut controller = CameraController::default();
        let constraints = CameraConstraints::default();

        // Zoom in toward a point to the right of camera
        let cursor_pos = Vec2::new(100.0, 0.0);
        zoom_toward_point(&mut controller, cursor_pos, 0.5, &constraints);

        // Camera should have moved toward the cursor
        assert!(controller.position.x > 0.0);
        assert!(controller.zoom > 1.0);
    }

    #[test]
    fn test_begin_transition() {
        let mut controller = CameraController::default();
        assert!(controller.transition.is_none());

        controller.begin_transition(Vec2::new(100.0, 100.0), 2.0, 1.0);

        assert!(controller.transition.is_some());
        let transition = controller.transition.as_ref().unwrap();
        assert_eq!(transition.to_pos, Vec2::new(100.0, 100.0));
        assert_eq!(transition.to_zoom, 2.0);
        assert_eq!(transition.duration, 1.0);
    }
}
