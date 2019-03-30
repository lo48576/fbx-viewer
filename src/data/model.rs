//! Model.

use crate::data::mesh::Mesh;

/// Model.
#[derive(Debug, Clone)]
pub struct Model {
    /// Name.
    pub name: Option<String>,
    /// Meshes.
    pub meshes: Vec<Mesh>,
}
