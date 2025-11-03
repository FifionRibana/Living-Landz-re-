use bevy::prelude::*;
use shared::{BiomeChunkData, BiomeChunkId, TerrainChunkData, TerrainChunkId};
use std::collections::{HashMap, HashSet};

#[derive(Resource, Default)]
pub struct WorldCache {
    terrains: TerrainCache,
    biomes: BiomeCache,
}

#[derive(Default, Clone)]
pub struct TerrainCache {
    loaded: HashMap<String, TerrainChunkData>,
    requested: HashSet<String>,
}

impl TerrainCache {
    pub fn insert_terrain(&mut self, terrain_data: &TerrainChunkData) {
        info!(
            "Inserting chunk ({},{}) in terrain {}",
            terrain_data.id.x, terrain_data.id.y, terrain_data.name
        );
        let key = &terrain_data.get_storage_key();

        if self.loaded.contains_key(key) {
            warn!("Chunk '{}' already inserted. Ignoring.", key);
            return;
        }

        self.loaded.insert(key.clone(), terrain_data.clone());
        self.requested.remove(key);
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

    pub fn unload_distant(&mut self, center: &TerrainChunkId, max_distance: i32) -> (Vec<String>, Vec<TerrainChunkData>) {
        
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
            warn!("📦 Unloaded {} chunks: {:?}", removed_ids.len(), removed_ids);
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
        self.requested.insert(format!("{}_{}_{}_{:?}", name, id.x, id.y, id.biome));
    }

    pub fn unload_distant(&mut self, center: &BiomeChunkId, max_distance: i32) -> (Vec<String>, Vec<BiomeChunkData>) {
        
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
            warn!("📦 Unloaded {} chunks: {:?}", removed_ids.len(), removed_ids);
        }

        (removed_ids, removed)
    }
}

impl WorldCache {
    // TERRAIN
    pub fn insert_terrain(&mut self, terrain_data: &TerrainChunkData) {
        self.terrains.insert_terrain(terrain_data);
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

    pub fn unload_distant_terrain(&mut self, center: &TerrainChunkId, max_distance: i32) -> (Vec<String>, Vec<TerrainChunkData>) {
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

    pub fn unload_distant_biome(&mut self, center: &BiomeChunkId, max_distance: i32) -> (Vec<String>, Vec<BiomeChunkData>) {
        self.biomes.unload_distant(center, max_distance)
    }
    

}
