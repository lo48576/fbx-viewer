//! Vertex.
// Allow `clippy::needless_borrow` for `vulkano::impl_vertex` macro.
#![allow(clippy::needless_borrow)]

/// Vertex.
#[derive(Default, Debug, Clone, Copy)]
pub struct Vertex {
    /// Position.
    pub position: [f32; 3],
    /// Normal.
    pub normal: [f32; 3],
    /// UV.
    pub uv: [f32; 2],
}

vulkano::impl_vertex!(Vertex, position, normal, uv);
