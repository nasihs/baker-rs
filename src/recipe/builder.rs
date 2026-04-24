use std::path::{Path, PathBuf};
use std::collections::HashMap;
use log::{trace};
use crate::config::{Bootloader, Config, ConvertTarget, MergeTarget, PackTarget, OutputFormat, PostBuildHook, Target, VersionSource};
use crate::firmware::{self, ImageReader, ImageWriter};
use crate::version::{TemplateExtractor, VersionError};
use super::{Recipe, RecipeError, MergeRecipe, PackRecipe, ConvertRecipe, BuiltinHeaders};
use super::hook::HookRunner;
use super::pack::HeaderBuilder;

pub struct RecipeBuilder<'a> {
    config: &'a Config,
    base_dir: PathBuf,
    env: HashMap<String, delbin::Value>,  // Environment variables (unified type)
}

impl<'a> RecipeBuilder<'a> {
    pub fn new(config: &'a Config, base_dir: &Path) -> Result<Self, RecipeError> {
        let mut env = HashMap::new();

        env.insert("PROJECT".to_string(), delbin::Value::String(config.project.name.clone()));
        Self::register_time_variables(&mut env);
        Self::register_git_variables(&mut env);

        if let Some(ref version_config) = config.env.version {
            let ver_vars = Self::extract_version(version_config, base_dir)?;
            Self::register_version_variables(&mut env, ver_vars);
        }

        // Add underscore aliases for all dotted keys so they are usable inside
        // the delbin DSL, which only allows simple identifiers (no dots).
        // Example: "VER.MAJOR" → "VER_MAJOR", "TIME.EPOCH32" → "TIME_EPOCH32".
        Self::register_flat_aliases(&mut env);

        Ok(Self {
            config,
            base_dir: base_dir.to_path_buf(),
            env,
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
            hook: self.build_hook(t.post_build.as_ref(), name),
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
            hook: self.build_hook(t.post_build.as_ref(), name),
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
            dsl,
            self.env.clone()  // Pass environment variables to header builder
        )?;
        
        trace!("Built pack recipe: {}", name);
        Ok(PackRecipe {
            name: name.to_string(),
            description: t.description.clone(),
            app_reader,
            writer,
            output_path,
            header_builder,
            hook: self.build_hook(t.post_build.as_ref(), name),
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
            "elf" | "axf" => Ok(Box::new(firmware::elf::ElfReader::new(path))),
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
    
    /// Extract version variables from the configured source.
    /// Returns a map of `VER.*` keys ready to merge into the env.
    fn extract_version(
        version_config: &crate::config::VersionConfig,
        base_dir: &Path,
    ) -> Result<std::collections::HashMap<String, delbin::Value>, RecipeError> {
        match version_config.source {
            VersionSource::File => {
                let full_path = if version_config.file.is_absolute() {
                    version_config.file.clone()
                } else {
                    base_dir.join(&version_config.file)
                };

                let extractor = TemplateExtractor::new(full_path, version_config.template.clone());
                Ok(extractor.extract_vars()?)
            }
            _ => Err(RecipeError::VersionError(
                VersionError::UnsupportedSource(format!("{:?}", version_config.source))
            )),
        }
    }

    /// Merge extracted version variables (VER.*) into the env map.
    fn register_version_variables(
        env: &mut std::collections::HashMap<String, delbin::Value>,
        ver_vars: std::collections::HashMap<String, delbin::Value>,
    ) {
        env.extend(ver_vars);
    }
    
    /// Register date/time environment variables
    fn register_time_variables(vars: &mut HashMap<String, delbin::Value>) {
        use chrono::Local;
        let now = Local::now();
        let s = |v: String| delbin::Value::String(v);
        vars.insert("TIME.YYYYMMDD".to_string(),   s(now.format("%Y%m%d").to_string()));
        vars.insert("TIME.YYMMDD".to_string(),     s(now.format("%y%m%d").to_string()));
        vars.insert("TIME.HHMMSS".to_string(),     s(now.format("%H%M%S").to_string()));
        vars.insert("TIME.YYMMDDHHMM".to_string(), s(now.format("%y%m%d%H%M").to_string()));
        vars.insert("TIME.DATETIME".to_string(),   s(now.format("%Y%m%d_%H%M%S").to_string()));
        let epoch = now.timestamp() as u64;
        vars.insert("TIME.EPOCH".to_string(),   delbin::Value::U64(epoch));
        vars.insert("TIME.EPOCH32".to_string(), delbin::Value::U32(epoch as u32)); // truncates high bits; safe until year 2106
    }

    /// Register git variables. Only `GIT.HASH` is provided; it is only inserted
    /// when a git repository and HEAD commit are available. If the caller uses
    /// `${GIT.HASH}` outside a git repo, render_template reports MissingVariable.
    fn register_git_variables(vars: &mut HashMap<String, delbin::Value>) {
        use std::process::Command;
        if let Ok(out) = Command::new("git").args(["rev-parse", "--short", "HEAD"]).output() {
            if out.status.success() {
                let hash = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !hash.is_empty() {
                    vars.insert("GIT.HASH".to_string(), delbin::Value::String(hash));
                }
            }
        }
    }
    
    /// Add underscore aliases for all dotted keys (e.g. "VER.MAJOR" → "VER_MAJOR").
    /// Required so dotted baker variables can be referenced in the delbin DSL,
    /// whose identifier grammar does not allow dots.
    fn register_flat_aliases(vars: &mut HashMap<String, delbin::Value>) {
        let aliases: Vec<(String, delbin::Value)> = vars
            .iter()
            .filter(|(k, _)| k.contains('.'))
            .map(|(k, v)| (k.replace('.', "_"), v.clone()))
            .collect();
        for (flat_key, value) in aliases {
            vars.entry(flat_key).or_insert(value);
        }
    }

    fn render_template(&self, template: &str, target_name: &str) -> Result<String, RecipeError> {
        let mut vars = self.env.clone();
        vars.insert("TARGET".to_string(), delbin::Value::String(target_name.to_string()));
        super::render::render_template(&vars, template)
    }

    fn build_hook(&self, cfg: Option<&PostBuildHook>, target_name: &str) -> Option<HookRunner> {
        cfg.map(|hook_cfg| {
            let mut vars = self.env.clone();
            vars.insert("TARGET".to_string(), delbin::Value::String(target_name.to_string()));
            HookRunner {
                command: hook_cfg.command.clone(),
                arg_templates: hook_cfg.args.clone(),
                vars,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::Builder;

    /// Minimal ELF32 LE binary: one PT_LOAD, VMA=0x20000000, LMA=0x08000000, 4 bytes payload.
    fn make_elf32_le_bytes() -> Vec<u8> {
        vec![
            0x7f, 0x45, 0x4c, 0x46, 0x01, 0x01, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x02, 0x00, 0x28, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x08,
            0x34, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x34, 0x00, 0x20, 0x00, 0x01, 0x00, 0x28, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0x54, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x20,
            0x00, 0x00, 0x00, 0x08,
            0x04, 0x00, 0x00, 0x00,
            0x04, 0x00, 0x00, 0x00,
            0x05, 0x00, 0x00, 0x00,
            0x04, 0x00, 0x00, 0x00,
            0xDE, 0xAD, 0xBE, 0xEF,
        ]
    }

    fn write_temp_elf(suffix: &str) -> tempfile::NamedTempFile {
        let mut f = Builder::new().suffix(suffix).tempfile().unwrap();
        f.write_all(&make_elf32_le_bytes()).unwrap();
        f
    }

    #[test]
    fn test_create_reader_dispatches_elf_extension() {
        let f = write_temp_elf(".elf");
        let config: crate::config::Config = toml::from_str(
            "[project]\nname = \"test\"\ndefault = \"t\"\n"
        ).unwrap();
        let builder = RecipeBuilder::new(&config, f.path().parent().unwrap()).unwrap();
        let reader = builder.create_reader(f.path(), None).unwrap();
        let image = reader.read().unwrap();
        assert_eq!(image.address_range(), Some((0x08000000, 0x08000003)));
    }

    #[test]
    fn test_create_reader_dispatches_axf_extension() {
        let f = write_temp_elf(".axf");
        let config: crate::config::Config = toml::from_str(
            "[project]\nname = \"test\"\ndefault = \"t\"\n"
        ).unwrap();
        let builder = RecipeBuilder::new(&config, f.path().parent().unwrap()).unwrap();
        let reader = builder.create_reader(f.path(), None).unwrap();
        let image = reader.read().unwrap();
        assert_eq!(image.address_range(), Some((0x08000000, 0x08000003)));
    }
}