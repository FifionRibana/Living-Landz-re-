use bevy::prelude::*;
use bevy::sprite_render::Material2dPlugin;
use hexx::*;

use crate::materials::{TerritoryChunkMaterial, TerritoryMaterial};
use crate::territory::HexMesh;

use super::territory;
pub struct TerritoryPlugin;

use shared::constants;
use shared::grid::GridConfig;

pub fn setup_grid_config(mut commands: Commands) {
    let radius = constants::HEX_SIZE;
    let orientation = HexOrientation::Flat;
    let ratio = Vec2::new(constants::HEX_RATIO.x, constants::HEX_RATIO.y);
    let chunk_size = 3u8; //constants::CHUNK_SIZE.x as u8;
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

impl Plugin for TerritoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<territory::TerritorySettings>()
            .add_plugins(Material2dPlugin::<TerritoryMaterial>::default())
            .add_plugins(Material2dPlugin::<TerritoryChunkMaterial>::default())
            .add_systems(PreStartup, (setup_grid_config, setup_meshes).chain())
            .add_systems(Startup, territory::compute_contour)
            .add_systems(Update, territory::draw_countour);
    }
}
