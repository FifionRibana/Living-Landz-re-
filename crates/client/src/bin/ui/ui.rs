use bevy::{prelude::*, window::PrimaryWindow};
use shared::grid::GridConfig;

use crate::camera::MainCamera;

#[derive(Component)]
pub struct CameraPositionText;

#[derive(Component)]
pub struct HoveredCellInfoText;

pub fn setup_debug_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Column,
                padding: UiRect {
                    left: Val::Px(10.),
                    top: Val::Px(10.),
                    right: Val::Px(10.),
                    bottom: Val::Px(10.),
                },
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Position: --"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 1.0)),
                Node { ..default() },
                CameraPositionText,
            ));

            parent.spawn((
                Text::new("Cell: --"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 1.0)),
                Node { ..default() },
                HoveredCellInfoText,
            ));
        });
}

pub fn update_debug_ui(
    grid_config: Res<GridConfig>,
    camera_query: Query<(&Camera, &GlobalTransform, &Transform), With<MainCamera>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut text_query: Query<(
        &mut Text,
        Option<&CameraPositionText>,
        Option<&HoveredCellInfoText>,
    )>,
) {
    let Ok((camera, global_transform, transform)) = camera_query.single() else {
        return;
    };

    let Ok(window) = windows.single() else {
        return;
    };

    let position = window
        .cursor_position()
        .and_then(|p| camera.viewport_to_world_2d(global_transform, p).ok())
        .unwrap_or_default();

    for (mut text, position_query, hovered_cell_query) in &mut text_query {
        if position_query.is_some() {
            **text = format!(
                "Position: ({:.0}, {:.0})",
                transform.translation.x, transform.translation.y
            );
        } else if hovered_cell_query.is_some() {
            let hovered_cell = grid_config
                .layout
                .world_pos_to_hex(Vec2::new(position.x, position.y));

            **text = format!("Cell: (q: {}, r: {})", hovered_cell.x, hovered_cell.y);
        }
    }
}
