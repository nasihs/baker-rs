use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub project: Project,
    #[serde(default)]
    pub version: Option<VersionConfig>,
    #[serde(default)]
    pub output: OutputConfig,
    #[serde(default)]
    pub bootloaders: HashMap<String, Bootloader>,
    #[serde(default)]
    pub targets: HashMap<String, Target>,
    #[serde(default)]
    pub groups: HashMap<String, Group>,
}

// project info
#[derive(Debug, Deserialize)]
pub struct Project {
    pub name: String,
    pub default: String,
}

/// 版本提取配置
#[derive(Debug, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum VersionConfig {
    /// extract version num from an C/C++ header
    Header {
        file: PathBuf,
        /// 正则表达式，需包含 major, minor, patch 捕获组
        /// 或单个 version 捕获组
        pattern: Option<String>,
    },
    /// 从分离的宏定义提取
    SplitMacro {
        file: PathBuf,
        major: String,
        minor: String,
        patch: Option<String>,
    },
    /// 从 CMake 文件提取
    Cmake {
        file: PathBuf,
    },
    /// 从 Git tag 提取
    Git {
        pattern: Option<String>,
    },
    /// 从 JSON/TOML 文件提取
    File {
        file: PathBuf,
        /// JSON path，如 "version.major"
        path: Option<String>,
    },
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

// fn default_date_format() -> String {
//     "%Y%m%d".to_string()
// }

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
    Ota(OtaTarget),
}

impl Target {

    pub fn description(&self) -> Option<&str> {
        match self {
            Target::Merge(t) => t.description.as_deref(),
            Target::Ota(t) => t.description.as_deref(),
        }
    }

    pub fn output_name(&self) -> Option<&str> {
        match self {
            Target::Merge(t) => t.output_name.as_deref(),
            Target::Ota(t) => t.output_name.as_deref(),
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
pub struct OtaTarget {
    pub description: Option<String>,
    pub header: HeaderType,
    #[serde(default)]
    pub header_def: Option<String>,  // for header: custom only
    pub app_file: PathBuf,
    #[serde(default = "default_fill_byte")]
    pub fill_byte: u8,
    #[serde(default)]
    pub output_format: OutputFormat,  // bin for default
    pub output_name: Option<String>,  // if not defined, use target name as default
    pub output_dir: Option<PathBuf>,
}

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HeaderType {
    #[default]
    None,
    OpenBlt,
    Custom,
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

