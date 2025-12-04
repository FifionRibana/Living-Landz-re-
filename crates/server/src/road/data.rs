use bevy::prelude::*;
use shared::{constants, RoadSegmentData, grid::GridCell};
use std::collections::HashMap;

// ============================================================================
// CONFIGURATION GLOBALE
// ============================================================================

/// Configuration du système de routes (Resource Bevy)
#[derive(Resource, Clone)]
pub struct RoadConfig {
    // --- Dimensions ---
    /// Résolution de la texture SDF des routes (adaptée au chunk)
    pub sdf_resolution: UVec2,
    /// Taille du chunk en unités monde (utilise les constantes existantes)
    pub chunk_size: Vec2,

    // --- Largeurs ---
    /// Largeur de base d'une route (unités monde / pixels)
    pub base_width: f32,
    /// Largeur additionnelle par niveau d'importance
    pub width_per_importance: f32,

    // --- Ornières (routes importantes) ---
    /// Décalage de l'ornière gauche par rapport au centre
    pub track_offset_left: f32,
    /// Décalage de l'ornière droite (légèrement différent pour asymétrie)
    pub track_offset_right: f32,
    /// Largeur des ornières
    pub track_width: f32,
    /// Seuil d'importance pour afficher les ornières doubles
    pub double_track_threshold: u8,

    // --- Intersections ---
    /// Rayon de base d'une intersection
    pub intersection_base_radius: f32,
    /// Rayon additionnel par connexion
    pub intersection_radius_per_connection: f32,
    /// Angle minimum (radians) pour considérer une fourche vs continuation
    pub fork_angle_threshold: f32,
    /// Facteur de lissage pour l'union route/intersection (smooth min)
    pub intersection_smoothness: f32,

    // --- Rendu visuel ---
    /// Couleur terre claire (centre de la route) - RGB normalisé
    pub color_light: Vec3,
    /// Couleur terre sombre (bords de la route)
    pub color_dark: Vec3,
    /// Couleur des ornières (encore plus sombre)
    pub color_tracks: Vec3,
    /// Largeur de la transition douce vers l'herbe
    pub edge_softness: f32,
    /// Fréquence du bruit pour les bords organiques
    pub noise_frequency: f32,
    /// Amplitude du bruit
    pub noise_amplitude: f32,
}

impl Default for RoadConfig {
    fn default() -> Self {
        Self {
            // Résolution du SDF (1024x1024 pour une meilleure qualité visuelle)
            sdf_resolution: UVec2::new(1024, 1024),
            chunk_size: constants::CHUNK_SIZE,

            base_width: 12.0,  // Largeur de base plus visible
            width_per_importance: 4.0,  // Augmentation par niveau d'importance

            track_offset_left: 3.0,
            track_offset_right: 3.75,  // Asymétrie subtile
            track_width: 1.5,
            double_track_threshold: 2,

            intersection_base_radius: 18.0,  // Proportionnel à la largeur de base
            intersection_radius_per_connection: 6.0,  // Proportionnel aussi
            fork_angle_threshold: 0.4,  // ~23 degrés
            intersection_smoothness: 3.0,

            color_light: Vec3::new(0.76, 0.70, 0.55),
            color_dark: Vec3::new(0.55, 0.48, 0.38),
            color_tracks: Vec3::new(0.40, 0.35, 0.28),
            edge_softness: 2.0,
            noise_frequency: 0.15,
            noise_amplitude: 3.0,
        }
    }
}

// ============================================================================
// DONNÉES DES ROUTES (côté serveur)
// ============================================================================

/// Un segment de route côté serveur (version étendue de RoadSegmentData)
#[derive(Clone, Debug)]
pub struct RoadSegment {
    /// ID du segment (provient de la DB)
    pub id: i64,

    /// Cellule de départ
    pub start_cell: GridCell,

    /// Cellule d'arrivée
    pub end_cell: GridCell,

    /// Points de passage formant une polyline (coordonnées monde)
    pub points: Vec<Vec2>,

    /// Importance du segment (0-3)
    pub importance: u8,
}

impl RoadSegment {
    /// Convertit vers le format réseau
    pub fn to_network_data(&self) -> RoadSegmentData {
        RoadSegmentData {
            id: self.id,
            start_cell: self.start_cell,
            end_cell: self.end_cell,
            points: self.points.iter().map(|p| p.to_array()).collect(),
            importance: self.importance,
        }
    }

    /// Crée depuis le format réseau
    pub fn from_network_data(data: &RoadSegmentData) -> Self {
        Self {
            id: data.id,
            start_cell: data.start_cell,
            end_cell: data.end_cell,
            points: data.points.iter().map(|&p| Vec2::from(p)).collect(),
            importance: data.importance,
        }
    }

