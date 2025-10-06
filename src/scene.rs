//! Scene building functionality for FBX files.

use crate::error::FbxError;
use crate::label::FbxAssetLabel;
use crate::loader::FbxLoaderSettings;
use crate::utils::convert_matrix;
use bevy::asset::{Handle, LoadContext};
use bevy::pbr::StandardMaterial;
use bevy::prelude::*;
use bevy::scene::Scene;
use std::collections::HashMap;

/// Build the final scene with all entities.
pub fn build_scene(
    scene: &ufbx::Scene,
    meshes: &[Handle<Mesh>],
    materials: &[Handle<StandardMaterial>],
    named_materials: &HashMap<Box<str>, Handle<StandardMaterial>>,
    mesh_transforms: &[ufbx::Matrix],
    mesh_material_info: &[Vec<String>],
    settings: &FbxLoaderSettings,
    load_context: &mut LoadContext,
) -> Result<Handle<Scene>, FbxError> {
    let mut world = World::new();

    // Create default material if needed
    let default_material = materials.get(0).cloned().unwrap_or_else(|| {
        load_context.add_labeled_asset(
            FbxAssetLabel::DefaultMaterial.to_string(),
            StandardMaterial::default(),
        )
    });

    // Spawn meshes
    for (mesh_index, ((mesh_handle, transform_matrix), mat_names)) in meshes
        .iter()
        .zip(mesh_transforms.iter())
        .zip(mesh_material_info.iter())
        .enumerate()
    {
        let transform = Transform::from_matrix(convert_matrix(transform_matrix));

        // Find material
        let material = mat_names
            .iter()
            .find_map(|name| named_materials.get(name as &str))
            .cloned()
            .or_else(|| {
                materials
                    .get(mesh_index.min(materials.len().saturating_sub(1)))
                    .cloned()
            })
            .unwrap_or_else(|| default_material.clone());

        world.spawn((
            Mesh3d(mesh_handle.clone()),
            MeshMaterial3d(material),
            transform,
            GlobalTransform::default(),
            Visibility::default(),
        ));
    }

    // Spawn lights
    if settings.load_lights {
        spawn_lights(scene, &mut world);
    }

    let scene_handle =
        load_context.add_labeled_asset(FbxAssetLabel::Scene(0).to_string(), Scene::new(world));

    Ok(scene_handle)
}

/// Spawn lights in the scene.
pub fn spawn_lights(scene: &ufbx::Scene, world: &mut World) {
    for light in scene.lights.as_ref().iter() {
        if let Some(light_node) = scene.nodes.as_ref().iter().find(|n| {
            n.light.is_some()
                && n.light.as_ref().unwrap().element.element_id == light.element.element_id
        }) {
            let transform = Transform::from_matrix(convert_matrix(&light_node.node_to_world));

            match light.type_ {
                ufbx::LightType::Directional => {
                    world.spawn((
                        DirectionalLight {
                            color: Color::srgb(
                                light.color.x as f32,
                                light.color.y as f32,
                                light.color.z as f32,
                            ),
                            illuminance: light.intensity as f32 * 10000.0,
                            shadows_enabled: light.cast_shadows,
                            ..Default::default()
                        },
                        transform,
                        GlobalTransform::default(),
                        Visibility::default(),
                    ));
                }
                ufbx::LightType::Point => {
                    world.spawn((
                        PointLight {
                            color: Color::srgb(
                                light.color.x as f32,
                                light.color.y as f32,
                                light.color.z as f32,
                            ),
                            intensity: light.intensity as f32 * 1000.0,
                            shadows_enabled: light.cast_shadows,
                            ..Default::default()
                        },
                        transform,
                        GlobalTransform::default(),
                        Visibility::default(),
                    ));
                }
                ufbx::LightType::Spot => {
                    world.spawn((
                        SpotLight {
                            color: Color::srgb(
                                light.color.x as f32,
                                light.color.y as f32,
                                light.color.z as f32,
                            ),
                            intensity: light.intensity as f32 * 1000.0,
                            shadows_enabled: light.cast_shadows,
                            inner_angle: light.inner_angle as f32,
                            outer_angle: light.outer_angle as f32,
                            ..Default::default()
                        },
                        transform,
                        GlobalTransform::default(),
                        Visibility::default(),
                    ));
                }
                _ => {}
            }
        }
    }
}
