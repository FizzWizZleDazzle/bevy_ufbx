//! Tests for FBX types.

use bevy::prelude::*;
use bevy_ufbx::types::*;
use std::collections::HashMap;

#[test]
fn test_handedness_equality() {
    assert_eq!(Handedness::Right, Handedness::Right);
    assert_eq!(Handedness::Left, Handedness::Left);
    assert_ne!(Handedness::Right, Handedness::Left);
}

#[test]
fn test_fbx_axis_system_creation() {
    let axis_system = FbxAxisSystem {
        up: Vec3::Y,
        front: Vec3::Z,
        handedness: Handedness::Right,
    };

    assert_eq!(axis_system.up, Vec3::Y);
    assert_eq!(axis_system.front, Vec3::Z);
    assert_eq!(axis_system.handedness, Handedness::Right);
}

#[test]
fn test_fbx_meta_default() {
    let meta = FbxMeta::default();

    assert!(meta.creator.is_none());
    assert!(meta.creation_time.is_none());
    assert!(meta.original_application.is_none());
}

#[test]
fn test_fbx_wrap_mode_equality() {
    assert_eq!(FbxWrapMode::Repeat, FbxWrapMode::Repeat);
    assert_eq!(FbxWrapMode::Clamp, FbxWrapMode::Clamp);
    assert_ne!(FbxWrapMode::Repeat, FbxWrapMode::Clamp);
}

#[test]
fn test_fbx_texture_type_hash() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(FbxTextureType::BaseColor);
    set.insert(FbxTextureType::Normal);
    set.insert(FbxTextureType::Metallic);

    assert!(set.contains(&FbxTextureType::BaseColor));
    assert!(set.contains(&FbxTextureType::Normal));
    assert!(set.contains(&FbxTextureType::Metallic));
    assert!(!set.contains(&FbxTextureType::Roughness));
}

#[test]
fn test_fbx_light_type_equality() {
    assert_eq!(FbxLightType::Directional, FbxLightType::Directional);
    assert_eq!(FbxLightType::Point, FbxLightType::Point);
    assert_eq!(FbxLightType::Spot, FbxLightType::Spot);
    assert_ne!(FbxLightType::Point, FbxLightType::Spot);
}

#[test]
fn test_fbx_projection_mode_equality() {
    assert_eq!(
        FbxProjectionMode::Perspective,
        FbxProjectionMode::Perspective
    );
    assert_eq!(
        FbxProjectionMode::Orthographic,
        FbxProjectionMode::Orthographic
    );
    assert_ne!(
        FbxProjectionMode::Perspective,
        FbxProjectionMode::Orthographic
    );
}

#[test]
fn test_fbx_interpolation_equality() {
    assert_eq!(FbxInterpolation::Constant, FbxInterpolation::Constant);
    assert_eq!(FbxInterpolation::Linear, FbxInterpolation::Linear);
    assert_eq!(FbxInterpolation::Cubic, FbxInterpolation::Cubic);
    assert_ne!(FbxInterpolation::Linear, FbxInterpolation::Cubic);
}

#[test]
fn test_fbx_material_creation() {
    let material = FbxMaterial {
        name: "TestMaterial".to_string(),
        base_color: Color::WHITE,
        metallic: 0.5,
        roughness: 0.7,
        emission: Color::BLACK,
        normal_scale: 1.0,
        alpha: 1.0,
        alpha_cutoff: 0.5,
        double_sided: false,
        textures: HashMap::new(),
    };

    assert_eq!(material.name, "TestMaterial");
    assert_eq!(material.base_color, Color::WHITE);
    assert_eq!(material.metallic, 0.5);
    assert_eq!(material.roughness, 0.7);
    assert!(!material.double_sided);
}

#[test]
fn test_fbx_light_creation() {
    let light = FbxLight {
        name: "TestLight".to_string(),
        light_type: FbxLightType::Point,
        color: Color::WHITE,
        intensity: 100.0,
        cast_shadows: true,
        inner_angle: None,
        outer_angle: None,
    };

    assert_eq!(light.name, "TestLight");
    assert_eq!(light.light_type, FbxLightType::Point);
    assert!(light.cast_shadows);
    assert!(light.inner_angle.is_none());
}

#[test]
fn test_fbx_camera_creation() {
    let camera = FbxCamera {
        name: "TestCamera".to_string(),
        projection_mode: FbxProjectionMode::Perspective,
        field_of_view_deg: 60.0,
        aspect_ratio: 1.777,
        near_plane: 0.1,
        far_plane: 1000.0,
        focal_length_mm: 35.0,
    };

    assert_eq!(camera.name, "TestCamera");
    assert_eq!(camera.projection_mode, FbxProjectionMode::Perspective);
    assert_eq!(camera.field_of_view_deg, 60.0);
    assert!((camera.aspect_ratio - 1.777).abs() < 0.001);
}
