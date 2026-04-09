use bevy::prelude::*;

use super::layer_state::{DebugLayer, DebugLayerVisibility};

use crate::rendering::lake::systems::LakeEntity;
use crate::rendering::mist::systems::{GroundFogEntity, MistEntity};
use crate::rendering::ocean::systems::OceanEntity;
use crate::rendering::terrain::components::{Building, Terrain, TreeChunkMesh};
use crate::rendering::territory::systems::TerritoryContourEntity;

/// Toggle visibility of all rendering layers based on DebugLayerVisibility.
/// Only runs when the resource changes.
pub fn apply_layer_visibility(
    layer_vis: Res<DebugLayerVisibility>,
    mut ocean: Query<&mut Visibility, (With<OceanEntity>, Without<Terrain>, Without<MistEntity>, Without<GroundFogEntity>, Without<TreeChunkMesh>, Without<Building>, Without<TerritoryContourEntity>, Without<LakeEntity>)>,
    mut lake: Query<&mut Visibility, (With<LakeEntity>, Without<OceanEntity>, Without<Terrain>, Without<MistEntity>, Without<GroundFogEntity>, Without<TreeChunkMesh>, Without<Building>, Without<TerritoryContourEntity>)>,
    mut terrain: Query<&mut Visibility, (With<Terrain>, Without<OceanEntity>, Without<MistEntity>, Without<GroundFogEntity>, Without<TreeChunkMesh>, Without<Building>, Without<TerritoryContourEntity>, Without<LakeEntity>)>,
    mut mist: Query<&mut Visibility, (With<MistEntity>, Without<OceanEntity>, Without<Terrain>, Without<GroundFogEntity>, Without<TreeChunkMesh>, Without<Building>, Without<TerritoryContourEntity>, Without<LakeEntity>)>,
    mut ground_fog: Query<&mut Visibility, (With<GroundFogEntity>, Without<OceanEntity>, Without<Terrain>, Without<MistEntity>, Without<TreeChunkMesh>, Without<Building>, Without<TerritoryContourEntity>, Without<LakeEntity>)>,
    mut trees: Query<&mut Visibility, (With<TreeChunkMesh>, Without<OceanEntity>, Without<Terrain>, Without<MistEntity>, Without<GroundFogEntity>, Without<Building>, Without<TerritoryContourEntity>, Without<LakeEntity>)>,
    mut buildings: Query<&mut Visibility, (With<Building>, Without<OceanEntity>, Without<Terrain>, Without<MistEntity>, Without<GroundFogEntity>, Without<TreeChunkMesh>, Without<TerritoryContourEntity>, Without<LakeEntity>)>,
    mut borders: Query<&mut Visibility, (With<TerritoryContourEntity>, Without<OceanEntity>, Without<Terrain>, Without<MistEntity>, Without<GroundFogEntity>, Without<TreeChunkMesh>, Without<Building>, Without<LakeEntity>)>,
) {
    if !layer_vis.is_changed() {
        return;
    }

    fn set_vis(query: &mut Query<&mut Visibility, impl bevy::ecs::query::QueryFilter>, visible: bool) {
        let v = if visible { Visibility::Inherited } else { Visibility::Hidden };
        for mut vis in query.iter_mut() {
            *vis = v;
        }
    }

    set_vis(&mut ocean, layer_vis.is_visible(DebugLayer::Ocean));
    set_vis(&mut lake, layer_vis.is_visible(DebugLayer::Lake));
    set_vis(&mut terrain, layer_vis.is_visible(DebugLayer::Terrain));
    set_vis(&mut mist, layer_vis.is_visible(DebugLayer::Mist));
    set_vis(&mut ground_fog, layer_vis.is_visible(DebugLayer::GroundFog));
    set_vis(&mut trees, layer_vis.is_visible(DebugLayer::Trees));
    set_vis(&mut buildings, layer_vis.is_visible(DebugLayer::Buildings));
    set_vis(&mut borders, layer_vis.is_visible(DebugLayer::DomainBorders));
}
