use std::path::PathBuf;
use std::fmt::{Display, Formatter};
use crate::config::HeaderType;
use crate::firmware::{ImageReader, ImageWriter};
use super::{Recipe, CookResult, RecipeError};

pub struct PackRecipe {
    name: String,
    description: Option<String>,
    app_reader: Box<dyn ImageReader>,
    writer: Box<dyn ImageWriter>,
    output_path: PathBuf,
    header_type: HeaderType,
}

impl PackRecipe {
    pub fn new(
        name: String,
        description: Option<String>,
        app_reader: Box<dyn ImageReader>,
        writer: Box<dyn ImageWriter>,
        output_path: PathBuf,
        header_type: HeaderType,
    ) -> Self {
        Self {
            name,
            description,
            app_reader,
            writer,
            output_path,
            header_type,
        }
    }
}

impl Recipe for PackRecipe {
    fn name(&self) -> &str { &self.name }
    fn description(&self) -> Option<&str> { self.description.as_deref() }
    
    fn cook(&self) -> Result<CookResult, RecipeError> {
        println!("  Loading application...");
        let image = self.app_reader.read()?;
        
        // Add header based on type
        match self.header_type {
            HeaderType::None => {
                println!("  No header needed");
            }
            HeaderType::OpenBlt => {
                println!("  Adding OpenBLT header...");
                // TODO: Implement OpenBLT header
                return Err(RecipeError::BuildFailed {
                    name: self.name.clone(),
                    reason: "OpenBLT header not yet implemented".to_string(),
                });
            }
            HeaderType::Custom => {
                println!("  Adding custom header...");
                // TODO: Implement custom header
                return Err(RecipeError::BuildFailed {
                    name: self.name.clone(),
                    reason: "Custom header not yet implemented".to_string(),
                });
            }
        }
        
        // Ensure output directory exists
        if let Some(parent) = self.output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        println!("  Writing: {}", self.output_path.display());
        self.writer.write(&image)?;
        
        Ok(CookResult::Single {
            name: self.name.clone(),
            output_path: self.output_path.clone(),
        })
    }
    
    fn validate(&self) -> Result<(), RecipeError> {
        // Validation is now done by trying to read via readers
        Ok(())
    }
}

impl Display for PackRecipe {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:<15} [pack]", self.name)?;
        if let Some(desc) = &self.description {
            write!(f, " {}", desc)?;
        }
        Ok(())
    }
}