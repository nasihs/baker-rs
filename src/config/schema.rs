use std::path::PathBuf;
use std::collections::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub project: Project,
    // #[serde(default)]
    // pub version: Option<VersionConfig>,  // TODO
    #[serde(default)]
    pub env: Env,
    #[serde(default)]
    pub bootloaders: HashMap<String, Bootloader>,
    #[serde(default)]
    pub headers: HashMap<String, HeaderDef>,
    #[serde(default)]
    pub targets: HashMap<String, Target>,
    #[serde(default)]
    pub groups: HashMap<String, Group>,
}

#[derive(Debug, Deserialize)]
pub struct Project {
    pub name: String,
    pub default: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct Env {
    #[serde(default)]
    pub output: OutputConfig,
}

#[derive(Debug, Deserialize)]
pub struct OutputConfig {
    #[serde(default = "default_output_dir")]
    pub dir: PathBuf,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            dir: default_output_dir(),
        }
    }
}

fn default_output_dir() -> PathBuf {
    PathBuf::from("output")
}

#[derive(Debug, Deserialize, Clone)]
pub struct HeaderDef {
    pub def: String,
}

#[derive(Debug, Deserialize)]
pub struct Bootloader {
    pub file: Option<PathBuf>,   // check when recipe runs
    // TODO: u32->Addr
    pub base_addr: u32,  // used to check whether bootloader's base addr is correct when file isn't a bin
    pub app_offset: u32,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Target {
    Merge(MergeTarget),
    Pack(PackTarget),
    Convert(ConvertTarget),
}

impl Target {

    pub fn description(&self) -> Option<&str> {
        match self {
            Target::Merge(t) => t.description.as_deref(),
            Target::Pack(t) => t.description.as_deref(),
            Target::Convert(t) => t.description.as_deref(),
        }
    }

    pub fn output_name(&self) -> Option<&str> {
        match self {
            Target::Merge(t) => t.output_name.as_deref(),
            Target::Pack(t) => t.output_name.as_deref(),
            Target::Convert(t) => t.output_name.as_deref(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct MergeTarget {
    pub description: Option<String>,
    pub bootloader: String,  // refrence of bootloaders
    pub app_file: PathBuf,
    #[serde(default = "default_fill_byte")]
    pub fill_byte: u8,
    
    #[serde(default)]
    pub output_format: OutputFormat,
    pub output_name: Option<String>,  // if not defined, use target name as default
    pub output_dir: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
pub struct ConvertTarget {
    pub description: Option<String>,
    pub input_file: PathBuf,
    
    #[serde(default = "default_fill_byte")]
    pub fill_byte: u8,
    
    #[serde(default)]
    pub output_format: OutputFormat,
    
    pub output_name: Option<String>,
    pub output_dir: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
pub struct PackTarget {
    pub description: Option<String>,
    pub header: String,  // use builtin header type or custom which defined in baker.toml -> headers
    pub app_file: PathBuf,
    #[serde(default = "default_fill_byte")]
    pub fill_byte: u8,
    
    #[serde(default)]
    pub output_format: OutputFormat,  // bin for default
    pub output_name: Option<String>,  // if not defined, use target name as default
    pub output_dir: Option<PathBuf>,
}


#[derive(Debug, Default, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    #[default]
    Bin,
    Hex,
    Srec,
}

impl OutputFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::Bin => "bin",
            OutputFormat::Hex => "hex",
            OutputFormat::Srec => "srec",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Group {
    /// group_name = ["target1", "target2"]
    Simple(Vec<String>),
    Detailed {
        targets: Vec<String>,
        description: Option<String>,
    },
}

impl Group {
    pub fn targets(&self) -> &[String] {
        match self {
            Group::Simple(targets) => targets,
            Group::Detailed { targets, .. } => targets,
        }
    }

    pub fn description(&self) -> Option<&str> {
        match self {
            Group::Simple(_) => None,
            Group::Detailed { description, .. } => description.as_deref(),
        }
    }
}

fn default_fill_byte() -> u8 {
    0xFF
}