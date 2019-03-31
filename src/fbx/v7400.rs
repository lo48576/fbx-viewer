//! FBX v7400 support.

use std::collections::HashMap;

use failure::{format_err, Fallible};
use fbxcel_dom::v7400::{
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
            let vertices = triangle_pvi_indices
                .iter_control_point_indices()
                .map(|cpi| {
                    let cpi =
                        cpi.ok_or_else(|| format_err!("Failed to get control point index"))?;
                    control_points
                        .get_cp_f32(cpi)
                        .map(|position| Vertex { position })
                        .ok_or_else(|| format_err!("Failed to get control point"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let indices = (0..triangle_pvi_indices.len() as u32).collect::<Vec<_>>();
            Mesh {
                name: mesh_obj.name().map(Into::into),
                position: vertices,
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
