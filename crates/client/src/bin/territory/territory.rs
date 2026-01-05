use std::collections::{HashMap, HashSet};

use bevy::{
    asset::RenderAssetUsages,
    gizmos::gizmos::Gizmos,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
    render::storage::ShaderStorageBuffer,
};
use hexx::*;
use shared::grid::GridConfig;

#[derive(Component)]
pub struct TerritoryHex;

pub fn create_hexagonal_mesh(layout: HexLayout, _radius: f32) -> Mesh {
    // Utilise ColumnMeshBuilder de hexx pour un hexagone plat
    let mesh_info = PlaneMeshBuilder::new(&layout)
        .facing(Vec3::Z)
        .center_aligned()
        .build();

    hexagonal_mesh(mesh_info)
}

pub fn hexagonal_mesh(mesh_info: MeshInfo) -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        // Means you won't interact with the mesh on the CPU afterwards
        // Check bevy docs for more information
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh_info.vertices)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_info.normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, mesh_info.uvs)
    .with_inserted_indices(Indices::U16(mesh_info.indices))
}

#[derive(Resource, Clone)]
pub struct HexMesh {
    pub mesh: Handle<Mesh>,
}

impl HexMesh {
    pub fn create(mut meshes: ResMut<Assets<Mesh>>, grid_config: Res<GridConfig>) -> Self {
        let mesh = meshes.add(create_hexagonal_mesh(
            grid_config.layout.clone(),
            grid_config.hex_radius,
        ));

        Self { mesh }
    }
}

#[derive(Resource)]
pub struct TerritorySettings {
    pub cells: HashMap<u64, HashSet<Hex>>,
}

