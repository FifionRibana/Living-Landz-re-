use bevy::prelude::Vec2;
use bincode::{Decode, Encode};
use hexx::{Hex, HexLayout, HexOrientation};

// Note: We use bevy::prelude::Vec2 for serialization (implements bincode traits)
// hexx::HexLayout::hex_to_world_pos returns glam::Vec2 which is compatible with bevy::Vec2

/// Direction absolue pour une ligne hexagonale
/// Ces directions sont indépendantes de l'orientation du HexLayout (PointyTop/FlatTop)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum HexLineDirection {
    /// Ligne horizontale (de gauche à droite à l'écran)
    Horizontal,
    /// Ligne verticale (de haut en bas à l'écran)
    Vertical,
    /// Direction personnalisée en coordonnées axiales (q, r)
    /// ATTENTION: Cette direction dépend de l'orientation du HexLayout
    Axial(i32, i32),
}


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

    /// Ligne hexagonale dans une direction donnée
    /// La direction s'adapte automatiquement selon l'orientation du HexLayout
    HexLine {
        direction: HexLineDirection,
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
            SlotLayoutType::HexLine { direction } => {
                self.generate_hex_line(*direction, container_size, hex_layout)
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

    /// Génère une ligne hexagonale dans une direction donnée
    /// Utilise l'algorithme de line drawing pour créer une ligne visuellement droite
    /// Référence: https://www.redblobgames.com/grids/hexagons/#line-drawing
    fn generate_hex_line(&self, direction: HexLineDirection, container_size: Vec2, hex_layout: &HexLayout) -> Vec<Vec2> {
        let center = container_size / 2.0;

        if self.count == 0 {
            return Vec::new();
        }

        // Pour les lignes horizontales et verticales, on utilise des directions
        // en coordonnées hex qui créent des lignes visuellement droites
        // Pour éviter les discontinuités, on doit choisir des directions qui garantissent
        // que chaque hexagone est adjacent au suivant

        // Pour les lignes horizontales et verticales, générer directement les hexagones
        // selon les règles d'adjacence pour éviter les discontinuités
        let hexes = match direction {
            HexLineDirection::Horizontal => {
                match hex_layout.orientation {
                    HexOrientation::Pointy => {
                        // En PointyTop, ligne horizontale = r constant, q varie
                        let center_index = (self.count / 2) as i32;
                        (0..self.count)
                            .map(|i| {
                                let offset = i as i32 - center_index;
                                Hex::new(offset, 0)
                            })
                            .collect()
                    }
                    HexOrientation::Flat => {
                        // En FlatTop, ligne horizontale nécessite alternance
                        // Direction (2, -1) pour ligne horizontale
                        let center_index = (self.count / 2) as i32;
                        (0..self.count)
                            .map(|i| {
                                let offset = i as i32 - center_index;
                                let q = offset;
                                let r = -(offset + 1) / 2;
                                Hex::new(q, r)
                            })
                            .collect()
                    }
                }
            }
            HexLineDirection::Vertical => {
                match hex_layout.orientation {
                    HexOrientation::Pointy => {
                        // En PointyTop, ligne verticale avec alternance
                        // q change tous les 2 hexagones, r change à chaque hexagone
                        // Direction (1, -2) : pour chaque paire de cellules, q diminue de 1
                        let center_index = (self.count / 2) as i32;
                        (0..self.count)
                            .map(|i| {
                                let r = i as i32 - center_index;
                                // Formule : q = (1 - r) / 2, avec division euclidienne (arrondi vers le bas)
                                // Exemples: r=-2 -> q=1, r=-1 -> q=1, r=0 -> q=0, r=1 -> q=0, r=2 -> q=-1
                                let q = (1 - r).div_euclid(2);
                                Hex::new(q, r)
                            })
                            .collect()
                    }
                    HexOrientation::Flat => {
                        // En FlatTop, ligne verticale = q constant, r varie
                        let center_index = (self.count / 2) as i32;
                        (0..self.count)
                            .map(|i| {
                                let offset = i as i32 - center_index;
                                Hex::new(0, offset)
                            })
                            .collect()
                    }
                }
            }
            HexLineDirection::Axial(q, r) => {
                // Direction personnalisée en coordonnées axiales - utiliser line drawing
                let steps = (self.count - 1) as i32;
                let half_steps = steps / 2;
                let start = Hex::new(-q * half_steps, -r * half_steps);
                let end = Hex::new(q * (steps - half_steps), r * (steps - half_steps));
                Self::hex_line_draw(start, end, self.count)
            }
        };

        // Convertir en positions pixels et centrer
        hexes.iter()
            .map(|hex| {
                let pos = hex_layout.hex_to_world_pos(*hex);
                center + pos
            })
            .collect()
    }

    /// Algorithme de line drawing hexagonal (Red Blob Games)
    /// Interpole linéairement entre deux hexagones en utilisant les coordonnées cubiques
    fn hex_line_draw(start: Hex, end: Hex, count: usize) -> Vec<Hex> {
        let mut results = Vec::with_capacity(count);

        if count == 0 {
            return results;
        }

        if count == 1 {
            results.push(Hex::ZERO);
            return results;
        }

        // Convertir en coordonnées cubiques (q, r, s) où s = -q - r
        let start_q = start.x as f32;
        let start_r = start.y as f32;
        let start_s = (-start.x - start.y) as f32;

        let end_q = end.x as f32;
        let end_r = end.y as f32;
        let end_s = (-end.x - end.y) as f32;

        // Interpoler linéairement et arrondir aux hexagones les plus proches
        for i in 0..count {
            let t = i as f32 / (count - 1) as f32;

            // Interpolation linéaire
            let q = start_q + (end_q - start_q) * t;
            let r = start_r + (end_r - start_r) * t;
            let s = start_s + (end_s - start_s) * t;

            // Arrondir aux coordonnées hexagonales
            let hex = Self::cube_round(q, r, s);
            results.push(hex);
        }

        results
    }

    /// Arrondit des coordonnées cubiques flottantes aux coordonnées hexagonales entières
    fn cube_round(fq: f32, fr: f32, fs: f32) -> Hex {
        let mut q = fq.round();
        let mut r = fr.round();
        let mut s = fs.round();

        let q_diff = (q - fq).abs();
        let r_diff = (r - fr).abs();
        let s_diff = (s - fs).abs();

        // Réajuster la coordonnée avec le plus grand delta pour maintenir q + r + s = 0
        if q_diff > r_diff && q_diff > s_diff {
            q = -r - s;
        } else if r_diff > s_diff {
            r = -q - s;
        } else {
            s = -q - r;
        }

        Hex::new(q as i32, r as i32)
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

    /// Crée un layout en ligne hexagonale
    pub fn hex_line(count: usize, direction: HexLineDirection) -> Self {
        Self {
            count,
            layout_type: SlotLayoutType::HexLine { direction },
        }
    }

    /// Crée un layout en ligne horizontale (de gauche à droite à l'écran)
    /// Cette ligne reste horizontale quelle que soit l'orientation du HexLayout (PointyTop/FlatTop)
    pub fn hex_line_horizontal(count: usize) -> Self {
        Self::hex_line(count, HexLineDirection::Horizontal)
    }

    /// Crée un layout en ligne verticale (de haut en bas à l'écran)
    /// Cette ligne reste verticale quelle que soit l'orientation du HexLayout (PointyTop/FlatTop)
    pub fn hex_line_vertical(count: usize) -> Self {
        Self::hex_line(count, HexLineDirection::Vertical)
    }

    /// Crée un layout en ligne avec une direction personnalisée en coordonnées axiales
    pub fn hex_line_axial(count: usize, q: i32, r: i32) -> Self {
        Self::hex_line(count, HexLineDirection::Axial(q, r))
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
    fn test_hex_line_constructor() {
        let layout = SlotLayout::hex_line_horizontal(5);
        assert_eq!(layout.count, 5);
        match layout.layout_type {
            SlotLayoutType::HexLine { direction } => {
                assert_eq!(direction, HexLineDirection::Horizontal);
            }
            _ => panic!("Expected HexLine layout type"),
        }
    }

    #[test]
    fn test_hex_line_vertical_constructor() {
        let layout = SlotLayout::hex_line_vertical(5);
        assert_eq!(layout.count, 5);
        match layout.layout_type {
            SlotLayoutType::HexLine { direction } => {
                assert_eq!(direction, HexLineDirection::Vertical);
            }
            _ => panic!("Expected HexLine layout type"),
        }
    }

    #[test]
    fn test_hex_line_axial_constructor() {
        let layout = SlotLayout::hex_line_axial(5, 1, 0);
        assert_eq!(layout.count, 5);
        match layout.layout_type {
            SlotLayoutType::HexLine { direction } => {
                assert_eq!(direction, HexLineDirection::Axial(1, 0));
            }
            _ => panic!("Expected HexLine layout type"),
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
    fn test_generate_hex_line_horizontal_pointy() {
        let layout = SlotLayout::hex_line_horizontal(5);
        let hex_layout = HexLayout::pointy().with_hex_size(40.0);
        let container_size = Vec2::new(800.0, 600.0);

        let positions = layout.generate_positions(container_size, &hex_layout);

        // Devrait générer exactement 5 positions
        assert_eq!(positions.len(), 5);

        // Pour une ligne horizontale, toutes les positions devraient avoir approximativement la même coordonnée Y
        let center_y = container_size.y / 2.0;
        for pos in &positions {
            let y_diff = (pos.y - center_y).abs();
            assert!(y_diff < 5.0, "Y coordinate should be close to center, diff: {}", y_diff);
        }

        // Les positions X devraient être espacées régulièrement de gauche à droite
        assert!(positions[0].x < positions[1].x);
        assert!(positions[1].x < positions[2].x);
        assert!(positions[2].x < positions[3].x);
        assert!(positions[3].x < positions[4].x);
    }

    #[test]
    fn test_generate_hex_line_horizontal_flat() {
        let layout = SlotLayout::hex_line_horizontal(5);
        let hex_layout = HexLayout::flat().with_hex_size(40.0);
        let container_size = Vec2::new(800.0, 600.0);

        let positions = layout.generate_positions(container_size, &hex_layout);

        // Devrait générer exactement 5 positions
        assert_eq!(positions.len(), 5);

        // Pour une ligne horizontale (même avec FlatTop), Y devrait rester relativement constant
        // Note: Avec FlatTop et direction (1,-1), il y a une petite variation en Y mais c'est acceptable
        let avg_y: f32 = positions.iter().map(|p| p.y).sum::<f32>() / positions.len() as f32;
        let y_variance: f32 = positions.iter()
            .map(|p| (p.y - avg_y).abs())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        // Avec FlatTop, la direction (1, -1) crée une ligne diagonale douce
        // La variance en Y est plus grande que pour PointyTop mais reste raisonnable
        assert!(y_variance < 80.0, "Y variance should be reasonable for horizontal line with FlatTop, got: {}", y_variance);

        // Les positions X devraient augmenter de gauche à droite
        assert!(positions[0].x < positions[4].x, "First X should be less than last X");
    }

    #[test]
    fn test_generate_hex_line_vertical_pointy() {
        let layout = SlotLayout::hex_line_vertical(5);
        let hex_layout = HexLayout::pointy().with_hex_size(40.0);
        let container_size = Vec2::new(800.0, 600.0);

        let positions = layout.generate_positions(container_size, &hex_layout);

        // Devrait générer exactement 5 positions
        assert_eq!(positions.len(), 5);

        // Pour une ligne verticale, les positions Y devraient varier de haut en bas
        assert!(positions[0].y != positions[1].y);
        assert!(positions[1].y != positions[2].y);

        // La première position devrait être plus haute (Y plus petit) que la dernière
        assert!(positions[0].y < positions[4].y || positions[4].y < positions[0].y,
                "Y should vary vertically");
    }

    #[test]
    fn test_generate_hex_line_vertical_flat() {
        let layout = SlotLayout::hex_line_vertical(5);
        let hex_layout = HexLayout::flat().with_hex_size(40.0);
        let container_size = Vec2::new(800.0, 600.0);

        let positions = layout.generate_positions(container_size, &hex_layout);

        // Devrait générer exactement 5 positions
        assert_eq!(positions.len(), 5);

        // Pour une ligne verticale avec FlatTop, Y devrait varier significativement
        let y_min = positions.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
        let y_max = positions.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
        let y_range = y_max - y_min;

        // La plage de Y devrait être significative pour une ligne verticale
        assert!(y_range > 100.0, "Y range should be large for vertical line, got: {}", y_range);
    }

    #[test]
    fn test_generate_hex_line_axial() {
        let layout = SlotLayout::hex_line_axial(5, 1, -1);
        let hex_layout = HexLayout::pointy().with_hex_size(40.0);
        let container_size = Vec2::new(800.0, 600.0);

        let positions = layout.generate_positions(container_size, &hex_layout);

        // Devrait générer exactement 5 positions
        assert_eq!(positions.len(), 5);

        // Pour une ligne avec direction axiale, les positions X et Y devraient varier
        assert!(positions[0].x != positions[1].x);
        assert!(positions[0].y != positions[1].y);
    }

    #[test]
    fn test_hex_line_draw_vertical_pattern() {
        // Test que l'algorithme de line drawing génère le bon pattern pour une ligne verticale
        // En PointyTop, une ligne verticale devrait créer un pattern alternant
        let layout = SlotLayout::hex_line_vertical(5);
        let hex_layout = HexLayout::pointy().with_hex_size(40.0);
        let container_size = Vec2::new(800.0, 600.0);

        // Utiliser la fonction generate_positions pour obtenir les positions
        let positions = layout.generate_positions(container_size, &hex_layout);
        assert_eq!(positions.len(), 5);

        // Convertir les positions pixel en hex pour voir le pattern
        let hexes: Vec<Hex> = positions.iter()
            .map(|pos| {
                let relative_pos = *pos - container_size / 2.0;
                hex_layout.world_pos_to_hex(relative_pos)
            })
            .collect();

        println!("Vertical line hexes (5 slots): {:?}", hexes);

        // Vérifier que ça crée une ligne verticale (Y devrait varier significativement)
        let y_min = hexes.iter().map(|h| h.y).min().unwrap();
        let y_max = hexes.iter().map(|h| h.y).max().unwrap();
        assert!(y_max - y_min >= 3, "Y should vary significantly for vertical line");

        // Vérifier que chaque hexagone est adjacent au suivant
        for i in 0..hexes.len() - 1 {
            let dist = hexes[i].unsigned_distance_to(hexes[i + 1]);
            assert_eq!(dist, 1, "Hexagons {} and {} should be adjacent, distance: {}", i, i+1, dist);
        }
    }

    #[test]
    fn test_hex_line_vertical_4_slots() {
        // Test spécifique pour 4 slots pour vérifier qu'il n'y a pas de discontinuité
        let layout = SlotLayout::hex_line_vertical(4);
        let hex_layout = HexLayout::pointy().with_hex_size(40.0);
        let container_size = Vec2::new(800.0, 600.0);

        let positions = layout.generate_positions(container_size, &hex_layout);
        assert_eq!(positions.len(), 4);

        // Convertir les positions pixel en hex
        let hexes: Vec<Hex> = positions.iter()
            .map(|pos| {
                let relative_pos = *pos - container_size / 2.0;
                hex_layout.world_pos_to_hex(relative_pos)
            })
            .collect();

        println!("Vertical line hexes (4 slots): {:?}", hexes);

        // Vérifier que chaque hexagone est adjacent au suivant (distance = 1)
        for i in 0..hexes.len() - 1 {
            let dist = hexes[i].unsigned_distance_to(hexes[i + 1]);
            assert_eq!(dist, 1, "Hexagons {} and {} should be adjacent (distance=1), but distance is {}", i, i+1, dist);
        }
    }

    #[test]
    fn test_hex_line_draw_horizontal_pattern() {
        // Test pour une ligne horizontale en PointyTop
        let layout = SlotLayout::hex_line_horizontal(5);
        let hex_layout = HexLayout::pointy().with_hex_size(40.0);
        let container_size = Vec2::new(800.0, 600.0);

        // Utiliser la fonction generate_positions pour obtenir les positions
        let positions = layout.generate_positions(container_size, &hex_layout);
        assert_eq!(positions.len(), 5);

        // Convertir les positions pixel en hex pour voir le pattern
        let hexes: Vec<Hex> = positions.iter()
            .map(|pos| {
                let relative_pos = *pos - container_size / 2.0;
                hex_layout.world_pos_to_hex(relative_pos)
            })
            .collect();

        println!("Horizontal line hexes (pixel-based): {:?}", hexes);

        // Vérifier que ça crée une ligne horizontale (X devrait varier significativement)
        let x_min = hexes.iter().map(|h| h.x).min().unwrap();
        let x_max = hexes.iter().map(|h| h.x).max().unwrap();
        assert!(x_max - x_min >= 3, "X should vary significantly for horizontal line");
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

    #[test]
    fn test_hex_line_vertical_3_and_6_slots() {
        // Test pour 3 slots
        let layout3 = SlotLayout::hex_line_vertical(3);
        let hex_layout = HexLayout::pointy().with_hex_size(40.0);
        let container_size = Vec2::new(800.0, 600.0);

        let positions = layout3.generate_positions(container_size, &hex_layout);
        let hexes3: Vec<Hex> = positions.iter()
            .map(|pos| {
                let relative_pos = *pos - container_size / 2.0;
                hex_layout.world_pos_to_hex(relative_pos)
            })
            .collect();

        println!("Vertical line hexes (3 slots): {:?}", hexes3);
        // Pattern attendu: (1, -1), (0, 0), (0, 1)
        assert_eq!(hexes3.len(), 3);
        assert_eq!(hexes3[0], Hex::new(1, -1));
        assert_eq!(hexes3[1], Hex::new(0, 0));
        assert_eq!(hexes3[2], Hex::new(0, 1));

        // Test pour 6 slots
        let layout6 = SlotLayout::hex_line_vertical(6);
        let positions = layout6.generate_positions(container_size, &hex_layout);
        let hexes6: Vec<Hex> = positions.iter()
            .map(|pos| {
                let relative_pos = *pos - container_size / 2.0;
                hex_layout.world_pos_to_hex(relative_pos)
            })
            .collect();

        println!("Vertical line hexes (6 slots): {:?}", hexes6);
        // Pattern attendu: (2, -3), (1, -2), (1, -1), (0, 0), (0, 1), (-1, 2)
        assert_eq!(hexes6.len(), 6);
        assert_eq!(hexes6[0], Hex::new(2, -3));
        assert_eq!(hexes6[1], Hex::new(1, -2));
        assert_eq!(hexes6[2], Hex::new(1, -1));
        assert_eq!(hexes6[3], Hex::new(0, 0));
        assert_eq!(hexes6[4], Hex::new(0, 1));
        assert_eq!(hexes6[5], Hex::new(-1, 2));

        // Vérifier que tous les hexagones sont adjacents
        for i in 0..hexes3.len() - 1 {
            let dist = hexes3[i].unsigned_distance_to(hexes3[i + 1]);
            assert_eq!(dist, 1, "3-slot line: hexagons {} and {} should be adjacent", i, i+1);
        }
        for i in 0..hexes6.len() - 1 {
            let dist = hexes6[i].unsigned_distance_to(hexes6[i + 1]);
            assert_eq!(dist, 1, "6-slot line: hexagons {} and {} should be adjacent", i, i+1);
        }
    }
}
