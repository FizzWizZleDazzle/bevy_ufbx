//! Utility functions for converting between ufbx and Bevy types.

use bevy::math::{Affine2, Mat4};
use bevy::prelude::*;

/// Convert ufbx texture UV transform to Bevy Affine2.
pub fn convert_texture_uv_transform(texture: &ufbx::Texture) -> Affine2 {
    let translation = Vec2::new(
        texture.uv_transform.translation.x as f32,
        texture.uv_transform.translation.y as f32,
    );
    let scale = Vec2::new(
        texture.uv_transform.scale.x as f32,
        texture.uv_transform.scale.y as f32,
    );
    let rotation_z = texture.uv_transform.rotation.z as f32;
    Affine2::from_scale_angle_translation(scale, rotation_z, translation)
}

/// Convert ufbx matrix to Bevy Mat4.
pub fn convert_matrix(m: &ufbx::Matrix) -> Mat4 {
    Mat4::from_cols_array(&[
        m.m00 as f32,
        m.m10 as f32,
        m.m20 as f32,
        0.0,
        m.m01 as f32,
        m.m11 as f32,
        m.m21 as f32,
        0.0,
        m.m02 as f32,
        m.m12 as f32,
        m.m22 as f32,
        0.0,
        m.m03 as f32,
        m.m13 as f32,
        m.m23 as f32,
        1.0,
    ])
}

/// Convert ufbx transform to Bevy Transform.
pub fn convert_transform(t: &ufbx::Transform) -> Transform {
    Transform {
        translation: Vec3::new(
            t.translation.x as f32,
            t.translation.y as f32,
            t.translation.z as f32,
        ),
        rotation: Quat::from_xyzw(
            t.rotation.x as f32,
            t.rotation.y as f32,
            t.rotation.z as f32,
            t.rotation.w as f32,
        ),
        scale: Vec3::new(t.scale.x as f32, t.scale.y as f32, t.scale.z as f32),
    }
}
