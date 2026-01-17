use std::path::{Path, PathBuf};
use crate::config::{Bootloader, Config, MergeTarget, PackTarget, OutputFormat, Target};
use crate::firmware::{self, ImageReader, ImageWriter};
use super::{Recipe, RecipeError, MergeRecipe, PackRecipe, GroupRecipe};

pub struct RecipeBuilder<'a> {
    config: &'a Config,
    base_dir: PathBuf,
}

impl<'a> RecipeBuilder<'a> {
    pub fn new(config: &'a Config, base_dir: &Path) -> Self {
        Self {
            config,
            base_dir: base_dir.to_path_buf(),
        }
    }
    
    /// Creates a Recipe by name (can be a target or group)
    pub fn build(&self, name: &str) -> Result<Box<dyn Recipe>, RecipeError> {
        if let Some(group) = self.config.groups.get(name) {
            return self.build_group(name, group.targets());
        }
        if let Some(target) = self.config.targets.get(name) {
            return self.build_target(name, target);
        }
        Err(RecipeError::TargetNotFound(name.to_string()))
    }
    
    /// Creates multiple recipes by names (can be targets or groups)
    pub fn build_batch(&self, names: &[&str]) -> Result<Vec<Box<dyn Recipe>>, RecipeError> {
        names.iter()
            .map(|name| self.build(name))
            .collect()
    }
    
    fn build_target(&self, name: &str, target: &Target) -> Result<Box<dyn Recipe>, RecipeError> {
        match target {
            Target::Merge(t) => Ok(Box::new(self.build_merge(name, t)?) as Box<dyn Recipe>),
            Target::Pack(t) => Ok(Box::new(self.build_pack(name, t)?) as Box<dyn Recipe>),
        }
    }
    
    fn build_merge(&self, name: &str, t: &MergeTarget) -> Result<MergeRecipe, RecipeError> {
        let bl: &Bootloader = self.config.bootloaders.get(&t.bootloader)
            .ok_or_else(|| RecipeError::BootloaderNotFound(t.bootloader.clone()))?;
        
        let bl_file = bl.file.as_ref()
            .ok_or_else(|| RecipeError::BootloaderFileNotSpecified(t.bootloader.clone()))?;
        
        let output_dir = self.resolve_path(
            t.output_dir.as_deref().unwrap_or(&self.config.output.dir)
        );
        let output_name = t.output_name.as_deref().unwrap_or(name);
        let output_path = output_dir.join(format!("{}.{}", output_name, t.output_format.extension()));
        
        let bootloader_path = self.resolve_path(&bl_file);
        let app_path = self.resolve_path(&t.app_file);
        
        // Create readers
        let bootloader_reader = self.create_reader(&bootloader_path, Some(bl.base_addr))?;
        let app_reader = self.create_reader(&app_path, Some(bl.base_addr + bl.app_offset))?;
        
        // Create writer
        let writer = self.create_writer(&output_path, t.output_format, t.fill_byte)?;
        
        Ok(MergeRecipe::new(
            name.to_string(),
            t.description.clone(),
            bootloader_reader,
            app_reader,
            writer,
            output_path,
        ))
    }
    
    fn build_pack(&self, name: &str, t: &PackTarget) -> Result<PackRecipe, RecipeError> {
        let output_dir = self.resolve_path(
            t.output_dir.as_deref().unwrap_or(&self.config.output.dir)
        );
        let output_name = t.output_name.as_deref().unwrap_or(name);
        let output_path = output_dir.join(format!("{}.{}", output_name, t.output_format.extension()));
        let app_path = self.resolve_path(&t.app_file);
        let app_reader = self.create_reader(&app_path, None)?;
        let writer = self.create_writer(&output_path, t.output_format, t.fill_byte)?;
        
        Ok(PackRecipe::new(
            name.to_string(),
            t.description.clone(),
            app_reader,
            writer,
            output_path,
            t.header.clone(),
        ))
    }
    
    fn build_group(&self, name: &str, targets: &[String]) -> Result<Box<dyn Recipe>, RecipeError> {
        let recipes: Result<Vec<_>, _> = targets
            .iter()
            .map(|t| self.build(t))
            .collect();
        
        Ok(Box::new(GroupRecipe::new(name.to_string(), recipes?)))
    }
    
    fn resolve_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let path = path.as_ref();
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.base_dir.join(path)
        }
    }
    
    /// Create an ImageReader based on file extension
    fn create_reader(&self, path: &Path, base_addr: Option<u32>) -> Result<Box<dyn ImageReader>, RecipeError> {
        if !path.exists() {
            return Err(RecipeError::InputNotFound(path.to_path_buf()));
        }
        
        let extension = path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        
        match extension.to_lowercase().as_str() {
            "hex" => Ok(Box::new(firmware::hex::HexReader::new(path))),
            "bin" => {
                let addr = base_addr.ok_or_else(|| RecipeError::BuildFailed {
                    name: path.display().to_string(),
                    reason: "Binary file requires base address".to_string(),
                })?;
                Ok(Box::new(firmware::bin::BinReader::new(path, addr)))
            }
            "srec" | "s19" | "s28" | "s37" => {
                Ok(Box::new(firmware::srec::SrecReader::new(path)))
            }
            _ => {
                // Try hex as default
                Ok(Box::new(firmware::hex::HexReader::new(path)))
            }
        }
    }
    
    /// Create an ImageWriter based on output format
    fn create_writer(&self, path: &Path, format: OutputFormat, fill_byte: u8) -> Result<Box<dyn ImageWriter>, RecipeError> {
        match format {
            OutputFormat::Hex => Ok(Box::new(firmware::hex::HexWriter::new(path))),
            OutputFormat::Bin => Ok(Box::new(firmware::bin::BinWriter::new(path, fill_byte))),
            OutputFormat::Srec => Ok(Box::new(firmware::srec::SrecWriter::new(path))),
        }
    }
}