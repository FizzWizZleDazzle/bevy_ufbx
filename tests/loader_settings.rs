//! Tests for FBX loader settings.

use bevy::asset::RenderAssetUsages;
use bevy_ufbx::FbxLoaderSettings;

#[test]
fn test_loader_settings_default() {
    let settings = FbxLoaderSettings::default();

    assert_eq!(settings.load_meshes, RenderAssetUsages::default());
    assert_eq!(settings.load_materials, RenderAssetUsages::default());
    assert!(settings.load_cameras);
    assert!(settings.load_lights);
    assert!(!settings.include_source);
    assert!(!settings.convert_coordinates);
}

#[test]
fn test_loader_settings_custom() {
    let settings = FbxLoaderSettings {
        load_meshes: RenderAssetUsages::RENDER_WORLD,
        load_materials: RenderAssetUsages::MAIN_WORLD,
        load_cameras: false,
        load_lights: false,
        include_source: true,
        convert_coordinates: true,
    };

    assert_eq!(settings.load_meshes, RenderAssetUsages::RENDER_WORLD);
    assert_eq!(settings.load_materials, RenderAssetUsages::MAIN_WORLD);
    assert!(!settings.load_cameras);
    assert!(!settings.load_lights);
    assert!(settings.include_source);
    assert!(settings.convert_coordinates);
}

#[test]
fn test_loader_settings_serialization() {
    let original = FbxLoaderSettings {
        load_meshes: RenderAssetUsages::RENDER_WORLD,
        load_materials: RenderAssetUsages::MAIN_WORLD,
        load_cameras: false,
        load_lights: true,
        include_source: false,
        convert_coordinates: true,
    };

    // Serialize
    let serialized = serde_json::to_string(&original).expect("Failed to serialize");

    // Deserialize
    let deserialized: FbxLoaderSettings =
        serde_json::from_str(&serialized).expect("Failed to deserialize");

    // Check equality
    assert_eq!(deserialized.load_meshes, original.load_meshes);
    assert_eq!(deserialized.load_materials, original.load_materials);
    assert_eq!(deserialized.load_cameras, original.load_cameras);
    assert_eq!(deserialized.load_lights, original.load_lights);
    assert_eq!(deserialized.include_source, original.include_source);
    assert_eq!(
        deserialized.convert_coordinates,
        original.convert_coordinates
    );
}
