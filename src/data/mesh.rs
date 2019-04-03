//! Mesh.

/// Mesh.
#[derive(Debug, Clone)]
pub struct Mesh {
    /// Name.
    pub name: Option<String>,
    /// Vertices.
    pub vertices: Vec<Vertex>,
    /// Submeshes.
    pub submeshes: Vec<SubMesh>,
}

/// Sub mesh.
#[derive(Debug, Clone)]
pub struct SubMesh {
    /// Material index.
    pub material_index: u32,
    /// Indices.
    pub indices: Vec<u32>,
}

/// Vertex.
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    /// Position.
    pub position: [f32; 3],
    /// Normal.
    pub normal: [f32; 3],
    /// Material.
    pub material: u32,
}

vulkano::impl_vertex!(Vertex, position, normal, material);
