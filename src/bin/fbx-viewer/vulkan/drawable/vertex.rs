//! Vertex.
// Allow `clippy::ref_in_deref` for `vulkano::impl_vertex` macro.
#![allow(clippy::ref_in_deref)]

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
