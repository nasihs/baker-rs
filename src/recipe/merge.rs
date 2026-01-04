use std::path::PathBuf;
use std::fmt::{Display, Formatter};
use crate::config::OutputFormat;
use crate::firmware;
use super::{Recipe, CookResult, RecipeError};

pub struct MergeRecipe {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) bootloader_path: PathBuf,
    pub(crate) app_path: PathBuf,
    pub(crate) base_addr: u32,
    pub(crate) app_offset: u32,
    pub(crate) fill_byte: u8,
    pub(crate) output_path: PathBuf,
    pub(crate) output_format: OutputFormat,
}

impl Recipe for MergeRecipe {
    fn name(&self) -> &str { &self.name }
    fn description(&self) -> Option<&str> { self.description.as_deref() }
    
    fn cook(&self) -> Result<CookResult, RecipeError> {
        println!("  Loading bootloader: {}", self.bootloader_path.display());
        let mut image = firmware::ihex::read(&self.bootloader_path)?;
        
        println!("  Loading app: {}", self.app_path.display());
        let app = firmware::ihex::read(&self.app_path)?;
        
        image.merge(&app)?;
        
        // Ensure output directory exists
        if let Some(parent) = self.output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        println!("  Writing: {}", self.output_path.display());
        match self.output_format {
            OutputFormat::Hex => firmware::ihex::write(&image, &self.output_path)?,
            OutputFormat::Bin => firmware::binary::write(&image, &self.output_path, self.fill_byte)?,
            OutputFormat::Srec => {
                return Err(RecipeError::BuildFailed {
                    name: self.name.clone(),
                    reason: "SREC format not yet supported".to_string(),
                });
            }
        }
        
        Ok(CookResult::Single {
            name: self.name.clone(),
            output_path: self.output_path.clone(),
        })
    }
    
    fn validate(&self) -> Result<(), RecipeError> {
        if !self.bootloader_path.exists() {
            return Err(RecipeError::InputNotFound(self.bootloader_path.clone()));
        }
        if !self.app_path.exists() {
            return Err(RecipeError::InputNotFound(self.app_path.clone()));
        }
        Ok(())
    }
}

impl Display for MergeRecipe {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:<15} [merge]", self.name)?;
        if let Some(desc) = &self.description {
            write!(f, " {}", desc)?;
        }
        Ok(())
    }
}