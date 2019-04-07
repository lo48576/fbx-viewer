//! 3D content data.

pub use self::{
    geometry::GeometryMesh,
    material::Material,
    mesh::Mesh,
    scene::{GeometryMeshIndex, MaterialIndex, MeshIndex, Scene},
};

mod geometry;
mod material;
mod mesh;
mod scene;
