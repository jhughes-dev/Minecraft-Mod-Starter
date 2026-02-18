use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum McmodError {
    #[error("Invalid mod ID '{0}': must match ^[a-z][a-z0-9_]*$")]
    InvalidModId(String),

    #[error("Invalid package '{0}': must match ^[a-z][a-z0-9_]*(\\.[a-z][a-z0-9_]*)*$")]
    InvalidPackage(String),

    #[error("Feature '{0}' is already enabled")]
    AlreadyEnabled(String),

    #[error("Feature '{0}' is not enabled")]
    NotEnabled(String),

    #[error("mcmod.toml not found — run `mcmod init` first")]
    ConfigNotFound,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML serialization error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, McmodError>;
