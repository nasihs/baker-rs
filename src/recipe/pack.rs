use std::path::PathBuf;
use std::fmt::{Display, Formatter};
use std::collections::HashMap;
use crate::firmware::{ImageReader, ImageWriter};
use super::{Recipe, CookResult, RecipeError};

/// Create .delbin file under builtin_headers to add new builtin header support
macro_rules! define_builtin_headers {
    ($($name:literal => ($file:literal, $suffix:literal)),* $(,)?) => {
        pub struct BuiltinHeaders;

        impl BuiltinHeaders {
            pub fn names() -> &'static [&'static str] {
                &[$($name),*]
            }
            
            pub fn is_builtin(name: &str) -> bool {
                Self::names().contains(&name)
            }
            
            pub fn get_dsl(name: &str) -> Option<&'static str> {
                match name {
                    $($name => Some(include_str!($file)),)*
                    _ => None,
                }
            }
            
            pub fn get_suffix(name: &str) -> Option<&'static str> {
                match name {
                    $($name => Some($suffix),)*
                    _ => None,
                }
            }
        }
    };
}

// add new header definition here
define_builtin_headers! {
    "mota" => ("builtin_headers/mota.delbin", "fpk"),
}

pub struct PackRecipe {
    pub(super) name: String,
    pub(super) description: Option<String>,
    pub(super) app_reader: Box<dyn ImageReader>,
    pub(super) writer: Box<dyn ImageWriter>,
    pub(super) output_path: PathBuf,
    pub(super) header_builder: HeaderBuilder,
}

pub(super) struct HeaderBuilder {
    header_name: String,
    dsl: String,
    env: HashMap<String, delbin::Value>,  // Environment variables for header generation
}

impl HeaderBuilder {
    pub fn new_validated(header_name: String, dsl: String, env: HashMap<String, delbin::Value>) -> Result<Self, RecipeError> {
        // validate grammar with blank image
        let mut sections: HashMap<String, Vec<u8>> = HashMap::new();
        sections.insert("image".to_string(), Vec::new());
        
        match delbin::generate(&dsl, &env, &sections) {
            Ok(_) => {
                Ok(Self { 
                    header_name,
                    dsl,
                    env,
                })
            }
            Err(e) => {
                Err(RecipeError::HeaderInvalid {
                    header_name: header_name.clone(),
                    reason: format!("{}", e),
                })
            }
        }
    }
    
    /// Generate binary header
    pub fn generate(&self, app_data: &[u8]) -> Result<Vec<u8>, RecipeError> {
        let mut sections: HashMap<String, Vec<u8>> = HashMap::new();
        sections.insert("image".to_string(), app_data.to_vec());
        
        delbin::generate(&self.dsl, &self.env, &sections)
            .map(|r| r.data)
            .map_err(|e| RecipeError::BuildFailed {
                name: self.header_name.clone(),
                reason: format!("Failed to generate header: {}", e),
            })
    }
}

impl Recipe for PackRecipe {
    fn name(&self) -> &str { &self.name }
    fn description(&self) -> Option<&str> { self.description.as_deref() }
    
    fn cook(&self) -> Result<CookResult, RecipeError> {
        println!("  Loading application...");
        let image = self.app_reader.read()?;
        
        println!("  Generating custom header...");
        
        // Get continuous app data for header generation
        let app_data = image.to_continuous_data()?;
        let header_data = self.header_builder.generate(&app_data)?;
        
        // Get original base address
        let (base_addr, _) = image.address_range()
            .ok_or_else(|| RecipeError::BuildFailed {
                name: self.name.clone(),
                reason: "Empty image".to_string(),
            })?;
        
        // Create new image with header prepended
        let mut new_image = crate::firmware::Image::new();
        new_image.add_data(base_addr, header_data.clone());
        new_image.add_data(base_addr + header_data.len() as u32, app_data);
        
        println!("  Header generated ({} bytes)", header_data.len());
        
        // Ensure output directory exists
        if let Some(parent) = self.output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        println!("  Writing: {}", self.output_path.display());
        self.writer.write(&new_image)?;
        
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