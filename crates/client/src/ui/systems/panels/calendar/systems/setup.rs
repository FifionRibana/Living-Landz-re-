use bevy::prelude::*;
use bevy::state::state_scoped::DespawnOnExit;

use crate::states::GameView;
use crate::ui::systems::panels::components::CalendarPanel;

pub fn setup_calendar_panel(mut commands: Commands) {
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
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.95)),
        DespawnOnExit(GameView::Calendar),
        CalendarPanel,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("CALENDAR"),
            TextFont {
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });
}
