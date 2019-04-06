//! Mesh.

use crate::data::texture::TextureId;

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
    /// Texture ID.
    pub texture_id: Option<TextureId>,
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
    /// UV.
    pub uv: [f32; 2],
    /// Material.
    pub material: u32,
}

vulkano::impl_vertex!(Vertex, position, normal, uv, material);
