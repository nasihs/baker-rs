mod error;
mod loader;
mod schema;

pub use error::ConfigError;
pub use loader::load;
pub use schema::{
    Bootloader, Config, Group, HeaderType, MergeTarget, OtaTarget, OutputConfig, OutputFormat,
    Project, Target, VersionConfig,
};
