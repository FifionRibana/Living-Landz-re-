use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy_ui_text_input::TextInputNode;

use crate::camera::components::MainCamera;
use crate::camera::resources::CameraSettings;

pub fn camera_movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    settings: Res<CameraSettings>,
    mut camera: Query<(&mut Transform, &Projection), With<MainCamera>>,
    input_focus: Res<InputFocus>,
    text_inputs: Query<(), With<TextInputNode>>,
) {
    // Don't move camera if any text input has focus
    if let Some(focused_entity) = input_focus.0
        && text_inputs.get(focused_entity).is_ok()
    {
        return;
    }

    let Ok((mut transform, projection)) = camera.single_mut() else {
        return;
    };

    let scale = if let Projection::Orthographic(ortho) = projection {
        ortho.scale
    } else {
        1.0
    };

    let speed = settings.speed * time.delta_secs() * scale;

    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        transform.translation.y += speed;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        transform.translation.y -= speed;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        transform.translation.x -= speed;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        transform.translation.x += speed;
    }
}

pub fn camera_zoom(
    mut scroll: MessageReader<bevy::input::mouse::MouseWheel>,
    keys: Res<ButtonInput<KeyCode>>,
    settings: Res<CameraSettings>,
    mut camera: Query<&mut Projection, With<MainCamera>>,
) {
    let Ok(mut projection) = camera.single_mut() else {
        return;
    };

    if keys.pressed(KeyCode::NumpadAdd) {
        if let Projection::Orthographic(ortho) = projection.as_mut() {
            ortho.scale -= 0.1;
            ortho.scale = ortho.scale.clamp(settings.min_zoom, settings.max_zoom);
        }
    } else if keys.pressed(KeyCode::NumpadSubtract) {
        if let Projection::Orthographic(ortho) = projection.as_mut() {
            ortho.scale += 0.1;
            ortho.scale = ortho.scale.clamp(settings.min_zoom, settings.max_zoom);
        }
    }

    for event in scroll.read() {
        if let Projection::Orthographic(ortho) = projection.as_mut() {
            ortho.scale -= event.y * settings.zoom_speed * 0.1;
            ortho.scale = ortho.scale.clamp(settings.min_zoom, settings.max_zoom);
        }
    }
}
