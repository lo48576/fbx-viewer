//! 3D content data.

pub use self::{
    mesh::{Mesh, SubMesh},
    model::Model,
    scene::Scene,
    texture::{Texture, TextureId},
};

pub mod mesh;
mod model;
mod scene;
mod texture;
