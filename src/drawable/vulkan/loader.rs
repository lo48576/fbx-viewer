//! Loader.

use std::{collections::HashMap, sync::Arc};

use failure::{format_err, Fallible};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    descriptor::descriptor_set::PersistentDescriptorSet,
    device::{Device, Queue},
    format::R8G8B8A8Srgb,
    image::{Dimensions, ImmutableImage},
    pipeline::GraphicsPipelineAbstract,
    sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode},
    sync::GpuFuture,
};

use crate::drawable::vulkan::{Mesh, Model, Scene, SubMesh, Texture};

/// Loader.
pub struct Loader {
    /// Loaded textures.
    loaded_textures: HashMap<crate::data::TextureId, Arc<Texture>>,
    /// Device.
    device: Arc<Device>,
    /// Queue.
    queue: Arc<Queue>,
    /// Future.
    future: Option<Box<dyn GpuFuture>>,
    /// Pipeline.
    pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
}

impl Loader {
    /// Creates a new `Loader`.
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    ) -> Self {
        Self {
            loaded_textures: Default::default(),
            device,
            queue,
            future: None,
            pipeline,
        }
    }

    /// Loads the given scene.
    pub fn load(
        mut self,
        scene: &crate::data::Scene,
    ) -> Fallible<(Scene, Option<Box<dyn GpuFuture>>)> {
        let models = scene
            .models
            .iter()
            .map(|model| self.load_model(model, scene))
            .collect::<Fallible<_>>()?;

        Ok((
            Scene {
                name: scene.name.clone(),
                models,
            },
            self.future,
        ))
    }

    /// Loads a model.
    fn load_model(
        &mut self,
        model: &crate::data::Model,
        scene: &crate::data::Scene,
    ) -> Fallible<Model> {
        let meshes = model
            .meshes
            .iter()
            .map(|mesh| self.load_mesh(mesh, scene))
            .collect::<Fallible<_>>()?;

        Ok(Model {
            name: model.name.clone(),
            meshes,
        })
    }

    /// Loads a mesh.
    fn load_mesh(
        &mut self,
        mesh: &crate::data::Mesh,
        scene: &crate::data::Scene,
    ) -> Fallible<Mesh> {
        let vertex = CpuAccessibleBuffer::from_iter(
            self.device.clone(),
            BufferUsage::all(),
            mesh.vertices.iter().cloned(),
        )?;
        let submeshes = mesh
            .submeshes
            .iter()
            .map(|submesh| self.load_submesh(submesh, scene))
            .collect::<Fallible<_>>()?;

        Ok(Mesh {
            name: mesh.name.clone(),
            vertex,
            submeshes,
        })
    }

    /// Loads a submesh.
    fn load_submesh(
        &mut self,
        submesh: &crate::data::SubMesh,
        scene: &crate::data::Scene,
    ) -> Fallible<SubMesh> {
        let indices = CpuAccessibleBuffer::from_iter(
            self.device.clone(),
            BufferUsage::all(),
            submesh.indices.iter().cloned(),
        )?;
        let texture = submesh
            .texture_id
            .map(|texture_id| self.load_texture(texture_id, scene))
            .transpose()?
            .cloned();

        Ok(SubMesh {
            material_index: submesh.material_index,
            indices,
            texture,
        })
    }

    /// Loads a texture and a sampler.
    fn load_texture(
        &mut self,
        texture_id: crate::data::TextureId,
        scene: &crate::data::Scene,
    ) -> Fallible<&Arc<Texture>> {
        use image::GenericImageView;
        use std::collections::hash_map::Entry;

        let entry = match self.loaded_textures.entry(texture_id) {
            Entry::Occupied(entry) => return Ok(entry.into_mut()),
            Entry::Vacant(entry) => entry,
        };
        let tex_data = scene
            .textures
            .get(&texture_id)
            .ok_or_else(|| format_err!("Failed to get texture: texture_id={:?}", texture_id))?;
        let dim = Dimensions::Dim2d {
            width: tex_data.data.width(),
            height: tex_data.data.height(),
        };
        let (image, img_future) = ImmutableImage::from_iter(
            tex_data.data.to_rgba().into_raw().iter().cloned(),
            dim,
            R8G8B8A8Srgb,
            self.queue.clone(),
        )?;
        join_futures(&mut self.future, img_future);
        // TODO: Use properties of a texture object.
        let sampler = Sampler::new(
            self.device.clone(),
            Filter::Linear,
            Filter::Linear,
            MipmapMode::Nearest,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            0.0,
            1.0,
            0.0,
            0.0,
        )?;
        let descriptor_set = Arc::new(
            PersistentDescriptorSet::start(self.pipeline.clone(), 1)
                .add_sampled_image(image.clone(), sampler.clone())
                .expect("Failed to add sampled image")
                .build()
                .expect("Failed to build descriptor set"),
        ) as Arc<_>;
        let texture = Texture {
            name: tex_data.name.clone(),
            transparent: tex_data.transparent,
            texture: image,
            sampler,
            descriptor_set,
        };
        Ok(entry.insert(Arc::new(texture)))
    }
}

/// Joins the given futures.
fn join_futures(prev: &mut Option<Box<dyn GpuFuture>>, f: impl GpuFuture + 'static) {
    let new = match prev.take() {
        Some(prev) => Box::new(prev.join(f)) as Box<_>,
        None => Box::new(f) as Box<_>,
    };
    *prev = Some(new);
}
