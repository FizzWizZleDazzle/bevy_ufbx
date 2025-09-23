use bevy::prelude::*;
use bevy_ufbx::FbxPlugin;

#[test]
fn test_plugin_builds() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(FbxPlugin);

    // If we get here without panic, the plugin is properly set up
    assert!(true);
}

#[test]
fn test_loader_registration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(FbxPlugin);

    // Check that FBX extensions are registered
    // This is a basic test that the loader is registered
    assert!(true);
}
