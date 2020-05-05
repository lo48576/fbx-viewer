//! FBX v7400 support.

use std::{collections::HashMap, path::Path};

use anyhow::{anyhow, bail, Context};
use cgmath::{Point2, Point3, Vector3};
use fbxcel_dom::v7400::{
    data::{
        material::ShadingModel, mesh::layer::TypedLayerElementHandle,
        texture::WrapMode as RawWrapMode,
    },
    object::{self, model::TypedModelHandle, ObjectId, TypedObjectHandle},
    Document,
};
use log::{debug, trace};
use rgb::ComponentMap;

use crate::{
    data::{
        GeometryMesh, GeometryMeshIndex, LambertData, Material, MaterialIndex, Mesh, MeshIndex,
        Scene, ShadingData, Texture, TextureIndex, WrapMode,
    },
    util::iter::{OptionIteratorExt, ResultIteratorExt},
};

use self::triangulator::triangulator;

mod triangulator;

/// Loads the data from the document.
pub fn from_doc(doc: Box<Document>) -> anyhow::Result<Scene> {
    Loader::new(&doc).load()
}

/// FBX data loader.
pub struct Loader<'a> {
    /// Document.
    doc: &'a Document,
    /// Scene.
    scene: Scene,
    /// Geometry mesh indices.
    geometry_mesh_indices: HashMap<ObjectId, GeometryMeshIndex>,
    /// Material indices.
    material_indices: HashMap<ObjectId, MaterialIndex>,
    /// Mesh indices.
    mesh_indices: HashMap<ObjectId, MeshIndex>,
    /// Texture indices.
    texture_indices: HashMap<ObjectId, TextureIndex>,
}

impl<'a> Loader<'a> {
    /// Creates a new `Loader`.
    fn new(doc: &'a Document) -> Self {
        Self {
            doc,
            scene: Default::default(),
            geometry_mesh_indices: Default::default(),
            material_indices: Default::default(),
            mesh_indices: Default::default(),
            texture_indices: Default::default(),
        }
    }

    /// Loads the document.
    fn load(mut self) -> anyhow::Result<Scene> {
        for obj in self.doc.objects() {
            if let TypedObjectHandle::Model(TypedModelHandle::Mesh(mesh)) = obj.get_typed() {
                self.load_mesh(mesh)?;
            }
        }

        Ok(self.scene)
    }

