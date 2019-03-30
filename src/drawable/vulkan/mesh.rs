//! Mesh.

use std::sync::Arc;

use failure::Fallible;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    device::Device,
};

use crate::data::mesh::Vertex;

/// Drawable mesh.
#[derive(Debug, Clone)]
pub struct Mesh {
    /// Name.
    name: Option<String>,
    /// Vertex buffer.
    position: Arc<CpuAccessibleBuffer<[Vertex]>>,
    /// Index.
    index: Arc<CpuAccessibleBuffer<[u32]>>,
}

impl Mesh {
    /// Creates a new `Mesh` from the given mesh.
    pub fn from_mesh(device: &Arc<Device>, mesh: &crate::data::Mesh) -> Fallible<Self> {
        let position = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            mesh.position.iter().cloned(),
        )?;
        let index = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            mesh.indices.iter().cloned(),
        )?;

        Ok(Self {
            name: mesh.name.clone(),
            position,
            index,
        })
    }

    /// Returns the mesh name if available.
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(AsRef::as_ref)
    }

    /// Returns the vertex buffer.
    pub fn position(&self) -> &Arc<CpuAccessibleBuffer<[Vertex]>> {
        &self.position
    }

    /// Returns the index buffer.
    pub fn index(&self) -> &Arc<CpuAccessibleBuffer<[u32]>> {
        &self.index
    }
}