impl Default for TerritorySettings {
    fn default() -> Self {
        Self {
            cells: HashMap::from([
                // (2, {
                //     [
                //         Hex::new(53, 24),
                //         Hex::new(53, 25),
                //         Hex::new(53, 26),
                //         Hex::new(53, 27),
                //         Hex::new(53, 28),
                //         Hex::new(53, 29),
                //         Hex::new(54, 22),
                //         Hex::new(54, 23),
                //         Hex::new(54, 24),
                //         Hex::new(54, 25),
                //         Hex::new(54, 26),
                //         Hex::new(54, 27),
                //         Hex::new(54, 28),
                //         Hex::new(55, 21),
                //         Hex::new(55, 22),
                //         Hex::new(55, 23),
                //         Hex::new(55, 24),
                //         Hex::new(55, 25),
                //         Hex::new(55, 26),
                //         Hex::new(55, 27),
                //         Hex::new(56, 20),
                //         Hex::new(56, 21),
                //         Hex::new(56, 22),
                //         Hex::new(56, 23),
                //         Hex::new(56, 24),
                //         Hex::new(56, 25),
                //         Hex::new(56, 26),
                //         Hex::new(56, 27),
                //         Hex::new(57, 19),
                //         Hex::new(57, 20),
                //         Hex::new(57, 21),
                //         Hex::new(57, 22),
                //         Hex::new(57, 23),
                //         Hex::new(57, 24),
                //         Hex::new(57, 25),
                //         Hex::new(57, 26),
                //         Hex::new(58, 19),
                //         Hex::new(58, 20),
                //         Hex::new(58, 21),
                //         Hex::new(58, 22),
                //         Hex::new(58, 23),
                //         Hex::new(58, 24),
                //         Hex::new(58, 25),
                //         Hex::new(58, 26),
                //         Hex::new(59, 18),
                //         Hex::new(59, 19),
                //         Hex::new(59, 20),
                //         Hex::new(59, 21),
                //         Hex::new(59, 22),
                //         Hex::new(59, 23),
                //         Hex::new(59, 24),
                //         Hex::new(59, 25),
                //         Hex::new(60, 18),
                //         Hex::new(60, 19),
                //         Hex::new(60, 20),
                //         Hex::new(60, 21),
                //         Hex::new(60, 22),
                //         Hex::new(60, 23),
                //         Hex::new(60, 24),
                //         Hex::new(60, 25),
                //         Hex::new(61, 17),
                //         Hex::new(61, 18),
                //         Hex::new(61, 19),
                //         Hex::new(61, 20),
                //         Hex::new(61, 21),
                //         Hex::new(61, 22),
                //         Hex::new(61, 23),
                //         Hex::new(61, 24),
                //         Hex::new(62, 17),
                //         Hex::new(62, 18),
                //         Hex::new(62, 19),
                //         Hex::new(62, 20),
                //         Hex::new(62, 21),
                //         Hex::new(62, 22),
                //         Hex::new(62, 23),
                //         Hex::new(62, 24),
                //         Hex::new(63, 17),
                //         Hex::new(63, 18),
                //         Hex::new(63, 19),
                //         Hex::new(63, 20),
                //         Hex::new(63, 21),
                //         Hex::new(63, 22),
                //         Hex::new(63, 23),
                //         Hex::new(64, 16),
                //         Hex::new(64, 17),
                //         Hex::new(64, 18),
                //         Hex::new(64, 19),
                //         Hex::new(64, 20),
                //         Hex::new(64, 21),
                //         Hex::new(64, 22),
                //         Hex::new(64, 23),
                //         Hex::new(65, 16),
                //         Hex::new(65, 17),
                //         Hex::new(65, 18),
                //         Hex::new(65, 19),
                //         Hex::new(65, 20),
                //         Hex::new(65, 21),
                //         Hex::new(65, 22),
                //         Hex::new(66, 16),
                //         Hex::new(66, 17),
                //         Hex::new(66, 18),
                //         Hex::new(66, 19),
                //         Hex::new(66, 20),
                //         Hex::new(67, 16),
                //         Hex::new(67, 17),
                //         Hex::new(67, 18),
                //     ]
                //     .into_iter()
                //     .collect()
                // }),
                // (8, {
                //     [
                //         Hex::new(65, 42),
                //         Hex::new(65, 43),
                //         Hex::new(66, 41),
                //         Hex::new(66, 42),
                //         Hex::new(66, 43),
                //         Hex::new(66, 44),
                //         Hex::new(67, 40),
                //         Hex::new(67, 41),
                //         Hex::new(67, 42),
                //         Hex::new(67, 43),
                //         Hex::new(67, 44),
                //         Hex::new(67, 45),
                //         Hex::new(67, 46),
                //         Hex::new(67, 47),
                //         Hex::new(67, 48),
                //         Hex::new(68, 39),
                //         Hex::new(68, 40),
                //         Hex::new(68, 41),
                //         Hex::new(68, 42),
                //         Hex::new(68, 43),
                //         Hex::new(68, 44),
                //         Hex::new(68, 45),
                //         Hex::new(68, 46),
                //         Hex::new(68, 47),
                //         Hex::new(68, 48),
                //         Hex::new(69, 39),
                //         Hex::new(69, 40),
                //         Hex::new(69, 41),
                //         Hex::new(69, 42),
                //         Hex::new(69, 43),
                //         Hex::new(69, 44),
                //         Hex::new(69, 45),
                //         Hex::new(69, 46),
                //         Hex::new(69, 47),
                //         Hex::new(70, 38),
                //         Hex::new(70, 39),
                //         Hex::new(70, 40),
                //         Hex::new(70, 41),
                //         Hex::new(70, 42),
                //         Hex::new(70, 43),
                //         Hex::new(70, 44),
                //         Hex::new(70, 45),
                //         Hex::new(70, 46),
                //         Hex::new(70, 47),
                //         Hex::new(71, 38),
                //         Hex::new(71, 39),
                //         Hex::new(71, 40),
                //         Hex::new(71, 41),
                //         Hex::new(71, 42),
                //         Hex::new(71, 43),
                //         Hex::new(71, 44),
                //         Hex::new(71, 45),
                //         Hex::new(71, 46),
                //         Hex::new(72, 37),
                //         Hex::new(72, 38),
                //         Hex::new(72, 39),
                //         Hex::new(72, 40),
                //         Hex::new(72, 41),
                //         Hex::new(72, 42),
                //         Hex::new(72, 43),
                //         Hex::new(72, 44),
                //         Hex::new(72, 45),
                //         Hex::new(72, 46),
                //         Hex::new(73, 37),
                //         Hex::new(73, 38),
                //         Hex::new(73, 39),
                //         Hex::new(73, 40),
                //         Hex::new(73, 41),
                //         Hex::new(73, 42),
                //         Hex::new(73, 43),
                //         Hex::new(73, 44),
                //         Hex::new(73, 45),
                //         Hex::new(73, 46),
                //         Hex::new(74, 36),
                //         Hex::new(74, 37),
                //         Hex::new(74, 38),
                //         Hex::new(74, 39),
                //         Hex::new(74, 40),
                //         Hex::new(74, 41),
                //         Hex::new(74, 42),
                //         Hex::new(74, 43),
                //         Hex::new(74, 44),
                //         Hex::new(74, 45),
                //         Hex::new(74, 46),
                //         Hex::new(75, 36),
                //         Hex::new(75, 37),
                //         Hex::new(75, 38),
                //         Hex::new(75, 39),
                //         Hex::new(75, 40),
                //         Hex::new(75, 41),
                //         Hex::new(75, 42),
                //         Hex::new(75, 43),
                //         Hex::new(75, 44),
                //         Hex::new(75, 45),
                //         Hex::new(76, 35),
                //         Hex::new(76, 36),
                //         Hex::new(76, 37),
                //         Hex::new(76, 38),
                //         Hex::new(76, 39),
                //         Hex::new(76, 40),
                //         Hex::new(76, 41),
                //         Hex::new(76, 42),
                //         Hex::new(76, 43),
                //         Hex::new(76, 44),
                //         Hex::new(76, 45),
                //         Hex::new(77, 35),
                //         Hex::new(77, 36),
                //         Hex::new(77, 37),
                //         Hex::new(77, 38),
                //         Hex::new(77, 39),
                //         Hex::new(77, 40),
                //         Hex::new(77, 41),
                //         Hex::new(77, 42),
                //         Hex::new(77, 43),
                //         Hex::new(77, 44),
                //         Hex::new(78, 34),
                //         Hex::new(78, 35),
                //         Hex::new(78, 36),
                //         Hex::new(78, 37),
                //         Hex::new(78, 38),
                //         Hex::new(78, 39),
                //         Hex::new(78, 40),
                //         Hex::new(78, 41),
                //         Hex::new(78, 42),
                //         Hex::new(78, 43),
                //         Hex::new(79, 34),
                //         Hex::new(79, 35),
                //         Hex::new(79, 36),
                //         Hex::new(79, 37),
                //         Hex::new(79, 38),
                //         Hex::new(79, 39),
                //         Hex::new(79, 40),
                //         Hex::new(79, 41),
                //         Hex::new(79, 42),
                //         Hex::new(80, 34),
                //         Hex::new(80, 35),
                //         Hex::new(80, 36),
                //         Hex::new(80, 37),
                //         Hex::new(80, 38),
                //         Hex::new(80, 39),
                //         Hex::new(80, 40),
                //         Hex::new(80, 41),
                //         Hex::new(80, 42),
                //         Hex::new(81, 34),
                //         Hex::new(81, 35),
                //         Hex::new(81, 36),
                //         Hex::new(81, 37),
                //         Hex::new(81, 38),
                //         Hex::new(81, 39),
                //     ]
                //     .into_iter()
                //     .collect()
                // }),
                (11, {
                    [
                        Hex::new(42, 41),
                        Hex::new(43, 40),
                        Hex::new(43, 41),
                        Hex::new(44, 39),
                        Hex::new(44, 40),
                        Hex::new(44, 41),
                        Hex::new(45, 38),
                        Hex::new(45, 39),
                        Hex::new(45, 40),
                        Hex::new(45, 41),
                        Hex::new(46, 37),
                        Hex::new(46, 38),
                        Hex::new(46, 39),
                        Hex::new(46, 40),
                        Hex::new(46, 41),
                        Hex::new(47, 36),
                        Hex::new(47, 37),
                        Hex::new(47, 38),
                        Hex::new(47, 39),
                        Hex::new(47, 40),
                        Hex::new(47, 41),
                        Hex::new(48, 35),
                        Hex::new(48, 36),
                        Hex::new(48, 37),
                        Hex::new(48, 38),
                        Hex::new(48, 39),
                        Hex::new(48, 40),
                        Hex::new(48, 41),
                        Hex::new(48, 42),
                        Hex::new(49, 34),
                        Hex::new(49, 35),
                        Hex::new(49, 36),
                        Hex::new(49, 37),
                        Hex::new(49, 38),
                        Hex::new(49, 39),
                        Hex::new(49, 40),
                        Hex::new(49, 41),
                        Hex::new(49, 42),
                        Hex::new(50, 33),
                        Hex::new(50, 34),
                        Hex::new(50, 35),
                        Hex::new(50, 36),
                        Hex::new(50, 37),
                        Hex::new(50, 38),
                        Hex::new(50, 39),
                        Hex::new(50, 40),
                        Hex::new(50, 41),
                        Hex::new(50, 42),
                        Hex::new(51, 32),
                        Hex::new(51, 33),
                        Hex::new(51, 34),
                        Hex::new(51, 35),
                        Hex::new(51, 36),
                        Hex::new(51, 37),
                        Hex::new(51, 38),
                        Hex::new(51, 39),
                        Hex::new(51, 40),
                        Hex::new(51, 41),
                        Hex::new(51, 42),
                        Hex::new(51, 43),
                        Hex::new(52, 31),
                        Hex::new(52, 32),
                        Hex::new(52, 33),
                        Hex::new(52, 34),
                        Hex::new(52, 35),
                        Hex::new(52, 36),
                        Hex::new(52, 37),
                        Hex::new(52, 38),
                        Hex::new(52, 39),
                        Hex::new(52, 40),
                        Hex::new(52, 41),
                        Hex::new(52, 42),
                        Hex::new(52, 43),
                        Hex::new(53, 32),
                        Hex::new(53, 33),
                        Hex::new(53, 34),
                        Hex::new(53, 35),
                        Hex::new(53, 36),
                        Hex::new(53, 37),
                        Hex::new(53, 38),
                        Hex::new(53, 39),
                        Hex::new(53, 40),
                        Hex::new(53, 41),
                        Hex::new(53, 42),
                        Hex::new(54, 33),
                        Hex::new(54, 34),
                        Hex::new(54, 35),
                        Hex::new(54, 36),
                        Hex::new(54, 37),
                        Hex::new(54, 38),
                        Hex::new(54, 39),
                        Hex::new(54, 40),
                        Hex::new(55, 34),
                        Hex::new(55, 35),
                        Hex::new(55, 36),
                        Hex::new(55, 37),
                        Hex::new(55, 38),
                        Hex::new(55, 39),
                        Hex::new(56, 34),
                        Hex::new(56, 35),
                        Hex::new(56, 36),
                        Hex::new(56, 37),
                        Hex::new(56, 38),
                        Hex::new(57, 35),
                        Hex::new(57, 36),
                    ]
                    .into_iter()
                    .collect()
                }),
                // (15, {
                //     [
                //         Hex::new(61, 5),
                //         Hex::new(61, 6),
                //         Hex::new(62, 3),
                //         Hex::new(62, 4),
                //         Hex::new(62, 5),
                //         Hex::new(62, 6),
                //         Hex::new(63, 1),
                //         Hex::new(63, 2),
                //         Hex::new(63, 3),
                //         Hex::new(63, 4),
                //         Hex::new(63, 5),
                //         Hex::new(63, 6),
                //         Hex::new(64, 0),
                //         Hex::new(64, 1),
                //         Hex::new(64, 2),
                //         Hex::new(64, 3),
                //         Hex::new(64, 4),
                //         Hex::new(64, 5),
                //         Hex::new(64, 6),
                //         Hex::new(65, -2),
                //         Hex::new(65, -1),
                //         Hex::new(65, 0),
                //         Hex::new(65, 1),
                //         Hex::new(65, 2),
                //         Hex::new(65, 3),
                //         Hex::new(65, 4),
                //         Hex::new(65, 5),
                //         Hex::new(65, 6),
                //         Hex::new(66, -2),
                //         Hex::new(66, -1),
                //         Hex::new(66, 0),
                //         Hex::new(66, 1),
                //         Hex::new(66, 2),
                //         Hex::new(66, 3),
                //         Hex::new(66, 4),
                //         Hex::new(66, 5),
                //         Hex::new(66, 6),
                //         Hex::new(67, -3),
                //         Hex::new(67, -2),
                //         Hex::new(67, -1),
                //         Hex::new(67, 0),
                //         Hex::new(67, 1),
                //         Hex::new(67, 2),
                //         Hex::new(67, 3),
                //         Hex::new(67, 4),
                //         Hex::new(67, 5),
                //         Hex::new(67, 6),
                //         Hex::new(68, -3),
                //         Hex::new(68, -2),
                //         Hex::new(68, -1),
                //         Hex::new(68, 0),
                //         Hex::new(68, 1),
                //         Hex::new(68, 2),
                //         Hex::new(68, 3),
                //         Hex::new(68, 4),
                //         Hex::new(68, 5),
                //         Hex::new(68, 6),
                //         Hex::new(69, -3),
                //         Hex::new(69, -2),
                //         Hex::new(69, -1),
                //         Hex::new(69, 0),
                //         Hex::new(69, 1),
                //         Hex::new(69, 2),
                //         Hex::new(69, 3),
                //         Hex::new(69, 4),
                //         Hex::new(69, 5),
                //         Hex::new(70, -4),
                //         Hex::new(70, -3),
                //         Hex::new(70, -2),
                //         Hex::new(70, -1),
                //         Hex::new(70, 0),
                //         Hex::new(70, 1),
                //         Hex::new(70, 2),
                //         Hex::new(70, 3),
                //         Hex::new(70, 4),
                //         Hex::new(71, -4),
                //         Hex::new(71, -3),
                //         Hex::new(71, -2),
                //         Hex::new(71, -1),
                //         Hex::new(71, 0),
                //         Hex::new(71, 1),
                //         Hex::new(71, 2),
                //         Hex::new(71, 3),
                //         Hex::new(72, -5),
                //         Hex::new(72, -4),
                //         Hex::new(72, -3),
                //         Hex::new(72, -2),
                //         Hex::new(72, -1),
                //         Hex::new(72, 0),
                //         Hex::new(72, 1),
                //         Hex::new(72, 2),
                //         Hex::new(72, 3),
                //         Hex::new(73, -4),
                //         Hex::new(73, -3),
                //         Hex::new(73, -2),
                //         Hex::new(73, -1),
                //         Hex::new(73, 0),
                //         Hex::new(73, 1),
                //     ]
                //     .into_iter()
                //     .collect()
                // }),
            ]),
        }
    }
}

