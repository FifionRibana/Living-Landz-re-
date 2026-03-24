use bevy::prelude::*;
use shared::{
    BiomeChunkData, BiomeChunkId, BuildingData, LakeData, OceanData, TerrainChunkData, TerrainChunkId, TerrainGlobalData, grid::{CellData, GridCell}
};
use std::collections::{HashMap, HashSet};

#[derive(Resource, Default)]
pub struct WorldCache {
    terrains: TerrainCache,
    biomes: BiomeCache,
    cells: CellCache,
    buildings: BuildingCache,
    ocean: OceanCache,
    lake: LakeCache,
    terrain_global: TerrainGlobalCache,
}

#[derive(Default, Clone)]
pub struct TerrainCache {
    loaded: HashMap<String, TerrainChunkData>,
    requested: HashSet<String>,
}

impl TerrainCache {
    /// Inserts or updates a terrain chunk.
    /// Returns true if the chunk was updated (already existed), false if newly inserted.
    pub fn insert_terrain(&mut self, terrain_data: &TerrainChunkData) -> bool {
        let key = &terrain_data.get_storage_key();
        let is_update = self.loaded.contains_key(key);

        if is_update {
            info!(
                "Updating chunk ({},{}) in terrain {} (has roads: {})",
                terrain_data.id.x, terrain_data.id.y, terrain_data.name,
                terrain_data.road_sdf_data.is_some()
            );
        } else {
            info!(
                "Inserting chunk ({},{}) in terrain {}",
                terrain_data.id.x, terrain_data.id.y, terrain_data.name
            );
        }

        self.loaded.insert(key.clone(), terrain_data.clone());
        self.requested.remove(key);

        is_update
    }

    pub fn is_loaded(&self, name: &str, id: &TerrainChunkId) -> bool {
        self.loaded
            .contains_key(&format!("{}_{}_{}", name, id.x, id.y))
    }

    pub fn is_requested(&self, name: &str, id: &TerrainChunkId) -> bool {
        self.requested
            .contains(&format!("{}_{}_{}", name, id.x, id.y))
    }

    pub fn mark_requested(&mut self, name: &str, id: &TerrainChunkId) {
        self.requested.insert(format!("{}_{}_{}", name, id.x, id.y));
    }

    pub fn unload_distant(
        &mut self,
        center: &TerrainChunkId,
        max_distance: i32,
    ) -> (Vec<String>, Vec<TerrainChunkData>) {
        let mut removed_ids = Vec::new();
        let mut removed = Vec::new();

        self.loaded.retain(|chunk_key, data| {
            let id = &data.id;
            let keep =
                (id.x - center.x).abs() <= max_distance && (id.y - center.y).abs() <= max_distance;

            if !keep {
                removed_ids.push(chunk_key.clone());
                removed.push(data.clone());
            }

            keep
        });

        if !removed_ids.is_empty() {
            warn!(
                "📦 Unloaded {} chunks: {:?}",
                removed_ids.len(),
                removed_ids
            );
        }

        (removed_ids, removed)
    }
}

#[derive(Default, Clone)]
pub struct CellCache {
    loaded: HashMap<GridCell, CellData>,
}

impl CellCache {
    pub fn insert_cells(&mut self, cells: &[CellData]) {
        info!("Inserting {} cells into cache", cells.len());
        cells.iter().for_each(|cell_data| {
            self.loaded.insert(cell_data.cell, *cell_data);
        });
    }

    pub fn get_cell(&self, cell: &GridCell) -> Option<&CellData> {
        self.loaded.get(cell)
    }
}

#[derive(Default, Clone)]
pub struct BuildingCache {
    loaded: HashMap<GridCell, BuildingData>,
    pub dirty: bool,
}

impl BuildingCache {
    pub fn insert_buildings(&mut self, buildings: &[BuildingData]) {
        info!("Inserting {} buildings into cache", buildings.len());
        buildings.iter().for_each(|building_data| {
            self.loaded.insert(building_data.base_data.cell, *building_data);
        });
        self.dirty = true;
    }

    pub fn get_building(&self, cell: &GridCell) -> Option<&BuildingData> {
        self.loaded.get(cell)
    }
    
