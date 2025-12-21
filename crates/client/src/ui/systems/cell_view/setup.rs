use bevy::prelude::*;
use crate::ui::components::CellViewContainer;

/// Setup the cell view UI container (runs once at startup)
pub fn setup_cell_view_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.95)), // Dark semi-transparent background
            Visibility::Hidden, // Hidden by default
            CellViewContainer,
        ))
        .with_children(|parent| {
            // Main content container
            parent.spawn((
                Node {
                    width: Val::Percent(90.0),
                    height: Val::Percent(85.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(20.0),
                    ..default()
                },
            ));
        });

    info!("Cell view UI container created");
}