// ============================================================
// Relations entre hexagones
// ============================================================

#[derive(Debug, Clone, PartialEq)]
pub enum HexRelation {
    /// Voisins directs (partagent une arête)
    Adjacent(usize),
    /// En diagonale (partagent un sommet)
    Diagonal {
        diag_index: usize,
        junction_a: Hex, // via dir[(diag_index+5)%6]
        junction_b: Hex, // via dir[diag_index]
    },
    /// Non connectés ou même hexagone
    Other,
}

pub fn hex_relation(a: Hex, b: Hex) -> HexRelation {
    let delta = b - a;

    // Voisin direct ?
    for (i, &neighbor_delta) in Hex::NEIGHBORS_COORDS.iter().enumerate() {
        if delta == neighbor_delta {
            return HexRelation::Adjacent(i);
        }
    }

    // Diagonale ?
    for (i, &diag_delta) in Hex::DIAGONAL_COORDS.iter().enumerate() {
        if delta == diag_delta {
            let dir_a = (i + 5) % 6;
            let dir_b = i;
            return HexRelation::Diagonal {
                diag_index: i,
                junction_a: a + Hex::NEIGHBORS_COORDS[dir_a],
                junction_b: a + Hex::NEIGHBORS_COORDS[dir_b],
            };
        }
    }

    HexRelation::Other
}

