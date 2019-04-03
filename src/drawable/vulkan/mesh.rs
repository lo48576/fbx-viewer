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
    /// Submeshes.
    submeshes: Vec<SubMesh>,
}

impl Mesh {
    /// Creates a new `Mesh` from the given mesh.
    pub fn from_mesh(device: &Arc<Device>, mesh: &crate::data::Mesh) -> Fallible<Self> {
        let vertex = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            mesh.vertices.iter().cloned(),
        )?;
        let submeshes = mesh
            .submeshes
            .iter()
            .map(|submesh| SubMesh::from_submesh(device, submesh))
            .collect::<Fallible<_>>()?;

        Ok(Self {
            name: mesh.name.clone(),
            vertex,
            submeshes,
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

    /// Returns the submeshes.
    pub fn submeshes(&self) -> &[SubMesh] {
        &self.submeshes
    }
}

/// Drawable submesh.
#[derive(Debug, Clone)]
pub struct SubMesh {
    /// Material index.
    material_index: u32,
    /// Index buffer.
    indices: Arc<CpuAccessibleBuffer<[u32]>>,
}

impl SubMesh {
    /// Creates a new `SubMesh` from the given submesh.
    pub fn from_submesh(device: &Arc<Device>, submesh: &crate::data::SubMesh) -> Fallible<Self> {
        let indices = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            submesh.indices.iter().cloned(),
        )?;

        Ok(SubMesh {
            material_index: submesh.material_index,
            indices,
        })
    }

    /// Returns the material index.
    pub fn material_index(&self) -> u32 {
        self.material_index
    }

    /// Returns the index buffer.
    pub fn index(&self) -> &Arc<CpuAccessibleBuffer<[u32]>> {
        &self.indices
    }
}
