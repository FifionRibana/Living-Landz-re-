// === Channels ===

/// RPC-style messages (actions, login data). Bidirectional, ordered reliable.
pub struct ReliableGameChannel;

/// Terrain chunk data. Server → client, unordered reliable (chunks arrive in any order).
pub struct TerrainChunkChannel;

/// Ocean SDF + heightmap. Server → client, ordered reliable.
pub struct OceanDataChannel;

/// Lake mask + SDF. Server → client, ordered reliable.
pub struct LakeDataChannel;

/// Biome + heightmap global textures. Server → client, ordered reliable.
pub struct TerrainGlobalChannel;

/// Full exploration map + incremental patches. Server → client, unordered reliable.
pub struct ExplorationChannel;
