use bincode::{Decode, Encode};

#[derive(Debug, Clone, Default, Encode, Decode)]
pub struct HeightmapChunkData {
    /// Résolution de la heightmap (ex: 64x64)
    pub resolution: u8,

    /// Valeurs de la heightmap (normalisées 0-255)
    /// Taille: resolution * resolution
    pub values: Vec<u8>,
}

impl HeightmapChunkData {
    pub fn new(resolution: u8) -> Self {
        Self {
            resolution,
            values: vec![0; (resolution as usize) * (resolution as usize)],
        }
    }

    pub fn from_values(resolution: u8, values: Vec<u8>) -> Self {
        assert_eq!(
            values.len(),
            (resolution as usize) * (resolution as usize),
            "Heightmap values size mismatch"
        );
        Self { resolution, values }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<u8> {
        let idx = y * self.resolution as usize + x;
        self.values.get(idx).copied()
    }

    pub fn set(&mut self, x: usize, y: usize, value: u8) {
        let idx = y * self.resolution as usize + x;
        if idx < self.values.len() {
            self.values[idx] = value;
        }
    }
}
