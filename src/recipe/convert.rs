use std::path::PathBuf;
use std::fmt::{Display, Formatter};
use crate::firmware::{ImageReader, ImageWriter};
use super::{Recipe, CookResult, RecipeError};

/// Format convertion
pub struct ConvertRecipe {
    pub(super) name: String,
    pub(super) description: Option<String>,
    pub(super) reader: Box<dyn ImageReader>,
    pub(super) writer: Box<dyn ImageWriter>,
    pub(super) output_path: PathBuf,
}

impl Recipe for ConvertRecipe {
    fn name(&self) -> &str { &self.name }
    fn description(&self) -> Option<&str> { self.description.as_deref() }
    
    fn cook(&self) -> Result<CookResult, RecipeError> {
        println!("  Loading image...");
        let image = self.reader.read()?;
        
        // Ensure output directory exists
        if let Some(parent) = self.output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        println!("  Converting: {}", self.output_path.display());
        self.writer.write(&image)?;
        
        Ok(CookResult::Single {
            name: self.name.clone(),
            output_path: self.output_path.clone(),
        })
    }
    
    fn validate(&self) -> Result<(), RecipeError> {
        Ok(())
    }
}

impl Display for ConvertRecipe {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:<15} [convert]", self.name)?;
        if let Some(desc) = &self.description {
            write!(f, " {}", desc)?;
        }
        Ok(())
    }
}
