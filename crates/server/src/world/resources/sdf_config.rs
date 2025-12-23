use bevy::prelude::*;

#[derive(Resource, Clone)]
pub struct SdfConfig {
    /// Résolution de la texture (32, 64, 128...)
    pub resolution: u32,
    /// Taille du chunk en unités monde
    pub chunk_world_size_x: f32,
    pub chunk_world_size_y: f32,
    /// Distance max encodée dans la SDF (en unités monde)
    pub max_distance: f32,
}

impl Default for SdfConfig {
    fn default() -> Self {
        Self {
            resolution: 64,
            chunk_world_size_x: 160.0, // 16 hex * ~10 unités par hex
            chunk_world_size_y: 160.0, // 16 hex * ~10 unités par hex
            max_distance: 30.0,        // 30 unités de transition plage
        }
    }
}
