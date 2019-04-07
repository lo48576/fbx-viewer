//! FBX v7400 support.

use std::collections::HashMap;

use failure::{bail, format_err, Fallible, ResultExt};
use fbxcel_dom::v7400::{
    data::mesh::layer::TypedLayerElementHandle,
    object::{self, model::TypedModelHandle, ObjectId, TypedObjectHandle},
    Document,
};
use log::{debug, trace};

use crate::data::{
    GeometryMesh, GeometryMeshIndex, Material, MaterialIndex, Mesh, MeshIndex, Scene,
};

use self::triangulator::triangulator;

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
    /// Geometry mesh indices.
    geometry_mesh_indices: HashMap<ObjectId, GeometryMeshIndex>,
    /// Material indices.
    material_indices: HashMap<ObjectId, MaterialIndex>,
    /// Mesh indices.
    mesh_indices: HashMap<ObjectId, MeshIndex>,
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

    /// Loads the geometry.
    fn load_geometry_mesh(
        &mut self,
        mesh_obj: object::geometry::MeshHandle<'a>,
        num_materials: usize,
    ) -> Fallible<GeometryMeshIndex> {
        if let Some(index) = self.geometry_mesh_indices.get(&mesh_obj.object_id()) {
            return Ok(*index);
        }

        debug!("Loading geometry mesh: {:?}", mesh_obj);

        let control_points = mesh_obj
            .control_points()
            .with_context(|e| format_err!("Failed to get control points: {}", e))?;
        let polygon_vertices = mesh_obj
            .polygon_vertex_indices()
            .with_context(|e| format_err!("Failed to get polygon vertices: {}", e))?;
        let triangle_pvi_indices = polygon_vertices
            .triangulate_each(&control_points, triangulator)
            .with_context(|e| format_err!("Triangulation failed: {}", e))?;

        let positions = triangle_pvi_indices
            .iter_control_point_indices()
            .map(|cpi| {
                let cpi = cpi.ok_or_else(|| format_err!("Failed to get control point index"))?;
                control_points
                    .get_cp_f32(cpi)
                    .ok_or_else(|| format_err!("Failed to get control point"))
            })
            .collect::<Result<Vec<_>, _>>()
            .with_context(|e| format_err!("Failed to reconstruct position vertices: {}", e))?;
        trace!("Expanded positions len: {:?}", positions.len());

        let layer = mesh_obj
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
                .with_context(|e| format_err!("Failed to reconstruct normals vertices: {}", e))?
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

        let indices_per_material = {
            let mut indices_per_material = vec![Vec::new(); num_materials];
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
            for tri_vi in triangle_pvi_indices.triangle_vertex_indices() {
                let local_material_index = materials
                    .get_material_index_by_tri_vi(&triangle_pvi_indices, tri_vi)
                    .with_context(|e| {
                        format_err!("Failed to get mesh-local material index: {}", e)
                    })?
                    .get_u32();
                indices_per_material
                    .get_mut(local_material_index as usize)
                    .ok_or_else(|| {
                        format_err!(
                            "Mesh-local material index out of range: num_materials={:?}, got={:?}",
                            num_materials,
                            local_material_index
                        )
                    })?
                    .push(tri_vi.get() as u32);
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
    ) -> Fallible<MaterialIndex> {
        if let Some(index) = self.material_indices.get(&material_obj.object_id()) {
            return Ok(*index);
        }

        debug!("Loading material: {:?}", material_obj);

        let material = Material {};

        debug!("Successfully loaded material: {:?}", material_obj);

        Ok(self.scene.add_material(material))
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

        let materials = mesh_obj
            .materials()
            .map(|material_obj| self.load_material(material_obj))
            .collect::<Fallible<Vec<_>>>()
            .with_context(|e| format_err!("Failed to load materials for mesh: {}", e))?;

        let geometry_index = self
            .load_geometry_mesh(geometry_obj, materials.len())
            .with_context(|e| format_err!("Failed to load geometry mesh: {}", e))?;

        let mesh = Mesh {
            geometry_mesh_index: geometry_index,
            materials,
        };

        debug!("Successfully loaded mesh: {:?}", mesh_obj);

        Ok(self.scene.add_mesh(mesh))
    }
}
