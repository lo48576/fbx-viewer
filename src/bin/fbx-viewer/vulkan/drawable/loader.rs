//! Loader.

use std::sync::Arc;

use failure::{format_err, Fallible, ResultExt};
use fbx_viewer::data;
use vulkano::{
    buffer::{BufferUsage, ImmutableBuffer},
    device::{Device, Queue},
    format::R8G8B8A8Srgb,
    image::{Dimensions, ImmutableImage},
    sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode},
    sync::GpuFuture,
};

use crate::vulkan::{
    drawable::{self, join_futures},
    fs,
};

/// Loader.
pub struct Loader {
    /// Device.
    device: Arc<Device>,
    /// Queue.
    queue: Arc<Queue>,
    /// GPU future.
    future: Option<Box<dyn GpuFuture>>,
}

impl Loader {
    /// Creates a new `Loader`.
    pub fn new(device: Arc<Device>, queue: Arc<Queue>) -> Self {
        Self {
            device,
            queue,
            future: None,
        }
    }

    /// Loads the scene.
    pub(crate) fn load(
        mut self,
        src_scene: &data::Scene,
    ) -> Fallible<(drawable::Scene, Option<Box<dyn GpuFuture>>)> {
        let mut scene = drawable::Scene::default();

        for src_geometry in src_scene.geometry_meshes() {
            let vertices = src_geometry
                .positions
                .iter()
                .cloned()
                .map(Into::into)
                .zip(src_geometry.normals.iter().cloned().map(Into::into))
                .zip(src_geometry.uv.iter().cloned().map(Into::into))
                .map(|((position, normal), uv)| drawable::Vertex {
                    position,
                    normal,
                    uv,
                })
                .collect::<Vec<_>>();
            let (vertices, vertices_future) = ImmutableBuffer::from_iter(
                vertices.into_iter(),
                BufferUsage::all(),
                self.queue.clone(),
            )?;
            join_futures(&mut self.future, vertices_future);

            let indices_per_material = src_geometry
                .indices_per_material
                .iter()
                .map(|indices| {
                    let (buf, buf_future) = ImmutableBuffer::from_iter(
                        indices.iter().cloned(),
                        BufferUsage::all(),
                        self.queue.clone(),
                    )?;
                    join_futures(&mut self.future, buf_future);
                    Ok(buf)
                })
                .collect::<Fallible<Vec<_>>>()
                .with_context(|e| format_err!("Failed to upload index buffers: {}", e))?;
            let bounding_box = src_geometry.bbox_mesh();
            let geometry = drawable::GeometryMesh {
                name: src_geometry.name.clone(),
                vertices,
                indices_per_material,
                bounding_box,
            };
            scene.geometry_meshes.push(geometry);
        }

        for src_material in src_scene.materials() {
            let diffuse_texture_exists = src_material.diffuse_texture.is_some();
            let data = match src_material.data {
                data::ShadingData::Lambert(lambert) => fs::ty::Material {
                    ambient: lambert.ambient,
                    _dummy0: [0; 4],
                    diffuse: lambert.diffuse,
                    emissive: lambert.emissive,
                    _dummy1: [0; 4],
                    enabled: !diffuse_texture_exists as u32,
                },
            };
            let (data, data_future) =
                ImmutableBuffer::from_data(data, BufferUsage::all(), self.queue.clone())
                    .with_context(|e| format_err!("Failed to upload material: {}", e))?;
            join_futures(&mut self.future, data_future);

            let material = drawable::Material {
                name: src_material.name.clone(),
                diffuse_texture: src_material.diffuse_texture,
                data,
                cache: Default::default(),
            };
            scene.materials.push(material);
        }

        for src_mesh in src_scene.meshes() {
            scene.meshes.push(src_mesh.clone());
        }

        for src_texture in src_scene.textures() {
            use image::GenericImageView;

            let dim = Dimensions::Dim2d {
                width: src_texture.image.width(),
                height: src_texture.image.height(),
            };
            let (image, image_future) = ImmutableImage::from_iter(
                src_texture.image.to_rgba().into_raw().into_iter(),
                dim,
                R8G8B8A8Srgb,
                self.queue.clone(),
            )
            .with_context(|e| format_err!("Failed to upload texture image: {}", e))?;
            join_futures(&mut self.future, image_future);
            let wrap_mode_u = match src_texture.wrap_mode_u {
                data::WrapMode::Repeat => SamplerAddressMode::Repeat,
                data::WrapMode::ClampToEdge => SamplerAddressMode::ClampToEdge,
            };
            let wrap_mode_v = match src_texture.wrap_mode_v {
                data::WrapMode::Repeat => SamplerAddressMode::Repeat,
                data::WrapMode::ClampToEdge => SamplerAddressMode::ClampToEdge,
            };
            let sampler = Sampler::new(
                self.device.clone(),
                Filter::Linear,
                Filter::Linear,
                MipmapMode::Nearest,
                wrap_mode_u,
                wrap_mode_v,
                SamplerAddressMode::Repeat,
                0.0,
                1.0,
                0.0,
                0.0,
            )
            .with_context(|e| format_err!("Failed to create sampler: {}", e))?;

            let texture = drawable::Texture {
                name: src_texture.name.clone(),
                image,
                sampler,
                transparent: src_texture.transparent,
                cache: Default::default(),
            };
            scene.textures.push(texture);
        }

        Ok((scene, self.future))
    }
}
