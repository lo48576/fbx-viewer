//! FBX v7400 support.

use std::collections::HashMap;

use failure::{format_err, Fallible, ResultExt};
use fbxcel_dom::v7400::{
    object::{self, model::TypedModelHandle, ObjectId, TypedObjectHandle},
    Document,
};
use log::debug;

use crate::data::{GeometryMesh, GeometryMeshIndex, Mesh, MeshIndex, Scene};

#[allow(dead_code)]
mod triangulator;

/// Loads the data from the document.
pub fn from_doc(doc: Box<Document>) -> Fallible<Scene> {
    Loader::new(&doc).load()
}

/// FBX data loader.
pub struct Loader<'a> {
    /// Document.
    doc: &'a Document,
    /// Scene.
    scene: Scene,
    /// Mesh indices.
    mesh_indices: HashMap<ObjectId, MeshIndex>,
    /// Geometry mesh indices.
    geometry_mesh_indices: HashMap<ObjectId, GeometryMeshIndex>,
}

impl<'a> Loader<'a> {
    /// Creates a new `Loader`.
    fn new(doc: &'a Document) -> Self {
        Self {
            doc,
            scene: Default::default(),
            mesh_indices: Default::default(),
            geometry_mesh_indices: Default::default(),
        }
    }

    /// Loads the document.
    fn load(mut self) -> Fallible<Scene> {
        for obj in self.doc.objects() {
            if let TypedObjectHandle::Model(TypedModelHandle::Mesh(mesh)) = obj.get_typed() {
                self.load_mesh(mesh)?;
            }
        }

        Ok(self.scene)
    }

    /// Loads the mesh.
    fn load_mesh(&mut self, mesh_obj: object::model::MeshHandle<'a>) -> Fallible<MeshIndex> {
        if let Some(index) = self.mesh_indices.get(&mesh_obj.object_id()) {
            return Ok(*index);
        }

        debug!("Loading mesh: {:?}", mesh_obj);

        let geometry_obj = mesh_obj
            .geometry()
            .with_context(|e| format_err!("Failed to get geometry: {}", e))?;

        let geometry_index = self
            .load_geometry_mesh(geometry_obj)
            .with_context(|e| format_err!("Failed to load geometry mesh: {}", e))?;

        let mesh = Mesh {
            geometry_mesh_index: geometry_index,
        };

        debug!("Successfully loaded mesh: {:?}", mesh_obj);

        Ok(self.scene.add_mesh(mesh))
    }

    /// Loads the geometry.
    fn load_geometry_mesh(
        &mut self,
        mesh_obj: object::geometry::MeshHandle<'a>,
    ) -> Fallible<GeometryMeshIndex> {
        if let Some(index) = self.geometry_mesh_indices.get(&mesh_obj.object_id()) {
            return Ok(*index);
        }

        debug!("Loading geometry mesh: {:?}", mesh_obj);

        let mesh = GeometryMesh {};

        debug!("Successfully loaded geometry mesh: {:?}", mesh_obj);

        Ok(self.scene.add_geometry_mesh(mesh))
    }
}
