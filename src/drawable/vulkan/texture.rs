//! Texture.

use std::{fmt, sync::Arc};

use vulkano::{
    descriptor::descriptor_set::DescriptorSet, format::R8G8B8A8Srgb, image::ImmutableImage,
    sampler::Sampler,
};

/// Texture.
#[derive(Clone)]
pub struct Texture {
    /// Name.
    pub(crate) name: Option<String>,
    /// Whether the texture has transparent data or not.
    pub(crate) transparent: bool,
    /// Texture.
    pub(crate) texture: Arc<ImmutableImage<R8G8B8A8Srgb>>,
    /// Sampler.
    pub(crate) sampler: Arc<Sampler>,
    /// Descriptor set.
    pub(crate) descriptor_set: Arc<dyn DescriptorSet + Send + Sync>,
}

impl fmt::Debug for Texture {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Texture")
            .field("name", &self.name)
            .field("transparent", &self.transparent)
            .field("texture", &self.texture)
            .field("sampler", &self.sampler)
            .finish()
    }
}

impl Texture {
    /// Returns the model name if available.
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(AsRef::as_ref)
    }

    /// Returns whether the texture can be transparent.
    pub fn is_transparent(&self) -> bool {
        self.transparent
    }

    /// Returns texture image.
    pub fn texture(&self) -> &Arc<ImmutableImage<R8G8B8A8Srgb>> {
        &self.texture
    }

    /// Returns sampler.
    pub fn sampler(&self) -> &Arc<Sampler> {
        &self.sampler
    }

    /// Returns descriptor set.
    pub fn descriptor_set(&self) -> &Arc<dyn DescriptorSet + Send + Sync> {
        &self.descriptor_set
    }
}
