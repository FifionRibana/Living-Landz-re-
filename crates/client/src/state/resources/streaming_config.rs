use bevy::prelude::*;
use shared::TerrainChunkId;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Max chunks that can be in-flight (requested but not yet received) at once.
pub const MAX_IN_FLIGHT_CHUNKS: usize = 12;

/// Don't re-request a chunk that was unloaded less than this long ago.
const RECENTLY_UNLOADED_GRACE: Duration = Duration::from_secs(5);

/// Don't unload a chunk that was loaded less than this long ago.
pub const LOAD_GRACE_PERIOD: Duration = Duration::from_secs(3);

#[derive(Resource)]
pub struct StreamingConfig {
    pub view_radius: i32,
    pub unload_distance: i32,
    pub request_cooldown: f32,
    pub request_timeout: f32,
    pub last_request: f32,
    /// Chunks that were recently unloaded — don't re-request them.
    pub recently_unloaded: HashMap<TerrainChunkId, Instant>,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            view_radius: 4,
            unload_distance: 6,
            request_cooldown: 0.3,
            request_timeout: 3.0,
            last_request: -999.0,
            recently_unloaded: HashMap::new(),
        }
    }
}

impl StreamingConfig {
    /// Returns true if the chunk was unloaded recently and should not be re-requested.
    pub fn was_recently_unloaded(&self, id: &TerrainChunkId) -> bool {
        self.recently_unloaded
            .get(id)
            .map(|t| t.elapsed() < RECENTLY_UNLOADED_GRACE)
            .unwrap_or(false)
    }

    /// Mark a chunk as recently unloaded.
    pub fn mark_unloaded(&mut self, id: TerrainChunkId) {
        self.recently_unloaded.insert(id, Instant::now());
    }

    /// Prune old entries from recently_unloaded.
    pub fn prune_old_unloads(&mut self) {
        self.recently_unloaded
            .retain(|_, t| t.elapsed() < RECENTLY_UNLOADED_GRACE);
    }
}