// ============================================================
// Calcul des points géométriques
// ============================================================

/// Milieu de l'arête dans la direction donnée
pub fn edge_midpoint(layout: &HexLayout, hex: Hex, dir_index: usize) -> Vec2 {
    let center = layout.hex_to_world_pos(hex);
    let neighbor = hex + Hex::NEIGHBORS_COORDS[dir_index];
    let neighbor_center = layout.hex_to_world_pos(neighbor);
    (center + neighbor_center) * 0.5
}

// ============================================================
// Construction du contour
// ============================================================

fn next_exterior_edge(hex: Hex, dir: usize, territory: &HashSet<Hex>) -> (Hex, usize) {
    // Le voisin dans la direction dir (extérieur par définition)
    // On tourne en sens horaire pour trouver la prochaine arête extérieure

    // D'abord, vérifier si le voisin "au coin" (sens horaire) est dans le territoire
    let next_dir_cw = (dir + 5) % 6; // sens horaire = index décroissant
    let corner_hex = hex + Hex::NEIGHBORS_COORDS[next_dir_cw];

    if territory.contains(&corner_hex) {
        // On passe sur cet hexagone et on cherche son arête extérieure
        // La direction "entrante" sur corner_hex est l'opposée de next_dir_cw
        let opposite = (next_dir_cw + 3) % 6;

        // Chercher la prochaine arête extérieure en tournant horaire depuis opposite
        for offset in 1..6 {
            let test_dir = (opposite + 6 - offset) % 6; // sens horaire
            if !territory.contains(&(corner_hex + Hex::NEIGHBORS_COORDS[test_dir])) {
                return (corner_hex, test_dir);
            }
        }
    }

    // Sinon, rester sur le même hex et prendre l'arête suivante (sens horaire)
    for offset in 1..6 {
        let test_dir = (dir + 6 - offset) % 6;
        if !territory.contains(&(hex + Hex::NEIGHBORS_COORDS[test_dir])) {
            return (hex, test_dir);
        }
    }

    // Ne devrait pas arriver
    (hex, dir)
}

/// Une arête de bordure identifiée par son hexagone et sa direction
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BorderEdge {
    pub hex: Hex,
    pub dir: usize,
}

/// Parcourt le contour et retourne les arêtes extérieures dans l'ordre
pub fn trace_border_edges(territory: &HashSet<Hex>) -> Vec<BorderEdge> {
    if territory.is_empty() {
        return Vec::new();
    }

    let start = *territory.iter().min_by_key(|h| (h.x(), h.y())).unwrap();

    let start_dir = (0..6)
        .find(|&d| !territory.contains(&(start + Hex::NEIGHBORS_COORDS[d])))
        .expect("L'hex de départ doit avoir une arête extérieure");

    let mut result = Vec::new();
    let mut current_hex = start;
    let mut current_dir = start_dir;
    let mut is_first = true;

    loop {
        // Vérifier si on a bouclé (retour au point de départ)
        if !is_first && current_hex == start && current_dir == start_dir {
            break;
        }
        is_first = false;

        // Ajouter cette arête au résultat
        result.push(BorderEdge {
            hex: current_hex,
            dir: current_dir,
        });

        // Trouver la prochaine arête extérieure
        (current_hex, current_dir) = next_exterior_edge(current_hex, current_dir, territory);

        // Sécurité anti-boucle infinie
        if result.len() > territory.len() * 6 {
            panic!("Boucle infinie dans trace_border_edges");
        }
    }

    result
}

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::materials::{
    self, CHUNK_HEIGHT, CHUNK_WIDTH, ChunkCoord, ContourSegment, TerritoryChunkMaterial,
    TerritoryMaterial, create_chunk_contour_material,
};

