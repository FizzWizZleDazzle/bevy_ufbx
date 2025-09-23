//! Example showing how to load and render an FBX model.

use bevy::prelude::*;
use bevy_ufbx::FbxPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FbxPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_model)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -45f32.to_radians(),
            45f32.to_radians(),
            0.0,
        )),
    ));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 200.0,
        affects_lightmapped_meshes: false,
    });

    // Spawn the default scene from the FBX
    commands.spawn((
        SceneRoot(asset_server.load("cube.fbx#Scene0")),
        ModelRotator,
    ));

    println!("Loading FBX model: cube.fbx");
}

#[derive(Component)]
struct ModelRotator;

fn rotate_model(time: Res<Time>, mut query: Query<&mut Transform, With<ModelRotator>>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_secs() * 0.5);
    }
}
