use bevy::prelude::*;
use bevy::sprite_render::MeshMaterial2d;
use shared::{TerrainChunkId, constants};

use super::components::Terrain;
use super::materials::TerrainMaterial;
use crate::rendering::ocean::materials::{OceanMaterial, OceanDebugParams};
use crate::state::resources::WorldCache;
use crate::ui::debug::DebugOverlayState;

/// Debug mode for chunk visualization (F8)
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum ChunkDebugMode {
    #[default]
    Off,
    ChunksOnly,
    OutlineOnly,
    Both,
}

impl ChunkDebugMode {
    pub fn next(&self) -> Self {
        match self {
            Self::Off => Self::ChunksOnly,
            Self::ChunksOnly => Self::OutlineOnly,
            Self::OutlineOnly => Self::Both,
            Self::Both => Self::Off,
        }
    }

    pub fn show_chunks(&self) -> bool {
        matches!(self, Self::ChunksOnly | Self::Both)
    }

    pub fn show_outline(&self) -> bool {
        matches!(self, Self::OutlineOnly | Self::Both)
    }
}

/// Resource to toggle chunk debug visualization (F8)
#[derive(Resource, Default)]
pub struct ChunkDebugEnabled(pub ChunkDebugMode);

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
        debug_enabled.0 = debug_enabled.0.next();
        let mode_str = match debug_enabled.0 {
            ChunkDebugMode::Off => "OFF",
            ChunkDebugMode::ChunksOnly => "CHUNKS ONLY",
            ChunkDebugMode::OutlineOnly => "OUTLINE ONLY",
            ChunkDebugMode::Both => "CHUNKS + OUTLINE",
        };
        info!("Chunk debug mode: {}", mode_str);
    }
}

