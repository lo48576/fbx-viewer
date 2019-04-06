//! FBX v7400 support.

use std::{
    collections::{BTreeMap, HashMap},
    path::Path,
};

use failure::{format_err, Fallible, ResultExt};
use fbxcel_dom::v7400::{
    data::mesh::layer::TypedLayerElementHandle,
    object::{
        model::{self, TypedModelHandle},
        ObjectId, TypedObjectHandle,
    },
    Document,
};
use log::{debug, trace};

use crate::data::{mesh::Vertex, Mesh, Model, Scene, SubMesh, Texture, TextureId};

use self::triangulator::triangulator;

mod triangulator;

/// Loads the data from the document.
pub fn from_doc(doc: Box<Document>) -> Fallible<Scene> {
    let mut mesh_objs = HashMap::new();

    for obj in doc.objects() {
        if let TypedObjectHandle::Model(TypedModelHandle::Mesh(mesh)) = obj.get_typed() {
            mesh_objs.insert(mesh.object_id(), mesh);
        }
    }

    let mut scene = Scene::new();

    let mut models = HashMap::<ObjectId, (Option<String>, Vec<Mesh>)>::new();
    let mut textures = HashMap::<TextureId, Texture>::new();

    // Load meshes.
    // TODO: support instantiation (geometry sharing among different models).
    for (_mesh_id, mesh_obj) in mesh_objs {
        debug!(
            "Loading mesh: name={:?}, object_id={:?}",
            mesh_obj.name(),
            mesh_obj.object_id()
        );
        let root_model = {
            let mut current: model::ModelHandle = *mesh_obj;
            while let Some(parent) = current.parent_model() {
                current = *parent;
            }
            current
        };
        debug!(
            "Root model of mesh {:?}: name={:?}, object_id={:?}",
            mesh_obj.object_id(),
            root_model.name(),
            root_model.object_id()
        );

        let mesh: Mesh = {
            let geometry = mesh_obj
                .geometry()
                .with_context(|e| format_err!("Failed to get geometry: {}", e))?;
            trace!("Geometry ID: {:?}", geometry.object_id());
            // Mesh.
            let control_points = geometry
                .control_points()
                .with_context(|e| format_err!("Failed to get control points: {}", e))?;
            let polygon_vertex_indices = geometry
                .polygon_vertex_indices()
                .with_context(|e| format_err!("Failed to get polygon vertices: {}", e))?;
            let triangle_pvi_indices = polygon_vertex_indices
                .triangulate_each(&control_points, triangulator)
                .with_context(|e| format_err!("Triangulation failed: {}", e))?;
            let positions = triangle_pvi_indices
                .iter_control_point_indices()
                .map(|cpi| {
                    let cpi =
                        cpi.ok_or_else(|| format_err!("Failed to get control point index"))?;
                    control_points
                        .get_cp_f32(cpi)
                        .ok_or_else(|| format_err!("Failed to get control point"))
                })
                .collect::<Result<Vec<_>, _>>()
                .with_context(|e| format_err!("Failed to reconstruct position vertices: {}", e))?;
            let layer = geometry
                .layers()
                .next()
                .ok_or_else(|| format_err!("Failed to get layer"))?;
            let normals = {
                let normals = layer
                    .layer_element_entries()
                    .filter_map(|entry| match entry.typed_layer_element() {
                        Ok(TypedLayerElementHandle::Normal(handle)) => Some(handle),
                        _ => None,
                    })
                    .next()
                    .ok_or_else(|| format_err!("Failed to get normals"))?
                    .normals()?;
                triangle_pvi_indices
                    .triangle_vertex_indices()
                    .map(|tri_vi| normals.get_xyz_f32_by_tri_vi(&triangle_pvi_indices, tri_vi))
                    .collect::<Result<Vec<_>, _>>()
                    .with_context(|e| {
                        format_err!("Failed to reconstruct normals vertices: {}", e)
                    })?
            };
            let uv = {
                let uv = layer
                    .layer_element_entries()
                    .filter_map(|entry| match entry.typed_layer_element() {
                        Ok(TypedLayerElementHandle::Uv(handle)) => Some(handle),
                        _ => None,
                    })
                    .next()
                    .ok_or_else(|| format_err!("Failed to get UV"))?
                    .uv()?;
                triangle_pvi_indices
                    .triangle_vertex_indices()
                    .map(|tri_vi| uv.get_uv_f32_by_tri_vi(&triangle_pvi_indices, tri_vi))
                    .collect::<Result<Vec<_>, _>>()
                    .with_context(|e| format_err!("Failed to reconstruct UV vertices: {}", e))?
            };
            let material_indices = {
                let materials = layer
                    .layer_element_entries()
                    .filter_map(|entry| match entry.typed_layer_element() {
                        Ok(TypedLayerElementHandle::Material(handle)) => Some(handle),
                        _ => None,
                    })
                    .next()
                    .ok_or_else(|| format_err!("Materials not found for mesh {:?}", mesh_obj))?
                    .materials()
                    .with_context(|e| format_err!("Failed to get materials: {}", e))?;
                triangle_pvi_indices
                    .triangle_vertex_indices()
                    .map(|tri_vi| {
                        materials.get_material_index_by_tri_vi(&triangle_pvi_indices, tri_vi)
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .with_context(|e| {
                        format_err!("Failed to reconstruct material indices: {}", e)
                    })?
            };
            trace!("vertices len: {:?}", positions.len());
            let vertices = positions
                .into_iter()
                .zip(normals)
                .zip(uv)
                .zip(material_indices.iter().map(|i| i.get_u32()))
                .map(|(((position, normal), uv), material)| Vertex {
                    position,
                    normal,
                    uv,
                    material,
                })
                .collect();

            // Load texture.
            let texture_ids = {
                let mut ids = Vec::new();
                for material_obj in mesh_obj.materials() {
                    trace!("Material {}: {:?}", ids.len(), material_obj);
                    let texture_obj_opt = material_obj
                        .transparent_texture()
                        .map(|v| (true, v))
                        .or_else(|| material_obj.diffuse_texture().map(|v| (false, v)));
                    let (transparent, texture_obj) = match texture_obj_opt {
                        Some(v) => v,
                        None => {
                            trace!("No texture object for material {:?}", material_obj);
                            ids.push(None);
                            continue;
                        }
                    };
                    let name = texture_obj.name();
                    let video_clip_obj = texture_obj
                        .video_clip()
                        .ok_or_else(|| {
                            format_err!("No image data for texture object: {:?}", texture_obj)
                        })
                        .with_context(|e| format_err!("Failed to get video clip object: {}", e))?;
                    trace!("Video clip object found: {:?}", video_clip_obj);

                    let tex_id = TextureId(video_clip_obj.object_id().raw());
                    ids.push(Some(tex_id));

                    let entry = match textures.entry(tex_id) {
                        std::collections::hash_map::Entry::Occupied(_) => continue,
                        std::collections::hash_map::Entry::Vacant(entry) => entry,
                    };
                    trace!("Not cached, loading texture");
                    let file_ext = Path::new(video_clip_obj.relative_filename()?)
                        .extension()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_ascii_lowercase());
                    trace!("filename: {:?}", video_clip_obj.relative_filename()?);
                    let content = video_clip_obj
                        .content()
                        .ok_or_else(|| format_err!("Currently, only embedded texture is supported"))
                        .with_context(|e| format_err!("Failed to get texture image: {}", e))?;
                    let image = match file_ext.as_ref().map(AsRef::as_ref) {
                        Some("tga") => {
                            image::load_from_memory_with_format(content, image::ImageFormat::TGA)
                                .with_context(|e| format_err!("Failed to load TGA image: {}", e))?
                        }
                        _ => image::load_from_memory(content)
                            .with_context(|e| format_err!("Failed to load image: {}", e))?,
                    };
                    entry.insert(Texture {
                        name: name.map(Into::into),
                        transparent,
                        data: image,
                    });
                }
                ids
            };

            // Create submeshes.
            let mut submeshes = BTreeMap::new();
            assert_eq!(triangle_pvi_indices.len(), material_indices.len());
            // Split meshes.
            for (pvii, &material_i) in material_indices.iter().enumerate() {
                submeshes
                    .entry(material_i)
                    .or_insert_with(Vec::new)
                    .push(pvii as u32);
            }
            let submeshes = submeshes
                .into_iter()
                .map(|(material_index, indices)| SubMesh {
                    material_index: material_index.get_u32(),
                    texture_id: texture_ids[material_index.get_u32() as usize],
                    indices,
                })
                .collect();

            Mesh {
                name: mesh_obj.name().map(Into::into),
                vertices,
                submeshes,
            }
        };

        models
            .entry(root_model.object_id())
            .or_insert_with(|| (root_model.name().map(Into::into), Vec::new()))
            .1
            .push(mesh);
    }

    scene.models = models
        .into_iter()
        .map(|(_, (name, meshes))| Model { name, meshes })
        .collect();

    scene.textures = textures;

    Ok(scene)
}
