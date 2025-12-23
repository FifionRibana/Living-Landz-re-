use bevy::prelude::Vec2;
use bincode::{Decode, Encode};
use hexx::{Hex, HexLayout};

// Note: We use bevy::prelude::Vec2 for serialization (implements bincode traits)
// hexx::HexLayout::hex_to_world_pos returns glam::Vec2 which is compatible with bevy::Vec2

/// Type de layout pour les emplacements d'unités
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub enum SlotLayoutType {
    /// Anneau hexagonal autour d'un centre (utilise hexx::Hex::ring)
    /// Un anneau de rayon 1 contient 6 slots, rayon 2 contient 12 slots, etc.
    HexRing {
        radius: u32,
    },

    /// Zone hexagonale remplie (utilise hexx::Hex::range)
    /// Une zone de rayon 1 contient 7 slots (centre + 6 autour), rayon 2 contient 19 slots, etc.
    HexRange {
        radius: u32,
    },

    /// Grille hexagonale rectangulaire
    /// Génère une grille de width × height hexagones en coordonnées axiales
    HexGrid {
        width: u32,
        height: u32,
    },

    /// Positions personnalisées en pixels relatifs au centre du conteneur
    /// (0, 0) représente le centre du conteneur
    /// Stockées comme (x, y) tuples pour la sérialisation bincode
    Custom {
        positions: Vec<(f32, f32)>,
    },
}

/// Configuration de layout pour un ensemble de slots
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct SlotLayout {
    /// Nombre total de slots dans ce layout
    pub count: usize,
    /// Type de layout à utiliser
    pub layout_type: SlotLayoutType,
}

impl SlotLayout {
    /// Génère les positions absolues (x, y) en pixels pour tous les slots
    ///
    /// # Arguments
    /// * `container_size` - Taille du conteneur parent en pixels
    /// * `hex_layout` - Configuration hexx pour convertir coordonnées hex → pixels
    ///
    /// # Returns
    /// Vecteur de positions Vec2 en coordonnées absolues (pixels depuis le coin supérieur gauche)
    pub fn generate_positions(&self, container_size: Vec2, hex_layout: &HexLayout) -> Vec<Vec2> {
        match &self.layout_type {
            SlotLayoutType::HexRing { radius } => {
                self.generate_hex_ring(*radius, container_size, hex_layout)
            }
            SlotLayoutType::HexRange { radius } => {
                self.generate_hex_range(*radius, container_size, hex_layout)
            }
            SlotLayoutType::HexGrid { width, height } => {
                self.generate_hex_grid(*width, *height, container_size, hex_layout)
            }
            SlotLayoutType::Custom { positions } => {
                self.apply_custom_positions(positions, container_size)
            }
        }
    }

    /// Génère un anneau hexagonal en utilisant hexx::Hex::ring
    fn generate_hex_ring(&self, radius: u32, container_size: Vec2, hex_layout: &HexLayout) -> Vec<Vec2> {
        let center = container_size / 2.0;

        // Générer l'anneau avec hexx
        let hexes: Vec<Hex> = Hex::ZERO
            .ring(radius)
            .take(self.count)
            .collect();

        // Convertir en positions pixels et centrer
        hexes.iter()
            .map(|hex| {
                let pos = hex_layout.hex_to_world_pos(*hex);
                center + pos
            })
            .collect()
    }

    /// Génère une zone hexagonale remplie en utilisant hexx::Hex::range
    fn generate_hex_range(&self, radius: u32, container_size: Vec2, hex_layout: &HexLayout) -> Vec<Vec2> {
        let center = container_size / 2.0;

        // Générer la zone remplie avec hexx
        let hexes: Vec<Hex> = Hex::ZERO
            .range(radius)
            .take(self.count)
            .collect();

        // Convertir en positions pixels et centrer
        hexes.iter()
            .map(|hex| {
                let pos = hex_layout.hex_to_world_pos(*hex);
                center + pos
            })
            .collect()
    }

    /// Génère une grille hexagonale rectangulaire
    fn generate_hex_grid(&self, width: u32, height: u32, container_size: Vec2, hex_layout: &HexLayout) -> Vec<Vec2> {
        let center = container_size / 2.0;
        let mut hexes = Vec::new();

        // Générer une grille rectangulaire en coordonnées axiales
        for row in 0..height as i32 {
            for col in 0..width as i32 {
                if hexes.len() >= self.count {
                    break;
                }

                // Offset pour centrer la grille
                let q = col - (width as i32 / 2);
                let r = row - (height as i32 / 2);
                hexes.push(Hex::new(q, r));
            }
            if hexes.len() >= self.count {
                break;
            }
        }

        // Convertir en positions pixels
        hexes.iter()
            .map(|hex| {
                let pos = hex_layout.hex_to_world_pos(*hex);
                center + pos
            })
            .collect()
    }

