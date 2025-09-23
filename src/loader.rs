//! FBX loader implementation for Bevy.

use crate::error::FbxError;
use crate::material::process_materials;
use crate::mesh::process_meshes;
use crate::node::{process_nodes, process_skins};
use crate::scene::build_scene;
use crate::types::{Fbx, FbxAxisSystem, FbxMeta, Handedness};
use bevy::asset::{io::Reader, AssetLoader, LoadContext, RenderAssetUsages};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Settings for FBX file loading.
///
/// These settings allow customizing which parts of the FBX file are loaded
/// and how they are processed.
#[derive(Serialize, Deserialize)]
pub struct FbxLoaderSettings {
    /// How meshes should be loaded and used
    pub load_meshes: RenderAssetUsages,
    /// How materials should be loaded and used
    pub load_materials: RenderAssetUsages,
    /// Whether to load cameras from the FBX file
    pub load_cameras: bool,
    /// Whether to load lights from the FBX file
    pub load_lights: bool,
    /// Whether to include raw source data in the loaded asset
    pub include_source: bool,
    /// Whether to convert coordinate systems (e.g., Y-up to Z-up)
    pub convert_coordinates: bool,
}

impl Default for FbxLoaderSettings {
    fn default() -> Self {
        Self {
            load_meshes: RenderAssetUsages::default(),
            load_materials: RenderAssetUsages::default(),
            load_cameras: true,
            load_lights: true,
            include_source: false,
            convert_coordinates: false,
        }
    }
}

/// Loader implementation for FBX files.
///
/// This loader handles reading FBX files and converting them into Bevy assets,
/// including meshes, materials, animations, and scene hierarchies.
#[derive(Default)]
pub struct FbxLoader;

impl AssetLoader for FbxLoader {
    type Asset = Fbx;
    type Settings = FbxLoaderSettings;
    type Error = FbxError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Fbx, FbxError> {
        // Read file
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        // Basic validation
        if bytes.is_empty() {
            return Err(FbxError::InvalidData("Empty FBX file".to_string()));
        }
        if bytes.len() < 32 {
            return Err(FbxError::InvalidData("FBX file too small".to_string()));
        }

        // Parse with ufbx
        let root = ufbx::load_memory(
            &bytes,
            ufbx::LoadOpts {
                target_unit_meters: 1.0,
                target_axes: ufbx::CoordinateAxes::right_handed_y_up(),
                ..Default::default()
            },
        )
        .map_err(|e| FbxError::UfbxError(format!("{:?}", e)))?;
        let scene: &ufbx::Scene = &*root;

        // Process meshes
        let (meshes, named_meshes, mesh_transforms, mesh_material_info) =
            process_meshes(scene, settings, load_context)?;

        // Process materials and textures
        let (materials, named_materials) = if !settings.load_materials.is_empty() {
            process_materials(scene, settings, load_context)?
        } else {
            (Vec::new(), HashMap::new())
        };

        // Process nodes and hierarchy
        let (nodes, named_nodes, node_map) = process_nodes(scene, &meshes, load_context)?;

        // Process skins
        let (skins, named_skins) = process_skins(scene, &node_map, load_context)?;

        // Build scene
        let scene_handle = build_scene(
            scene,
            &meshes,
            &materials,
            &named_materials,
            &mesh_transforms,
            &mesh_material_info,
            settings,
            load_context,
        )?;

        // Extract metadata
        let metadata = FbxMeta::default();

        // Build final FBX asset
        Ok(Fbx {
            scenes: vec![scene_handle.clone()],
            named_scenes: HashMap::new(),
            meshes,
            named_meshes,
            materials,
            named_materials,
            nodes,
            named_nodes,
            skins,
            named_skins,
            default_scene: Some(scene_handle),
            axis_system: FbxAxisSystem {
                up: Vec3::Y,
                front: Vec3::Z,
                handedness: Handedness::Right,
            },
            unit_scale: 1.0,
            metadata,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["fbx"]
    }
}
