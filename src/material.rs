//! Material and texture processing for FBX files.

use crate::error::FbxError;
use crate::label::FbxAssetLabel;
use crate::loader::FbxLoaderSettings;
use crate::utils::convert_texture_uv_transform;
use bevy::asset::{Handle, LoadContext};
use bevy::pbr::StandardMaterial;
use bevy::prelude::*;
use bevy::render::alpha::AlphaMode;
use bevy_image::{CompressedImageFormats, ImageSampler, ImageType};
use std::collections::HashMap;

/// Process all materials from the FBX scene.
pub fn process_materials(
    scene: &ufbx::Scene,
    settings: &FbxLoaderSettings,
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
    let texture_handles = process_textures(scene, settings, load_context)?;

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
///
/// Uses the following priority for each texture:
/// 1. Embedded textures (`texture.content` non-empty) — decoded via `Image::from_buffer`
/// 2. External textures via `texture.relative_filename` — already relative to the FBX file
/// 3. External textures via `.fbm` folder extraction from `texture.filename` / `texture.absolute_filename`
pub fn process_textures(
    scene: &ufbx::Scene,
    settings: &FbxLoaderSettings,
    load_context: &mut LoadContext,
) -> Result<HashMap<u32, Handle<Image>>, FbxError> {
    let mut texture_handles = HashMap::new();

    for (index, texture) in scene.textures.as_ref().iter().enumerate() {
        let asset_path = load_context.path().clone();
        let fbx_dir = asset_path
            .path()
            .parent()
            .unwrap_or_else(|| std::path::Path::new(""))
            .to_path_buf();

        // Priority 1: Embedded texture data
        if !texture.content.is_empty() {
            let ext = extract_extension(&texture.filename).unwrap_or("png");
            match Image::from_buffer(
                &texture.content,
                ImageType::Extension(ext),
                CompressedImageFormats::NONE,
                true, // is_srgb
                ImageSampler::Default,
                settings.load_materials,
            ) {
                Ok(image) => {
                    let handle = load_context.add_labeled_asset(
                        FbxAssetLabel::Texture(index).to_string(),
                        image,
                    );
                    texture_handles.insert(texture.element.element_id, handle);
                    continue;
                }
                Err(e) => {
                    eprintln!("bevy_ufbx: failed to load embedded texture {index}: {e}");
                    // Fall through to try external paths
                }
            }
        }

        // Priority 2: relative_filename from ufbx (relative to FBX file)
        if !texture.relative_filename.is_empty() {
            let rel = texture.relative_filename.as_ref();
            let is_absolute = rel.starts_with('/')
                || (rel.len() >= 3 && rel.chars().nth(1) == Some(':'));
            if !is_absolute {
                let texture_path = fbx_dir.join(rel).to_string_lossy().to_string();
                let image_handle = load_context.load(texture_path);
                texture_handles.insert(texture.element.element_id, image_handle);
                continue;
            }
        }

        // Priority 3: Extract relative path from filename / absolute_filename
        // Preserves .fbm folder structure if present (e.g., "model.fbm/texture.jpg")
        let relative_path = extract_relative_texture_path(texture);

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

/// Extract file extension from a texture filename string.
fn extract_extension(filename: &ufbx::String) -> Option<&str> {
    if filename.is_empty() {
        return None;
    }
    let s = filename.as_ref();
    s.rsplit('.')
        .next()
        .filter(|ext| !ext.contains('/') && !ext.contains('\\'))
}

/// Extract a relative texture path from a ufbx Texture's filename or absolute_filename.
///
/// Handles absolute paths by looking for `.fbm` folders (FBX's standard embedded texture
/// directory) and extracting from there, or falling back to just the filename.
fn extract_relative_texture_path<'a>(texture: &'a ufbx::Texture) -> &'a str {
    if !texture.filename.is_empty() {
        let filename = texture.filename.as_ref();

        let is_absolute = filename.starts_with('/')
            || (filename.len() >= 3 && filename.chars().nth(1) == Some(':'));

        if is_absolute {
            extract_from_absolute(filename)
        } else {
            filename
        }
    } else if !texture.absolute_filename.is_empty() {
        extract_from_absolute(texture.absolute_filename.as_ref())
    } else {
        ""
    }
}

/// Extract a relative path from an absolute path, preserving `.fbm` folder structure.
fn extract_from_absolute(path: &str) -> &str {
    if let Some(fbm_pos) = path.rfind(".fbm/").or_else(|| path.rfind(".fbm\\")) {
        let before_fbm = &path[..fbm_pos];
        let folder_start = before_fbm
            .rfind(&['/', '\\'][..])
            .map(|p| p + 1)
            .unwrap_or(0);
        &path[folder_start..]
    } else {
        let last_slash = path.rfind(&['/', '\\'][..]);
        if let Some(pos) = last_slash {
            &path[pos + 1..]
        } else {
            path
        }
    }
}

/// Create a StandardMaterial from ufbx material.
pub fn create_standard_material(
    ufbx_material: &ufbx::Material,
    texture_handles: &HashMap<u32, Handle<Image>>,
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