/// Applique un jitter déterministe aux points du contour.
/// Le même point produira toujours le même jitter, garantissant
/// que les frontières adjacentes coïncident.
pub fn apply_jitter(points: &mut Vec<Vec2>, amplitude: f32, seed: u64) {
    for point in points.iter_mut() {
        let jitter = deterministic_jitter(*point, amplitude, seed);
        *point += jitter;
    }
}

/// Calcule un jitter déterministe basé sur les coordonnées du point.
///
/// On utilise un hash des coordonnées quantifiées pour garantir que
/// deux points suffisamment proches (même après erreurs de flottants)
/// produisent le même jitter.
fn deterministic_jitter(point: Vec2, amplitude: f32, seed: u64) -> Vec2 {
    // Quantifier les coordonnées pour éviter les problèmes de précision
    // On arrondit à 0.1 unité près (ajuster selon la taille de tes hexagones)
    let quantization = 0.1;
    let qx = (point.x / quantization).round() as i64;
    let qy = (point.y / quantization).round() as i64;

    // Créer un hash déterministe à partir des coordonnées quantifiées
    let hash_x = compute_hash((qx, qy, seed, 0u8));
    let hash_y = compute_hash((qx, qy, seed, 1u8));

    // Convertir les hash en valeurs dans [-1, 1]
    let normalized_x = hash_to_normalized(hash_x);
    let normalized_y = hash_to_normalized(hash_y);

    // Appliquer l'amplitude
    Vec2::new(normalized_x * amplitude, normalized_y * amplitude)
}

