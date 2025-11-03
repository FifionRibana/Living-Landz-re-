use bincode::{Decode, Encode};

#[derive(Default, Debug, Clone, Encode, Decode)]
pub struct MeshData {
    pub triangles: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
}
