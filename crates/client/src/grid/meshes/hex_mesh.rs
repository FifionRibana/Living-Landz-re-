use bevy::asset::RenderAssetUsages;
use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use hexx::{HexLayout, MeshInfo, PlaneMeshBuilder};

/// Crée un mesh hexagonal en utilisant hexx::ColumnMeshBuilder
pub fn create_hexagonal_mesh(layout: HexLayout, radius: f32) -> Mesh {
    // Utilise ColumnMeshBuilder de hexx pour un hexagone plat
    let mesh_info = PlaneMeshBuilder::new(&layout)
        .facing(Vec3::Z)
        .center_aligned()
        .build();

    hexagonal_mesh(mesh_info, true)
}

/// Convertit hexx::MeshInfo en bevy::Mesh
/// Source: https://docs.rs/hexx/latest/hexx/#bevy-integration
pub fn hexagonal_mesh(mesh_info: MeshInfo, is_interactive: bool) -> Mesh {
    let usage = if is_interactive {
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD
    } else {
        RenderAssetUsages::RENDER_WORLD
    };
    Mesh::new(PrimitiveTopology::TriangleList, usage)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh_info.vertices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_info.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, mesh_info.uvs)
        .with_inserted_indices(Indices::U16(mesh_info.indices))
}