/// System to draw gizmos around chunks
pub fn draw_chunk_gizmos(
    mut gizmos: Gizmos,
    debug_enabled: Res<ChunkDebugEnabled>,
    terrains: Query<&Terrain>,
) {
    if !debug_enabled.0.show_chunks() {
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
    if debug_enabled.0.show_chunks() {
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

/// System to draw outline points from TerrainChunkData
pub fn draw_outline_points(
    mut gizmos: Gizmos,
    debug_enabled: Res<ChunkDebugEnabled>,
    world_cache_opt: Option<Res<WorldCache>>,
) {
    if !debug_enabled.0.show_outline() {
        return;
    }

    let Some(world_cache) = world_cache_opt else {
        return;
    };

    // Parcourir tous les terrains chargés
    for terrain in world_cache.loaded_terrains() {
        let world_position = Vec2::new(
            terrain.id.x as f32 * constants::CHUNK_SIZE.x,
            terrain.id.y as f32 * constants::CHUNK_SIZE.y,
        );

        // Dessiner chaque contour (outline) du terrain
        for contour in &terrain.outline {
            // Dessiner les points du contour
            for point in contour {
                let point_world_pos = world_position + Vec2::new(point[0] as f32, point[1] as f32);

                // Dessiner un petit cercle pour chaque point
                gizmos.circle_2d(
                    point_world_pos,
                    0.5, // rayon du cercle
                    Color::srgba(1.0, 0.0, 1.0, 1.0), // Rouge
                );
            }

            // Dessiner les lignes entre les points du contour
            if contour.len() > 1 {
                for i in 0..contour.len() {
                    let current = contour[i];
                    let next = contour[(i + 1) % contour.len()];

                    let current_world = world_position + Vec2::new(current[0] as f32, current[1] as f32);
                    let next_world = world_position + Vec2::new(next[0] as f32, next[1] as f32);

                    gizmos.line_2d(
                        current_world,
                        next_world,
                        Color::srgba(1.0, 0.0, 0.0, 0.6), // Rouge semi-transparent
                    );
                }
            }
        }
    }
}

// ─── Terrain Debug Uniforms ─────────────────────────────────────────
//
// Reads from DebugOverlayState (UI panel) and syncs debug_params on
// all terrain materials each frame.
//
// debug_params.x = shader base mode (0..5)
// debug_params.y = overlay bit-flags  (bit 0 = level_lines)

/// Sync DebugOverlayState → terrain material debug_params uniform.
pub fn sync_debug_uniforms(
    debug: Res<DebugOverlayState>,
    terrains: Query<&MeshMaterial2d<TerrainMaterial>>,
    mut materials: ResMut<Assets<TerrainMaterial>>,
) {
    let base = debug.shader_base.shader_value();
    let mut overlay_flags: u32 = 0;
    if debug.level_lines {
        overlay_flags |= 1;
    }
    let overlay_f = overlay_flags as f32;

    for mat_handle in terrains.iter() {
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.debug_params.slope_debug = base;
            mat.debug_params._padding1 = overlay_f;
        }
    }
}

/// Sync DebugOverlayState → ocean material debug_params uniform.
pub fn sync_ocean_debug_uniforms(
    debug: Res<DebugOverlayState>,
    oceans: Query<&MeshMaterial2d<OceanMaterial>>,
    mut materials: ResMut<Assets<OceanMaterial>>,
) {
    let base = debug.shader_base.shader_value();
    let mut overlay_flags: u32 = 0;
    if debug.level_lines {
        overlay_flags |= 1;
    }

    for mat_handle in oceans.iter() {
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.debug_params.debug_base = base;
            mat.debug_params.overlay_flags = overlay_flags as f32;
        }
    }
}

// ─── Shore Type Gizmos Overlay ──────────────────────────────────────

/// Draw hex outlines for cells tagged Shoreline or Lakebank.
pub fn draw_shore_type_gizmos(
    mut gizmos: Gizmos,
    debug: Res<DebugOverlayState>,
    world_cache: Option<Res<WorldCache>>,
    grid_config: Res<shared::grid::GridConfig>,
) {
    if !debug.hex_shore_types {
        return;
    }

    let Some(cache) = world_cache else {
        return;
    };

    for cell_data in cache.iter_cells() {
        let color = match cell_data.shore_type {
            shared::ShoreType::Shoreline => Color::srgba(0.3, 0.6, 1.0, 0.8),
            shared::ShoreType::Lakebank => Color::srgba(0.2, 0.8, 0.6, 0.8),
            _ => continue,
        };

        let center = grid_config.layout.hex_to_world_pos(cell_data.cell.to_hex());
        draw_hex_outline(&mut gizmos, center, &grid_config.layout, color);
    }
}

/// Draw a hex outline using the layout's corner positions.
fn draw_hex_outline(gizmos: &mut Gizmos, center: Vec2, layout: &hexx::HexLayout, color: Color) {
    let corners = layout.hex_corners(hexx::Hex::ZERO);
    for i in 0..6 {
        let start = center + corners[i];
        let end = center + corners[(i + 1) % 6];
        gizmos.line_2d(start, end, color);
    }
}

// ─── Chunk Boundary Gizmos + Labels (from DebugOverlayState) ────────

/// Marker for chunk label text entities managed by the overlay panel.
#[derive(Component)]
pub struct OverlayChunkLabel {
    pub chunk_id: shared::TerrainChunkId,
}

/// Draw chunk boundary rectangles and manage chunk label text entities.
pub fn draw_chunk_boundary_gizmos(
    mut commands: Commands,
    mut gizmos: Gizmos,
    debug: Res<DebugOverlayState>,
    terrains: Query<&Terrain>,
    labels: Query<(Entity, &OverlayChunkLabel)>,
) {
    if !debug.chunk_boundaries {
        // Despawn labels when disabled
        for (entity, _) in labels.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    let existing: std::collections::HashSet<_> = labels
        .iter()
        .map(|(_, l)| l.chunk_id)
        .collect();

    for terrain in terrains.iter() {
        let pos = Vec2::new(
            terrain.id.x as f32 * constants::CHUNK_SIZE.x,
            terrain.id.y as f32 * constants::CHUNK_SIZE.y,
        );
        let center = pos + Vec2::new(constants::CHUNK_SIZE.x, constants::CHUNK_SIZE.y) * 0.5;
        gizmos.rect_2d(
            center,
            Vec2::new(constants::CHUNK_SIZE.x, constants::CHUNK_SIZE.y),
            Color::srgba(1.0, 1.0, 1.0, 0.3),
        );

        // Spawn label if not yet existing
        if !existing.contains(&terrain.id) {
            let text_pos = pos + Vec2::new(40.0, constants::CHUNK_SIZE.y - 15.0);
            commands.spawn((
                Text2d::new(format!("({}, {})", terrain.id.x, terrain.id.y)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
                Transform::from_translation(text_pos.extend(0.0)),
                OverlayChunkLabel {
                    chunk_id: terrain.id,
                },
            ));
        }
    }
}
