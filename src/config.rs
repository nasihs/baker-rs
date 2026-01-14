mod error;
mod loader;
mod schema;

pub use error::ConfigError;
pub use schema::{
    Bootloader, Config, Group, HeaderType, MergeTarget, OtaTarget, OutputConfig, OutputFormat,
    Project, Target,
};
