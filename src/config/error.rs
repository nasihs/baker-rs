use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("config file not found: {}", path.display())]
    NotFound { path: PathBuf },

    #[error("failed to read config file: {0}")]
    Read(#[from] std::io::Error),

    #[error("failed to parse config file")]
    Parse(#[from] toml::de::Error),

    #[error("target not found: {0}")]
    TargetNotFound(String),

    #[error("group not found: {0}")]
    GroupNotFound(String),

    #[error("bootloader not found: {0}")]
    BootloaderNotFound(String),

    #[error("invalid target '{name}': {reason}")]
    InvalidTarget { name: String, reason: String },

    #[error("circular group reference detected: {0}")]
    CircularReference(String),

    #[error("no default target or group specified")]
    NoDefault,
}


