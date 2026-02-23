# bevy_ufbx

FBX asset loader for [Bevy](https://bevyengine.org) powered by [ufbx](https://github.com/ufbx/ufbx).

## Bevy compatibility

| bevy | bevy_ufbx |
|------|-----------|
| 0.18 | 0.18      |
| 0.17 | 0.17      |

## Installation

```toml
[dependencies]
bevy     = "0.18"
bevy_ufbx = "0.18"
```

## Quick start

```rust
use bevy::prelude::*;
use bevy_ufbx::FbxPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FbxPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera3d::default());
    commands.spawn(SceneRoot(
        asset_server.load("character.fbx#Scene0"),
    ));
}
```

## Loading with custom settings

```rust
use bevy_ufbx::{Fbx, FbxLoaderSettings};

fn setup(asset_server: Res<AssetServer>) {
    asset_server.load_with_settings::<Fbx, FbxLoaderSettings>(
        "environment.fbx",
        |s| {
            s.load_cameras = false;
            s.load_lights  = false;
        },
    );
}
```

### `FbxLoaderSettings` fields

| Field                | Type                 | Default                       | Description                                 |
|----------------------|----------------------|-------------------------------|---------------------------------------------|
| `load_meshes`        | `RenderAssetUsages`  | `RenderAssetUsages::default()` | Which worlds the mesh is available in       |
| `load_materials`     | `RenderAssetUsages`  | `RenderAssetUsages::default()` | Which worlds the material is available in   |
| `load_cameras`       | `bool`               | `true`                        | Import cameras from the FBX                 |
| `load_lights`        | `bool`               | `true`                        | Import lights from the FBX                  |
| `include_source`     | `bool`               | `false`                       | Keep raw bytes in the loaded asset          |
| `convert_coordinates`| `bool`               | `false`                       | Remap axes to Bevy's right-handed Y-up space|

## Asset labels

Individual sub-assets can be addressed with `#Label` path suffixes:

| Label             | Type                | Description                             |
|-------------------|---------------------|-----------------------------------------|
| `Scene{N}`        | `Scene`             | Scene hierarchy (N = scene index)       |
| `Mesh{N}`         | `Mesh`              | Triangulated mesh                       |
| `Material{N}`     | `StandardMaterial`  | PBR material                            |
| `Node{N}`         | `FbxNode`           | Transform node                          |
| `Skin{N}`         | `FbxSkin`           | Skeletal skin                           |
| `DefaultMaterial` | `StandardMaterial`  | Fallback material when none is present  |

```rust
let scene    = asset_server.load::<Scene>("model.fbx#Scene0");
let mesh     = asset_server.load::<Mesh>("model.fbx#Mesh0");
let material = asset_server.load::<StandardMaterial>("model.fbx#Material0");
```

## Supported features

- Triangle meshes with positions, normals, and UVs
- Multi-material meshes (face groups per material)
- PBR materials (base color, metallic, roughness, normal, emission, AO)
- Texture mapping, including `.fbm` embedded texture folders
- Skeletal skinning data (bone weights / bind poses)
- Scene hierarchy (node transforms)
- Directional, point, and spot lights

## Limitations

- Animation curves are parsed but **not yet forwarded to Bevy's animation system**
- Cameras are not imported into Bevy camera components
- NURBS and subdivision surfaces are not supported (ufbx triangulates on load)

## Example

```sh
cargo run --example load_fbx -- my_model.fbx
```

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE) at your option.
