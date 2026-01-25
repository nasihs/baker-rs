use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("config file not found: {}", path.display())]
    NotFound { path: PathBuf },

    #[error(transparent)]
    Read(#[from] std::io::Error),

    #[error(transparent)]
    Parse(#[from] toml::de::Error),

    #[error("undefined target or group: {0}")]
    TargetUndefined(String),

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

    /////////////
    /// 
    /// 
    /// 
    
    #[error("Config file not found: {}", path.display())]
    ConfigNotFound { path: PathBuf },

    // #[error("failed to read config file: {0}")]
    // Read(#[from] std::io::Error),

    // #[error("Failed to parse config file")]
    // Parse(#[from] toml::de::Error),

    #[error("Invalid config file ({0})")]
    Invalid(String),  // 字段错误 / 选项错误 / 循环引用

    #[error("Firmware not found ({0})")]
    FirmwareNotFound(String),  // bootloader / app 固件路径不存在
}


