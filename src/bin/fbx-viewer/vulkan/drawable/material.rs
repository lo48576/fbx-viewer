//! Material.

use std::{fmt, sync::Arc};

use fbx_viewer::data::TextureIndex;
use vulkano::{buffer::ImmutableBuffer, descriptor::descriptor_set::DescriptorSet};

use crate::vulkan::fs::ty::Material as ShaderMaterial;

/// Material.
#[derive(Clone)]
pub struct Material {
    /// Name.
    pub(crate) name: Option<String>,
    /// Texture index.
    pub(crate) diffuse_texture: Option<TextureIndex>,
    /// Shading parameters.
    pub(crate) data: Arc<ImmutableBuffer<ShaderMaterial>>,
    /// Cache.
    pub(crate) cache: MaterialCache,
}

impl fmt::Debug for Material {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Material")
            .field("name", &self.name)
            .field("diffuse_texture", &self.diffuse_texture)
            .finish()
    }
}

/// Material cache.
#[derive(Default, Clone)]
pub struct MaterialCache {
    /// Uniform buffer.
    pub(crate) uniform_buffer: Option<Arc<dyn DescriptorSet + Send + Sync>>,
}

impl MaterialCache {
    /// Resets the cache.
    pub fn reset(&mut self) {
        *self = Default::default();
    }
}
