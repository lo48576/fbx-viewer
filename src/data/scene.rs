//! Scene.

use std::collections::HashMap;

use crate::data::{
    model::Model,
    texture::{Texture, TextureId},
};

/// Scene.
#[derive(Default, Debug, Clone)]
pub struct Scene {
    /// Name.
    pub name: Option<String>,
    /// Textures.
    pub textures: HashMap<TextureId, Texture>,
    /// Models.
    pub models: Vec<Model>,
}

impl Scene {
    /// Creates a new `Scene`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns texture.
    pub fn texture(&self, id: TextureId) -> Option<&Texture> {
        self.textures.get(&id)
    }
}
