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
    vertex: Arc<CpuAccessibleBuffer<[Vertex]>>,
    /// Index buffers.
    indices: Vec<Arc<CpuAccessibleBuffer<[u32]>>>,
}

impl Mesh {
    /// Creates a new `Mesh` from the given mesh.
    pub fn from_mesh(device: &Arc<Device>, mesh: &crate::data::Mesh) -> Fallible<Self> {
        let vertex = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            mesh.vertices.iter().cloned(),
        )?;
        let indices = mesh
            .indices
            .iter()
            .map(|indices| {
                CpuAccessibleBuffer::from_iter(
                    device.clone(),
                    BufferUsage::all(),
                    indices.iter().cloned(),
                )
                .map_err(Into::into)
            })
            .collect::<Fallible<_>>()?;

        Ok(Self {
            name: mesh.name.clone(),
            vertex,
            indices,
        })
    }

    /// Returns the mesh name if available.
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(AsRef::as_ref)
    }

    /// Returns the vertex buffer.
    pub fn vertex(&self) -> &Arc<CpuAccessibleBuffer<[Vertex]>> {
        &self.vertex
    }

    /// Returns the index buffer.
    pub fn indices(&self) -> &[Arc<CpuAccessibleBuffer<[u32]>>] {
        &self.indices
    }
}
