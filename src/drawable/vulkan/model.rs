//! Model.

use crate::drawable::vulkan::Mesh;

/// Model.
#[derive(Debug, Clone)]
pub struct Model {
    /// Name.
    pub(crate) name: Option<String>,
    /// Meshes.
    pub(crate) meshes: Vec<Mesh>,
}

impl Model {
    /// Returns the model name if available.
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(AsRef::as_ref)
    }

    /// Returns an iterator of meshes.
    pub fn iter_meshes(&self) -> impl Iterator<Item = &Mesh> {
        self.meshes.iter()
    }
}
