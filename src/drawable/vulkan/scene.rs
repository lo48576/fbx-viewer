//! Scene.

use std::sync::Arc;

use failure::Fallible;
use vulkano::device::Device;

use crate::drawable::vulkan::Model;

/// Scene.
#[derive(Debug, Clone)]
pub struct Scene {
    /// Name.
    name: Option<String>,
    /// Models.
    models: Vec<Model>,
}

impl Scene {
    /// Creates a new `Scene` from the given scene.
    pub fn from_scene(device: &Arc<Device>, scene: &crate::data::Scene) -> Fallible<Self> {
        let models = scene
            .models
            .iter()
            .map(|model| Model::from_model(device, model))
            .collect::<Fallible<_>>()?;

        Ok(Self { name: None, models })
    }

    /// Returns the model name if available.
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(AsRef::as_ref)
    }

    /// Returns an iterator of meshes.
    pub fn iter_models(&self) -> impl Iterator<Item = &Model> {
        self.models.iter()
    }
}
