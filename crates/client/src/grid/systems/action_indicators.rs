use bevy::prelude::*;
use shared::{ActionStatusEnum, TerrainChunkId, grid::GridCell};

use crate::{
    grid::components::{ActionIndicator, CompletedIndicator, InProgressIndicator, PendingIndicator},
    state::resources::ActionTracker,
};

/// Système pour créer/mettre à jour les indicateurs visuels d'actions sur les cellules
pub fn update_action_indicators(
    mut commands: Commands,
    action_tracker: Res<ActionTracker>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    action_id: u64,
    chunk_id: &TerrainChunkId,
    cell: &GridCell,
    status: ActionStatusEnum,
) {
    // Calculer la position 3D de la cellule
    let hex_radius = 1.0;
    let hex_height = (3.0_f32).sqrt() * hex_radius;
    let x = hex_radius * 1.5 * cell.q as f32;
    let z = hex_height * (cell.r as f32 + cell.q as f32 * 0.5);
    let y = 0.5; // Au-dessus de la cellule

    let position = Vec3::new(x, y, z);

    // Créer un petit mesh pour l'indicateur (un petit cercle ou icône)
    let indicator_mesh = meshes.add(Circle::new(0.2));

    // Couleur selon le statut
    let color = match status {
        ActionStatusEnum::Pending => Color::srgb(1.0, 1.0, 0.0), // Jaune
        ActionStatusEnum::InProgress => Color::srgb(0.0, 0.5, 1.0), // Bleu
        ActionStatusEnum::Completed => Color::srgb(0.0, 1.0, 0.0), // Vert
        ActionStatusEnum::Failed => Color::srgb(1.0, 0.0, 0.0), // Rouge
    };

    let material = materials.add(StandardMaterial {
        base_color: color,
        unlit: true,
        ..default()
    });

    // Spawn l'entité indicateur avec les bons composants Bevy
    let mut entity = commands.spawn((
        Mesh3d(indicator_mesh),
        MeshMaterial3d(material),
        Transform::from_translation(position)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
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
        }
        ActionStatusEnum::Completed => {
            entity.insert(CompletedIndicator);
        }
    }
}

/// Système pour animer les indicateurs InProgress (rotation)
pub fn animate_in_progress_indicators(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<InProgressIndicator>>,
) {
    for mut transform in query.iter_mut() {
        transform.rotate_z(time.delta_secs() * 2.0);
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
