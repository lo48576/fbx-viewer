//! Scene.

use crate::drawable::vulkan::Model;

/// Scene.
#[derive(Debug, Clone)]
pub struct Scene {
    /// Name.
    pub(crate) name: Option<String>,
    /// Models.
    pub(crate) models: Vec<Model>,
}

impl Scene {
    /// Returns the model name if available.
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(AsRef::as_ref)
    }

    /// Returns an iterator of meshes.
    pub fn iter_models(&self) -> impl Iterator<Item = &Model> {
        self.models.iter()
    }
}
