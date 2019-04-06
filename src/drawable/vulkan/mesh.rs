//! Mesh.

use std::sync::Arc;

use vulkano::buffer::CpuAccessibleBuffer;

use crate::{data::mesh::Vertex, drawable::vulkan::Texture};

/// Drawable mesh.
#[derive(Debug, Clone)]
pub struct Mesh {
    /// Name.
    pub(crate) name: Option<String>,
    /// Vertex buffer.
    pub(crate) vertex: Arc<CpuAccessibleBuffer<[Vertex]>>,
    /// Submeshes.
    pub(crate) submeshes: Vec<SubMesh>,
}

impl Mesh {
    /// Returns the mesh name if available.
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(AsRef::as_ref)
    }

    /// Returns the vertex buffer.
    pub fn vertex(&self) -> &Arc<CpuAccessibleBuffer<[Vertex]>> {
        &self.vertex
    }

    /// Returns the submeshes.
    pub fn submeshes(&self) -> &[SubMesh] {
        &self.submeshes
    }
}

/// Drawable submesh.
#[derive(Debug, Clone)]
pub struct SubMesh {
    /// Material index.
    pub(crate) material_index: u32,
    /// Index buffer.
    pub(crate) indices: Arc<CpuAccessibleBuffer<[u32]>>,
    /// texture.
    pub(crate) texture: Option<Arc<Texture>>,
}

impl SubMesh {
    /// Returns the material index.
    pub fn material_index(&self) -> u32 {
        self.material_index
    }

    /// Returns the index buffer.
    pub fn index(&self) -> &Arc<CpuAccessibleBuffer<[u32]>> {
        &self.indices
    }

    /// Returns the texture.
    pub fn texture(&self) -> Option<&Arc<Texture>> {
        self.texture.as_ref()
    }
}
