use bevy::prelude::*;

use crate::grid;

use shared::grid::GridConfig;

#[derive(Resource, Clone)]
pub struct HexMesh {
    pub mesh: Handle<Mesh>,
}

impl HexMesh {
    pub fn create(mut meshes: ResMut<Assets<Mesh>>, grid_config: Res<GridConfig>) -> Self {
        let mesh = meshes.add(grid::meshes::create_hexagonal_mesh(
            grid_config.layout.clone(),
            grid_config.hex_radius,
        ));

        Self { mesh }
    }
}
