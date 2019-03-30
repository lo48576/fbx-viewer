//! Scene.

use crate::data::model::Model;

/// Scene.
#[derive(Default, Debug, Clone)]
pub struct Scene {
    /// Name.
    pub name: Option<String>,
    /// Models.
    pub models: Vec<Model>,
}

impl Scene {
    /// Creates a new `Scene`.
    pub fn new() -> Self {
        Self::default()
    }
}
