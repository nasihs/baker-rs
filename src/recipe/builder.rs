use std::path::{Path, PathBuf};
use log::trace;
use crate::config::{Bootloader, Config, ConvertTarget, MergeTarget, PackTarget, OutputFormat, Target};
use crate::firmware::{self, ImageReader, ImageWriter};
use super::{Recipe, RecipeError, MergeRecipe, PackRecipe, ConvertRecipe, BuiltinHeaders};
use super::pack::HeaderBuilder;

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
    
    /// Creates a Recipe by target name
    pub fn build(&self, name: &str) -> Result<Box<dyn Recipe>, RecipeError> {  // TODO name type -> Target/Group        
        let target = self.config.targets.get(name).unwrap();  // target existance has been checked in config.resolve_targets
        return self.build_target(name, target);
    }
    
    fn validate_headers(&self) -> Result<(), RecipeError> {
        for header_name in self.config.headers.keys() {
            if BuiltinHeaders::is_builtin(header_name) {
                return Err(RecipeError::HeaderExists {
                    name: header_name.clone(),
                });
            }
        }
        Ok(())
    }
    
    /// Creates multiple recipes by target names
    pub fn build_batch(&self, names: &[&str]) -> Result<Vec<Box<dyn Recipe>>, RecipeError> {
        self.validate_headers()?;

        names.iter()
            .map(|name| self.build(name))
            .collect()
    }
    
    // Check referenced bootoladers/headers, and path existance
    fn build_target(&self, name: &str, target: &Target) -> Result<Box<dyn Recipe>, RecipeError> {
        match target {
            Target::Merge(t) => Ok(Box::new(self.build_merge(name, t)?)),
            Target::Pack(t) => Ok(Box::new(self.build_pack(name, t)?)),
            Target::Convert(t) => Ok(Box::new(self.build_convert(name, t)?)),
        }
    }
    
    fn build_merge(&self, name: &str, t: &MergeTarget) -> Result<MergeRecipe, RecipeError> {
        trace!("Check whether bootloader is defined");
        let bl: &Bootloader = self.config.bootloaders.get(&t.bootloader)
            .ok_or_else(|| RecipeError::MissingBootloader(t.bootloader.clone()))?;
        
        let output_dir = self.resolve_path(
            t.output_dir.as_deref().unwrap_or(&self.config.env.output.dir)
        );
        let output_name = t.output_name.as_deref().unwrap_or(name);
        let output_path = output_dir.join(format!("{}.{}", output_name, t.output_format.extension()));
        
        let bootloader_path = self.resolve_path(&bl.file);
        let app_path = self.resolve_path(&t.app_file);
        
        // Create readers
        let bootloader_reader = self.create_reader(&bootloader_path, Some(bl.base_addr))?;
        let app_reader = self.create_reader(&app_path, Some(bl.base_addr + bl.app_offset))?;
        
        // Create writer
        let writer = self.create_writer(&output_path, t.output_format, t.fill_byte)?;
        
        trace!("Built merge recipe: {}", name);
        Ok(MergeRecipe {
            name: name.to_string(),
            description: t.description.clone(),
            bootloader_reader,
            app_reader,
            writer,
            output_path,
        })
    }
    
    fn build_convert(&self, name: &str, t: &ConvertTarget) -> Result<ConvertRecipe, RecipeError> {
        let output_dir = self.resolve_path(
            t.output_dir.as_deref().unwrap_or(&self.config.env.output.dir)
        );
        let output_name = t.output_name.as_deref().unwrap_or(name);
        let output_path = output_dir.join(format!("{}.{}", output_name, t.output_format.extension()));
        
        let input_path = self.resolve_path(&t.input_file);
        let reader = self.create_reader(&input_path, None)?;
        let writer = self.create_writer(&output_path, t.output_format, t.fill_byte)?;
        
        trace!("Built convert recipe: {}", name);
        Ok(ConvertRecipe {
            name: name.to_string(),
            description: t.description.clone(),
            reader,
            writer,
            output_path,
        })
    }
    
    fn build_pack(&self, name: &str, t: &PackTarget) -> Result<PackRecipe, RecipeError> {
        let header_name = &t.header;
        
        trace!("Check whether header is defined");
        let (dsl, suffix) = if let Some(builtin_dsl) = BuiltinHeaders::get_dsl(header_name) {
            let suffix = BuiltinHeaders::get_suffix(header_name)
                .expect("builtin header must have suffix");
            (builtin_dsl.to_string(), suffix.to_string())
        } else if let Some(header_def) = self.config.headers.get(header_name) {
            (header_def.def.clone(), header_def.suffix.clone())
        } else {
            return Err(RecipeError::MissingHeader(header_name.clone()));
        };
        
        let output_dir = self.resolve_path(
            t.output_dir.as_deref().unwrap_or(&self.config.env.output.dir)
        );
        let output_name = t.output_name.as_deref().unwrap_or(name);
        let output_path = output_dir.join(format!("{}.{}", output_name, suffix));
        let app_path = self.resolve_path(&t.app_file);
        
        let app_reader = self.create_reader(&app_path, t.app_offset)?;
        let writer = self.create_writer(&output_path, OutputFormat::Bin, t.fill_byte)?;
        
        trace!("Build header builder: {}", header_name);
        let header_builder = HeaderBuilder::new_validated(
            header_name.clone(),
            dsl
        )?;
        
        trace!("Built pack recipe: {}", name);
        Ok(PackRecipe {
            name: name.to_string(),
            description: t.description.clone(),
            app_reader,
            writer,
            output_path,
            header_builder,
        })
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
            return Err(RecipeError::NotFound(path.to_path_buf()));
        }
        
        let extension = path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        
        match extension.to_lowercase().as_str() {
            "hex" => Ok(Box::new(firmware::hex::HexReader::new(path))),
            "bin" => {
                let addr = base_addr.ok_or_else(|| RecipeError::MissingBaseAddr(path.display().to_string()))?;
                Ok(Box::new(firmware::bin::BinReader::new(path, addr)))
            }
            "srec" | "s19" | "s28" | "s37" => {
                Ok(Box::new(firmware::srec::SrecReader::new(path)))
            }
            _ => {
                Err(RecipeError::UnsupportedFormat(extension.to_owned()))
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