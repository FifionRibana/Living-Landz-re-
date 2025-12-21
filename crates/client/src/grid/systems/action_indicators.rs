use bevy::prelude::*;
use hexx::Hex;
use shared::{ActionStatusEnum, TerrainChunkId, grid::{GridCell, GridConfig}};

use crate::{
    grid::components::{ActionIndicator, CompletedIndicator, InProgressIndicator, PendingIndicator},
    state::resources::ActionTracker,
};

/// Système pour créer/mettre à jour les indicateurs visuels d'actions sur les cellules
pub fn update_action_indicators(
    mut commands: Commands,
    action_tracker: Res<ActionTracker>,
    grid_config: Res<GridConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    existing_indicators: Query<(Entity, &ActionIndicator)>,
) {
    // Collecter les indicateurs existants
    let mut existing_map = std::collections::HashMap::new();
    for (entity, indicator) in existing_indicators.iter() {
        existing_map.insert(indicator.action_id, (entity, indicator.status));
    }

    // Parcourir toutes les actions actives
    for action in action_tracker.get_all_actions() {
        match existing_map.get(&action.action_id) {
            Some((entity, old_status)) => {
                // L'indicateur existe déjà
                if *old_status != action.status {
                    // Le statut a changé, mettre à jour l'indicateur
                    commands.entity(*entity).despawn();
                    spawn_action_indicator(
                        &mut commands,
                        &grid_config,
                        &mut meshes,
                        &mut materials,
                        action.action_id,
                        &action.chunk_id,
                        &action.cell,
                        action.status,
                    );
                }
            }
            None => {
                // Nouvel indicateur à créer
                spawn_action_indicator(
                    &mut commands,
                    &grid_config,
                    &mut meshes,
                    &mut materials,
                    action.action_id,
                    &action.chunk_id,
                    &action.cell,
                    action.status,
                );
            }
        }
    }

    // Nettoyer les indicateurs des actions qui n'existent plus
    for (entity, indicator) in existing_indicators.iter() {
        if action_tracker.get_action(indicator.action_id).is_none() {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_action_indicator(
    commands: &mut Commands,
    grid_config: &GridConfig,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    action_id: u64,
    chunk_id: &TerrainChunkId,
    cell: &GridCell,
    status: ActionStatusEnum,
) {
    // Calculer la position 2D de la cellule en utilisant le layout de la grille
    let hex = Hex::new(cell.q, cell.r);
    let world_pos = grid_config.layout.hex_to_world_pos(hex);

    // Position avec Z pour le layering (au-dessus des hexagones)
    let position = world_pos.extend(0.5);

    // Créer un petit mesh pour l'indicateur (un petit cercle)
    let indicator_mesh = meshes.add(Circle::new(0.3));

    // Couleur selon le statut
    let color = match status {
        ActionStatusEnum::Pending => Color::srgb(1.0, 1.0, 0.0), // Jaune
        ActionStatusEnum::InProgress => Color::srgb(0.0, 0.5, 1.0), // Bleu
        ActionStatusEnum::Completed => Color::srgb(0.0, 1.0, 0.0), // Vert
        ActionStatusEnum::Failed => Color::srgb(1.0, 0.0, 0.0), // Rouge
    };

    let material = materials.add(ColorMaterial::from_color(color));

    // Spawn l'entité indicateur avec les bons composants Bevy 2D
    let mut entity = commands.spawn((
        Mesh2d(indicator_mesh),
        MeshMaterial2d(material),
        Transform::from_translation(position),
        ActionIndicator {
            action_id,
            chunk_id: chunk_id.clone(),
            cell: cell.clone(),
            status,
        },
    ));

    // Ajouter le marker spécifique au statut
    match status {
        ActionStatusEnum::Pending | ActionStatusEnum::Failed => {
            entity.insert(PendingIndicator);
        }
        ActionStatusEnum::InProgress => {
            entity.insert(InProgressIndicator);

            // Ajouter un texte pour afficher le timer sur la cellule
            entity.with_children(|parent| {
                parent.spawn((
                    Text2d::new(""),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_translation(Vec3::new(0.0, 0.6, 0.1)),
                    ActionTimerText,
                ));
            });
        }
        ActionStatusEnum::Completed => {
            entity.insert(CompletedIndicator);
        }
    }
}

/// Marker component pour le texte du timer d'action
#[derive(Component)]
pub struct ActionTimerText;

/// Système pour animer les indicateurs InProgress (pulse d'échelle)
pub fn animate_in_progress_indicators(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<InProgressIndicator>>,
) {
    let time_secs = time.elapsed_secs();
    for mut transform in query.iter_mut() {
        // Animation de pulsation avec un cycle de 2 secondes
        let scale = 1.0 + (time_secs * 3.14159).sin() * 0.15;
        transform.scale = Vec3::splat(scale);
    }
}

/// Système pour nettoyer automatiquement les indicateurs Completed après un délai
pub fn cleanup_completed_indicators(
    mut commands: Commands,
    time: Res<Time>,
    query: Query<(Entity, &ActionIndicator), With<CompletedIndicator>>,
    action_tracker: Res<ActionTracker>,
) {
    let current_time = time.elapsed_secs() as u64;

    for (entity, indicator) in query.iter() {
        if let Some(action) = action_tracker.get_action(indicator.action_id) {
            // Retirer l'indicateur 3 secondes après la complétion
            if current_time > action.completion_time + 3 {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Système pour mettre à jour le texte du timer sur les indicateurs en cours
pub fn update_action_timer_text(
    action_tracker: Res<ActionTracker>,
    indicator_query: Query<(&ActionIndicator, &Children), With<InProgressIndicator>>,
    mut text_query: Query<&mut Text2d, With<ActionTimerText>>,
) {
    for (indicator, children) in indicator_query.iter() {
        if let Some(action) = action_tracker.get_action(indicator.action_id) {
            // Calculer le temps restant
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let remaining_seconds = if current_time < action.completion_time {
                action.completion_time - current_time
            } else {
                0
            };

            let minutes = remaining_seconds / 60;
            let seconds = remaining_seconds % 60;

            // Mettre à jour le texte dans les enfants
            for child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    **text = format!("{}:{:02}", minutes, seconds);
                }
            }
        }
    }
}
