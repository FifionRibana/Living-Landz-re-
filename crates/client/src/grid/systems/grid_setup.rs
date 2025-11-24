use bevy::prelude::*;
use hexx::HexOrientation;

use shared::constants;
use shared::grid::GridConfig;
use crate::grid::resources::HexMesh;

pub fn setup_grid_config(mut commands: Commands) {
    let radius = constants::HEX_SIZE;
    let orientation = HexOrientation::Flat;
    let ratio = Vec2::new(constants::HEX_RATIO.x, constants::HEX_RATIO.y);
    let chunk_size = 3u8;//constants::CHUNK_SIZE.x as u8;
    let grid_config = GridConfig::new(radius, orientation, ratio, chunk_size);
    commands.insert_resource(grid_config);
    info!(
        "✓ HexConfig configuré (rayon: {}, orientation: {:?}, ratio: {:?})",
        radius, orientation, ratio
    );
}

pub fn setup_meshes(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    grid_config: Res<GridConfig>,
) {
    let hex_mesh = HexMesh::create(meshes, grid_config);
    commands.insert_resource(hex_mesh);
}
