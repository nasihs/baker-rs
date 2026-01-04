use std::path::{Path, PathBuf};
use crate::config::{Config, Target, MergeTarget, OtaTarget};
use super::{Recipe, RecipeError, MergeRecipe, OtaRecipe, GroupRecipe};

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
    
    fn build_target(&self, name: &str, target: &Target) -> Result<Box<dyn Recipe>, RecipeError> {
        match target {
            Target::Merge(t) => Ok(Box::new(self.build_merge(name, t)?) as Box<dyn Recipe>),
            Target::Ota(t) => Ok(Box::new(self.build_ota(name, t)?) as Box<dyn Recipe>),
        }
    }
    
    fn build_merge(&self, name: &str, t: &MergeTarget) -> Result<MergeRecipe, RecipeError> {
        let bl = self.config.bootloaders.get(&t.bootloader)
            .ok_or_else(|| RecipeError::BootloaderNotFound(t.bootloader.clone()))?;
        
        let bl_file = bl.file.as_ref()
            .ok_or_else(|| RecipeError::BootloaderFileNotSpecified(t.bootloader.clone()))?;
        
        let output_dir = self.resolve_path(
            t.output_dir.as_deref().unwrap_or(&self.config.output.dir)
        );
        let output_name = t.output_name.as_deref().unwrap_or(name);
        let output_path = output_dir.join(format!("{}.{}", output_name, t.output_format.extension()));
        
        Ok(MergeRecipe {
            name: name.to_string(),
            description: t.description.clone(),
            output_path,
            bootloader_path: self.resolve_path(&bl_file),
            app_path: self.resolve_path(&t.app_file),
            base_addr: bl.base_addr,
            app_offset: bl.app_offset,
            fill_byte: t.fill_byte,
            output_format: t.output_format,
        })
    }
    
    fn build_ota(&self, name: &str, t: &OtaTarget) -> Result<OtaRecipe, RecipeError> {
        let output_dir = self.resolve_path(
            t.output_dir.as_deref().unwrap_or(&self.config.output.dir)
        );
        let output_name = t.output_name.as_deref().unwrap_or(name);
        let output_path = output_dir.join(format!("{}.{}", output_name, t.output_format.extension()));
        
        Ok(OtaRecipe {
            name: name.to_string(),
            description: t.description.clone(),
            output_path,
            app_path: self.resolve_path(&t.app_file),
            header_type: t.header.clone(),
            output_format: t.output_format,
        })
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
}