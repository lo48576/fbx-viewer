//! FBX v7400 support.

use std::collections::{BTreeMap, HashMap};

use failure::{format_err, Fallible};
use fbxcel_dom::v7400::{
    data::mesh::layer::TypedLayerElementHandle,
    object::{
        model::{self, TypedModelHandle},
        ObjectId, TypedObjectHandle,
    },
    Document,
};
use log::debug;

use crate::data::{mesh::Vertex, Mesh, Model, Scene};

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
            let geometry = mesh_obj.geometry()?;
            // Mesh.
            let control_points = geometry.control_points()?;
            let polygon_vertex_indices = geometry.polygon_vertex_indices()?;
            let triangle_pvi_indices =
                polygon_vertex_indices.triangulate_each(&control_points, triangulator)?;
            let positions = triangle_pvi_indices
                .iter_control_point_indices()
                .map(|cpi| {
                    let cpi =
                        cpi.ok_or_else(|| format_err!("Failed to get control point index"))?;
                    control_points
                        .get_cp_f32(cpi)
                        .ok_or_else(|| format_err!("Failed to get control point"))
                })
                .collect::<Result<Vec<_>, _>>()?;
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
                    .collect::<Result<Vec<_>, _>>()?
            };
            let material_indices = {
                let materials = layer
                    .layer_element_entries()
                    .filter_map(|entry| match entry.typed_layer_element() {
                        Ok(TypedLayerElementHandle::Material(handle)) => Some(handle),
                        _ => None,
                    })
                    .next()
                    .ok_or_else(|| format_err!("Failed to get materials"))?
                    .materials()?;
                triangle_pvi_indices
                    .triangle_vertex_indices()
                    .map(|tri_vi| {
                        materials.get_material_index_by_tri_vi(&triangle_pvi_indices, tri_vi)
                    })
                    .collect::<Result<Vec<_>, _>>()?
            };
            let vertices = positions
                .into_iter()
                .zip(normals)
                .zip(material_indices.iter().map(|i| i.get_u32()))
                .map(|((position, normal), material)| Vertex {
                    position,
                    normal,
                    material,
                })
                .collect();
            let mut indices = BTreeMap::new();
            assert_eq!(triangle_pvi_indices.len(), material_indices.len());
            for (pvii, &material_i) in material_indices.iter().enumerate() {
                indices
                    .entry(material_i)
                    .or_insert_with(Vec::new)
                    .push(pvii as u32);
            }
            let indices = indices.into_iter().map(|(_, v)| v).collect();
            Mesh {
                name: mesh_obj.name().map(Into::into),
                vertices,
                indices,
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

    Ok(scene)
}