    /// Loads the geometry.
    fn load_geometry_mesh(
        &mut self,
        mesh_obj: object::geometry::MeshHandle<'a>,
        num_materials: usize,
    ) -> anyhow::Result<GeometryMeshIndex> {
        if let Some(index) = self.geometry_mesh_indices.get(&mesh_obj.object_id()) {
            return Ok(*index);
        }

        debug!("Loading geometry mesh: {:?}", mesh_obj);

        let polygon_vertices = mesh_obj
            .polygon_vertices()
            .context("Failed to get polygon vertices")?;
        let triangle_pvi_indices = polygon_vertices
            .triangulate_each(triangulator)
            .context("Triangulation failed")?;

        let positions = triangle_pvi_indices
            .iter_control_point_indices()
            .ok_or_else(|| anyhow!("Failed to get control point index"))
            .and_then(|cpi| {
                polygon_vertices
                    .control_point(cpi)
                    .map(Point3::from)
                    .ok_or_else(|| anyhow!("Failed to get control point: cpi={:?}", cpi))
            })
            .and_then(|p| {
                p.cast().ok_or_else(|| {
                    anyhow!("Failed to convert floating point values: point={:?}", p)
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to reconstruct position vertices")?;
        trace!("Expanded positions len: {:?}", positions.len());

        let layer = mesh_obj
            .layers()
            .next()
            .ok_or_else(|| anyhow!("Failed to get layer"))?;

        let normals = {
            let normals = layer
                .layer_element_entries()
                .filter_map(|entry| match entry.typed_layer_element() {
                    Ok(TypedLayerElementHandle::Normal(handle)) => Some(handle),
                    _ => None,
                })
                .next()
                .ok_or_else(|| anyhow!("Failed to get normals"))?
                .normals()
                .context("Failed to get normals")?;
            triangle_pvi_indices
                .triangle_vertex_indices()
                .map(|tri_vi| -> Result<_, _> {
                    normals
                        .normal(&triangle_pvi_indices, tri_vi)
                        .map(Vector3::from)
                })
                .and_then(|v| {
                    v.cast().ok_or_else(|| {
                        anyhow!("Failed to convert floating point values: vector={:?}", v)
                    })
                })
                .collect::<Result<Vec<_>, _>>()
                .context("Failed to reconstruct normals vertices")?
        };
        let uv = {
            let uv = layer
                .layer_element_entries()
                .filter_map(|entry| match entry.typed_layer_element() {
                    Ok(TypedLayerElementHandle::Uv(handle)) => Some(handle),
                    _ => None,
                })
                .next()
                .ok_or_else(|| anyhow!("Failed to get UV"))?
                .uv()?;
            triangle_pvi_indices
                .triangle_vertex_indices()
                .map(|tri_vi| uv.uv(&triangle_pvi_indices, tri_vi).map(Point2::from))
                .and_then(|p| {
                    p.cast().ok_or_else(|| {
                        anyhow!("Failed to convert floating point values: point={:?}", p)
                    })
                })
                .collect::<Result<Vec<_>, _>>()
                .context("Failed to reconstruct UV vertices")?
        };

        let indices_per_material = {
            let mut indices_per_material = vec![Vec::new(); num_materials];
            let materials = layer
                .layer_element_entries()
                .filter_map(|entry| match entry.typed_layer_element() {
                    Ok(TypedLayerElementHandle::Material(handle)) => Some(handle),
                    _ => None,
                })
                .next()
                .ok_or_else(|| anyhow!("Materials not found for mesh {:?}", mesh_obj))?
                .materials()
                .context("Failed to get materials")?;
            for tri_vi in triangle_pvi_indices.triangle_vertex_indices() {
                let local_material_index = materials
                    .material_index(&triangle_pvi_indices, tri_vi)
                    .context("Failed to get mesh-local material index")?
                    .to_u32();
                indices_per_material
                    .get_mut(local_material_index as usize)
                    .ok_or_else(|| {
                        anyhow!(
                            "Mesh-local material index out of range: num_materials={:?}, got={:?}",
                            num_materials,
                            local_material_index
                        )
                    })?
                    .push(tri_vi.to_usize() as u32);
            }
            indices_per_material
        };

        if positions.len() != normals.len() {
            bail!(
                "Vertices length mismatch: positions.len={:?}, normals.len={:?}",
                positions.len(),
                normals.len()
            );
        }
        if positions.len() != uv.len() {
            bail!(
                "Vertices length mismatch: positions.len={:?}, uv.len={:?}",
                positions.len(),
                uv.len()
            );
        }

        let mesh = GeometryMesh {
            name: mesh_obj.name().map(Into::into),
            positions,
            normals,
            uv,
            indices_per_material,
        };

        debug!("Successfully loaded geometry mesh: {:?}", mesh_obj);

        Ok(self.scene.add_geometry_mesh(mesh))
    }

    /// Loads the material.
    fn load_material(
        &mut self,
        material_obj: object::material::MaterialHandle<'a>,
    ) -> anyhow::Result<MaterialIndex> {
        if let Some(index) = self.material_indices.get(&material_obj.object_id()) {
            return Ok(*index);
        }

        debug!("Loading material: {:?}", material_obj);

        let diffuse_texture = material_obj
            .transparent_texture()
            .map(|v| (true, v))
            .or_else(|| material_obj.diffuse_texture().map(|v| (false, v)))
            .map(|(transparent, texture_obj)| {
                self.load_texture(texture_obj, transparent)
                    .context("Failed to load diffuse texture")
            })
            .transpose()?;

        let properties = material_obj.properties();
        let shading_data = match properties
            .shading_model_or_default()
            .context("Failed to get shading model")?
        {
            ShadingModel::Lambert | ShadingModel::Phong => {
                let ambient_color = properties
                    .ambient_color_or_default()
                    .context("Failed to get ambient color")?;
                let ambient_factor = properties
                    .ambient_factor_or_default()
                    .context("Failed to get ambient factor")?;
                let ambient = (ambient_color * ambient_factor).map(|v| v as f32);
                let diffuse_color = properties
                    .diffuse_color_or_default()
                    .context("Failed to get diffuse color")?;
                let diffuse_factor = properties
                    .diffuse_factor_or_default()
                    .context("Failed to get diffuse factor")?;
                let diffuse = (diffuse_color * diffuse_factor).map(|v| v as f32);
                let emissive_color = properties
                    .emissive_color_or_default()
                    .context("Failed to get emissive color")?;
                let emissive_factor = properties
                    .emissive_factor_or_default()
                    .context("Failed to get emissive factor")?;
                let emissive = (emissive_color * emissive_factor).map(|v| v as f32);
                ShadingData::Lambert(LambertData {
                    ambient,
                    diffuse,
                    emissive,
                })
            }
            v => bail!("Unknown shading model: {:?}", v),
        };

        let material = Material {
            name: material_obj.name().map(Into::into),
            diffuse_texture,
            data: shading_data,
        };

        debug!("Successfully loaded material: {:?}", material_obj);

        Ok(self.scene.add_material(material))
    }

    /// Loads the mesh.
    fn load_mesh(&mut self, mesh_obj: object::model::MeshHandle<'a>) -> anyhow::Result<MeshIndex> {
        if let Some(index) = self.mesh_indices.get(&mesh_obj.object_id()) {
            return Ok(*index);
        }

        debug!("Loading mesh: {:?}", mesh_obj);

        let geometry_obj = mesh_obj.geometry().context("Failed to get geometry")?;

        let materials = mesh_obj
            .materials()
            .map(|material_obj| self.load_material(material_obj))
            .collect::<anyhow::Result<Vec<_>>>()
            .context("Failed to load materials for mesh")?;

        let geometry_index = self
            .load_geometry_mesh(geometry_obj, materials.len())
            .context("Failed to load geometry mesh")?;

        let mesh = Mesh {
            name: mesh_obj.name().map(Into::into),
            geometry_mesh_index: geometry_index,
            materials,
        };

        debug!("Successfully loaded mesh: {:?}", mesh_obj);

        Ok(self.scene.add_mesh(mesh))
    }

    /// Loads the texture.
    fn load_texture(
        &mut self,
        texture_obj: object::texture::TextureHandle<'a>,
        transparent: bool,
    ) -> anyhow::Result<TextureIndex> {
        if let Some(index) = self.texture_indices.get(&texture_obj.object_id()) {
            return Ok(*index);
        }

        debug!("Loading texture: {:?}", texture_obj);

        let properties = texture_obj.properties();
        let wrap_mode_u = {
            let val = properties
                .wrap_mode_u_or_default()
                .context("Failed to load wrap mode for U axis")?;
            match val {
                RawWrapMode::Repeat => WrapMode::Repeat,
                RawWrapMode::Clamp => WrapMode::ClampToEdge,
            }
        };
        let wrap_mode_v = {
            let val = properties
                .wrap_mode_v_or_default()
                .context("Failed to load wrap mode for V axis")?;
            match val {
                RawWrapMode::Repeat => WrapMode::Repeat,
                RawWrapMode::Clamp => WrapMode::ClampToEdge,
            }
        };
        let video_clip_obj = texture_obj
            .video_clip()
            .ok_or_else(|| anyhow!("No image data for texture object: {:?}", texture_obj))?;
        let image = self
            .load_video_clip(video_clip_obj)
            .context("Failed to load texture image")?;

        let texture = Texture {
            name: texture_obj.name().map(Into::into),
            image,
            transparent,
            wrap_mode_u,
            wrap_mode_v,
        };

        debug!("Successfully loaded texture: {:?}", texture_obj);

        Ok(self.scene.add_texture(texture))
    }

    /// Loads the texture image.
    fn load_video_clip(
        &mut self,
        video_clip_obj: object::video::ClipHandle<'a>,
    ) -> anyhow::Result<image::DynamicImage> {
        debug!("Loading texture image: {:?}", video_clip_obj);

        let relative_filename = video_clip_obj
            .relative_filename()
            .context("Failed to get relative filename of texture image")?;
        trace!("Relative filename: {:?}", relative_filename);
        let file_ext = Path::new(&relative_filename)
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .map(str::to_ascii_lowercase);
        trace!("File extension: {:?}", file_ext);
        let content = video_clip_obj
            .content()
            .ok_or_else(|| anyhow!("Currently, only embedded texture is supported"))?;
        let image = match file_ext.as_ref().map(AsRef::as_ref) {
            Some("tga") => image::load_from_memory_with_format(content, image::ImageFormat::Tga)
                .context("Failed to load TGA image")?,
            _ => image::load_from_memory(content).context("Failed to load image")?,
        };

        debug!("Successfully loaded texture image: {:?}", video_clip_obj);

        Ok(image)
    }
}
