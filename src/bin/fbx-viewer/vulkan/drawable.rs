//! Drawable types.

use vulkano::sync::GpuFuture;

pub use self::{
    geometry::GeometryMesh, loader::Loader, material::Material, mesh::Mesh, scene::Scene,
    texture::Texture, vertex::Vertex,
};

pub mod geometry;
mod loader;
pub mod material;
pub mod mesh;
pub mod scene;
pub mod texture;
pub mod vertex;

/// Joins the given futures.
fn join_futures(prev: &mut Option<Box<dyn GpuFuture>>, f: impl GpuFuture + 'static) {
    let new = match prev.take() {
        Some(prev) => Box::new(prev.join(f)) as Box<_>,
        None => Box::new(f) as Box<_>,
    };
    *prev = Some(new);
}
