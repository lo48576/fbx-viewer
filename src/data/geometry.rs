//! Geometry.

/// Geometry mesh.
#[derive(Debug, Clone)]
pub struct GeometryMesh {
    /// Name.
    pub name: Option<String>,
    /// Positions.
    pub positions: Vec<[f32; 3]>,
    /// Normals.
    pub normals: Vec<[f32; 3]>,
    /// UV.
    pub uv: Vec<[f32; 2]>,
    /// Indices per materials.
    pub indices_per_material: Vec<Vec<u32>>,
}
