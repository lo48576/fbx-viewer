//! Material.

use crate::data::TextureIndex;

/// Material.
#[derive(Debug, Clone)]
pub struct Material {
    /// Texture index.
    pub(crate) diffuse_texture: Option<TextureIndex>,
}
