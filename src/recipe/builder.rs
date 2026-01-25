use std::path::{Path, PathBuf};
use std::collections::HashMap;
use log::{trace};
use crate::config::{Bootloader, Config, ConvertTarget, MergeTarget, PackTarget, OutputFormat, Target, VersionSource};
use crate::firmware::{self, ImageReader, ImageWriter};
use crate::version::{VersionInfo, VersionExtractor, HeaderExtractor, VersionError};
use super::{Recipe, RecipeError, MergeRecipe, PackRecipe, ConvertRecipe, BuiltinHeaders};
use super::pack::HeaderBuilder;

pub struct RecipeBuilder<'a> {
    config: &'a Config,
    base_dir: PathBuf,
    variables: HashMap<String, String>,  // Template variables
}

impl<'a> RecipeBuilder<'a> {
    pub fn new(config: &'a Config, base_dir: &Path) -> Result<Self, RecipeError> {
        // Extract version information and build template variables
        let mut variables = HashMap::new();
        
        // Add project name
        variables.insert("PROJECT".to_string(), config.project.name.clone());
        
        // Add date/time variables
        Self::register_datetime_variables(&mut variables);
        
        // Extract and register version variables
        if let Some(ref version_config) = config.env.version {
            let version_info = Self::extract_version(version_config, base_dir)?;
            Self::register_version_variables(&mut variables, &version_info);
        }
        
        Ok(Self {
            config,
            base_dir: base_dir.to_path_buf(),
            variables,
        })
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
        let rendered_name = self.render_template(output_name, name)?;
        let output_path = output_dir.join(format!("{}.{}", rendered_name, t.output_format.extension()));
        
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
        let rendered_name = self.render_template(output_name, name)?;
        let output_path = output_dir.join(format!("{}.{}", rendered_name, t.output_format.extension()));
        
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
        let rendered_name = self.render_template(output_name, name)?;
        let output_path = output_dir.join(format!("{}.{}", rendered_name, suffix));
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
    
    /// Extract version information from configured source
    fn extract_version(
        version_config: &crate::config::VersionConfig,
        base_dir: &Path,
    ) -> Result<VersionInfo, RecipeError> {
        match version_config.source {
            VersionSource::Header => {
                let file_path = version_config.file.as_ref()
                    .ok_or_else(|| RecipeError::VersionError(
                        VersionError::MissingConfig("file".to_string())
                    ))?;
                
                let full_path = if file_path.is_absolute() {
                    file_path.clone()
                } else {
                    base_dir.join(file_path)
                };
                
                // Validate configuration: either string or (major+minor+patch) required
                if version_config.string.is_none() 
                    && (version_config.major.is_none() 
                        || version_config.minor.is_none() 
                        || version_config.patch.is_none()) {
                    return Err(RecipeError::VersionError(
                        VersionError::MissingConfig(
                            "'string' or 'major'/'minor'/'patch'".to_string()
                        )
                    ));
                }
                
                let extractor = HeaderExtractor::new(
                    full_path,
                    version_config.major.clone(),
                    version_config.minor.clone(),
                    version_config.patch.clone(),
                )
                .with_build(version_config.build.clone())
                .with_pre_release(version_config.pre_release.clone())
                .with_string(version_config.string.clone());
                
                Ok(extractor.extract()?)
            }
            _ => {
                Err(RecipeError::VersionError(
                    VersionError::UnsupportedSource(format!("{:?}", version_config.source))
                ))
            }
        }
    }
    
    /// Register version-related template variables
    fn register_version_variables(vars: &mut HashMap<String, String>, version: &VersionInfo) {
        vars.insert("MAJOR".to_string(), version.major.to_string());
        vars.insert("MINOR".to_string(), version.minor.to_string());
        vars.insert("PATCH".to_string(), version.patch.to_string());
        vars.insert("VERSION".to_string(), version.version_string());
        vars.insert("VERSION_FULL".to_string(), version.full_string());
        
        if let Some(build) = version.build {
            vars.insert("BUILD".to_string(), build.to_string());
        }
        
        if let Some(ref pre) = version.pre_release {
            vars.insert("PRE_RELEASE".to_string(), pre.clone());
        }
        
        if let Some(ref meta) = version.build_metadata {
            vars.insert("BUILD_METADATA".to_string(), meta.clone());
        }
    }
    
    /// Register date/time template variables
    fn register_datetime_variables(vars: &mut HashMap<String, String>) {
        use chrono::Local;
        let now = Local::now();
        
        vars.insert("DATE".to_string(), now.format("%Y%m%d").to_string());
        vars.insert("TIME".to_string(), now.format("%H%M%S").to_string());
        vars.insert("DATETIME".to_string(), now.format("%Y%m%d_%H%M%S").to_string());
        vars.insert("TIMESTAMP".to_string(), now.timestamp().to_string());
    }
    
    /// Render template string with variables
    fn render_template(&self, template: &str, target_name: &str) -> Result<String, RecipeError> {
        let mut result = template.to_string();
        
        // Add target name to temporary variables
        let mut vars = self.variables.clone();
        vars.insert("TARGET".to_string(), target_name.to_string());
        
        // Replace all {VAR} placeholders
        for (key, value) in &vars {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }
        
        // Check for undefined variables (remaining placeholders)
        let re = regex::Regex::new(r"\{([A-Z_][A-Z0-9_]*)\}").unwrap();
        if let Some(cap) = re.captures(&result) {
            let var_name = cap[1].to_string();
            return Err(RecipeError::MissingVariable(var_name));
        }
        
        Ok(result)
    }
}