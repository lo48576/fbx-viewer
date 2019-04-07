//! 3D content data.

pub use self::{
    geometry::GeometryMesh,
    mesh::Mesh,
    scene::{GeometryMeshIndex, MeshIndex, Scene},
};

mod geometry;
mod mesh;
mod scene;
