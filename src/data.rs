//! 3D content data.

pub use self::{
    mesh::{Mesh, SubMesh},
    model::Model,
    scene::Scene,
};

pub mod mesh;
mod model;
mod scene;
