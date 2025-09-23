//! Example demonstrating custom loader settings for FBX files.

use bevy::prelude::*;
use bevy_ufbx::{FbxPlugin, FbxLoaderSettings, Fbx};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FbxPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, handle_loaded_fbx)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)),
    ));

    // Load FBX with custom settings
    let fbx_handle = asset_server.load_with_settings::<Fbx, FbxLoaderSettings>(
        "maya_cube_7400_binary.fbx",
        |settings: &mut FbxLoaderSettings| {
            // Don't load cameras or lights from the FBX
            settings.load_cameras = false;
            settings.load_lights = false;
            // Convert coordinate system if needed
            settings.convert_coordinates = true;
        },
    );

    // Store the handle for later use
    commands.insert_resource(LoadedFbxHandle(fbx_handle));
}

#[derive(Resource)]
struct LoadedFbxHandle(Handle<Fbx>);

fn handle_loaded_fbx(
    mut commands: Commands,
    fbx_handle: Res<LoadedFbxHandle>,
    fbx_assets: Res<Assets<Fbx>>,
    asset_server: Res<AssetServer>,
    mut spawned: Local<bool>,
) {
    if *spawned {
        return;
    }

    if let Some(fbx) = fbx_assets.get(&fbx_handle.0) {
        info!("FBX loaded successfully!");
        info!("  Scenes: {}", fbx.scenes.len());
        info!("  Nodes: {}", fbx.nodes.len());
        info!("  Meshes: {}", fbx.meshes.len());
        info!("  Materials: {}", fbx.materials.len());

        // Load the first scene
        if !fbx.scenes.is_empty() {
            let scene = asset_server.load::<Scene>("maya_cube_7400_binary.fbx#Scene0");
            commands.spawn(SceneRoot(scene));
            *spawned = true;
        }
    }
}