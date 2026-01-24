use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RecipeError {
    #[error("file not found at path: '{}'", .0.display())]
    NotFound(PathBuf),

    #[error("bootloader '{0}' is missing in config")]
    MissingBootloader(String),

    #[error("missing header '{0}' (not built-in or defined in [headers])")]
    MissingHeader(String),

    #[error("missing base addr for binary file '{0}'")]
    MissingBaseAddr(String),

    #[error("config error: {0}")]
    Config(#[from] crate::config::ConfigError),

    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("firmware error: {0}")]
    Firmware(#[from] crate::firmware::FirmwareError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("build failed for '{name}': {reason}")]
    BuildFailed { name: String, reason: String },

    #[error("header '{header_name}' has invalid DSL definition: {reason}")]
    HeaderInvalid { 
        header_name: String, 
        reason: String 
    },

    #[error("header '{name}' already exists as built-in header, please use a different name")]
    HeaderExists { 
        name: String,
    },
}
