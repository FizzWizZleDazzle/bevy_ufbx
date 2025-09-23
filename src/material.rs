//! Material and texture processing for FBX files.

use crate::error::FbxError;
use crate::label::FbxAssetLabel;
use crate::loader::FbxLoaderSettings;
use crate::utils::convert_texture_uv_transform;
use bevy::asset::{Handle, LoadContext};
use bevy::pbr::StandardMaterial;
use bevy::prelude::*;
use bevy::render::alpha::AlphaMode;
use std::collections::HashMap;

/// Process all materials from the FBX scene.
pub fn process_materials(
    scene: &ufbx::Scene,
    _settings: &FbxLoaderSettings,
    load_context: &mut LoadContext,
) -> Result<
    (
        Vec<Handle<StandardMaterial>>,
        HashMap<Box<str>, Handle<StandardMaterial>>,
    ),
    FbxError,
> {
    let mut materials = Vec::new();
    let mut named_materials = HashMap::new();
    let texture_handles = process_textures(scene, load_context)?;

    for (index, ufbx_material) in scene.materials.as_ref().iter().enumerate() {
        if ufbx_material.element.element_id == 0 {
            continue;
        }

        let standard_material = create_standard_material(ufbx_material, &texture_handles)?;
        let handle = load_context.add_labeled_asset(
            FbxAssetLabel::Material(index).to_string(),
            standard_material,
        );

        if !ufbx_material.element.name.is_empty() {
            named_materials.insert(
                Box::from(ufbx_material.element.name.as_ref()),
                handle.clone(),
            );
        }

        materials.push(handle);
    }

    Ok((materials, named_materials))
}

/// Process textures from materials.
pub fn process_textures(
    scene: &ufbx::Scene,
    load_context: &mut LoadContext,
) -> Result<HashMap<u32, Handle<bevy::prelude::Image>>, FbxError> {
    let mut texture_handles = HashMap::new();

    for texture in scene.textures.as_ref().iter() {
        if !texture.filename.is_empty() {
            let texture_path = if !texture.absolute_filename.is_empty() {
                texture.absolute_filename.to_string()
            } else {
                let fbx_dir = load_context
                    .path()
                    .parent()
                    .unwrap_or_else(|| std::path::Path::new(""));
                fbx_dir
                    .join(texture.filename.as_ref())
                    .to_string_lossy()
                    .to_string()
            };

            let image_handle = load_context.load(texture_path);
            texture_handles.insert(texture.element.element_id, image_handle);
        }
    }

    Ok(texture_handles)
}

/// Create a StandardMaterial from ufbx material.
pub fn create_standard_material(
    ufbx_material: &ufbx::Material,
    texture_handles: &HashMap<u32, Handle<bevy::prelude::Image>>,
) -> Result<StandardMaterial, FbxError> {
    let mut material = StandardMaterial::default();

    // Base color
    if let Ok(diffuse) = std::panic::catch_unwind(|| ufbx_material.fbx.diffuse_color.value_vec4) {
        material.base_color = Color::srgb(diffuse.x as f32, diffuse.y as f32, diffuse.z as f32);
    } else if let Ok(pbr_base) =
        std::panic::catch_unwind(|| ufbx_material.pbr.base_color.value_vec4)
    {
        material.base_color = Color::srgb(pbr_base.x as f32, pbr_base.y as f32, pbr_base.z as f32);
    }

    // Metallic and roughness
    if let Ok(metallic) = std::panic::catch_unwind(|| ufbx_material.pbr.metalness.value_vec4) {
        material.metallic = metallic.x as f32;
    }
    if let Ok(roughness) = std::panic::catch_unwind(|| ufbx_material.pbr.roughness.value_vec4) {
        material.perceptual_roughness = roughness.x as f32;
    }

    // Emission
    if let Ok(emission) = std::panic::catch_unwind(|| ufbx_material.fbx.emission_color.value_vec4) {
        material.emissive =
            LinearRgba::rgb(emission.x as f32, emission.y as f32, emission.z as f32);
    }

    // Alpha
    if ufbx_material.pbr.opacity.value_vec4.x < 1.0 {
        let alpha = ufbx_material.pbr.opacity.value_vec4.x as f32;
        material.alpha_mode = if alpha < 0.98 {
            AlphaMode::Blend
        } else {
            AlphaMode::Opaque
        };
    }

    // Textures
    for texture_ref in &ufbx_material.textures {
        if let Some(image_handle) = texture_handles.get(&texture_ref.texture.element.element_id) {
            match texture_ref.material_prop.as_ref() {
                "DiffuseColor" | "BaseColor" => {
                    material.base_color_texture = Some(image_handle.clone());
                    material.uv_transform = convert_texture_uv_transform(&texture_ref.texture);
                }
                "NormalMap" => material.normal_map_texture = Some(image_handle.clone()),
                "Metallic" => material.metallic_roughness_texture = Some(image_handle.clone()),
                "Roughness" if material.metallic_roughness_texture.is_none() => {
                    material.metallic_roughness_texture = Some(image_handle.clone());
                }
                "EmissiveColor" => material.emissive_texture = Some(image_handle.clone()),
                "AmbientOcclusion" => material.occlusion_texture = Some(image_handle.clone()),
                _ => {}
            }
        }
    }

    Ok(material)
}
