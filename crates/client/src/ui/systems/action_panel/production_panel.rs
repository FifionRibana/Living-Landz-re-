use std::collections::HashMap;

use bevy::prelude::*;
use shared::ActionStatusEnum;

use crate::camera::resources::{CellSceneRenderTarget, SceneRenderTarget};
use crate::state::resources::{UnitWorkState, UnitsDataCache};
use crate::states::GameView;
use crate::ui::frosted_glass::{FrostedGlassConfig, FrostedGlassMaterial};
use crate::ui::resources::UIState;
use crate::{
    state::resources::{ActionTracker, TrackedAction},
    ui::resources::CellState,
};

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

#[derive(Component)]
pub struct ProductionSlotClickable;

#[derive(Component)]
pub struct ProductionSlotPortraits {
    pub slot_index: usize,
}

pub fn setup_production_panel(
    mut commands: Commands,
    cell_state: Res<CellState>,
    existing: Query<Entity, With<ProductionPanel>>,
    mut materials: ResMut<Assets<FrostedGlassMaterial>>,
    render_target: Res<SceneRenderTarget>,
    cell_render_target: Res<CellSceneRenderTarget>,
    game_view: Res<State<GameView>>,
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
    let scene_texture = match game_view.get() {
        GameView::Cell => cell_render_target
            .handle
            .clone()
            .unwrap_or_else(|| render_target.0.clone()),
        _ => render_target.0.clone(),
    };

    let panel_material = materials.add(FrostedGlassMaterial::from(
        FrostedGlassConfig::dialog()
            .with_border_radius(8.0)
            .with_scene_texture(scene_texture.clone()),
    ));

    commands
        .spawn((
            // MaterialNode(panel_material),
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(10.0),
                top: Val::Px(74.0),
                bottom: Val::Px(240.0),
                width: Val::Px(160.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            // BorderRadius::all(Val::Px(8.0)),
            // BorderColor::all(Color::srgba_u8(235, 225, 209, 120)),
            ProductionPanel,
            Pickable::IGNORE,
            // Pickable {
            //     should_block_lower: false,
            //     is_hoverable: false,
            // },
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

            let slot_material = materials.add(FrostedGlassMaterial::from(
                FrostedGlassConfig::card()
                    .with_border_radius(4.0)
                    .with_scene_texture(scene_texture.clone()),
            ));
            // Slots
            for i in 0..max_lines {
                panel
                    .spawn((
                        MaterialNode(slot_material.clone()),
                        Node {
                            width: Val::Percent(100.0),
                            padding: UiRect::all(Val::Px(6.0)),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(4.0),
                            ..default()
                        },
                        BorderRadius::all(Val::Px(4.0)),
                        BorderColor::all(Color::srgba_u8(235, 225, 209, 80)),
                        ProductionSlotEntry {
                            slot_index: i as usize,
                            action_id: None,
                        },
                        Interaction::default(),
                        ProductionSlotClickable,
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

                        // Unit portraits container
                        slot.spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(2.0),
                                height: Val::Px(24.0),
                                ..default()
                            },
                            ProductionSlotPortraits {
                                slot_index: i as usize,
                            },
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
    mut commands: Commands,
    action_tracker: Res<ActionTracker>,
    cell_state: Res<CellState>,
    unit_work_state: Res<UnitWorkState>,
    units_data_cache: Res<UnitsDataCache>,
    asset_server: Res<AssetServer>,
    portrait_containers: Query<(Entity, &ProductionSlotPortraits, Option<&Children>)>,
    mut slot_query: Query<(&mut ProductionSlotEntry, &Children)>,
    mut text_query: Query<(&mut Text, &mut TextColor), With<ProductionSlotTimerText>>,
    mut bar_query: Query<(&mut Node, &mut BackgroundColor), With<ProductionSlotProgressBar>>,
    mut displayed_units: Local<HashMap<usize, Vec<u64>>>,
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

                    let icon = match action.action_type {
                        shared::ActionTypeEnum::CraftResource => "⚒",
                        shared::ActionTypeEnum::HarvestResource => "🌾",
                        shared::ActionTypeEnum::TrainUnit => "⚔",
                        shared::ActionTypeEnum::BuildBuilding => "🏗",
                        _ => "⏳",
                    };

                    // Use recipe name if available, else action type name
                    let name = action
                        .action_name
                        .as_deref()
                        .unwrap_or(action.action_type.to_name());

                    **text = format!("{} {} — {}:{:02}", icon, name, minutes, seconds,);
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

    // Update portraits in each slot
    for (container_entity, portraits_comp, children) in portrait_containers.iter() {
        let action = active_actions.get(portraits_comp.slot_index);
        
        let current_units: Vec<u64> = action
            .map(|a| unit_work_state.units_for_action(a.action_id))
            .unwrap_or_default();

        let prev = displayed_units.get(&portraits_comp.slot_index);
        if prev == Some(&current_units) {
            continue;
        }
        displayed_units.insert(portraits_comp.slot_index, current_units.clone());
        
        // Despawn old portraits
        if let Some(children) = children {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }

        // Find the action for this slot

        if let Some(action) = action {
            // Get units working on this action
            let working_units = unit_work_state.units_for_action(action.action_id);

            commands.entity(container_entity).with_children(|parent| {
                for unit_id in &working_units {
                    if let Some(unit) = units_data_cache.get_unit(*unit_id) {
                        let avatar = unit
                            .avatar_url
                            .clone()
                            .unwrap_or_else(|| "ui/icons/unit_placeholder.png".to_string());

                        parent.spawn((
                            ImageNode {
                                image: asset_server.load(&avatar),
                                ..default()
                            },
                            Node {
                                width: Val::Px(22.0),
                                height: Val::Px(22.0),
                                ..default()
                            },
                            BorderRadius::all(Val::Px(11.0)),
                        ));
                    }
                }
            });
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

pub fn handle_production_slot_click(
    slot_query: Query<
        (&Interaction, &ProductionSlotEntry),
        (Changed<Interaction>, With<ProductionSlotClickable>),
    >,
    mut ui_state: ResMut<UIState>,
    cell_state: Res<CellState>,
) {
    for (interaction, slot_entry) in &slot_query {
        if !matches!(interaction, Interaction::Pressed) {
            continue;
        }

        // Only react on empty slots
        if slot_entry.action_id.is_some() {
            continue;
        }

        info!(
            "Production slot {} clicked — opening action panel",
            slot_entry.slot_index
        );

        // Open the production action mode
        if let Some(building_data) = cell_state.building_data {
            if let Some(_building_type) = building_data.to_building_type() {
                ui_state.action_mode = Some(shared::ActionModeEnum::ProductionActionMode);
            }
        }
    }
}