/// Calcule un hash 64 bits à partir d'une valeur hashable
fn compute_hash<T: Hash>(value: T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

/// Convertit un hash u64 en une valeur normalisée dans [-1, 1]
fn hash_to_normalized(hash: u64) -> f32 {
    // Prendre les 32 bits de poids faible et normaliser
    let normalized = (hash & 0xFFFFFFFF) as f32 / (0xFFFFFFFFu32 as f32);
    // Transformer [0, 1] en [-1, 1]
    normalized * 2.0 - 1.0
}

/// Simplifie le contour en utilisant les diagonales.
pub fn simplify_contour_with_diagonals(
    layout: &HexLayout,
    edges: &[BorderEdge],
    territory: &HashSet<Hex>,
) -> Vec<Vec2> {
    if edges.len() < 3 {
        return edges
            .iter()
            .map(|e| edge_midpoint(layout, e.hex, e.dir))
            .collect();
    }

    let n = edges.len();

    // Étape 1 : Identifier les jonctions et les arêtes qui génèrent un sommet
    let mut is_junction = vec![false; n];
    let mut vertex_diag_index: Vec<Option<usize>> = vec![None; n];

    for i in 0..n {
        let prev2_idx = (i + n - 2) % n;
        let prev_idx = (i + n - 1) % n;
        let next_idx = (i + 1) % n;
        let next2_idx = (i + 2) % n;

        let hex_prev2 = edges[prev2_idx].hex;
        let hex_prev = edges[prev_idx].hex;
        let hex_curr = edges[i].hex;
        let hex_next = edges[next_idx].hex;
        let hex_next2 = edges[next2_idx].hex;

        // info!("Transition: ({},{}) -> ({},{}) | ({},{})", hex_prev.x, hex_prev.y, hex_next.x, hex_next.y, hex_curr.x, hex_curr.y);

        // Vérifier si c'est une transition diagonale
        // Concave case
        if let Some(diag_info) = detect_diagonal_transition(hex_prev, hex_curr, hex_next, territory)
        {
            // info!("Transition ({:?}): ({},{}) -> ({},{})", diag_info.curvature, hex_prev.x, hex_prev.y, hex_next.x, hex_next.y);
            // edges[i] est une jonction : ne génère pas de point
            is_junction[i] = true;
            is_junction[(i + 2) % n] = true;

            // edges[prev_idx] doit générer un sommet diagonal
            // On stocke l'index de la diagonale pour savoir quel sommet générer
            vertex_diag_index[prev_idx] = Some(diag_info.diag_index);
        }

        // Convex case
        if let Some(diag_info) = detect_diagonal_transition(hex_prev2, hex_curr, hex_next2, territory)
        {
            // info!("Transition ({:?}): ({},{}) -> ({},{})", diag_info.curvature, hex_prev2.x, hex_prev2.y, hex_next2.x, hex_next2.y);
            // edges[i] est une jonction : ne génère pas de point
            is_junction[i] = true;

            // vertex_diag_index[pre]
        }
    }

    // Étape 2 : Générer les points
    let mut points = Vec::new();

    for i in 0..n {
        // Sauter les jonctions
        if is_junction[i] {
            continue;
        }

        if let Some(diag_index) = vertex_diag_index[i] {
            // Cette arête génère un sommet diagonal
            points.push(edge_midpoint(layout, edges[i].hex, diag_index));
        } else {
            // Cette arête génère un milieu d'arête normal
            points.push(edge_midpoint(layout, edges[i].hex, edges[i].dir));
        }
    }

    points
}

#[derive(Debug)]
pub enum DiagonalTransitionCurvature {
    Convexe,
    Concave,
}
/// Information sur une transition diagonale
struct DiagonalTransitionInfo {
    diag_index: usize,
    curvature: DiagonalTransitionCurvature
}

/// Détecte si le triplet (hex_prev, hex_curr, hex_next) forme une transition diagonale.
/// hex_curr est potentiellement la jonction entre hex_prev et hex_next.
fn detect_diagonal_transition(
    hex_prev: Hex,
    hex_curr: Hex,
    hex_next: Hex,
    territory: &HashSet<Hex>,
) -> Option<DiagonalTransitionInfo> {
    // Les 3 hexagones doivent être distincts
    if hex_prev == hex_curr || hex_curr == hex_next || hex_prev == hex_next {
        return None;
    }

    // hex_prev et hex_next doivent être en relation diagonale
    let relation = hex_relation(hex_prev, hex_next);

    match relation {
        HexRelation::Diagonal {
            diag_index,
            junction_a,
            junction_b,
        } => {
            // hex_curr doit être exactement une des jonctions
            if hex_curr != junction_a && hex_curr != junction_b {
                return None;
            }

            // L'AUTRE jonction doit être dans le territoire
            // (c'est ce qui permet de "couper" par la diagonale)
            let other_junction = if hex_curr == junction_a {
                junction_b
            } else {
                junction_a
            };

            if territory.contains(&other_junction) {
                return Some(DiagonalTransitionInfo { diag_index, curvature: DiagonalTransitionCurvature::Convexe });
            }
            
            Some(DiagonalTransitionInfo { diag_index, curvature: DiagonalTransitionCurvature::Concave })
        }
        _ => None,
    }
}

pub fn build_contour(
    layout: &HexLayout,
    territory: &HashSet<Hex>,
    jitter_amplitude: f32, // ex: 2.0 pixels
    jitter_seed: u64,      // ex: 12345 - même seed pour tous les territoires
) -> Vec<Vec2> {
    let edges = trace_border_edges(territory);
    let mut points = simplify_contour_with_diagonals(layout, &edges, territory);

    // Appliquer le jitter avant la suppression des points colinéaires
    // pour que le jitter puisse créer de légères variations
    apply_jitter(&mut points, jitter_amplitude, jitter_seed);

    points
}

pub fn compute_contour(
    mut commands: Commands,
    grid_config: Res<GridConfig>,
    hex_mesh: Res<HexMesh>,
    territory_settings: Res<TerritorySettings>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut territory_materials: ResMut<Assets<TerritoryMaterial>>,
    mut territory_chunk_materials: ResMut<Assets<TerritoryChunkMaterial>>,
) {
    let hex_material = &materials.add(ColorMaterial::from_color(Color::srgba(0.2, 0.7, 0.4, 0.1)));

    for (id, cells) in &territory_settings.cells {
        // let ordered_border = trace_border_hexes(&territory_settings.cells);
        let contour_points = &build_contour(&grid_config.layout, cells, 4.0, 12345);

        let hex_mesh_handle = hex_mesh.mesh.clone(); // Taille unitaire

        commands.spawn((
            Mesh2d(hex_mesh_handle.clone()),
            MeshMaterial2d(hex_material.clone()),
            Transform::default(),
            TerritoryHex, // Caché au démarrage
        ));

        let chunk_contours = split_contour_into_chunks(contour_points);

        for (i, (chunk, segments)) in chunk_contours.iter().enumerate() {
            if let Some((mesh_handle, material_handle)) = create_chunk_contour_material(
                *chunk,
                segments,
                &mut meshes,
                &mut territory_chunk_materials,
                &mut buffers,
                Color::srgba(0.0, 0.0, 0.0, 0.7), // border
                Color::srgba(
                    0.8 * (i as f32) / (chunk_contours.len() as f32),
                    (*id as f32) / 15.,
                    0.3,
                    0.3,
                ), // fill
                2.0,                              // border_width
                30.0,                             // fade_distance
            ) {
                let (chunk_min, chunk_max) = chunk.bounds();
                let chunk_center = (chunk_min + chunk_max) * 0.5;

                commands
                    .spawn((
                        Mesh2d(mesh_handle),
                        MeshMaterial2d(material_handle),
                        Transform::from_translation(chunk_center.extend(0.0)),
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Text2d(format!("Area: {}", *id)),
                            TextColor(Color::srgba(
                                0.8 * (i as f32) / (chunk_contours.len() as f32),
                                (*id as f32) / 15.,
                                0.3,
                                0.3,
                            )), // Yellow
                        ));
                    });
            }
        }

        for hex in cells {
            let position = grid_config.layout.hex_to_world_pos(*hex).extend(0.5);

            commands.spawn((
                Mesh2d(hex_mesh_handle.clone()),
                MeshMaterial2d(hex_material.clone()),
                Transform::from_translation(position),
                TerritoryHex, // Caché au démarrage
            ));
            // .observe(recolor_on::<Pointer<Over>>(Color::srgba(0.2, 0.7, 0.4, 0.75)))
            // .observe(recolor_on::<Pointer<Out>>(Color::srgba(0.2, 0.7, 0.4, 0.5)));
        }

        // let (min, max) = materials::compute_bounds(&contour_points);
        // let center = (min + max) * 0.5;

        // let (mesh_handle, material_handle) = materials::create_territory_material(
        //     &contour_points,
        //     &mut meshes,
        //     &mut territory_materials,
        //     &mut buffers,
        //     Color::srgba(0.0, 0.0, 0.0, 0.7), // border
        //     Color::srgba(0.8, 0.2, 0.3, 0.3), // fill
        // );

        // commands.spawn((
        //     Mesh2d(mesh_handle),
        //     MeshMaterial2d(material_handle),
        //     Transform::from_translation(center.extend(0.0)),
        // ));
    }
}

pub fn draw_countour(
    mut gizmos: Gizmos,
    grid_config: Res<GridConfig>,
    territory_settings: Res<TerritorySettings>,
) {
    // let mut contour_points =
    //     build_contour(&grid_config.layout, &territory_settings.cells, 4.0, 12345);
    // contour_points.push(contour_points.first().copied().unwrap());

    // let contour_points = &build_contour(&grid_config.layout, &territory_settings.cells, 4.0, 12345);

    // let chunk_contours = split_contour_into_chunks(contour_points);

    // for (chunk, _segments) in chunk_contours {
    //     // info!("Drawing chunk {:?}", chunk);
    //     gizmos.rect_2d(
    //         Vec2::new(
    //             ((chunk.x as f32) + 0.5) * CHUNK_WIDTH,
    //             ((chunk.y as f32) + 0.5) * CHUNK_HEIGHT,
    //         ),
    //         Vec2::new(CHUNK_WIDTH, CHUNK_HEIGHT),
    //         Color::srgba(1.0, 1.0, 0.0, 1.0),
    //     );
    // }

    // for (i, point) in contour_points.iter().enumerate() {
    //     gizmos.circle_2d(
    //         *point,
    //         3.0,
    //         Color::srgba(
    //             ((i as f32 + 1.) / contour_points.len() as f32),
    //             1.0,
    //             1.0 - ((i as f32 + 1.) / contour_points.len() as f32),
    //             0.3,
    //         ),
    //     );
    // }

    // gizmos.linestrip(
    //     contour_points.iter().map(|pt| pt.extend(1.0)),
    //     Color::srgba(1.0, 0.0, 0.0, 0.3),
    // );
}

