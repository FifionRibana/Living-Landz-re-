use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy::prelude::{Mesh2d, MeshMaterial2d};
use super::materials::{TerritoryBorderMaterial, BorderParams};
use shared::{TerritoryBorderChunkSdfData, grid::{GridCell, GridConfig}};
use std::collections::HashMap;

/// Resource to store territory border SDF data
/// Each chunk can have multiple SDFs (one per organization)
#[derive(Resource, Default)]
pub struct TerritoryBorderSdfCache {
    pub chunks: HashMap<(i32, i32), Vec<TerritoryBorderChunkSdfData>>,
}

/// Resource to store border cells for debug visualization
#[derive(Resource)]
pub struct TerritoryBorderCellsDebug {
    pub organization_id: u64,
    pub border_cells: Vec<GridCell>,
}

/// Component to mark territory border entities
#[derive(Component)]
pub struct TerritoryBorderLayer {
    pub chunk_x: i32,
    pub chunk_y: i32,
    pub organization_id: u64,
}

/// System to update time uniform for animations
pub fn update_territory_border_time(
    time: Res<Time>,
    mut materials: ResMut<Assets<TerritoryBorderMaterial>>,
) {
    for (_, material) in materials.iter_mut() {
        material.time = time.elapsed_secs();
    }
}

