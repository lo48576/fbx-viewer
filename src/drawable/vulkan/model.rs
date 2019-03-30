//! Model.

use std::sync::Arc;

use failure::Fallible;
use vulkano::device::Device;

use crate::drawable::vulkan::Mesh;

/// Model.
#[derive(Debug, Clone)]
pub struct Model {
    /// Name.
    name: Option<String>,
    /// Meshes.
    meshes: Vec<Mesh>,
}

impl Model {
    /// Creates a new `Model` from the given model.
    pub fn from_model(device: &Arc<Device>, model: &crate::data::Model) -> Fallible<Self> {
        let meshes = model
            .meshes
            .iter()
            .map(|mesh| Mesh::from_mesh(device, mesh))
            .collect::<Fallible<_>>()?;

        Ok(Self {
            name: model.name.clone(),
            meshes,
        })
    }

    /// Returns the model name if available.
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(AsRef::as_ref)
    }

    /// Returns an iterator of meshes.
    pub fn iter_meshes(&self) -> impl Iterator<Item = &Mesh> {
        self.meshes.iter()
    }
}
