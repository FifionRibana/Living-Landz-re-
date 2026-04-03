use bevy::prelude::*;
use shared::{
    BiomeChunkData, BiomeChunkId, BuildingData, LakeData, OceanData, TerrainChunkData,
    TerrainChunkId, TerrainGlobalData,
    grid::{CellData, GridCell},
};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

use super::streaming_config::LOAD_GRACE_PERIOD;

#[derive(Resource, Default)]
pub struct WorldCache {
    terrains: TerrainCache,
    biomes: BiomeCache,
    cells: CellCache,
    buildings: BuildingCache,
    ocean: OceanCache,
    lake: LakeCache,
    terrain_global: TerrainGlobalCache,
    exploration: ExplorationCache,
    /// Precomputed set of chunk coords that touch the coastline.
    /// Computed when ocean SDF data arrives.
    coastal_chunks: HashSet<(i32, i32)>,
}

#[derive(Default, Clone)]
pub struct TerrainCache {
    loaded: HashMap<String, TerrainChunkData>,
    /// When each chunk was loaded — used for unload grace period.
    loaded_at: HashMap<String, Instant>,
    requested: HashSet<String>,
    requested_at: HashMap<String, f32>,
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
                terrain_data.id.x,
                terrain_data.id.y,
                terrain_data.name,
                terrain_data.road_sdf_data.is_some()
            );
        } else {
            info!(
                "Inserting chunk ({},{}) in terrain {}",
                terrain_data.id.x, terrain_data.id.y, terrain_data.name
            );
        }

        self.loaded.insert(key.clone(), terrain_data.clone());
        self.loaded_at.insert(key.clone(), Instant::now());
        self.requested.remove(key);
        self.requested_at.remove(key);

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

    pub fn mark_requested_at(&mut self, name: &str, id: &TerrainChunkId, time: f32) {
        let key = TerrainChunkData::storage_key(name, *id);
        self.requested_at.insert(key, time);
    }

    pub fn is_requested_recently(
        &self,
        name: &str,
        id: &TerrainChunkId,
        timeout: f32,
        now: f32,
    ) -> bool {
        let key = TerrainChunkData::storage_key(name, *id);
        self.requested_at
            .get(&key)
            .map(|&t| now - t < timeout)
            .unwrap_or(false)
    }

    pub fn get_requested_time(&self, name: &str, id: &TerrainChunkId) -> Option<f32> {
        let key = TerrainChunkData::storage_key(name, *id);
        self.requested_at.get(&key).copied()
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
            let in_range =
                (id.x - center.x).abs() <= max_distance && (id.y - center.y).abs() <= max_distance;

            if in_range {
                return true; // Keep — in range
            }

            // Don't unload recently loaded chunks (grace period prevents thrashing)
            if let Some(loaded_at) = self.loaded_at.get(chunk_key) {
                if loaded_at.elapsed() < LOAD_GRACE_PERIOD {
                    info!(
                        "⏳ Grace period: keeping chunk ({},{}) (loaded {}ms ago)",
                        id.x, id.y, loaded_at.elapsed().as_millis()
                    );
                    return true; // Keep — loaded too recently
                }
            }

            removed_ids.push(chunk_key.clone());
            removed.push(data.clone());
            false
        });

        // Clean up tracking for removed chunks
        for key in &removed_ids {
            self.loaded_at.remove(key);
            self.requested_at.remove(key);
        }

        if !removed_ids.is_empty() {
            info!(
                "📦 Unloaded {} chunks",
                removed_ids.len(),
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
    pub dirty_since: f64,
}

impl BuildingCache {
    pub fn insert_buildings(&mut self, buildings: &[BuildingData]) {
        info!("Inserting {} buildings into cache", buildings.len());
        buildings.iter().for_each(|building_data| {
            self.loaded
                .insert(building_data.base_data.cell, *building_data);
        });
        if !self.dirty {
            self.dirty_since = 0.0; // will be set by rebuild system
        }
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

        self.loaded.retain(|_cell, data| {
            let keep = (data.base_data.chunk.x - center.x).abs() <= max_distance
                && (data.base_data.chunk.y - center.y).abs() <= max_distance;

            if !keep {
                // DEBUG: afficher le chunk du bâtiment vs le centre
                // warn!(
                //     "📦 Unloading building id={} cell=({},{}) chunk=({},{}) center=({},{}) dist=({},{})",
                //     data.base_data.id,
                //     cell.q, cell.r,
                //     data.base_data.chunk.x, data.base_data.chunk.y,
                //     center.x, center.y,
                //     (data.base_data.chunk.x - center.x).abs(),
                //     (data.base_data.chunk.y - center.y).abs(),
                // );
                removed_ids.push(data.base_data.id as i64);
                removed.push(*data);
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

#[derive(Default, Clone)]
pub struct ExplorationCache {
    /// Texture dimensions (high-res, e.g. 32× chunk grid)
    pub width: i32,
    pub height: i32,
    /// Exploration bitmap: 0 = fog, 255 = explored
    pub data: Vec<u8>,
    /// Chunk grid dimensions (for streaming is_chunk_explored check)
    pub n_chunk_x: i32,
    pub n_chunk_y: i32,
    /// Texels per chunk axis (width / n_chunk_x)
    pub resolution: i32,
    loaded: bool,
    requested: bool,
    pub dirty: bool,
}

impl ExplorationCache {
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }
    pub fn is_requested(&self) -> bool {
        self.requested
    }
    pub fn mark_requested(&mut self) {
        self.requested = true;
    }

    pub fn set_map(&mut self, width: i32, height: i32, data: Vec<u8>, n_chunk_x: i32, n_chunk_y: i32) {
        self.width = width;
        self.height = height;
        self.data = data;
        self.n_chunk_x = n_chunk_x;
        self.n_chunk_y = n_chunk_y;
        self.resolution = if n_chunk_x > 0 { width / n_chunk_x } else { 1 };
        self.loaded = true;
        self.dirty = true;
    }

    /// Apply a rectangular patch from an ExplorationPatch message.
    pub fn apply_patch(&mut self, px: i32, py: i32, pw: i32, ph: i32, patch_data: &[u8]) {
        for row in 0..ph {
            let ty = py + row;
            if ty < 0 || ty >= self.height {
                continue;
            }
            for col in 0..pw {
                let tx = px + col;
                if tx < 0 || tx >= self.width {
                    continue;
                }
                let src_idx = (row * pw + col) as usize;
                let dst_idx = (ty * self.width + tx) as usize;
                if src_idx < patch_data.len() && dst_idx < self.data.len() {
                    self.data[dst_idx] = patch_data[src_idx];
                }
            }
        }
        self.dirty = true;
    }

    /// Check if a chunk has any explored texels (for streaming).
    /// Samples a 3×3 grid within the chunk — if ANY point is explored, chunk loads.
    pub fn is_chunk_explored(&self, chunk: &shared::TerrainChunkId) -> bool {
        if !self.loaded || self.resolution <= 0 {
            return false;
        }
        if chunk.x < 0 || chunk.x >= self.n_chunk_x || chunk.y < 0 || chunk.y >= self.n_chunk_y {
            return false;
        }

        let base_tx = chunk.x * self.resolution;
        let base_ty = chunk.y * self.resolution;
        let step = self.resolution / 3;

        for sy in 0..3 {
            for sx in 0..3 {
                let tx = base_tx + step / 2 + sx * step;
                let ty = base_ty + step / 2 + sy * step;
                if tx < self.width && ty < self.height {
                    if self.data[(ty * self.width + tx) as usize] > 0 {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Check if a chunk OR any of its 8 neighbors is explored.
    /// This gives a 1-chunk buffer zone around explored territory,
    /// so fog animation has room to breathe without revealing unloaded chunks.
    pub fn is_chunk_near_explored(&self, chunk: &shared::TerrainChunkId) -> bool {
        for dy in -1..=1i32 {
            for dx in -1..=1i32 {
                let neighbor = shared::TerrainChunkId {
                    x: chunk.x + dx,
                    y: chunk.y + dy,
                };
                if self.is_chunk_explored(&neighbor) {
                    return true;
                }
            }
        }
        false
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

    pub fn mark_terrain_requested_at(&mut self, name: &str, id: &TerrainChunkId, time: f32) {
        self.terrains.mark_requested_at(name, id, time)
    }

    pub fn is_terrain_requested_recently(
        &self,
        name: &str,
        id: &TerrainChunkId,
        timeout: f32,
        now: f32,
    ) -> bool {
        self.terrains.is_requested_recently(name, id, timeout, now)
    }

    pub fn get_terrain_requested_time(&self, name: &str, id: &TerrainChunkId) -> Option<f32> {
        self.terrains.get_requested_time(name, id)
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

    /// Get all buildings that belong to a specific terrain chunk.
    pub fn buildings_for_chunk(&self, chunk_id: &TerrainChunkId) -> Vec<&BuildingData> {
        self.buildings
            .loaded
            .values()
            .filter(|b| b.base_data.chunk == *chunk_id)
            .collect()
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
        // Precompute coastal chunks from the SDF before storing.
        self.coastal_chunks = Self::compute_coastal_chunks(&ocean_data);
        info!(
            "🏖️ Computed {} coastal chunks from ocean SDF",
            self.coastal_chunks.len()
        );
        self.ocean.insert_ocean(ocean_data);
    }

    /// Returns true if the chunk contains or borders the coastline.
    pub fn is_chunk_coastal(&self, chunk: &TerrainChunkId) -> bool {
        self.coastal_chunks.contains(&(chunk.x, chunk.y))
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

    // EXPLORATION
    pub fn is_chunk_explored(&self, chunk: &TerrainChunkId) -> bool {
        self.exploration.is_chunk_explored(chunk)
    }
    pub fn is_chunk_near_explored(&self, chunk: &TerrainChunkId) -> bool {
        self.exploration.is_chunk_near_explored(chunk)
    }
    pub fn is_exploration_loaded(&self) -> bool {
        self.exploration.is_loaded()
    }
    pub fn is_exploration_requested(&self) -> bool {
        self.exploration.is_requested()
    }
    pub fn mark_exploration_requested(&mut self) {
        self.exploration.mark_requested();
    }
    pub fn set_exploration_map(&mut self, width: i32, height: i32, data: Vec<u8>, n_chunk_x: i32, n_chunk_y: i32) {
        self.exploration.set_map(width, height, data, n_chunk_x, n_chunk_y);
    }
    pub fn apply_exploration_patch(&mut self, px: i32, py: i32, pw: i32, ph: i32, data: &[u8]) {
        self.exploration.apply_patch(px, py, pw, ph, data);
    }
    pub fn exploration_cache(&self) -> &ExplorationCache {
        &self.exploration
    }
    pub fn exploration_cache_mut(&mut self) -> &mut ExplorationCache {
        &mut self.exploration
    }

    /// Scan the ocean SDF and mark every chunk that contains or borders coastline.
    /// A chunk is coastal if any sample point within it has an SDF raw value < 148
    /// (sdf_signed < ~0.16, i.e. water or very near the shore on the land side).
    fn compute_coastal_chunks(ocean: &OceanData) -> HashSet<(i32, i32)> {
        use shared::constants;

        let chunk_w = constants::CHUNK_SIZE.x;
        let chunk_h = constants::CHUNK_SIZE.y;
        let num_cx = (ocean.world_width / chunk_w).ceil() as i32;
        let num_cy = (ocean.world_height / chunk_h).ceil() as i32;
        let sdf_w = ocean.width as f32;
        let sdf_h = ocean.height as f32;

        // SDF threshold: 128 = coast.  We include anything up to ~148 ≈ slightly inland.
        const COASTAL_THRESHOLD: u8 = 148;
        // Number of sample points per chunk axis (3×3 grid)
        const SAMPLES: i32 = 3;

        let mut result = HashSet::new();

        for cy in 0..num_cy {
            for cx in 0..num_cx {
                let mut is_coastal = false;

                'samples: for sy in 0..SAMPLES {
                    for sx in 0..SAMPLES {
                        // Sample position within the chunk (0..1 range on each axis)
                        let fx = (sx as f32 + 0.5) / SAMPLES as f32;
                        let fy = (sy as f32 + 0.5) / SAMPLES as f32;

                        let world_x = (cx as f32 + fx) * chunk_w;
                        let world_y = (cy as f32 + fy) * chunk_h;

                        // Map to SDF pixel coordinates
                        let u = world_x / ocean.world_width;
                        let v = world_y / ocean.world_height;
                        let px = (u * sdf_w).min(sdf_w - 1.0) as usize;
                        let py = (v * sdf_h).min(sdf_h - 1.0) as usize;

                        let idx = py * ocean.width + px;
                        if idx < ocean.sdf_values.len()
                            && ocean.sdf_values[idx] < COASTAL_THRESHOLD
                        {
                            is_coastal = true;
                            break 'samples;
                        }
                    }
                }

                if is_coastal {
                    result.insert((cx, cy));
                }
            }
        }

        result
    }
}