    pub fn unload_distant(
        &mut self,
        center: &TerrainChunkId,
        max_distance: i32,
    ) -> (Vec<i64>, Vec<BuildingData>) {
        let mut removed_ids = Vec::new();
        let mut removed = Vec::new();

        self.loaded.retain(|cell, data| {
            let keep =
                (data.base_data.chunk.x - center.x).abs() <= max_distance 
                && (data.base_data.chunk.y - center.y).abs() <= max_distance;

            if !keep {
                // DEBUG: afficher le chunk du bâtiment vs le centre
                warn!(
                    "📦 Unloading building id={} cell=({},{}) chunk=({},{}) center=({},{}) dist=({},{})",
                    data.base_data.id,
                    cell.q, cell.r,
                    data.base_data.chunk.x, data.base_data.chunk.y,
                    center.x, center.y,
                    (data.base_data.chunk.x - center.x).abs(),
                    (data.base_data.chunk.y - center.y).abs(),
                );
                removed_ids.push(data.base_data.id as i64);
                removed.push(data.clone());
            }

            keep
        });

        if !removed_ids.is_empty() {
            warn!("📦 Unloaded {} buildings", removed_ids.len());
        }

        (removed_ids, removed)
    }
}

#[derive(Default, Clone)]
pub struct BiomeCache {
    loaded: HashMap<String, BiomeChunkData>,
    requested: HashSet<String>,
}

impl BiomeCache {
    pub fn insert_biome(&mut self, biome_data: &BiomeChunkData) {
        info!(
            "Inserting biome {:?} chunk ({},{}) in terrain {}",
            biome_data.id.biome, biome_data.id.x, biome_data.id.y, biome_data.name
        );
        let key = &biome_data.get_storage_key();

        if self.loaded.contains_key(key) {
            warn!("Chunk '{}' already inserted. Ignoring.", key);
            return;
        }

        self.loaded.insert(key.clone(), biome_data.clone());
        self.requested.remove(key);
    }

    pub fn is_loaded(&self, name: &str, id: &BiomeChunkId) -> bool {
        self.loaded
            .contains_key(&format!("{}_{}_{}_{:?}", name, id.x, id.y, id.biome))
    }

    pub fn is_requested(&self, name: &str, id: &BiomeChunkId) -> bool {
        self.requested
            .contains(&format!("{}_{}_{}_{:?}", name, id.x, id.y, id.biome))
    }

    pub fn mark_requested(&mut self, name: &str, id: &BiomeChunkId) {
        self.requested
            .insert(format!("{}_{}_{}_{:?}", name, id.x, id.y, id.biome));
    }

    pub fn unload_distant(
        &mut self,
        center: &BiomeChunkId,
        max_distance: i32,
    ) -> (Vec<String>, Vec<BiomeChunkData>) {
        let mut removed_ids = Vec::new();
        let mut removed = Vec::new();

        self.loaded.retain(|chunk_key, data| {
            let id = &data.id;
            let keep =
                (id.x - center.x).abs() <= max_distance && (id.y - center.y).abs() <= max_distance;

            if !keep {
                removed_ids.push(chunk_key.clone());
                removed.push(data.clone());
            }

            keep
        });

        if !removed_ids.is_empty() {
            warn!(
                "📦 Unloaded {} chunks: {:?}",
                removed_ids.len(),
                removed_ids
            );
        }

        (removed_ids, removed)
    }
}

#[derive(Default, Clone)]
pub struct OceanCache {
    loaded: Option<OceanData>,
    requested: bool,
}

impl OceanCache {
    pub fn insert_ocean(&mut self, ocean_data: OceanData) {
        info!("Inserting ocean data for world: {}", ocean_data.name);
        self.loaded = Some(ocean_data);
        self.requested = false;
    }

    pub fn get_ocean(&self) -> Option<&OceanData> {
        self.loaded.as_ref()
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded.is_some()
    }

    pub fn is_requested(&self) -> bool {
        self.requested
    }

    pub fn mark_requested(&mut self) {
        self.requested = true;
    }

    pub fn clear(&mut self) {
        self.loaded = None;
        self.requested = false;
    }
}

#[derive(Default, Clone)]
pub struct LakeCache {
    loaded: Option<LakeData>,
    pub mask_handle: Option<Handle<Image>>,
    pub sdf_handle: Option<Handle<Image>>,
    requested: bool,
}

impl LakeCache {
    pub fn insert_lake(&mut self, lake_data: LakeData) {
        info!("Inserting lake data for world: {}", lake_data.name);
        self.loaded = Some(lake_data);
    }

    pub fn get_lake(&self) -> Option<&LakeData> {
        self.loaded.as_ref()
    }

