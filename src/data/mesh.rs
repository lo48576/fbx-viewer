//! Mesh.

use crate::data::{GeometryMeshIndex, MaterialIndex};

/// Mesh.
#[derive(Debug, Clone)]
pub struct Mesh {
    /// Geometry mesh index.
    pub(crate) geometry_mesh_index: GeometryMeshIndex,
    /// Materials.
    pub(crate) materials: Vec<MaterialIndex>,
}

impl Mesh {
    /// Returns geometry mesh index.
    pub fn geometry_mesh_index(&self) -> GeometryMeshIndex {
        self.geometry_mesh_index
    }
}
