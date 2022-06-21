//! Texture.

use std::{fmt, sync::Arc};

use vulkano::{
    descriptor::descriptor_set::DescriptorSet,
    format::R8G8B8A8Srgb,
    image::{view::ImageView, ImmutableImage},
    sampler::Sampler,
};

/// Texture.
#[derive(Clone)]
pub struct Texture {
    /// Name.
    pub(crate) name: Option<String>,
    /// Image.
    pub(crate) image: Arc<ImageView<Arc<ImmutableImage<R8G8B8A8Srgb>>>>,
    /// Sampler.
    pub(crate) sampler: Arc<Sampler>,
    /// Whether the texture can be transparent.
    ///
    /// If `false`, the texture can be assumed to have no transparent texels.
    pub(crate) transparent: bool,
    /// Cache.
    pub(crate) cache: TextureCache,
}

impl fmt::Debug for Texture {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Texture")
            .field("name", &self.name)
            .field("transparent", &self.transparent)
            .field("image", self.image.image())
            .field("sampler", &self.sampler)
            .finish()
    }
}

/// Texture cache.
#[derive(Default, Clone)]
pub struct TextureCache {
    /// Descriptor set.
    pub(crate) descriptor_set: Option<Arc<dyn DescriptorSet + Send + Sync>>,
}

impl TextureCache {
    /// Resets the cache.
    pub fn reset(&mut self) {
        *self = Default::default();
    }
}
