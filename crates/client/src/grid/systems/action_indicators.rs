use std::f32;

use bevy::prelude::*;
use hexx::Hex;
use shared::{
    ActionStatusEnum, ActionTypeEnum, TerrainChunkId,
    grid::{GridCell, GridConfig},
};

use crate::{
    grid::components::{
        ActionIndicator, CompletedIndicator, InProgressIndicator, PendingIndicator, ProgressBarFill,
    },
    state::resources::ActionTracker,
};

const BAR_WIDTH: f32 = 40.0;
const BAR_HEIGHT: f32 = 5.0;
const BAR_Y_OFFSET: f32 = 25.0; // above hex center

/// Système pour créer/mettre à jour les indicateurs visuels d'actions sur les cellules
pub fn update_action_indicators(
    mut commands: Commands,
    mut action_tracker: ResMut<ActionTracker>,
    grid_config: Res<GridConfig>,
    existing_indicators: Query<(Entity, &ActionIndicator)>,
    asset_server: Res<AssetServer>,
) {
    let mut existing_map = std::collections::HashMap::new();
    for (entity, indicator) in existing_indicators.iter() {
        existing_map.insert(indicator.action_id, (entity, indicator.status));
    }

    // Track completed actions to remove from tracker after processing
    let mut completed_to_remove = Vec::new();

    for action in action_tracker.get_all_actions() {
        match existing_map.get(&action.action_id) {
            Some((entity, old_status)) => {
                if *old_status != action.status {
                    info!("Action {} status changed {:?} → {:?}", action.action_id, old_status, action.status);
                    commands.entity(*entity).despawn();
                    spawn_action_indicator(
                        &mut commands,
                        &grid_config,
                        &asset_server,
                        action.action_id,
                        &action.chunk_id,
                        &action.cell,
                        action.status,
                        action.action_type,
                    );

                    if action.status == ActionStatusEnum::Completed {
                        completed_to_remove.push(action.action_id);
                    }
                }
            }
            None => {
                spawn_action_indicator(
                    &mut commands,
                    &grid_config,
                    &asset_server,
                    action.action_id,
                    &action.chunk_id,
                    &action.cell,
                    action.status,
                    action.action_type,
                );

                if action.status == ActionStatusEnum::Completed {
                    completed_to_remove.push(action.action_id);
                }
            }
        }
    }

    // Clean up indicators for actions that no longer exist
    // BUT skip Completed indicators — they self-cleanup after 3s
    for (entity, indicator) in existing_indicators.iter() {
        if indicator.status == ActionStatusEnum::Completed {
            continue;
        }
        if action_tracker.get_action(indicator.action_id).is_none() {
            commands.entity(entity).despawn();
        }
    }

    // Remove completed actions from tracker so they don't trigger respawn
    for action_id in completed_to_remove {
        action_tracker.remove_action(action_id);
    }
}

