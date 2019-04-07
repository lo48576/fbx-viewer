//! 3D content data.

pub use self::{
    geometry::GeometryMesh,
    material::{LambertData, Material, ShadingData},
    mesh::Mesh,
    scene::{GeometryMeshIndex, MaterialIndex, MeshIndex, Scene, TextureIndex},
    texture::{Texture, WrapMode},
};

mod geometry;
mod material;
mod mesh;
mod scene;
mod texture;
