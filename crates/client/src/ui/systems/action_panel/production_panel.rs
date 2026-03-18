use bevy::prelude::*;
use shared::ActionStatusEnum;

use crate::{state::resources::{ActionTracker, TrackedAction}, ui::resources::CellState};

#[derive(Component)]
pub struct ProductionPanel;

#[derive(Component)]
pub struct ProductionSlotEntry {
    pub slot_index: usize,
    pub action_id: Option<u64>,
}

#[derive(Component)]
pub struct ProductionSlotProgressBar;

#[derive(Component)]
pub struct ProductionSlotTimerText;

pub fn setup_production_panel(
    mut commands: Commands,
    cell_state: Res<CellState>,
    existing: Query<Entity, With<ProductionPanel>>,
) {
    // Only rebuild when cell changes
    if !cell_state.is_changed() {
        return;
    }

    // Despawn existing
    for entity in &existing {
        commands.entity(entity).despawn();
    }

    // Only show if building has production lines
    let Some(building_data) = cell_state.building_data else {
        return;
    };
    let Some(building_type) = building_data.to_building_type() else {
        return;
    };

    let max_lines = building_type.production_lines();
    if max_lines == 0 {
        return;
    }

    // Panel container — right side
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(10.0),
                top: Val::Px(74.0),     // below top bar
                bottom: Val::Px(240.0), // above action panel
                width: Val::Px(160.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            ProductionPanel,
            Pickable {
                should_block_lower: false,
                is_hoverable: false,
            },
        ))
        .with_children(|panel| {
            // Title
            panel.spawn((
                Text::new("Production"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgba_u8(235, 225, 209, 200)),
            ));

            // Slots
            for i in 0..max_lines {
                panel
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            padding: UiRect::all(Val::Px(6.0)),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(4.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba_u8(0, 0, 0, 80)),
                        BorderRadius::all(Val::Px(4.0)),
                        ProductionSlotEntry {
                            slot_index: i as usize,
                            action_id: None,
                        },
                    ))
                    .with_children(|slot| {
                        // Action name (or "Disponible")
                        slot.spawn((
                            Text::new("Disponible"),
                            TextFont {
                                font_size: 10.0,
                                ..default()
                            },
                            TextColor(Color::srgba_u8(180, 170, 150, 180)),
                            ProductionSlotTimerText,
                        ));

                        // Progress bar background
                        slot.spawn(Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(4.0),
                            ..default()
                        })
                        .with_children(|bar_bg| {
                            bar_bg.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba_u8(40, 40, 40, 120)),
                                BorderRadius::all(Val::Px(2.0)),
                            ));
                        });

                        // Progress bar fill (overlaid)
                        slot.spawn((
                            Node {
                                width: Val::Percent(0.0),
                                height: Val::Px(4.0),
                                position_type: PositionType::Absolute,
                                bottom: Val::Px(6.0),
                                left: Val::Px(6.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba_u8(50, 150, 255, 200)),
                            BorderRadius::all(Val::Px(2.0)),
                            ProductionSlotProgressBar,
                        ));
                    });
            }
        });
}

pub fn update_production_panel(
    action_tracker: Res<ActionTracker>,
    cell_state: Res<CellState>,
    mut slot_query: Query<(&mut ProductionSlotEntry, &Children)>,
    mut text_query: Query<(&mut Text, &mut TextColor), With<ProductionSlotTimerText>>,
    mut bar_query: Query<(&mut Node, &mut BackgroundColor), With<ProductionSlotProgressBar>>,
) {
    let Some(cell) = cell_state.cell() else {
        return;
    };
    let Some(cell_data) = &cell_state.cell_data else {
        return;
    };

    let chunk_id = cell_data.chunk;

    // Get active actions on this cell (Pending + InProgress only)
    let mut active_actions: Vec<&TrackedAction> = action_tracker
        .get_actions_on_cell(&chunk_id, &cell)
        .into_iter()
        .filter(|a| {
            matches!(
                a.status,
                ActionStatusEnum::Pending | ActionStatusEnum::InProgress
            )
        })
        .collect();
    active_actions.sort_by_key(|a| a.start_time);

    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    for (mut slot_entry, children) in slot_query.iter_mut() {
        let action = active_actions.get(slot_entry.slot_index);
        slot_entry.action_id = action.map(|a| a.action_id);

        for child in children.iter() {
            // Update text
            if let Ok((mut text, mut color)) = text_query.get_mut(child) {
                if let Some(action) = action {
                    let remaining = action.completion_time.saturating_sub(current_time);
                    let minutes = remaining / 60;
                    let seconds = remaining % 60;
                    **text = format!(
                        "{} — {}:{:02}",
                        action.action_type.to_name(),
                        minutes,
                        seconds,
                    );
                    *color = match action.status {
                        ActionStatusEnum::InProgress => TextColor(Color::srgb_u8(200, 220, 255)),
                        ActionStatusEnum::Pending => TextColor(Color::srgb_u8(255, 220, 150)),
                        _ => TextColor(Color::srgba_u8(180, 170, 150, 180)),
                    };
                } else {
                    **text = "Disponible".to_string();
                    *color = TextColor(Color::srgba_u8(180, 170, 150, 180));
                }
            }

            // Update progress bar
            if let Ok((mut node, mut bg)) = bar_query.get_mut(child) {
                if let Some(action) = action {
                    let progress = action.progress(current_time);
                    node.width = Val::Percent(progress * 100.0);
                    *bg = BackgroundColor(match action.status {
                        ActionStatusEnum::InProgress => Color::srgba_u8(50, 150, 255, 200),
                        ActionStatusEnum::Pending => Color::srgba_u8(255, 200, 50, 180),
                        _ => Color::srgba_u8(80, 220, 80, 200),
                    });
                } else {
                    node.width = Val::Percent(0.0);
                }
            }
        }
    }
}

pub fn cleanup_production_panel(
    mut commands: Commands,
    query: Query<Entity, With<ProductionPanel>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}