/// Spawn territory border layer for a chunk
pub fn spawn_territory_border_layer(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<TerritoryBorderMaterial>>,
    images: &mut ResMut<Assets<Image>>,
    sdf_data: &TerritoryBorderChunkSdfData,
    chunk_size: Vec2,
) -> Entity {
    // Create SDF texture from data
    let texture = Image::new(
        Extent3d {
            width: sdf_data.width,
            height: sdf_data.height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        sdf_data.sdf_data.clone(),
        TextureFormat::R8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );

    let texture_handle = images.add(texture);

    // Create material with organization colors
    let border_color = Color::linear_rgba(
        sdf_data.border_color.0,
        sdf_data.border_color.1,
        sdf_data.border_color.2,
        sdf_data.border_color.3,
    );

    let material = TerritoryBorderMaterial {
        border_sdf_texture: texture_handle,
        border_params: BorderParams {
            line_width: 2.5,
            edge_softness: 1.5,
            glow_intensity: 0.0,
            color: border_color.into(),
        },
        time: 0.0,
    };

    // Create quad mesh for the chunk
    let mesh_handle = meshes.add(Rectangle::new(chunk_size.x, chunk_size.y));

    // Calculate world position for this chunk
    let world_pos = Vec3::new(
        sdf_data.chunk_x as f32 * chunk_size.x + chunk_size.x / 2.0,
        sdf_data.chunk_y as f32 * chunk_size.y + chunk_size.y / 2.0,
        0.25, // Z-layer above terrain (0.0) but below UI
    );

    commands
        .spawn((
            Mesh2d(mesh_handle),
            MeshMaterial2d(materials.add(material)),
            Transform::from_translation(world_pos),
            TerritoryBorderLayer {
                chunk_x: sdf_data.chunk_x,
                chunk_y: sdf_data.chunk_y,
                organization_id: sdf_data.organization_id,
            },
        ))
        .id()
}

/// System to process new SDF data and spawn border layers
pub fn process_territory_border_sdf_data(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TerritoryBorderMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut sdf_cache: ResMut<TerritoryBorderSdfCache>,
    existing_layers: Query<(Entity, &TerritoryBorderLayer)>,
) {
    // Example chunk size (should match terrain chunk size)
    let chunk_size = Vec2::new(600.0, 503.0);

    // Process cached chunks (each chunk may have multiple organizations)
    for ((chunk_x, chunk_y), sdf_data_list) in sdf_cache.chunks.iter() {
        for sdf_data in sdf_data_list {
            // Check if layer already exists for this chunk + organization
            let exists = existing_layers.iter().any(|(_, layer)| {
                layer.chunk_x == *chunk_x
                    && layer.chunk_y == *chunk_y
                    && layer.organization_id == sdf_data.organization_id
            });

            if !exists {
                spawn_territory_border_layer(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &mut images,
                    sdf_data,
                    chunk_size,
                );
            }
        }
    }
}

/// Debug system to visualize border detection
pub fn debug_territory_borders(
    mut gizmos: Gizmos,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut debug_enabled: Local<bool>,
    sdf_cache: Res<TerritoryBorderSdfCache>,
) {
    // Toggle with F10
    if keyboard.just_pressed(KeyCode::F10) {
        *debug_enabled = !*debug_enabled;
        info!("Territory border debug: {}", if *debug_enabled { "ON" } else { "OFF" });
    }

    if !*debug_enabled {
        return;
    }

    let chunk_size = Vec2::new(600.0, 503.0);

    // Visualize actual border pixels from SDF data (for all organizations)
    for ((chunk_x, chunk_y), sdf_data_list) in sdf_cache.chunks.iter() {
        // Calculate chunk world offset
        let chunk_offset = Vec2::new(
            *chunk_x as f32 * chunk_size.x,
            *chunk_y as f32 * chunk_size.y,
        );

        // Draw each organization's SDF
        for sdf_data in sdf_data_list {
            // Sample SDF at lower resolution for performance (every 4th pixel)
            let sample_step = 4;

            for y in (0..sdf_data.height).step_by(sample_step) {
                for x in (0..sdf_data.width).step_by(sample_step) {
                    let sdf_value = sdf_data.get_distance(x, y);

                    // Draw pixels close to border (distance < 15 pixels)
                    if sdf_value < 15 {
                        // Calculate world position
                        let pixel_pos = chunk_offset + Vec2::new(
                            (x as f32 / sdf_data.width as f32) * chunk_size.x,
                            (y as f32 / sdf_data.height as f32) * chunk_size.y,
                        );

                        // Use organization color
                        let base_color = Color::linear_rgba(
                            sdf_data.border_color.0,
                            sdf_data.border_color.1,
                            sdf_data.border_color.2,
                            sdf_data.border_color.3,
                        );

                        // Draw a small circle at this position
                        gizmos.circle_2d(pixel_pos, 2.0, base_color);
                    }
                }
            }
        }

        // Also draw chunk boundaries in cyan
        let chunk_center = Vec2::new(
            *chunk_x as f32 * chunk_size.x + chunk_size.x / 2.0,
            *chunk_y as f32 * chunk_size.y + chunk_size.y / 2.0,
        );
        gizmos.rect_2d(
            chunk_center,
            chunk_size,
            Color::srgba(0.0, 1.0, 1.0, 0.3), // Semi-transparent cyan
        );
    }
}

/// System to visualize territory border cells with colored hexagons
pub fn visualize_border_cells(
    mut gizmos: Gizmos,
    border_cells_debug: Option<Res<TerritoryBorderCellsDebug>>,
    grid_config: Res<GridConfig>,
) {
    let Some(border_cells) = border_cells_debug else {
        return;
    };

    // Draw each border cell as a colored hexagon
    for cell in &border_cells.border_cells {
        let center = grid_config.layout.hex_to_world_pos(cell.to_hex());
        let radius = grid_config.hex_radius;

        // Draw hexagon vertices
        let angles_deg = [0.0_f32, 60.0, 120.0, 180.0, 240.0, 300.0];
        let mut vertices: Vec<Vec2> = angles_deg
            .iter()
            .map(|&angle| {
                let rad = angle.to_radians();
                center + Vec2::new(radius * rad.cos(), radius * rad.sin())
            })
            .collect();

        // Close the hexagon
        vertices.push(vertices[0]);

        // Draw hexagon outline in bright green
        for i in 0..vertices.len() - 1 {
            gizmos.line_2d(vertices[i], vertices[i + 1], Color::srgb(0.0, 1.0, 0.0));
        }

        // Fill hexagon with semi-transparent green
        // Draw triangles from center to each edge
        for i in 0..6 {
            gizmos.line_2d(center, vertices[i], Color::srgba(0.0, 1.0, 0.0, 0.3));
        }
    }

    // Display count in the logs once per second
    // info!("Visualizing {} border cells for org {}",
    //     border_cells.border_cells.len(),
    //     border_cells.organization_id
    // );
}


// ===========================================================================================
// NEW CONTOUR-BASED RENDERING SYSTEM
// ===========================================================================================

use super::cache::{TerritoryContourCache, OrganizationContour};
use super::materials::{TerritoryMaterial, create_territory_material};
use bevy::render::storage::ShaderStorageBuffer;

/// Component to mark territory contour entities
#[derive(Component)]
pub struct TerritoryContourEntity {
    pub chunk_x: i32,
    pub chunk_y: i32,
    pub organization_id: u64,
}

/// System to render territory contours from cache
///
/// This system reads the TerritoryContourCache and spawns entities with TerritoryMaterial
/// for each contour that hasn't been rendered yet.
pub fn render_territory_contours(
    mut commands: Commands,
    cache: Res<TerritoryContourCache>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TerritoryMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    existing_query: Query<(Entity, &TerritoryContourEntity)>,
) {
    // Track which (chunk, org) pairs exist
    let mut existing_entities: std::collections::HashMap<((i32, i32), u64), Entity> =
        std::collections::HashMap::new();

    for (entity, contour_entity) in existing_query.iter() {
        existing_entities.insert(
            ((contour_entity.chunk_x, contour_entity.chunk_y), contour_entity.organization_id),
            entity,
        );
    }

    // Render contours from cache
    for ((chunk_x, chunk_y), contours) in cache.contours.iter() {
        for contour in contours {
            let key = ((*chunk_x, *chunk_y), contour.organization_id);

            // Skip if already rendered
            if existing_entities.contains_key(&key) {
                continue;
            }

            // Skip if no points
            if contour.points.is_empty() {
                warn!(
                    "Skipping contour for org {} in chunk ({},{}) - no points",
                    contour.organization_id, chunk_x, chunk_y
                );
                continue;
            }

            // Create material and mesh
            let (mesh_handle, material_handle) = create_territory_material(
                &contour.points,
                &mut meshes,
                &mut materials,
                &mut buffers,
                contour.border_color,
                contour.fill_color,
            );

            // Calculate center position
            let (min, max) = compute_contour_bounds(&contour.points);
            let center = (min + max) * 0.5;

            // Spawn entity at Z=10 (above terrain at Z=0)
            commands.spawn((
                Mesh2d(mesh_handle),
                MeshMaterial2d(material_handle),
                Transform::from_xyz(center.x, center.y, 10.0),
                TerritoryContourEntity {
                    chunk_x: *chunk_x,
                    chunk_y: *chunk_y,
                    organization_id: contour.organization_id,
                },
            ));

            info!(
                "✓ Spawned territory contour for org {} in chunk ({},{})",
                contour.organization_id, chunk_x, chunk_y
            );
        }
    }
}

/// Helper to compute bounds from contour points
fn compute_contour_bounds(points: &[Vec2]) -> (Vec2, Vec2) {
    let mut min = Vec2::splat(f32::MAX);
    let mut max = Vec2::splat(f32::MIN);

    for p in points {
        min = min.min(*p);
        max = max.max(*p);
    }

    (min, max)
}
