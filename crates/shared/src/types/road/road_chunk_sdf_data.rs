use bincode::{Decode, Encode};

/// Données SDF des routes pour un chunk
/// Format optimisé pour le transfert réseau et le rendu GPU
#[derive(Debug, Clone, Encode, Decode)]
pub struct RoadChunkSdfData {
    /// Résolution de la grille SDF (typiquement 64-128)
    pub resolution_x: u16,
    pub resolution_y: u16,

    /// Données SDF encodées en RG16
    /// Stocké en row-major order: index = y * resolution_x + x
    /// Chaque pixel = 4 bytes (R16 + G16)
    ///
    /// Format par pixel (4 bytes):
    /// - R16 (2 bytes): Distance signée à la route encodée
    ///   * 0 = loin de la route (négatif)
    ///   * 32768 = sur la route (distance 0)
    ///   * 65535 = loin de la route (positif)
    ///
    /// - G16 (2 bytes): Métadonnées encodées
    ///   * Bits 0-7: Importance (0-3) * 64
    ///   * Bit 8: Flag ornières (1 si présentes)
    ///   * Bit 9: Flag intersection (1 si dans une intersection)
    ///   * Bits 10-15: Réservés
    pub data: Vec<u8>,
}

impl Default for RoadChunkSdfData {
    fn default() -> Self {
        Self::new(64, 64)
    }
}

impl RoadChunkSdfData {
    /// Crée une nouvelle grille SDF vide (pas de routes)
    pub fn new(resolution_x: u16, resolution_y: u16) -> Self {
        let pixel_count = (resolution_x as usize) * (resolution_y as usize);
        let byte_size = pixel_count * 4; // RG16 = 4 bytes par pixel

        // Initialiser avec distance maximale (pas de routes)
        let mut data = vec![0u8; byte_size];

        // Remplir avec distance maximale (0xFFFF en little-endian pour R16)
        for i in 0..pixel_count {
            let offset = i * 4;
            // R16 = 0xFFFF (distance maximale)
            data[offset] = 0xFF;
            data[offset + 1] = 0xFF;
            // G16 = 0x0000 (pas de métadonnées)
            data[offset + 2] = 0x00;
            data[offset + 3] = 0x00;
        }

        Self {
            resolution_x,
            resolution_y,
            data,
        }
    }

    /// Taille en bytes pour stockage DB
    pub fn byte_size(&self) -> usize {
        4 + self.data.len() // 2 bytes resolution_x + 2 bytes resolution_y + data
    }

    /// Récupère la valeur SDF à une position (x, y)
    /// Retourne (distance_raw, metadata_raw) ou None si hors limites
    pub fn get_pixel(&self, x: u16, y: u16) -> Option<(u16, u16)> {
        if x >= self.resolution_x || y >= self.resolution_y {
            return None;
        }

        let index = (y as usize * self.resolution_x as usize + x as usize) * 4;

        if index + 3 >= self.data.len() {
            return None;
        }

        // Lire R16 et G16 en little-endian
        let distance_raw = u16::from_le_bytes([self.data[index], self.data[index + 1]]);
        let metadata_raw = u16::from_le_bytes([self.data[index + 2], self.data[index + 3]]);

        Some((distance_raw, metadata_raw))
    }

    /// Définit la valeur SDF à une position (x, y)
    pub fn set_pixel(&mut self, x: u16, y: u16, distance_raw: u16, metadata_raw: u16) {
        if x >= self.resolution_x || y >= self.resolution_y {
            return;
        }

        let index = (y as usize * self.resolution_x as usize + x as usize) * 4;

        if index + 3 >= self.data.len() {
            return;
        }

        // Écrire R16 et G16 en little-endian
        let distance_bytes = distance_raw.to_le_bytes();
        let metadata_bytes = metadata_raw.to_le_bytes();

        self.data[index] = distance_bytes[0];
        self.data[index + 1] = distance_bytes[1];
        self.data[index + 2] = metadata_bytes[0];
        self.data[index + 3] = metadata_bytes[1];
    }
}