    /// Retourne la direction au départ du segment (normalisée)
    pub fn start_direction(&self) -> Vec2 {
        if self.points.len() >= 2 {
            (self.points[1] - self.points[0]).normalize_or_zero()
        } else {
            Vec2::X
        }
    }

    /// Retourne la direction à la fin du segment (normalisée, pointant vers l'intérieur)
    pub fn end_direction(&self) -> Vec2 {
        let n = self.points.len();
        if n >= 2 {
            (self.points[n - 2] - self.points[n - 1]).normalize_or_zero()
        } else {
            Vec2::X
        }
    }
}

// ============================================================================
// INTERSECTIONS
// ============================================================================

/// Type d'intersection basé sur le nombre et l'angle des connexions
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IntersectionType {
    /// Terminus (1 seule connexion)
    Terminus,
    /// 2 connexions alignées (angle ~180°)
    Continuation,
    /// 2 connexions en angle
    Fork,
    /// 3 connexions
    Junction,
    /// 4 connexions
    Crossroad,
    /// 5+ connexions
    Plaza,
}

impl IntersectionType {
    /// Détermine le type basé sur les directions des routes connectées
    pub fn classify(directions: &[Vec2], fork_angle_threshold: f32) -> Self {
        match directions.len() {
            0 | 1 => Self::Terminus,
            2 => {
                let dot = directions[0].dot(directions[1]);
                if dot < -fork_angle_threshold.cos() {
                    Self::Continuation
                } else {
                    Self::Fork
                }
            }
            3 => Self::Junction,
            4 => Self::Crossroad,
            _ => Self::Plaza,
        }
    }

    /// Retourne le facteur multiplicateur de rayon pour ce type
    pub fn radius_factor(&self) -> f32 {
        match self {
            Self::Terminus => 0.5,
            Self::Continuation => 0.8,
            Self::Fork => 1.0,
            Self::Junction => 1.3,
            Self::Crossroad => 1.5,
            Self::Plaza => 2.0,
        }
    }
}

/// Données d'une intersection calculée
#[derive(Clone, Debug)]
pub struct Intersection {
    /// Position dans l'espace monde
    pub position: Vec2,

    /// Cellule hexagonale
    pub cell: GridCell,

    /// Type d'intersection détecté
    pub intersection_type: IntersectionType,

    /// Directions normalisées vers chaque route connectée
    pub connected_directions: Vec<Vec2>,

    /// Rayon calculé de la placette
    pub radius: f32,

    /// Importance maximale des routes connectées
    pub importance: u8,
}

// ============================================================================
// GESTION DES ROUTES PAR CHUNK
// ============================================================================

/// Gestionnaire des routes pour un chunk spécifique
/// Stocké comme Resource avec une HashMap par chunk
#[derive(Default, Clone)]
pub struct ChunkRoads {
    /// Segments de route présents dans ce chunk
    pub segments: Vec<RoadSegment>,

    /// Intersections calculées (regénérées quand dirty)
    pub intersections: Vec<Intersection>,

    /// Flag indiquant que le SDF doit être regénéré
    pub dirty: bool,
}

impl ChunkRoads {
    /// Ajoute un segment et marque le chunk comme dirty
    pub fn add_segment(&mut self, segment: RoadSegment) {
        self.segments.push(segment);
        self.dirty = true;
    }

    /// Supprime un segment par ID et marque dirty
    pub fn remove_segment(&mut self, segment_id: i64) -> bool {
        let initial_len = self.segments.len();
        self.segments.retain(|s| s.id != segment_id);
        let removed = self.segments.len() < initial_len;
        if removed {
            self.dirty = true;
        }
        removed
    }

    /// Vérifie si le chunk contient des routes
    pub fn has_roads(&self) -> bool {
        !self.segments.is_empty()
    }
}

/// Resource globale contenant toutes les routes par chunk
#[derive(Resource, Default)]
pub struct WorldRoads {
    pub chunks: HashMap<(i32, i32), ChunkRoads>,
}

impl WorldRoads {
    /// Récupère ou crée les routes d'un chunk
    pub fn get_or_create(&mut self, chunk_x: i32, chunk_y: i32) -> &mut ChunkRoads {
        self.chunks.entry((chunk_x, chunk_y)).or_default()
    }

    /// Récupère les routes d'un chunk (lecture seule)
    pub fn get(&self, chunk_x: i32, chunk_y: i32) -> Option<&ChunkRoads> {
        self.chunks.get(&(chunk_x, chunk_y))
    }
}
