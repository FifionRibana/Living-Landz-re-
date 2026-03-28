use bevy::prelude::*;

use crate::state::resources::{NotificationKind, NotificationState};

/// Marker for the notification container (anchored bottom-center).
#[derive(Component)]
pub struct NotificationContainer;

/// Marker for individual toast entities, with their spawn time.
#[derive(Component)]
pub struct ToastNotification {
    pub spawned_at: f64,
}

const TOAST_DURATION_SECS: f64 = 4.0;
const TOAST_FADE_SECS: f64 = 0.5;

/// Setup the notification container (call once, e.g. OnEnter(InGame)).
pub fn setup_notification_container(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            left: Val::Percent(50.0),
            width: Val::Px(0.0),
            flex_direction: FlexDirection::ColumnReverse,
            align_items: AlignItems::Center,
            row_gap: Val::Px(8.0),
            ..default()
        },
        GlobalZIndex(2000),
        NotificationContainer,
    ));
}

/// Spawn toast entities from the notification queue.
pub fn spawn_notifications(
    mut commands: Commands,
    mut state: ResMut<NotificationState>,
    time: Res<Time>,
    container_query: Query<Entity, With<NotificationContainer>>,
) {
    if state.queue.is_empty() {
        return;
    }

    let Ok(container) = container_query.single() else {
        return;
    };

    while let Some(mut notif) = state.queue.pop_front() {
        notif.spawned_at = time.elapsed_secs_f64();

        let (bg_color, text_color) = match notif.kind {
            NotificationKind::Error => (
                Color::srgba_u8(140, 30, 30, 230),
                Color::srgb_u8(255, 220, 220),
            ),
            NotificationKind::Success => (
                Color::srgba_u8(30, 120, 50, 230),
                Color::srgb_u8(220, 255, 220),
            ),
            NotificationKind::Info => (
                Color::srgba_u8(40, 60, 100, 230),
                Color::srgb_u8(220, 230, 255),
            ),
        };

        let toast = commands
            .spawn((
                Node {
                    padding: UiRect::new(
                        Val::Px(20.0),
                        Val::Px(20.0),
                        Val::Px(10.0),
                        Val::Px(10.0),
                    ),
                    max_width: Val::Px(450.0),
                    margin: UiRect::left(Val::Px(-225.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(bg_color),
                ToastNotification {
                    spawned_at: notif.spawned_at,
                },
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new(&notif.message),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(text_color),
                ));
            })
            .id();

        commands.entity(container).add_child(toast);
    }
}

/// Auto-despawn toasts after duration. Fade out during the last TOAST_FADE_SECS.
pub fn despawn_notifications(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &ToastNotification, &mut BackgroundColor)>,
) {
    let now = time.elapsed_secs_f64();

    for (entity, toast, mut bg) in &mut query {
        let age = now - toast.spawned_at;

        if age >= TOAST_DURATION_SECS {
            commands.entity(entity).despawn();
        } else if age >= TOAST_DURATION_SECS - TOAST_FADE_SECS {
            let fade = ((TOAST_DURATION_SECS - age) / TOAST_FADE_SECS) as f32;
            let mut color = bg.0;
            color.set_alpha(color.alpha() * fade);
            *bg = BackgroundColor(color);
        }
    }
}
