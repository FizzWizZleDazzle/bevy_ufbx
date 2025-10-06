//! Mesh processing functionality for FBX files.

use crate::error::FbxError;
use crate::label::FbxAssetLabel;
use crate::loader::FbxLoaderSettings;
use bevy::asset::{Handle, LoadContext};
use bevy::prelude::*;
use bevy::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use std::collections::HashMap;

/// Process all meshes from the FBX scene.
pub fn process_meshes(
    scene: &ufbx::Scene,
    settings: &FbxLoaderSettings,
    load_context: &mut LoadContext,
) -> Result<
    (
        Vec<Handle<Mesh>>,
        HashMap<Box<str>, Handle<Mesh>>,
        Vec<ufbx::Matrix>,
        Vec<Vec<String>>,
    ),
    FbxError,
> {
    let mut meshes = Vec::new();
    let mut named_meshes = HashMap::new();
    let mut transforms = Vec::new();
    let mut mesh_material_info = Vec::new();

    for (index, node) in scene.nodes.as_ref().iter().enumerate() {
        let Some(mesh_ref) = node.mesh.as_ref() else {
            continue;
        };
        let mesh = mesh_ref.as_ref();

        if mesh.num_vertices == 0 || mesh.faces.as_ref().is_empty() {
            continue;
        }

        // Group faces by material
        let material_groups = group_faces_by_material(mesh);

        // Create mesh for each material group
        for (material_idx, indices) in material_groups.iter() {
            let mesh_handle = create_mesh_from_group(
                mesh,
                indices,
                index,
                *material_idx,
                settings,
                load_context,
            )?;

            if *material_idx == 0 && !node.element.name.is_empty() {
                named_meshes.insert(Box::from(node.element.name.as_ref()), mesh_handle.clone());
            }

            meshes.push(mesh_handle);
            transforms.push(node.geometry_to_world);

            let material_name = if *material_idx < mesh.materials.len() {
                mesh.materials[*material_idx].element.name.to_string()
            } else {
                "default".to_string()
            };
            mesh_material_info.push(vec![material_name]);
        }
    }

    Ok((meshes, named_meshes, transforms, mesh_material_info))
}

/// Group mesh faces by material index.
pub fn group_faces_by_material(mesh: &ufbx::Mesh) -> HashMap<usize, Vec<u32>> {
    let mut material_groups: HashMap<usize, Vec<u32>> = HashMap::new();
    let mut scratch = Vec::new();

    if mesh.materials.is_empty() {
        // No materials - create single group
        let mut all_indices = Vec::new();
        for &face in mesh.faces.as_ref().iter() {
            scratch.clear();
            ufbx::triangulate_face_vec(&mut scratch, mesh, face);
            for idx in &scratch {
                if (*idx as usize) < mesh.vertex_indices.len() {
                    all_indices.push(mesh.vertex_indices[*idx as usize]);
                }
            }
        }
        material_groups.insert(0, all_indices);
    } else {
        // Group by material
        for (face_idx, &face) in mesh.faces.as_ref().iter().enumerate() {
            let material_idx =
                if !mesh.face_material.is_empty() && face_idx < mesh.face_material.len() {
                    mesh.face_material[face_idx] as usize
                } else {
                    0
                };

            scratch.clear();
            ufbx::triangulate_face_vec(&mut scratch, mesh, face);

            let indices = material_groups.entry(material_idx).or_insert_with(Vec::new);
            for idx in &scratch {
                if (*idx as usize) < mesh.vertex_indices.len() {
                    indices.push(mesh.vertex_indices[*idx as usize]);
                }
            }
        }
    }

    material_groups
}

/// Create a Bevy mesh from a material group.
pub fn create_mesh_from_group(
    ufbx_mesh: &ufbx::Mesh,
    indices: &[u32],
    mesh_index: usize,
    material_index: usize,
    settings: &FbxLoaderSettings,
    load_context: &mut LoadContext,
) -> Result<Handle<Mesh>, FbxError> {
    let label = FbxAssetLabel::Mesh(mesh_index * 1000 + material_index).to_string();

    let handle = load_context.labeled_asset_scope(label, |_| {
        let mut bevy_mesh = Mesh::new(PrimitiveTopology::TriangleList, settings.load_meshes);

        // Positions
        let positions: Vec<[f32; 3]> = ufbx_mesh
            .vertex_position
            .values
            .as_ref()
            .iter()
            .map(|v| [v.x as f32, v.y as f32, v.z as f32])
            .collect();
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);

        // Normals
        if ufbx_mesh.vertex_normal.exists {
            let normals: Vec<[f32; 3]> = (0..ufbx_mesh.num_vertices)
                .map(|i| {
                    let n = ufbx_mesh.vertex_normal[i];
                    [n.x as f32, n.y as f32, n.z as f32]
                })
                .collect();
            bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        }

        // UVs
        if ufbx_mesh.vertex_uv.exists {
            let uvs: Vec<[f32; 2]> = (0..ufbx_mesh.num_vertices)
                .map(|i| {
                    let uv = ufbx_mesh.vertex_uv[i];
                    [uv.x as f32, uv.y as f32]
                })
                .collect();
            bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        }

        // Skinning
        if !ufbx_mesh.skin_deformers.is_empty() {
            process_skinning_data(ufbx_mesh, &mut bevy_mesh);
        }

        // Indices
        bevy_mesh.insert_indices(Indices::U32(indices.to_vec()));

        Ok::<_, FbxError>(bevy_mesh)
    })?;

    Ok(handle)
}

/// Process skinning data for a mesh.
pub fn process_skinning_data(ufbx_mesh: &ufbx::Mesh, bevy_mesh: &mut Mesh) {
    let skin_deformer = &ufbx_mesh.skin_deformers[0];
    let mut joint_indices = vec![[0u16; 4]; ufbx_mesh.num_vertices];
    let mut joint_weights = vec![[0.0f32; 4]; ufbx_mesh.num_vertices];

    for vertex_index in 0..ufbx_mesh.num_vertices {
        let mut weight_count = 0;
        let mut total_weight = 0.0f32;

        for (cluster_index, cluster) in skin_deformer.clusters.iter().enumerate() {
            if weight_count >= 4 {
                break;
            }

            for (i, &vert_idx) in cluster.vertices.iter().enumerate() {
                if vert_idx as usize == vertex_index && i < cluster.weights.len() {
                    let weight = cluster.weights[i] as f32;
                    if weight > 0.0 {
                        joint_indices[vertex_index][weight_count] = cluster_index as u16;
                        joint_weights[vertex_index][weight_count] = weight;
                        total_weight += weight;
                        weight_count += 1;
                        break;
                    }
                }
            }
        }

        // Normalize weights
        if total_weight > 0.0 {
            for i in 0..weight_count {
                joint_weights[vertex_index][i] /= total_weight;
            }
        }
    }

    bevy_mesh.insert_attribute(
        Mesh::ATTRIBUTE_JOINT_INDEX,
        VertexAttributeValues::Uint16x4(joint_indices),
    );
    bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, joint_weights);
}
