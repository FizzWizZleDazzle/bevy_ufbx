use thiserror::Error;

#[derive(Error, Debug)]
pub enum FbxError {
    #[error("Failed to read FBX file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to load FBX: {0}")]
    UfbxError(String),

    #[error("Failed to convert FBX data: {0}")]
    ConversionError(String),

    #[error("Failed to convert mesh: {0}")]
    MeshConversion(String),

    #[error("Failed to convert material: {0}")]
    MaterialConversion(String),

    #[error("Failed to load texture: {0}")]
    TextureLoad(String),

    #[error("Invalid FBX data: {0}")]
    InvalidData(String),

    #[error("Unsupported FBX feature: {0}")]
    UnsupportedFeature(String),
}