    pub fn set_mask_handle(&mut self, handle: Handle<Image>) {
        self.mask_handle = Some(handle);
    }

    pub fn get_mask_handle(&self) -> Option<&Handle<Image>> {
        self.mask_handle.as_ref()
    }

    pub fn has_mask_handle(&self) -> bool {
        self.mask_handle.is_some()
    }

    pub fn set_sdf_handle(&mut self, handle: Handle<Image>) {
        self.sdf_handle = Some(handle);
    }

    pub fn get_sdf_handle(&self) -> Option<&Handle<Image>> {
        self.sdf_handle.as_ref()
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded.is_some()
    }

    pub fn is_requested(&self) -> bool {
        self.requested
    }

    pub fn mark_requested(&mut self) {
        self.requested = true;
    }

    pub fn clear(&mut self) {
        self.loaded = None;
        self.requested = false;
    }
}

#[derive(Default, Clone)]
pub struct TerrainGlobalCache {
    pub data: Option<TerrainGlobalData>,
    pub biome_handle: Option<Handle<Image>>,
    pub heightmap_handle: Option<Handle<Image>>,
    requested: bool,
}

impl TerrainGlobalCache {
    pub fn insert(&mut self, data: TerrainGlobalData) {
        info!("Inserting terrain global data for world: {}", data.name);
        self.data = Some(data);
        self.requested = false;
    }

    pub fn is_loaded(&self) -> bool {
        self.data.is_some()
    }

    pub fn get_terrain_global(&self) -> Option<&TerrainGlobalData> {
        self.data.as_ref()
    }

    pub fn has_handles(&self) -> bool {
        self.biome_handle.is_some() && self.heightmap_handle.is_some()
    }

    pub fn is_requested(&self) -> bool {
        self.requested
    }

    pub fn mark_requested(&mut self) {
        self.requested = true;
    }

    pub fn set_handles(&mut self, biome: Handle<Image>, heightmap: Handle<Image>) {
        self.biome_handle = Some(biome);
        self.heightmap_handle = Some(heightmap);
    }

    pub fn get_biome_handle(&self) -> Option<&Handle<Image>> {
        self.biome_handle.as_ref()
    }

    pub fn get_heightmap_handle(&self) -> Option<&Handle<Image>> {
        self.heightmap_handle.as_ref()
    }

    pub fn clear(&mut self) {
        self.data = None;
        self.biome_handle = None;
        self.heightmap_handle = None;
        self.requested = false;
    }
}

impl WorldCache {
    // TERRAIN
    /// Inserts or updates a terrain chunk.
    /// Returns true if the chunk was updated (already existed), false if newly inserted.
    pub fn insert_terrain(&mut self, terrain_data: &TerrainChunkData) -> bool {
        self.terrains.insert_terrain(terrain_data)
    }

    pub fn loaded_terrains(&self) -> impl Iterator<Item = &TerrainChunkData> {
        self.terrains.loaded.values()
    }

    pub fn is_terrain_loaded(&self, name: &str, id: &TerrainChunkId) -> bool {
        self.terrains.is_loaded(name, id)
    }

    pub fn is_terrain_requested(&self, name: &str, id: &TerrainChunkId) -> bool {
        self.terrains.is_requested(name, id)
    }

    pub fn mark_terrain_requested(&mut self, name: &str, id: &TerrainChunkId) {
        self.terrains.mark_requested(name, id);
    }

    pub fn unload_distant_terrain(
        &mut self,
        center: &TerrainChunkId,
        max_distance: i32,
    ) -> (Vec<String>, Vec<TerrainChunkData>) {
        self.terrains.unload_distant(center, max_distance)
    }

    // BIOME
    pub fn insert_biome(&mut self, biome_data: &BiomeChunkData) {
        self.biomes.insert_biome(biome_data);
    }

    pub fn loaded_biomes(&self) -> impl Iterator<Item = &BiomeChunkData> {
        self.biomes.loaded.values()
    }

    pub fn is_biome_loaded(&self, name: &str, id: &BiomeChunkId) -> bool {
        self.biomes.is_loaded(name, id)
    }

    pub fn is_biome_requested(&self, name: &str, id: &BiomeChunkId) -> bool {
        self.biomes.is_requested(name, id)
    }

    pub fn mark_biome_requested(&mut self, name: &str, id: &BiomeChunkId) {
        self.biomes.mark_requested(name, id);
    }

