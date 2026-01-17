use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RecipeError {
    #[error("target not found: {0}")]
    TargetNotFound(String),

    #[error("bootloader '{0}' has no file specified")]
    BootloaderFileNotSpecified(String),

    #[error("bootloader '{0}' not found")]
    BootloaderNotFound(String),

    #[error("input file not found: {}", .0.display())]
    InputNotFound(PathBuf),

    #[error("config error: {0}")]
    Config(#[from] crate::config::ConfigError),

    #[error("no extension")]
    NoExtension,

    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("firmware error: {0}")]
    Firmware(#[from] crate::firmware::FirmwareError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("build failed for '{name}': {reason}")]
    BuildFailed { name: String, reason: String },
}
