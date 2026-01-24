mod error;
mod merge;
mod pack;
mod convert;
mod builder;

pub use error::RecipeError;
pub use merge::MergeRecipe;
pub use pack::{PackRecipe, BuiltinHeaders};
pub use convert::ConvertRecipe;
pub use builder::RecipeBuilder;

use std::fmt::Display;
use std::path::PathBuf;

/// Recipe trait - abstraction for all build recipes
pub trait Recipe: Display {
    fn name(&self) -> &str;
    fn description(&self) -> Option<&str>;
    fn cook(&self) -> Result<CookResult, RecipeError>;
    fn validate(&self) -> Result<(), RecipeError>;
}

/// Build result
pub enum CookResult {
    Single { 
        name: String, 
        output_path: PathBuf,
    },
    Batch(Vec<CookResult>),
}