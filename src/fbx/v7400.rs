//! FBX v7400 support.

use std::collections::HashMap;

use failure::Fallible;
use fbxcel_dom::v7400::{
    object::{self, model::TypedModelHandle, ObjectId, TypedObjectHandle},
    Document,
};
use log::debug;

use crate::data::{Mesh, MeshIndex, Scene};

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
}

impl<'a> Loader<'a> {
    /// Creates a new `Loader`.
    fn new(doc: &'a Document) -> Self {
        Self {
            doc,
            scene: Default::default(),
            mesh_indices: Default::default(),
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

        let mesh = Mesh {};

        debug!("Successfully loaded mesh: {:?}", mesh_obj);

        Ok(self.scene.add_mesh(mesh))
    }
}
