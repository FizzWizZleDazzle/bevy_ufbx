#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![forbid(unsafe_code)]
#![doc(
    html_logo_url = "https://bevyengine.org/assets/icon.png",
    html_favicon_url = "https://bevyengine.org/assets/icon.png"
)]

//! Loader for FBX scenes using [`ufbx`](https://github.com/ufbx/ufbx-rust).
//!
//! This plugin provides comprehensive FBX file loading support for Bevy, including:
//! - Mesh geometry with multi-material support
//! - PBR materials with textures
//! - Skeletal animation and skinning
//! - Scene hierarchy with nodes
//! - Lights and cameras
//! - Animation clips with curves

use bevy::asset::AssetApp;
use bevy::prelude::*;

pub mod error;
pub mod label;
pub mod loader;
pub mod material;
pub mod mesh;
pub mod node;
pub mod scene;
pub mod types;
pub mod utils;

pub use error::FbxError;
pub use label::FbxAssetLabel;
pub use loader::{FbxLoader, FbxLoaderSettings};
pub use types::*;

pub mod prelude {
    //! Commonly used items.
    pub use crate::{Fbx, FbxAssetLabel, FbxLoaderSettings, FbxNode, FbxPlugin, FbxSkin, Skeleton};
}

/// Plugin adding the FBX loader to an [`App`].
#[derive(Default)]
pub struct FbxPlugin;

impl Plugin for FbxPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Fbx>()
            .init_asset::<FbxNode>()
            .init_asset::<FbxSkin>()
            .init_asset::<Skeleton>()
            .register_asset_loader(FbxLoader::default());
    }
}
