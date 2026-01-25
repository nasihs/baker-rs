use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VersionError {
    #[error("version file not found: {}", .0.display())]
    FileNotFound(PathBuf),

    #[error("failed to read version file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("macro '{0}' not found in header file")]
    MacroNotFound(String),

    #[error("invalid macro value '{value}' for {name}: {reason}")]
    InvalidMacroValue {
        name: String,
        value: String,
        reason: String,
    },

    #[error("failed to parse version string '{0}': {1}")]
    ParseError(String, String),

    #[error("version source '{0}' not supported yet")]
    UnsupportedSource(String),

    #[error("required field '{0}' not configured in [env.version]")]
    MissingConfig(String),

    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
}
