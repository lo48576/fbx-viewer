//! Mesh.

/// Mesh.
#[derive(Debug, Clone)]
pub struct Mesh {
    /// Name.
    pub name: Option<String>,
    /// Vertices.
    pub position: Vec<Vertex>,
    /// Indices.
    pub indices: Vec<u32>,
}

/// Vertex.
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    /// Position.
    pub position: [f32; 3],
}

vulkano::impl_vertex!(Vertex, position);
