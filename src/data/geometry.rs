//! Geometry.

/// Geometry mesh.
#[derive(Debug, Clone)]
pub struct GeometryMesh {
    /// Positions.
    pub(crate) positions: Vec<[f32; 3]>,
    /// Normals.
    pub(crate) normals: Vec<[f32; 3]>,
    /// UV.
    pub(crate) uv: Vec<[f32; 2]>,
    /// Indices per materials.
    pub(crate) indices_per_material: Vec<Vec<u32>>,
}
