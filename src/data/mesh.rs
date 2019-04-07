//! Mesh.

use crate::data::GeometryMeshIndex;

/// Mesh.
#[derive(Debug, Clone)]
pub struct Mesh {
    /// Geometry mesh index.
    pub(crate) geometry_mesh_index: GeometryMeshIndex,
}

impl Mesh {
    /// Returns geometry mesh index.
    pub fn geometry_mesh_index(&self) -> GeometryMeshIndex {
        self.geometry_mesh_index
    }
}
