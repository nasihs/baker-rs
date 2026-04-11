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

    #[error(transparent)]
    Config(#[from] crate::config::ConfigError),

    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error(transparent)]
    Firmware(#[from] crate::firmware::FirmwareError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    VersionError(#[from] crate::version::VersionError),

    #[error("undefined variable '${{{}}}' in template", .0)]
    MissingVariable(String),

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

    #[error("external tool not found: '{0}' (is it installed and on PATH?)")]
    ExternalToolNotFound(String),

    #[error("external tool '{command}' failed with exit code {exit_code}{}", format_tool_output(.stdout, .stderr))]
    ExternalToolFailed {
        command: String,
        exit_code: String,
        stdout: String,
        stderr: String,
    },
}

fn format_tool_output(stdout: &str, stderr: &str) -> String {
    let mut parts = Vec::new();
    if !stderr.is_empty() { parts.push(format!("stderr: {}", stderr)); }
    if !stdout.is_empty() { parts.push(format!("stdout: {}", stdout)); }
    if parts.is_empty() { String::new() } else { format!(": {}", parts.join("; ")) }
}