// fn recolor_on<E: EntityEvent>(
//     color: Color,
// ) -> impl Fn(On<E>, Query<(&Entity, With<TerritoryHex>)>) {
//     move |event, hex_query| {
//         if let Ok((_, mut material)) = material_query.get(event.event_target()) {

//         }
//     }
// }

/// Résultat du découpage : segments par chunk
pub type ChunkContours = HashMap<ChunkCoord, Vec<ContourSegment>>;

/// Découpe un contour en segments par chunk
pub fn split_contour_into_chunks(contour_points: &[Vec2]) -> ChunkContours {
    let mut result: ChunkContours = HashMap::new();

    if contour_points.len() < 2 {
        return result;
    }

    let n = contour_points.len();

    for i in 0..n {
        let start = contour_points[i];
        let end = contour_points[(i + 1) % n];

        // Découper ce segment selon les chunks qu'il traverse
        let segments = clip_segment_to_chunks(start, end);

        for (chunk, clipped_start, clipped_end) in segments {
            let segment = ContourSegment::from_contour_points(clipped_start, clipped_end);
            result.entry(chunk).or_default().push(segment);
        }
    }

    result
}

/// Découpe un segment aux frontières des chunks
/// Retourne une liste de (chunk, start, end) pour chaque portion du segment
fn clip_segment_to_chunks(start: Vec2, end: Vec2) -> Vec<(ChunkCoord, Vec2, Vec2)> {
    let mut result = Vec::new();

    // Collecter tous les points d'intersection avec les bordures de chunks
    let mut points = vec![(0.0f32, start)];

    let dir = end - start;
    let length = dir.length();

    if length < 0.0001 {
        // Segment dégénéré
        let chunk = ChunkCoord::from_world_pos(start);
        return vec![(chunk, start, end)];
    }

    // Intersections avec les lignes verticales (bordures X des chunks)
    let min_x = start.x.min(end.x);
    let max_x = start.x.max(end.x);
    let first_chunk_x = (min_x / CHUNK_WIDTH).floor() as i32;
    let last_chunk_x = (max_x / CHUNK_WIDTH).floor() as i32;

    for chunk_x in first_chunk_x..=last_chunk_x + 1 {
        let x = chunk_x as f32 * CHUNK_WIDTH;
        if x > min_x
            && x < max_x
            && let Some(t) = intersect_vertical(start, end, x)
        {
            let point = start + dir * t;
            points.push((t, point));
        }
    }

    // Intersections avec les lignes horizontales (bordures Y des chunks)
    let min_y = start.y.min(end.y);
    let max_y = start.y.max(end.y);
    let first_chunk_y = (min_y / CHUNK_HEIGHT).floor() as i32;
    let last_chunk_y = (max_y / CHUNK_HEIGHT).floor() as i32;

    for chunk_y in first_chunk_y..=last_chunk_y + 1 {
        let y = chunk_y as f32 * CHUNK_HEIGHT;
        if y > min_y
            && y < max_y
            && let Some(t) = intersect_horizontal(start, end, y)
        {
            let point = start + dir * t;
            points.push((t, point));
        }
    }

    // Ajouter le point final
    points.push((1.0, end));

    // Trier par t
    points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // Dédupliquer les points très proches
    points.dedup_by(|a, b| (a.0 - b.0).abs() < 0.0001);

    // Créer les segments
    for i in 0..points.len() - 1 {
        let seg_start = points[i].1;
        let seg_end = points[i + 1].1;

        // Déterminer le chunk de ce segment (utiliser le milieu)
        let midpoint = (seg_start + seg_end) * 0.5;
        let chunk = ChunkCoord::from_world_pos(midpoint);

        result.push((chunk, seg_start, seg_end));
    }

    result
}

/// Intersection avec une ligne verticale x = x_line
fn intersect_vertical(start: Vec2, end: Vec2, x_line: f32) -> Option<f32> {
    let dx = end.x - start.x;
    if dx.abs() < 0.0001 {
        return None; // Segment vertical, pas d'intersection unique
    }
    let t = (x_line - start.x) / dx;
    if t > 0.0 && t < 1.0 { Some(t) } else { None }
}

