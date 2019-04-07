//! Material.

use crate::data::TextureIndex;

/// Material.
#[derive(Debug, Clone)]
pub struct Material {
    /// Texture index.
    pub(crate) diffuse_texture: Option<TextureIndex>,
    /// Shading parameters.
    pub(crate) data: ShadingData,
}

/// Shading data.
#[derive(Debug, Clone, Copy)]
pub enum ShadingData {
    /// Lambert material.
    Lambert(LambertData),
}

/// Lambert data.
#[derive(Debug, Clone, Copy)]
pub struct LambertData {
    /// Ambient.
    pub(crate) ambient: [f32; 3],
    /// Diffuse.
    pub(crate) diffuse: [f32; 3],
    /// Emissive.
    pub(crate) emissive: [f32; 3],
}
