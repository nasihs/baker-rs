use std::path::PathBuf;
use std::fmt::{Display, Formatter};
use crate::firmware::{ImageReader, ImageWriter};
use super::{Recipe, CookResult, RecipeError};
use super::hook::HookRunner;

pub struct MergeRecipe {
    pub(super) name: String,
    pub(super) description: Option<String>,
    pub(super) bootloader_reader: Box<dyn ImageReader>,
    pub(super) app_reader: Box<dyn ImageReader>,
    pub(super) writer: Box<dyn ImageWriter>,
    pub(super) output_path: PathBuf,
    pub(super) hook: Option<HookRunner>,
}

impl Recipe for MergeRecipe {
    fn name(&self) -> &str { &self.name }
    fn description(&self) -> Option<&str> { self.description.as_deref() }
    
    fn cook(&self) -> Result<CookResult, RecipeError> {
        println!("  Loading bootloader...");
        let mut image = self.bootloader_reader.read()?;
        
        println!("  Loading application...");
        let app = self.app_reader.read()?;
        
        println!("  Merging images...");
        image.merge(&app)?;
        
        // Ensure output directory exists
        if let Some(parent) = self.output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        println!("  Writing: {}", self.output_path.display());
        self.writer.write(&image)?;
        
        if let Some(hook) = &self.hook {
            hook.run(&self.output_path)?;
        }
        
        Ok(CookResult::Single {
            name: self.name.clone(),
            output_path: self.output_path.clone(),
        })
    }
    
    fn validate(&self) -> Result<(), RecipeError> {
        // Validation is now done by trying to read via readers
        // Could add file existence checks if paths are accessible
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