/// Intersection avec une ligne horizontale y = y_line
fn intersect_horizontal(start: Vec2, end: Vec2, y_line: f32) -> Option<f32> {
    let dy = end.y - start.y;
    if dy.abs() < 0.0001 {
        return None; // Segment horizontal, pas d'intersection unique
    }
    let t = (y_line - start.y) / dy;
    if t > 0.0 && t < 1.0 { Some(t) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Vérifie si deux contours cycliques sont équivalents
    /// (même séquence mais possiblement décalée et/ou inversée)
    fn contours_equivalent(a: &[Hex], b: &[Hex]) -> bool {
        if a.len() != b.len() {
            return false;
        }

        if a.is_empty() {
            return true;
        }

        // Chercher le point de départ de 'a' dans 'b'
        let Some(start_idx) = b.iter().position(|h| *h == a[0]) else {
            return false;
        };

        // Vérifier dans le sens direct
        let matches_forward = a
            .iter()
            .enumerate()
            .all(|(i, hex)| b[(start_idx + i) % b.len()] == *hex);

        if matches_forward {
            return true;
        }

        // Vérifier dans le sens inverse (le parcours peut être dans l'autre sens)
        let matches_backward = a.iter().enumerate().all(|(i, hex)| {
            // +b.len() pour éviter les négatifs
            b[(start_idx + b.len() - i) % b.len()] == *hex
        });

        matches_backward
    }

    #[test]
    fn test_trace_border_matches_manual_contour() {
        let territory: HashSet<Hex> = [
            Hex::new(42, 41),
            Hex::new(43, 40),
            Hex::new(43, 41),
            Hex::new(44, 39),
            Hex::new(44, 40),
            Hex::new(44, 41),
            Hex::new(45, 38),
            Hex::new(45, 39),
            Hex::new(45, 40),
            Hex::new(45, 41),
            Hex::new(46, 37),
            Hex::new(46, 38),
            Hex::new(46, 39),
            Hex::new(46, 40),
            Hex::new(46, 41),
            Hex::new(47, 36),
            Hex::new(47, 37),
            Hex::new(47, 38),
            Hex::new(47, 39),
            Hex::new(47, 40),
            Hex::new(47, 41),
            Hex::new(48, 35),
            Hex::new(48, 36),
            Hex::new(48, 37),
            Hex::new(48, 38),
            Hex::new(48, 39),
            Hex::new(48, 40),
            Hex::new(48, 41),
            Hex::new(48, 42),
            Hex::new(49, 34),
            Hex::new(49, 35),
            Hex::new(49, 36),
            Hex::new(49, 37),
            Hex::new(49, 38),
            Hex::new(49, 39),
            Hex::new(49, 40),
            Hex::new(49, 41),
            Hex::new(49, 42),
            Hex::new(50, 33),
            Hex::new(50, 34),
            Hex::new(50, 35),
            Hex::new(50, 36),
            Hex::new(50, 37),
            Hex::new(50, 38),
            Hex::new(50, 39),
            Hex::new(50, 40),
            Hex::new(50, 41),
            Hex::new(50, 42),
            Hex::new(51, 32),
            Hex::new(51, 33),
            Hex::new(51, 34),
            Hex::new(51, 35),
            Hex::new(51, 36),
            Hex::new(51, 37),
            Hex::new(51, 38),
            Hex::new(51, 39),
            Hex::new(51, 40),
            Hex::new(51, 41),
            Hex::new(51, 42),
            Hex::new(51, 43),
            Hex::new(52, 31),
            Hex::new(52, 32),
            Hex::new(52, 33),
            Hex::new(52, 34),
            Hex::new(52, 35),
            Hex::new(52, 36),
            Hex::new(52, 37),
            Hex::new(52, 38),
            Hex::new(52, 39),
            Hex::new(52, 40),
            Hex::new(52, 41),
            Hex::new(52, 42),
            Hex::new(52, 43),
            Hex::new(53, 32),
            Hex::new(53, 33),
            Hex::new(53, 34),
            Hex::new(53, 35),
            Hex::new(53, 36),
            Hex::new(53, 37),
            Hex::new(53, 38),
            Hex::new(53, 39),
            Hex::new(53, 40),
            Hex::new(53, 41),
            Hex::new(53, 42),
            Hex::new(54, 33),
            Hex::new(54, 34),
            Hex::new(54, 35),
            Hex::new(54, 36),
            Hex::new(54, 37),
            Hex::new(54, 38),
            Hex::new(54, 39),
            Hex::new(54, 40),
            Hex::new(55, 34),
            Hex::new(55, 35),
            Hex::new(55, 36),
            Hex::new(55, 37),
            Hex::new(55, 38),
            Hex::new(55, 39),
            Hex::new(56, 34),
            Hex::new(56, 35),
            Hex::new(56, 36),
            Hex::new(56, 37),
            Hex::new(56, 38),
            Hex::new(57, 35),
            Hex::new(57, 36),
        ]
        .into_iter()
        .collect();

        let expected_contour = [
            Hex::new(42, 41),
            Hex::new(43, 41),
            Hex::new(44, 41),
            Hex::new(45, 41),
            Hex::new(46, 41),
            Hex::new(47, 41),
            Hex::new(48, 41),
            Hex::new(48, 42),
            Hex::new(49, 42),
            Hex::new(50, 42),
            Hex::new(51, 42),
            Hex::new(51, 43),
            Hex::new(52, 43),
            Hex::new(53, 42),
            Hex::new(53, 41),
            Hex::new(54, 40),
            Hex::new(55, 39),
            Hex::new(56, 38),
            Hex::new(56, 37),
            Hex::new(57, 36),
            Hex::new(57, 35),
            Hex::new(56, 35),
            Hex::new(56, 34),
            Hex::new(55, 34),
            Hex::new(54, 34),
            Hex::new(54, 33),
            Hex::new(53, 33),
            Hex::new(53, 32),
            Hex::new(52, 32),
            Hex::new(52, 31),
            Hex::new(51, 32),
            Hex::new(50, 33),
            Hex::new(49, 34),
            Hex::new(48, 35),
            Hex::new(47, 36),
            Hex::new(46, 37),
            Hex::new(45, 38),
            Hex::new(44, 39),
            Hex::new(43, 40),
        ];

        let traced = trace_border_hexes(&territory);

        println!("Expected length: {}", expected_contour.len());
        println!("Traced length: {}", traced.len());

        // Trouver l'élément divergent
        println!("\n=== Comparaison élément par élément ===");
        let max_len = expected_contour.len().max(traced.len());
        for i in 0..max_len {
            let exp = expected_contour.get(i);
            let trc = traced.get(i);

            let marker = if exp != trc { " <-- DIFF" } else { "" };
            println!("{:2}: expected {:?} | traced {:?}{}", i, exp, trc, marker);
        }

        // Vérifier les doublons dans traced
        let mut seen = HashSet::new();
        for (i, hex) in traced.iter().enumerate() {
            if !seen.insert(*hex) {
                println!("\n!!! Doublon trouvé: {:?} à l'index {}", hex, i);
            }
        }

        assert!(
            contours_equivalent(&expected_contour, &traced),
            "Contours do not match!\nExpected: {:?}\nTraced: {:?}",
            expected_contour,
            traced
        );

        println!("✓ Contours match!");
    }
}
