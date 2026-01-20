mod error;
mod loader;
mod schema;

pub use error::ConfigError;
pub use schema::{
    Bootloader, Config, ConvertTarget, Env, Group, HeaderDef, MergeTarget, PackTarget, OutputConfig, OutputFormat,
    Project, Target,
};
