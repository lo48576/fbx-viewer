//! Scene.

use std::sync::Arc;

use failure::{format_err, Fallible, ResultExt};
use fbx_viewer::data::{GeometryMeshIndex, MaterialIndex, TextureIndex};
use vulkano::{
    descriptor::descriptor_set::PersistentDescriptorSet, pipeline::GraphicsPipelineAbstract,
    sync::GpuFuture,
};

use crate::vulkan::{
    drawable::{GeometryMesh, Material, Mesh, Texture},
    setup::create_diffuse_texture_desc_set,
};

/// Scene.
#[derive(Default, Debug, Clone)]
pub struct Scene {
    /// Name.
    pub(crate) name: Option<String>,
    /// Geometry mesh.
    pub(crate) geometry_meshes: Vec<GeometryMesh>,
    /// Materials.
    pub(crate) materials: Vec<Material>,
    /// Meshes.
    pub(crate) meshes: Vec<Mesh>,
    /// Textures.
    pub(crate) textures: Vec<Texture>,
}

impl Scene {
    /// Returns a reference to the geometry mesh.
    pub fn geometry_mesh(&self, i: GeometryMeshIndex) -> Option<&GeometryMesh> {
        self.geometry_meshes.get(i.to_usize())
    }

    /// Returns a reference to the material.
    pub fn material(&self, i: MaterialIndex) -> Option<&Material> {
        self.materials.get(i.to_usize())
    }

    /// Returns a reference to the texture.
    pub fn texture(&self, i: TextureIndex) -> Option<&Texture> {
        self.textures.get(i.to_usize())
    }

    /// Reset and initialize caches with the given pipeline.
    pub fn reset_cache_with_pipeline(
        &mut self,
        pipeline: &Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    ) -> Fallible<Option<Box<dyn GpuFuture>>> {
        let future = None;

        for material in &mut self.materials {
            material.cache.reset();
            let uniform_buffer = PersistentDescriptorSet::start(pipeline.clone(), 2)
                .add_buffer(material.data.clone())
                .with_context(|e| {
                    format_err!("Failed to add material data to descriptor set: {}", e)
                })?
                .build()
                .with_context(|e| format_err!("Failed to build material descriptor set: {}", e))?;
            material.cache.uniform_buffer = Some(Arc::new(uniform_buffer) as Arc<_>);
        }

        for texture in &mut self.textures {
            texture.cache.reset();
            texture.cache.descriptor_set = Some(create_diffuse_texture_desc_set(
                texture.image.clone(),
                texture.sampler.clone(),
                pipeline.clone(),
            )?);
        }

        Ok(future)
    }
}
