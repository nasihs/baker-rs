use std::path::PathBuf;
use std::fmt::{Display, Formatter};
use crate::config::{OutputFormat, HeaderType};
use super::{Recipe, CookResult, RecipeError};

pub struct OtaRecipe {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) output_path: PathBuf,
    pub(crate) app_path: PathBuf,
    pub(crate) header_type: HeaderType,
    pub(crate) output_format: OutputFormat,
}

impl Recipe for OtaRecipe {
    fn name(&self) -> &str { &self.name }
    fn description(&self) -> Option<&str> { self.description.as_deref() }
    
    fn cook(&self) -> Result<CookResult, RecipeError> {
        // TODO: OTA implementation
        println!("[{}] OTA packaging not yet implemented", self.name);
        Ok(CookResult::Single {
            name: self.name.clone(),
            output_path: self.output_path.clone(),
        })
    }
    
    fn validate(&self) -> Result<(), RecipeError> {
        if !self.app_path.exists() {
            return Err(RecipeError::InputNotFound(self.app_path.clone()));
        }
        Ok(())
    }
}

impl Display for OtaRecipe {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:<15} [ota]", self.name)?;
        if let Some(desc) = &self.description {
            write!(f, " {}", desc)?;
        }
        Ok(())
    }
}