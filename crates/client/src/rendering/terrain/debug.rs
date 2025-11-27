use bevy::prelude::*;
use shared::{TerrainChunkId, constants};

use super::components::Terrain;

/// Resource to toggle chunk debug visualization (F8)
#[derive(Resource, Default)]
pub struct ChunkDebugEnabled(pub bool);

/// Component marker for debug text entities
#[derive(Component)]
pub struct ChunkDebugText {
    pub chunk_id: TerrainChunkId,
}

/// System to toggle debug mode with F8 key
pub fn toggle_chunk_debug(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut debug_enabled: ResMut<ChunkDebugEnabled>,
) {
    if keyboard.just_pressed(KeyCode::F8) {
        debug_enabled.0 = !debug_enabled.0;
        info!("Chunk debug mode: {}", if debug_enabled.0 { "ENABLED" } else { "DISABLED" });
    }
}

/// System to draw gizmos around chunks
pub fn draw_chunk_gizmos(
    mut gizmos: Gizmos,
    debug_enabled: Res<ChunkDebugEnabled>,
    terrains: Query<&Terrain>,
) {
    if !debug_enabled.0 {
        return;
    }

    for terrain in terrains.iter() {
        let world_position = Vec2::new(
            terrain.id.x as f32 * constants::CHUNK_SIZE.x,
            terrain.id.y as f32 * constants::CHUNK_SIZE.y,
        );

        // Draw rectangle around chunk
        let half_width = constants::CHUNK_SIZE.x / 2.0;
        let half_height = constants::CHUNK_SIZE.y / 2.0;

        let center = world_position + Vec2::new(half_width, half_height);

        gizmos.rect_2d(
            center,
            Vec2::new(constants::CHUNK_SIZE.x, constants::CHUNK_SIZE.y),
            Color::srgba(1.0, 1.0, 0.0, 0.8), // Yellow with transparency
        );
    }
}

/// System to spawn/update debug text for chunks
pub fn update_chunk_debug_text(
    mut commands: Commands,
    debug_enabled: Res<ChunkDebugEnabled>,
    terrains: Query<&Terrain>,
    debug_texts: Query<(Entity, &ChunkDebugText)>,
) {
    if debug_enabled.0 {
        // Get existing debug texts
        let existing_texts: std::collections::HashMap<_, _> = debug_texts
            .iter()
            .map(|(entity, text)| (text.chunk_id, entity))
            .collect();

        // Spawn debug text for each terrain chunk
        for terrain in terrains.iter() {
            if !existing_texts.contains_key(&terrain.id) {
                let world_position = Vec2::new(
                    terrain.id.x as f32 * constants::CHUNK_SIZE.x,
                    terrain.id.y as f32 * constants::CHUNK_SIZE.y,
                );

                // Offset the text slightly from the top-left corner (add 10px padding)
                let text_position = world_position + Vec2::new(40.0, constants::CHUNK_SIZE.y - 15.0);

                commands.spawn((
                    Text2d::new(format!("({}, {})", terrain.id.x, terrain.id.y)),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::srgba(1.0, 1.0, 0.0, 1.0)), // Yellow
                    Transform::from_translation(text_position.extend(0.0)), // High z-index to be on top
                    ChunkDebugText {
                        chunk_id: terrain.id,
                    },
                ));
            }
        }
    } else {
        // Remove all debug texts when disabled
        for (entity, _) in debug_texts.iter() {
            commands.entity(entity).despawn();
        }
    }
}
