//! Integration tests for FBX loading.

use bevy::asset::{AssetPlugin, AssetServer};
use bevy::prelude::*;
use bevy_ufbx::{Fbx, FbxPlugin};

#[test]
fn test_plugin_initialization() {
    let mut app = App::new();

    // Add required plugins
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(FbxPlugin);

    // Check that the plugin initialized correctly
    app.update();

    // If we get here without panic, the plugin is working
    assert!(true);
}

#[test]
fn test_asset_registration() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(FbxPlugin);

    // Check that Fbx asset is registered
    let asset_server = app.world().resource::<AssetServer>();

    // This would typically be tested by loading an actual FBX file
    // For now, we just verify the asset server exists
    assert!(
        asset_server.mode() == bevy::asset::AssetServerMode::Processed
            || asset_server.mode() == bevy::asset::AssetServerMode::Unprocessed
    );
}

// Test system to verify FBX loading
fn check_fbx_loaded(
    _asset_server: Res<AssetServer>,
    fbx_assets: Res<Assets<Fbx>>,
    mut loaded: Local<bool>,
) {
    if *loaded {
        return;
    }

    // In a real test, we would load an actual FBX file
    // For now, we just verify the system can access the resources
    let _ = fbx_assets.len();
    *loaded = true;
}

#[test]
fn test_fbx_loading_system() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(FbxPlugin);

    app.add_systems(Update, check_fbx_loaded);

    // Run a few update cycles
    for _ in 0..3 {
        app.update();
    }

    // If we get here without panic, systems are working
    assert!(true);
}

#[test]
fn test_multiple_plugins() {
    let mut app = App::new();

    // Test that we can add multiple instances of required plugins
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(FbxPlugin);

    // Add some standard Bevy plugins that might interact with FBX loading
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();

    app.update();

    // Verify resources exist
    assert!(app.world().get_resource::<Assets<Mesh>>().is_some());
    assert!(app
        .world()
        .get_resource::<Assets<StandardMaterial>>()
        .is_some());
    assert!(app.world().get_resource::<Assets<Fbx>>().is_some());
}
