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
        let asset_path = load_context.path();
        let fbx_dir = asset_path
            .path()
            .parent()
            .unwrap_or_else(|| std::path::Path::new(""));

        // Construct relative path for Bevy's asset system
        // Preserves .fbm folder structure if present (e.g., "model.fbm/texture.jpg")
        let relative_path = if !texture.filename.is_empty() {
            let filename = texture.filename.as_ref();

            // Check if filename contains an absolute path (Unix or Windows style)
            // Windows paths can have either : followed by / or \
            let is_absolute = filename.starts_with('/')
                || (filename.len() >= 3 && filename.chars().nth(1) == Some(':'));

            if is_absolute {
                // Extract relative path from absolute path
                // Look for .fbm folder (FBX's standard embedded texture directory)
                if let Some(fbm_pos) = filename.rfind(".fbm/").or_else(|| filename.rfind(".fbm\\")) {
                    // Find the start of the .fbm folder name
                    let before_fbm = &filename[..fbm_pos];
                    let folder_start = before_fbm.rfind(&['/', '\\'][..])
                        .map(|p| p + 1)
                        .unwrap_or(0);
                    // Extract from folder name onwards: "model.fbm/texture.jpg"
                    &filename[folder_start..]
                } else {
                    // No .fbm folder, extract just the filename
                    let last_slash = filename.rfind(&['/', '\\'][..]);
                    if let Some(pos) = last_slash {
                        &filename[pos + 1..]
                    } else {
                        filename
                    }
                }
            } else {
                // Use as-is (already relative)
                filename
            }
        } else if !texture.absolute_filename.is_empty() {
            let abs_path = texture.absolute_filename.as_ref();
            // Extract relative path from absolute_filename
            // Look for .fbm folder
            if let Some(fbm_pos) = abs_path.rfind(".fbm/").or_else(|| abs_path.rfind(".fbm\\")) {
                let before_fbm = &abs_path[..fbm_pos];
                let folder_start = before_fbm.rfind(&['/', '\\'][..])
                    .map(|p| p + 1)
                    .unwrap_or(0);
                &abs_path[folder_start..]
            } else {
                // No .fbm folder, extract just the filename
                let last_slash = abs_path.rfind(&['/', '\\'][..]);
                if let Some(pos) = last_slash {
                    &abs_path[pos + 1..]
                } else {
                    abs_path
                }
            }
        } else {
            ""
        };

        if !relative_path.is_empty() {
            let texture_path = fbx_dir
                .join(relative_path)
                .to_string_lossy()
                .to_string();

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