    /// Applique des positions personnalisées relatives au centre
    fn apply_custom_positions(&self, positions: &[(f32, f32)], container_size: Vec2) -> Vec<Vec2> {
        let center = container_size / 2.0;
        positions.iter()
            .map(|(x, y)| center + Vec2::new(*x, *y))
            .collect()
    }

    // ===== Constructeurs de commodité =====

    /// Crée un layout en anneau hexagonal
    pub fn hex_ring(count: usize, radius: u32) -> Self {
        Self {
            count,
            layout_type: SlotLayoutType::HexRing { radius },
        }
    }

    /// Crée un layout en zone hexagonale remplie
    pub fn hex_range(count: usize, radius: u32) -> Self {
        Self {
            count,
            layout_type: SlotLayoutType::HexRange { radius },
        }
    }

    /// Crée un layout en grille hexagonale rectangulaire
    pub fn hex_grid(count: usize, width: u32, height: u32) -> Self {
        Self {
            count,
            layout_type: SlotLayoutType::HexGrid { width, height },
        }
    }

    /// Crée un layout avec positions personnalisées
    pub fn custom(positions: Vec<Vec2>) -> Self {
        let stored_positions: Vec<(f32, f32)> = positions
            .iter()
            .map(|v| (v.x, v.y))
            .collect();
        Self {
            count: stored_positions.len(),
            layout_type: SlotLayoutType::Custom { positions: stored_positions },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_ring_constructor() {
        let layout = SlotLayout::hex_ring(6, 1);
        assert_eq!(layout.count, 6);
        match layout.layout_type {
            SlotLayoutType::HexRing { radius } => assert_eq!(radius, 1),
            _ => panic!("Expected HexRing layout type"),
        }
    }

    #[test]
    fn test_hex_range_constructor() {
        let layout = SlotLayout::hex_range(7, 1);
        assert_eq!(layout.count, 7);
        match layout.layout_type {
            SlotLayoutType::HexRange { radius } => assert_eq!(radius, 1),
            _ => panic!("Expected HexRange layout type"),
        }
    }

    #[test]
    fn test_hex_grid_constructor() {
        let layout = SlotLayout::hex_grid(6, 2, 3);
        assert_eq!(layout.count, 6);
        match layout.layout_type {
            SlotLayoutType::HexGrid { width, height } => {
                assert_eq!(width, 2);
                assert_eq!(height, 3);
            }
            _ => panic!("Expected HexGrid layout type"),
        }
    }

    #[test]
    fn test_custom_constructor() {
        let positions = vec![Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0)];
        let layout = SlotLayout::custom(positions.clone());
        assert_eq!(layout.count, 2);
        match layout.layout_type {
            SlotLayoutType::Custom { positions: pos } => {
                assert_eq!(pos.len(), 2);
                assert_eq!(pos[0], (0.0, 0.0));
                assert_eq!(pos[1], (10.0, 10.0));
            }
            _ => panic!("Expected Custom layout type"),
        }
    }

    #[test]
    fn test_generate_hex_ring_positions() {
        let layout = SlotLayout::hex_ring(6, 1);
        let hex_layout = HexLayout::pointy().with_hex_size(40.0);
        let container_size = Vec2::new(800.0, 600.0);

        let positions = layout.generate_positions(container_size, &hex_layout);

        // Un anneau de rayon 1 devrait générer 6 positions
        assert_eq!(positions.len(), 6);

        // Toutes les positions devraient être autour du centre
        let center = container_size / 2.0;
        for pos in &positions {
            let distance = pos.distance(center);
            // La distance devrait être approximativement la même pour toutes les positions
            assert!(distance > 30.0 && distance < 50.0, "Distance: {}", distance);
        }
    }

    #[test]
    fn test_generate_hex_grid_positions() {
        let layout = SlotLayout::hex_grid(6, 3, 2);
        let hex_layout = HexLayout::pointy().with_hex_size(40.0);
        let container_size = Vec2::new(800.0, 600.0);

        let positions = layout.generate_positions(container_size, &hex_layout);

        // Devrait générer exactement 6 positions
        assert_eq!(positions.len(), 6);
    }

    #[test]
    fn test_apply_custom_positions() {
        let custom_positions = vec![
            Vec2::new(-50.0, -50.0),
            Vec2::new(50.0, 50.0),
        ];
        let layout = SlotLayout::custom(custom_positions.clone());
        let container_size = Vec2::new(800.0, 600.0);
        let hex_layout = HexLayout::pointy().with_hex_size(40.0);

        let positions = layout.generate_positions(container_size, &hex_layout);

        assert_eq!(positions.len(), 2);

        // Les positions devraient être décalées par rapport au centre
        let center = container_size / 2.0;
        assert_eq!(positions[0], center + Vec2::new(-50.0, -50.0));
        assert_eq!(positions[1], center + Vec2::new(50.0, 50.0));
    }
}
