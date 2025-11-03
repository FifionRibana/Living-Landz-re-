use i_triangle::float::{triangulation::Triangulation};
use serde::{Deserialize, Serialize};
use rayon::prelude::*;
use bincode::{Encode, Decode};

#[derive(Default, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct MeshData {
    pub triangles: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
}

impl MeshData {
    #[inline]
    pub fn from_meshes(meshes: Triangulation<[f64; 2], u32>) -> Self {
        // let mut triangles = Vec::new();

        let triangles: Vec<u32> = (0..meshes.indices.len())
            .step_by(3)
            .par_bridge()
            .flat_map_iter(|i| {
                [
                    meshes.indices[i],
                    meshes.indices[i + 1],
                    meshes.indices[i + 2],
                ]
            })
            .collect::<Vec<_>>();

        // for i in (0..meshes.indices.len()).step_by(3) {
        //     triangles.push(meshes.indices[i]);
        //     triangles.push(meshes.indices[i + 1]);
        //     triangles.push(meshes.indices[i + 2]);
        // }

        Self {
            triangles: triangles
                .par_iter()
                .map(|&i| {
                    [
                        (meshes.points[i as usize][0]) as f32,
                        (meshes.points[i as usize][1]) as f32,
                        0.0,
                    ]
                })
                .collect::<Vec<_>>(),
            normals: triangles
                .par_iter()
                .map(|_| [0.0, 0.0, 1.0])
                .collect::<Vec<_>>(),
            uvs: triangles
                .par_iter()
                .map(|_| [0.5, 0.5])
                .collect::<Vec<_>>(),
        }
    }
}