fn spawn_action_indicator(
    commands: &mut Commands,
    grid_config: &GridConfig,
    asset_server: &AssetServer,
    action_id: u64,
    chunk_id: &TerrainChunkId,
    cell: &GridCell,
    status: ActionStatusEnum,
    action_type: ActionTypeEnum,
) {
    let hex = Hex::new(cell.q, cell.r);
    let world_pos = grid_config.layout.hex_to_world_pos(hex);
    let position = Vec3::new(world_pos.x, world_pos.y + BAR_Y_OFFSET, 0.5);

    // Colors based on status
    let (bar_color, bg_color) = match status {
        ActionStatusEnum::Pending => (
            Color::srgba_u8(255, 200, 50, 200), // yellow
            Color::srgba_u8(40, 40, 40, 160),
        ),
        ActionStatusEnum::InProgress => (
            Color::srgba_u8(50, 150, 255, 220), // blue
            Color::srgba_u8(40, 40, 40, 180),
        ),
        ActionStatusEnum::Completed => (
            Color::srgba_u8(80, 220, 80, 220), // green
            Color::srgba_u8(40, 40, 40, 160),
        ),
        ActionStatusEnum::Failed => (
            Color::srgba_u8(220, 50, 50, 220), // red
            Color::srgba_u8(40, 40, 40, 160),
        ),
    };

    // Initial fill width
    let fill_width = match status {
        ActionStatusEnum::Pending => 0.0,
        ActionStatusEnum::Completed => BAR_WIDTH,
        _ => 0.0, // will be updated by update system
    };

    let mut entity = commands.spawn((
        Transform::from_translation(position),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ActionIndicator {
            action_id,
            chunk_id: *chunk_id,
            cell: *cell,
            status,
        },
    ));

    entity.with_children(|parent| {
        // Background bar (full width, dark)
        parent.spawn((
            Sprite {
                color: bg_color,
                custom_size: Some(Vec2::new(BAR_WIDTH, BAR_HEIGHT)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ));

        // Fill bar (starts at 0 or full width, colored)
        parent.spawn((
            Sprite {
                color: bar_color,
                custom_size: Some(Vec2::new(fill_width, BAR_HEIGHT)),
                ..default()
            },
            Transform::from_translation(Vec3::new(-BAR_WIDTH / 2.0 + fill_width / 2.0, 0.0, 0.1)),
            ProgressBarFill,
        ));

        // Timer text (above bar)
        if matches!(
            status,
            ActionStatusEnum::InProgress | ActionStatusEnum::Pending
        ) {
            parent.spawn((
                Text2d::new(""),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Transform::from_translation(Vec3::new(0.0, 8.0, 0.1)),
                ActionTimerText,
            ));
        }

        // Action type icon (left of bar)
        // TODO: Probably add icons
        // let icon_path = match action_type {
        //     ActionTypeEnum::BuildBuilding => "ui/icons/action_build.png",
        //     ActionTypeEnum::BuildRoad => "ui/icons/action_road.png",
        //     ActionTypeEnum::HarvestResource => "ui/icons/action_harvest.png",
        //     ActionTypeEnum::CraftResource => "ui/icons/action_craft.png",
        //     ActionTypeEnum::TrainUnit => "ui/icons/action_train.png",
        //     ActionTypeEnum::MoveUnit => "ui/icons/action_move.png",
        //     _ => "ui/icons/action_default.png",
        // };

        // // Only spawn icon if asset exists — otherwise skip
        // parent.spawn((
        //     Sprite {
        //         image: asset_server.load(icon_path),
        //         custom_size: Some(Vec2::new(12.0, 12.0)),
        //         ..default()
        //     },
        //     Transform::from_translation(Vec3::new(-BAR_WIDTH / 2.0 - 10.0, 0.0, 0.1)),
        // ));
    });

    // Add status markers
    match status {
        ActionStatusEnum::Pending | ActionStatusEnum::Failed => {
            entity.insert(PendingIndicator);
        }
        ActionStatusEnum::InProgress => {
            entity.insert(InProgressIndicator);
        }
        ActionStatusEnum::Completed => {
            entity.insert(CompletedIndicator {
                completed_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            });
        }
    }
}

#[derive(Component)]
pub struct ActionTimerText;

/// Update progress bar fill and timer text
pub fn update_action_progress(
    action_tracker: Res<ActionTracker>,
    indicator_query: Query<(&ActionIndicator, &Children), With<InProgressIndicator>>,
    mut fill_query: Query<(&mut Sprite, &mut Transform), With<ProgressBarFill>>,
    mut text_query: Query<&mut Text2d, With<ActionTimerText>>,
) {
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    for (indicator, children) in indicator_query.iter() {
        let Some(action) = action_tracker.get_action(indicator.action_id) else {
            continue;
        };

        let progress = action.progress(current_time);
        let remaining = action.completion_time.saturating_sub(current_time);
        let minutes = remaining / 60;
        let seconds = remaining % 60;

        for child in children.iter() {
            // Update fill bar width
            if let Ok((mut sprite, mut transform)) = fill_query.get_mut(child) {
                let fill = BAR_WIDTH * progress;
                sprite.custom_size = Some(Vec2::new(fill, BAR_HEIGHT));
                transform.translation.x = -BAR_WIDTH / 2.0 + fill / 2.0;
            }

            // Update timer text
            if let Ok(mut text) = text_query.get_mut(child) {
                **text = format!("{}:{:02}", minutes, seconds);
            }
        }
    }
}

/// Animate InProgress indicators (subtle pulse on fill bar opacity)
pub fn animate_in_progress_indicators(
    time: Res<Time>,
    indicator_query: Query<&Children, With<InProgressIndicator>>,
    mut fill_query: Query<&mut Sprite, With<ProgressBarFill>>,
) {
    let t = time.elapsed_secs();
    let alpha = 0.8 + (t * std::f32::consts::PI * 2.0).sin() * 0.2;

    for children in indicator_query.iter() {
        for child in children.iter() {
            if let Ok(mut sprite) = fill_query.get_mut(child) {
                sprite.color.set_alpha(alpha);
            }
        }
    }
}

/// Auto-cleanup completed indicators after a delay
pub fn cleanup_completed_indicators(
    mut commands: Commands,
    query: Query<(Entity, &CompletedIndicator)>,
) {
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    for (entity, completed) in query.iter() {
        let age = current_time - completed.completed_at;
        if age > 3 {
            commands.entity(entity).despawn();
        }
    }
}