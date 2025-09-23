//! Tests for utility conversion functions.

use bevy::prelude::*;
use bevy_ufbx::utils::{convert_matrix, convert_transform};

#[test]
fn test_convert_matrix() {
    // Create a simple ufbx matrix
    let ufbx_matrix = ufbx::Matrix {
        m00: 1.0,
        m01: 0.0,
        m02: 0.0,
        m03: 10.0,
        m10: 0.0,
        m11: 1.0,
        m12: 0.0,
        m13: 20.0,
        m20: 0.0,
        m21: 0.0,
        m22: 1.0,
        m23: 30.0,
    };

    let mat4 = convert_matrix(&ufbx_matrix);

    // Check that translation is preserved
    assert_eq!(mat4.w_axis.x, 10.0);
    assert_eq!(mat4.w_axis.y, 20.0);
    assert_eq!(mat4.w_axis.z, 30.0);
    assert_eq!(mat4.w_axis.w, 1.0);

    // Check diagonal elements
    assert_eq!(mat4.x_axis.x, 1.0);
    assert_eq!(mat4.y_axis.y, 1.0);
    assert_eq!(mat4.z_axis.z, 1.0);
}

#[test]
fn test_convert_transform() {
    let ufbx_transform = ufbx::Transform {
        translation: ufbx::Vec3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        rotation: ufbx::Quat {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
        scale: ufbx::Vec3 {
            x: 2.0,
            y: 2.0,
            z: 2.0,
        },
    };

    let transform = convert_transform(&ufbx_transform);

    assert_eq!(transform.translation, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(transform.rotation, Quat::IDENTITY);
    assert_eq!(transform.scale, Vec3::new(2.0, 2.0, 2.0));
}

#[test]
fn test_convert_transform_with_rotation() {
    // Test with a 90-degree rotation around Y axis
    let half_sqrt2 = 0.7071067811865476;
    let ufbx_transform = ufbx::Transform {
        translation: ufbx::Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        rotation: ufbx::Quat {
            x: 0.0,
            y: half_sqrt2,
            z: 0.0,
            w: half_sqrt2,
        },
        scale: ufbx::Vec3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
    };

    let transform = convert_transform(&ufbx_transform);

    // Check rotation is preserved
    assert!((transform.rotation.x - 0.0).abs() < 0.001);
    assert!((transform.rotation.y - half_sqrt2 as f32).abs() < 0.001);
    assert!((transform.rotation.z - 0.0).abs() < 0.001);
    assert!((transform.rotation.w - half_sqrt2 as f32).abs() < 0.001);
}