    pub fn unload_distant_biome(
        &mut self,
        center: &BiomeChunkId,
        max_distance: i32,
    ) -> (Vec<String>, Vec<BiomeChunkData>) {
        self.biomes.unload_distant(center, max_distance)
    }

    // CELLS
    pub fn insert_cells(&mut self, cells: &[CellData]) {
        self.cells.insert_cells(cells);
    }

    pub fn get_cell(&self, cell: &GridCell) -> Option<&CellData> {
        self.cells.get_cell(cell)
    }

    // BUILDINGS
    pub fn insert_buildings(&mut self, buildings: &[BuildingData]) {
        self.buildings.insert_buildings(buildings);
    }

    pub fn loaded_buildings(&self) -> impl Iterator<Item = &BuildingData> {
        self.buildings.loaded.values()
    }

    pub fn get_building(&self, cell: &GridCell) -> Option<&BuildingData> {
        self.buildings.get_building(cell)
    }
    
    pub fn unload_distant_building(
        &mut self,
        center: &TerrainChunkId,
        max_distance: i32,
    ) -> (Vec<i64>, Vec<BuildingData>) {
        self.buildings.unload_distant(center, max_distance)
    }

    pub fn is_buildings_dirty(&self) -> bool {
        self.buildings.dirty
    }

    pub fn clear_buildings_dirty(&mut self) {
        self.buildings.dirty = false;
    }

    // OCEAN
    pub fn insert_ocean(&mut self, ocean_data: OceanData) {
        self.ocean.insert_ocean(ocean_data);
    }

    pub fn get_ocean(&self) -> Option<&OceanData> {
        self.ocean.get_ocean()
    }

    pub fn is_ocean_loaded(&self) -> bool {
        self.ocean.is_loaded()
    }

    pub fn is_ocean_requested(&self) -> bool {
        self.ocean.is_requested()
    }

    pub fn mark_ocean_requested(&mut self) {
        self.ocean.mark_requested();
    }

    pub fn clear_ocean(&mut self) {
        self.ocean.clear();
    }

    // LAKE
    pub fn insert_lake(&mut self, lake_data: LakeData) {
        self.lake.insert_lake(lake_data);
    }

    pub fn get_lake(&self) -> Option<&LakeData> {
        self.lake.get_lake()
    }

    pub fn get_lake_mask_handle(&self) -> Option<&Handle<Image>> {
        self.lake.get_mask_handle()
    }

    pub fn has_lake_mask_handle(&self) -> bool {
        self.lake.has_mask_handle()
    }

    pub fn set_lake_mask_handle(&mut self, handle: Handle<Image>) {
        self.lake.set_mask_handle(handle);
    }

    pub fn get_lake_sdf_handle(&self) -> Option<&Handle<Image>> {
        self.lake.get_sdf_handle()
    }

    pub fn set_lake_sdf_handle(&mut self, handle: Handle<Image>) {
        self.lake.set_sdf_handle(handle);
    }

    pub fn is_lake_loaded(&self) -> bool {
        self.lake.is_loaded()
    }

    pub fn is_lake_requested(&self) -> bool {
        self.lake.is_requested()
    }

    pub fn mark_lake_requested(&mut self) {
        self.lake.mark_requested();
    }

    pub fn clear_lake(&mut self) {
        self.lake.clear();
    }

    // TERRAIN GLOBAL
    pub fn insert_terrain_global(&mut self, data: TerrainGlobalData) {
        self.terrain_global.insert(data);
    }

    pub fn get_terrain_global(&self) -> Option<&TerrainGlobalData> {
        self.terrain_global.get_terrain_global()
    }

    pub fn is_terrain_global_loaded(&self) -> bool {
        self.terrain_global.is_loaded()
    }

    pub fn has_terrain_global_handles(&self) -> bool {
        self.terrain_global.has_handles()
    }
    
    pub fn set_terrain_global_handles(&mut self, biome: Handle<Image>, heightmap: Handle<Image>) {
        self.terrain_global.set_handles(biome, heightmap);
    }

    pub fn get_terrain_global_biome_handle(&self) -> Option<&Handle<Image>> {
        self.terrain_global.get_biome_handle()
    }

    pub fn get_terrain_global_heightmap_handle(&self) -> Option<&Handle<Image>> {
        self.terrain_global.get_heightmap_handle()
    }

    pub fn is_terrain_global_requested(&self) -> bool {
        self.terrain_global.is_requested()
    }

    pub fn mark_terrain_global_requested(&mut self) {
        self.terrain_global.mark_requested();
    }
}
