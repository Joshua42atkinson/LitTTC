// ar_capture.rs — XR Pinch-to-Capture Mechanic for AR Object Identification
// This module enables players to look at real-world objects, pinch to capture them,
// and trigger VLM inference for word identification.

use bevy::prelude::*;

/// Event emitted when a pinch triggers an image capture request
#[derive(Message, Debug, Clone)]
pub struct XRImageCaptureRequest {
    pub world_position: Vec3,
    pub rotation: Quat,
}

/// Component marking the holographic bounding box spawned at capture location
#[derive(Component)]
pub struct CaptureBoundingBox;

/// Component for the "Scanning..." text UI
#[derive(Component)]
pub struct ScanningText;

/// Resource tracking the current AR capture state
#[derive(Resource, Default, Debug, Clone)]
pub struct ARCaptureState {
    pub active_capture: Option<Entity>,
    pub last_capture_time: f32,
}

#[cfg(feature = "xr")]
const PINCH_CAPTURE_COOLDOWN: f32 = 0.5; // Seconds between captures

#[cfg(feature = "xr")]
pub fn handle_xr_pinch_capture(
    mut commands: Commands,
    mut pinch_events: ResMut<crate::hand_tracking::PinchEvents>,
    mut capture_state: ResMut<ARCaptureState>,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut capture_events: MessageWriter<XRImageCaptureRequest>,
) {
    let current_time = time.elapsed_secs();

    // Enforce cooldown
    if current_time - capture_state.last_capture_time < PINCH_CAPTURE_COOLDOWN {
        return;
    }

    for pinch in pinch_events.events.drain(..) {
        // Despawn previous capture if exists
        if let Some(old_entity) = capture_state.active_capture {
            commands.entity(old_entity).despawn();
        }

        // Calculate ray direction from camera through pinch point
        // In a real XR setup, this would use the headset pose
        let _camera_forward = Vec3::new(0.0, 0.0, -1.0);
        let camera_position = Vec3::new(0.0, 0.0, 0.0);
        
        // Cast ray from camera through pinch position to find world point
        // For now, we use the pinch position directly as the target
        let ray_direction = (pinch.position - camera_position).normalize();
        let ray_distance = 2.0; // Default capture distance
        let world_position = camera_position + ray_direction * ray_distance;

        // Spawn holographic bounding box
        let box_mat = materials.add(StandardMaterial {
            base_color: Color::srgba(0.0, 1.0, 1.0, 0.3),
            emissive: Color::srgb(0.0, 0.5, 0.5).into(),
            unlit: true,
            ..default()
        });

        let bounding_box = commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))),
            MeshMaterial3d(box_mat),
            Transform::from_translation(world_position),
            CaptureBoundingBox,
        )).id();

        // Spawn "Scanning..." text
        commands.spawn((
            Text2d::new("SCANNING..."),
            TextFont { font_size: 24.0, ..default() },
            TextColor(Color::srgb(0.0, 1.0, 1.0)),
            Transform::from_xyz(world_position.x, world_position.y + 0.4, world_position.z),
            ScanningText,
        ));

        // Update capture state
        capture_state.active_capture = Some(bounding_box);
        capture_state.last_capture_time = current_time;

        // Emit capture request event
        capture_events.write(XRImageCaptureRequest {
            world_position,
            rotation: Quat::IDENTITY,
        });

        info!("AR Capture triggered at position: {:?}", world_position);
    }
}

#[cfg(feature = "xr")]
pub fn update_capture_box_animation(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<CaptureBoundingBox>>,
) {
    let t = time.elapsed_secs();
    for mut transform in &mut query {
        // Gentle rotation and pulsing to indicate active scanning
        transform.rotation = Quat::from_rotation_y(t * 0.5);
        let scale = 1.0 + (t * 2.0).sin() * 0.1;
        transform.scale = Vec3::splat(scale);
    }
}

#[cfg(not(feature = "xr"))]
pub fn handle_xr_pinch_capture(
    // Desktop fallback: no-op
) {
    // AR capture is XR-only
}

#[cfg(not(feature = "xr"))]
pub fn update_capture_box_animation(
    // Desktop fallback: no-op
) {
    // AR capture is XR-only
}

pub struct ARCapturePlugin;

impl Plugin for ARCapturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ARCaptureState>();
        app.add_message::<XRImageCaptureRequest>();
        app.add_systems(
            Update,
            (
                handle_xr_pinch_capture,
                update_capture_box_animation,
            ).run_if(in_state(crate::components::GameState::Collecting))
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ar_capture_state_defaults_to_no_active_capture() {
        let state = ARCaptureState::default();
        assert!(state.active_capture.is_none());
        assert_eq!(state.last_capture_time, 0.0);
    }

    #[test]
    fn xr_capture_request_contains_position_and_rotation() {
        let request = XRImageCaptureRequest {
            world_position: Vec3::new(1.0, 2.0, 3.0),
            rotation: Quat::IDENTITY,
        };
        assert_eq!(request.world_position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(request.rotation, Quat::IDENTITY);
    }
}
