//! Node and hierarchy processing for FBX files.

use crate::error::FbxError;
use crate::label::FbxAssetLabel;
use crate::types::{FbxNode, FbxSkin};
use crate::utils::{convert_matrix, convert_transform};
use bevy::asset::{Handle, LoadContext};
use bevy::prelude::*;
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use std::collections::HashMap;

/// Process nodes and build hierarchy.
pub fn process_nodes(
    scene: &ufbx::Scene,
    meshes: &[Handle<Mesh>],
    load_context: &mut LoadContext,
) -> Result<
    (
        Vec<Handle<FbxNode>>,
        HashMap<Box<str>, Handle<FbxNode>>,
        HashMap<u32, Handle<FbxNode>>,
    ),
    FbxError,
> {
    let mut nodes = Vec::new();
    let mut named_nodes = HashMap::new();
    let mut node_map = HashMap::new();

    // First pass: create nodes
    for (index, ufbx_node) in scene.nodes.as_ref().iter().enumerate() {
        let name = if ufbx_node.element.name.is_empty() {
            format!("Node_{}", index)
        } else {
            ufbx_node.element.name.to_string()
        };

        let mesh_handle = if ufbx_node.mesh.is_some() && index < meshes.len() {
            Some(meshes[index].clone())
        } else {
            None
        };

        let fbx_node = FbxNode {
            index,
            name: name.clone(),
            children: Vec::new(),
            mesh: mesh_handle,
            skin: None,
            transform: convert_transform(&ufbx_node.local_transform),
            visible: ufbx_node.visible,
        };

        let handle =
            load_context.add_labeled_asset(FbxAssetLabel::Node(index).to_string(), fbx_node);

        node_map.insert(ufbx_node.element.element_id, handle.clone());
        nodes.push(handle.clone());

        if !ufbx_node.element.name.is_empty() {
            named_nodes.insert(Box::from(ufbx_node.element.name.as_ref()), handle);
        }
    }

    // Note: Hierarchy building would require mutable access to nodes
    // which is not possible with the current asset system.
    // Parent-child relationships would need to be established at runtime.

    Ok((nodes, named_nodes, node_map))
}

/// Process skins for skeletal animation.
pub fn process_skins(
    scene: &ufbx::Scene,
    node_map: &HashMap<u32, Handle<FbxNode>>,
    load_context: &mut LoadContext,
) -> Result<(Vec<Handle<FbxSkin>>, HashMap<Box<str>, Handle<FbxSkin>>), FbxError> {
    let mut skins = Vec::new();
    let mut named_skins = HashMap::new();

    for (skin_index, node) in scene.nodes.as_ref().iter().enumerate() {
        let Some(mesh_ref) = &node.mesh else {
            continue;
        };
        let mesh = mesh_ref.as_ref();

        if mesh.skin_deformers.is_empty() {
            continue;
        }

        let skin_deformer = &mesh.skin_deformers[0];
        let mut inverse_bind_matrices = Vec::new();
        let mut joint_handles = Vec::new();

        for cluster in &skin_deformer.clusters {
            let bind_matrix = convert_matrix(&cluster.bind_to_world);
            inverse_bind_matrices.push(bind_matrix.inverse());

            if let Some(bone_node) = cluster.bone_node.as_ref() {
                if let Some(joint_handle) = node_map.get(&bone_node.element.element_id) {
                    joint_handles.push(joint_handle.clone());
                }
            }
        }

        if !inverse_bind_matrices.is_empty() {
            let inverse_bindposes_handle = load_context.add_labeled_asset(
                format!("Skin_{}_InverseBindposes", skin_index),
                SkinnedMeshInverseBindposes::from(inverse_bind_matrices),
            );

            let skin_name = if node.element.name.is_empty() {
                format!("Skin_{}", skin_index)
            } else {
                format!("{}_Skin", node.element.name)
            };

            let fbx_skin = FbxSkin {
                index: skin_index,
                name: skin_name.clone(),
                joints: joint_handles,
                inverse_bind_matrices: inverse_bindposes_handle,
            };

            let handle = load_context
                .add_labeled_asset(FbxAssetLabel::Skin(skin_index).to_string(), fbx_skin);

            skins.push(handle.clone());
            if !skin_name.starts_with("Skin_") {
                named_skins.insert(Box::from(skin_name.as_str()), handle);
            }
        }
    }

    Ok((skins, named_skins))
}
