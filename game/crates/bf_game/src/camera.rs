use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

use bf_types::hex::hex_to_pixel;
use crate::game::GameSystems;
use crate::log;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
        app.add_systems(Update, camera_controls.in_set(GameSystems::Render));
    }
}

fn setup_camera(mut commands: Commands) {
    let (cx, cz) = hex_to_pixel(16, 16);
    let look_at = Vec3::new(cx, 0.0, cz);
    let cam_pos = look_at + Vec3::new(50.0, 50.0, 50.0);

    log!(
        "[game] Camera: look_at={:?}, pos={:?}, map center hex=(16,16) -> pixel=({},{})",
        look_at, cam_pos, cx, cz
    );

    commands.spawn((
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 80.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(cam_pos.x, cam_pos.y, cam_pos.z)
            .looking_at(look_at, Vec3::Y),
    ));

    let offsets = [
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(20.0, 0.0, 0.0),
        Vec3::new(-20.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 20.0),
        Vec3::new(0.0, 0.0, -20.0),
    ];
    for offset in offsets {
        commands.spawn((
            PointLight {
                intensity: 3_000_000.0,
                range: 80.0,
                ..default()
            },
            Transform::from_xyz(
                look_at.x + offset.x,
                25.0,
                look_at.z + offset.z,
            ),
        ));
    }
}

fn camera_controls(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut scroll_events: EventReader<MouseWheel>,
    windows: Query<&Window>,
    mut camera: Query<(&mut Transform, &mut Projection), With<Camera3d>>,
    time: Res<Time>,
    mut last_cursor: Local<Option<Vec2>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((mut transform, mut projection)) = camera.single_mut() else {
        return;
    };

    let cam_scale = if let Projection::Orthographic(ref ortho) = *projection {
        match ortho.scaling_mode {
            ScalingMode::FixedVertical { viewport_height } => viewport_height,
            _ => ortho.scale,
        }
    } else {
        1.0
    };

    let speed = 0.3 * cam_scale * time.delta_secs();
    let forward = Vec3::new(transform.forward().x, 0.0, transform.forward().z).normalize_or_zero();
    let right = Vec3::new(transform.right().x, 0.0, transform.right().z).normalize_or_zero();

    let mut move_dir = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        move_dir += forward;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        move_dir -= forward;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        move_dir -= right;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        move_dir += right;
    }

    if move_dir != Vec3::ZERO {
        transform.translation += move_dir.normalize() * speed;
    }

    for event in scroll_events.read() {
        if let Projection::Orthographic(ref mut ortho) = *projection {
            let factor = if event.y > 0.0 { 0.9 } else { 1.1 };
            if let ScalingMode::FixedVertical { ref mut viewport_height } = ortho.scaling_mode {
                *viewport_height = (*viewport_height * factor).clamp(10.0, 150.0);
            } else {
                ortho.scale = (ortho.scale * factor).clamp(5.0, 80.0);
            }
        }
    }

    let dragging = mouse_button.pressed(MouseButton::Left)
        || mouse_button.pressed(MouseButton::Middle)
        || mouse_button.pressed(MouseButton::Right);

    if let Some(cursor) = window.cursor_position() {
        if dragging {
            if let Some(last) = *last_cursor {
                let delta = cursor - last;
                let pan_speed = cam_scale * 0.002;
                transform.translation -= right * delta.x * pan_speed;
                transform.translation += forward * delta.y * pan_speed;
            }
        }
        *last_cursor = Some(cursor);
    }

    if !dragging {
        *last_cursor = None;
    }
}
