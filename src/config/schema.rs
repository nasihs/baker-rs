use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

/// 顶层配置结构
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

/// 项目基本信息
#[derive(Debug, Deserialize)]
pub struct Project {
    pub name: String,
    /// 默认构建的 target 或 group
    pub default: Option<String>,
}

/// 版本提取配置
#[derive(Debug, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum VersionConfig {
    /// 从 C/C++ 头文件提取
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

/// 输出配置
#[derive(Debug, Deserialize)]
pub struct OutputConfig {
    /// 输出目录
    #[serde(default = "default_output_dir")]
    pub dir: PathBuf,
    /// 命名模板
    pub name_template: Option<String>,
    /// 日期格式
    #[serde(default = "default_date_format")]  // TODO
    pub date_format: String,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            dir: default_output_dir(),
            name_template: None,
            date_format: default_date_format(),
        }
    }
}

fn default_output_dir() -> PathBuf {
    PathBuf::from("output")
}

fn default_date_format() -> String {
    "%Y%m%d".to_string()
}

/// Bootloader 定义
#[derive(Debug, Deserialize)]
pub struct Bootloader {
    pub file: PathBuf,
    pub version: Option<String>,
}

/// Target 定义 - 使用 tagged enum
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Target {
    Merge(MergeTarget),
    Ota(OtaTarget),
}

impl Target {
    /// 获取 target 的描述
    pub fn description(&self) -> Option<&str> {
        match self {
            Target::Merge(t) => t.description.as_deref(),
            Target::Ota(t) => t.description.as_deref(),
        }
    }

    /// 获取输出文件名模板
    pub fn output_name(&self) -> Option<&str> {
        match self {
            Target::Merge(t) => t.output_name.as_deref(),
            Target::Ota(t) => t.output_name.as_deref(),
        }
    }
}

/// Merge 类型 Target - 合并 bootloader 和 app
#[derive(Debug, Deserialize)]
pub struct MergeTarget {
    pub description: Option<String>,
    /// App 固件路径
    pub app: PathBuf,
    /// Bootloader 引用名 或 直接路径
    pub bootloader: String,
    /// App 偏移地址
    #[serde(deserialize_with = "deserialize_hex_u32")]
    pub app_offset: u32,
    /// 填充字节
    #[serde(default = "default_fill_byte")]
    pub fill_byte: u8,
    /// 输出格式
    #[serde(default)]
    pub output_format: OutputFormat,
    /// 输出文件名模板
    pub output_name: Option<String>,
}

/// OTA 类型 Target - OTA 打包或纯转换
#[derive(Debug, Deserialize)]
pub struct OtaTarget {
    pub description: Option<String>,
    /// 输入固件路径
    pub input: PathBuf,
    /// Header 类型
    #[serde(default)]
    pub header: HeaderType,
    /// 自定义 header 定义（当 header = custom 时）
    pub header_def: Option<String>,
    /// 输出格式
    #[serde(default)]
    pub output_format: OutputFormat,
    /// 输出文件名模板
    pub output_name: Option<String>,
}

/// Header 类型
#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HeaderType {
    #[default]
    None,
    OpenBlt,
    Custom,
}

/// 输出格式
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

/// Group 定义
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Group {
    /// 简单列表: group_name = ["target1", "target2"]
    Simple(Vec<String>),
    /// 详细定义
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

/// 反序列化十六进制数字 (支持 0x8000 或 32768)
fn deserialize_hex_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum HexOrInt {
        Int(u32),
        Str(String),
    }

    match HexOrInt::deserialize(deserializer)? {
        HexOrInt::Int(v) => Ok(v),
        HexOrInt::Str(s) => {
            let s = s.trim();
            if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
                u32::from_str_radix(hex, 16).map_err(D::Error::custom)
            } else {
                s.parse().map_err(D::Error::custom)
            }
        }
    }
}
