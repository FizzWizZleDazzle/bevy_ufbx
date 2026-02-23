//! Loads and displays an FBX file.
//!
//! Pass a path (relative to the `assets/` directory) as the first argument,
//! or drop a `cube.fbx` into `assets/` and run without arguments:
//!
//! ```sh
//! cargo run --example load_fbx -- spider.fbx
//! ```
//!
//! The loaded model spins slowly so you can inspect it from all sides.
//! Use `FbxLoaderSettings` to control what gets imported – see the
//! `load_with_settings` section at the bottom of `setup`.

use bevy::prelude::*;
use bevy_ufbx::{Fbx, FbxLoaderSettings, FbxPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FbxPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, rotate)
        .run();
}

// ── Components / resources ──────────────────────────────────────────────────

#[derive(Component)]
struct Spinning;

#[derive(Resource)]
struct FbxHandle(Handle<Fbx>);

// ── Setup ───────────────────────────────────────────────────────────────────

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // ── Camera ──────────────────────────────────────────────────────────────
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(4.0, 4.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // ── Lighting ─────────────────────────────────────────────────────────────
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -45_f32.to_radians(),
            45_f32.to_radians(),
            0.0,
        )),
    ));

    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 200.0,
        affects_lightmapped_meshes: false,
    });

    // ── FBX path from CLI args (defaults to "cube.fbx") ──────────────────────
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "cube.fbx".to_string());

    // --- Default loading ---
    let scene = asset_server.load(format!("{path}#Scene0"));
    commands.spawn((SceneRoot(scene), Spinning));

    // --- Custom settings (uncomment to use instead) ---
    // let fbx = asset_server.load_with_settings::<Fbx, FbxLoaderSettings>(
    //     path.clone(),
    //     |s| {
    //         s.load_cameras = false;
    //         s.load_lights = false;
    //     },
    // );
    // commands.insert_resource(FbxHandle(fbx));

    info!("Loading '{path}' …");
}

// ── Systems ──────────────────────────────────────────────────────────────────

fn rotate(time: Res<Time>, mut q: Query<&mut Transform, With<Spinning>>) {
    for mut t in &mut q {
        t.rotate_y(time.delta_secs() * 0.4);
    }
}
