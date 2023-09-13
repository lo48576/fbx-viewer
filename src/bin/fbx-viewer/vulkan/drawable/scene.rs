//! Scene.

use std::sync::Arc;

use anyhow::Context;
use fbx_viewer::{
    data::{GeometryMeshIndex, MaterialIndex, TextureIndex},
    util::bbox::OptionalBoundingBox3d,
};
use vulkano::{
    buffer::ImmutableBuffer,
    descriptor::{
        descriptor_set::{PersistentDescriptorSet, PersistentDescriptorSetBuf},
        pipeline_layout::PipelineLayoutAbstract,
    },
    pipeline::GraphicsPipeline,
    sync::GpuFuture,
};

use crate::vulkan::{
    drawable::{GeometryMesh, Material, Mesh, Texture},
    fs::ty::Material as ShaderMaterial,
    setup::create_diffuse_texture_desc_set,
};

/// Scene.
#[derive(Default, Debug, Clone)]
pub struct Scene {
    /// Name.
    #[allow(dead_code)]
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

    /// Returns bounding box of all geometries.
    pub fn bbox(&self) -> OptionalBoundingBox3d<f32> {
        self.geometry_meshes
            .iter()
            .map(|gm| &gm.bounding_box)
            .collect()
    }

    /// Reset and initialize caches with the given pipeline.
    pub fn reset_cache_with_pipeline<Mv, L, Rp>(
        &mut self,
        pipeline: &Arc<GraphicsPipeline<Mv, L, Rp>>,
    ) -> anyhow::Result<Option<Box<dyn GpuFuture>>>
    where
        L: PipelineLayoutAbstract,
    {
        let future = None;

        for material in &mut self.materials {
            material.cache.reset();
            material.cache.uniform_buffer = Some(create_material_desc_set(
                material.data.clone(),
                pipeline.clone(),
            )?);
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

/// Creates a descriptor set for the given material uniform buffer.
#[allow(clippy::type_complexity)]
fn create_material_desc_set<Mv, L, Rp>(
    material_buf: Arc<ImmutableBuffer<ShaderMaterial>>,
    pipeline: Arc<GraphicsPipeline<Mv, L, Rp>>,
) -> anyhow::Result<
    Arc<
        PersistentDescriptorSet<(
            (),
            PersistentDescriptorSetBuf<Arc<ImmutableBuffer<ShaderMaterial>>>,
        )>,
    >,
>
where
    L: PipelineLayoutAbstract,
{
    let layout = pipeline
        .layout()
        .descriptor_set_layout(2)
        .context("Failed to get the second descriptor set layout of the pipeline")?;
    let desc_set = PersistentDescriptorSet::start(layout.clone())
        .add_buffer(material_buf)
        .context("Failed to add material data to descriptor set")?
        .build()
        .context("Failed to build material descriptor set")?;

    Ok(Arc::new(desc_set) as Arc<_>)
}
