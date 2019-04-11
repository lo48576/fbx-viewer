//! Geometry.

use std::{fmt, sync::Arc};

use fbx_viewer::util::bbox::OptionalBoundingBox3d;
use vulkano::buffer::ImmutableBuffer;

use crate::vulkan::drawable::Vertex;

/// Geometry mesh.
#[derive(Clone)]
pub struct GeometryMesh {
    /// Name.
    pub(crate) name: Option<String>,
    /// Vertices.
    pub(crate) vertices: Arc<ImmutableBuffer<[Vertex]>>,
    /// Indices per materials.
    pub(crate) indices_per_material: Vec<Arc<ImmutableBuffer<[u32]>>>,
    /// Bounding box.
    pub(crate) bounding_box: OptionalBoundingBox3d<f32>,
}

impl fmt::Debug for GeometryMesh {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("GeometryMesh")
            .field("name", &self.name)
            .field("indices_per_material_len", &self.indices_per_material.len())
            .field("bounding_box", &self.bounding_box)
            .finish()
    }
}
