use bevy::prelude::*;

use crate::ui::{components::PanelContainer, resources::PanelEnum, systems::panels::components::RecordsPanel};

pub fn setup_records_panel(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.95)), // Dark semi-transparent background
        Visibility::Hidden,                                 // Hidden by default),
        PanelContainer{ panel: PanelEnum::RecordsPanel },
        RecordsPanel
    )).with_children(|parent| {
        parent.spawn((
            Text::new("RECORDS"),
            TextFont {
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });
}
