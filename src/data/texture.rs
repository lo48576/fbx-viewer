//! Texture.

use std::fmt;

use vulkano::sampler::SamplerAddressMode;

/// Texture ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextureId(pub i64);

/// Texture.
#[derive(Clone)]
pub struct Texture {
    /// Name.
    pub name: Option<String>,
    /// Data.
    pub data: image::DynamicImage,
    /// Whether the texture has transparent data or not.
    pub transparent: bool,
    /// Wrap mode U.
    pub wrap_mode_u: SamplerAddressMode,
    /// Wrap mode V.
    pub wrap_mode_v: SamplerAddressMode,
}

impl fmt::Debug for Texture {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Texture")
            .field("name", &self.name)
            .field("transparent", &self.transparent)
            .finish()
    }
}
