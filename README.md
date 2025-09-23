# bevy_ufbx

A Bevy plugin for loading FBX files using the ufbx library.

## Features

- Load FBX files directly into Bevy
- Support for meshes with multiple materials
- PBR material support with textures
- Skeletal animation with skinning
- Scene hierarchy preservation
- Support for lights and cameras
- Flexible loading options

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
bevy_ufbx = "0.1"
bevy = "0.16"
```

## Usage

### Basic Setup

```rust
use bevy::prelude::*;
use bevy_ufbx::FbxPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FbxPlugin)
        .run();
}
```

### Loading FBX Files

```rust
use bevy::prelude::*;
use bevy_ufbx::{Fbx, FbxAssetLabel};

fn load_fbx(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Load the entire FBX file
    let fbx_handle = asset_server.load::<Fbx>("models/character.fbx");

    // Or load specific sub-assets with labels
    let scene = asset_server.load::<Scene>("models/character.fbx#Scene0");
    let mesh = asset_server.load::<Mesh>("models/character.fbx#Mesh0");
    let material = asset_server.load::<StandardMaterial>("models/character.fbx#Material0");

    // Spawn the scene
    commands.spawn(SceneRoot(scene));
}
```

### Custom Loading Settings

```rust
use bevy::prelude::*;
use bevy_ufbx::FbxLoaderSettings;

fn load_with_settings(asset_server: Res<AssetServer>) {
    asset_server.load_with_settings::<Fbx>(
        "models/environment.fbx",
        |settings: &mut FbxLoaderSettings| {
            settings.load_cameras = false;
            settings.load_lights = false;
            settings.convert_coordinates = true;
        }
    );
}
```

## Asset Labels

The plugin uses labeled sub-assets to allow loading specific parts of an FBX file:

- `Scene{N}` - Scene hierarchy (N is the scene index)
- `Node{N}` - Individual nodes
- `Mesh{N}` - Mesh data
- `Material{N}` - Materials
- `Texture{N}` - Textures
- `Animation{N}` - Animations
- `Skin{N}` - Skinning data
- `DefaultMaterial` - Default material when none is specified

## Supported Features

### Geometry
- Triangle meshes
- Multi-material meshes (face groups)
- Vertex positions, normals, UVs
- Vertex colors
- Tangents

### Materials
- PBR materials (base color, metallic, roughness)
- Texture mapping
- Normal maps
- Emission
- Alpha blending

### Animation
- Skeletal animation
- Skinning with bone weights
- Transform animations

### Scene Elements
- Node hierarchy
- Lights (directional, point, spot)
- Cameras

## Limitations

- Requires FBX files to have been exported with triangulated meshes
- NURBS and subdivision surfaces are not directly supported
- Some advanced material features may not be fully supported

## Examples

See the `examples/` directory for more detailed usage examples.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.