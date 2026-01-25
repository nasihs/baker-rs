use std::path::{PathBuf};
use regex::Regex;
use super::{VersionInfo, VersionError, VersionExtractor};

/// Extractor for C/C++ header files
pub struct HeaderExtractor {
    file_path: PathBuf,
    major_macro: Option<String>,
    minor_macro: Option<String>,
    patch_macro: Option<String>,
    build_macro: Option<String>,
    pre_release_macro: Option<String>,
    string_macro: Option<String>,
}

impl HeaderExtractor {
    pub fn new(
        file_path: PathBuf,
        major_macro: Option<String>,
        minor_macro: Option<String>,
        patch_macro: Option<String>,
    ) -> Self {
        Self {
            file_path,
            major_macro,
            minor_macro,
            patch_macro,
            build_macro: None,
            pre_release_macro: None,
            string_macro: None,
        }
    }

    pub fn with_build(mut self, build_macro: Option<String>) -> Self {
        self.build_macro = build_macro;
        self
    }

    pub fn with_pre_release(mut self, pre_release_macro: Option<String>) -> Self {
        self.pre_release_macro = pre_release_macro;
        self
    }

    pub fn with_string(mut self, string_macro: Option<String>) -> Self {
        self.string_macro = string_macro;
        self
    }

    fn read_header(&self) -> Result<String, VersionError> {
        if !self.file_path.exists() {
            return Err(VersionError::FileNotFound(self.file_path.clone()));
        }
        Ok(std::fs::read_to_string(&self.file_path)?)
    }

    fn extract_macro_value(&self, content: &str, macro_name: &str) -> Option<String> {
        // Match: #define MACRO_NAME value
        // Supports: integers, hex, strings
        let pattern = format!(
            r#"(?m)^\s*#\s*define\s+{}\s+(.+?)(?://.*)?$"#,
            regex::escape(macro_name)
        );
        
        let re = Regex::new(&pattern).ok()?;
        let captures = re.captures(content)?;
        let value = captures.get(1)?.as_str().trim();
        
        Some(value.to_string())
    }

    fn parse_integer(&self, value: &str, macro_name: &str) -> Result<u32, VersionError> {
        let value = value.trim();
        
        // Try hex (0x...)
        if let Some(hex) = value.strip_prefix("0x").or_else(|| value.strip_prefix("0X")) {
            return u32::from_str_radix(hex, 16).map_err(|_| {
                VersionError::InvalidMacroValue {
                    name: macro_name.to_string(),
                    value: value.to_string(),
                    reason: "invalid hexadecimal number".to_string(),
                }
            });
        }
        
        // Try binary (0b...)
        if let Some(bin) = value.strip_prefix("0b").or_else(|| value.strip_prefix("0B")) {
            return u32::from_str_radix(bin, 2).map_err(|_| {
                VersionError::InvalidMacroValue {
                    name: macro_name.to_string(),
                    value: value.to_string(),
                    reason: "invalid binary number".to_string(),
                }
            });
        }
        
        // Try decimal
        value.parse::<u32>().map_err(|_| {
            VersionError::InvalidMacroValue {
                name: macro_name.to_string(),
                value: value.to_string(),
                reason: "not a valid integer".to_string(),
            }
        })
    }

    fn parse_string(&self, value: &str) -> String {
        // Remove quotes if present: "string" -> string
        let value = value.trim();
        if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
            value[1..value.len()-1].to_string()
        } else {
            value.to_string()
        }
    }

    fn parse_semver(&self, version_str: &str) -> Result<VersionInfo, VersionError> {
        // Simple semver parsing: [v]major.minor.patch[-pre_release][+build_metadata]
        let version_str = version_str.trim();
        
        // Strip optional 'v' or 'V' prefix
        let version_str = version_str.strip_prefix('v')
            .or_else(|| version_str.strip_prefix('V'))
            .unwrap_or(version_str);
        
        // Split by '+' to separate build metadata
        let (main_part, build_metadata) = if let Some(pos) = version_str.find('+') {
            let (main, meta) = version_str.split_at(pos);
            (main, Some(meta[1..].to_string()))
        } else {
            (version_str, None)
        };
        
        // Split by '-' to separate pre-release
        let (version_part, pre_release) = if let Some(pos) = main_part.find('-') {
            let (ver, pre) = main_part.split_at(pos);
            (ver, Some(pre[1..].to_string()))
        } else {
            (main_part, None)
        };
        
        // Parse version numbers
        let parts: Vec<&str> = version_part.split('.').collect();
        if parts.len() < 3 {
            return Err(VersionError::ParseError(
                version_str.to_string(),
                "expected format: major.minor.patch".to_string(),
            ));
        }
        
        let major = parts[0].parse::<u32>().map_err(|_| {
            VersionError::ParseError(version_str.to_string(), "invalid major version".to_string())
        })?;
        
        let minor = parts[1].parse::<u32>().map_err(|_| {
            VersionError::ParseError(version_str.to_string(), "invalid minor version".to_string())
        })?;
        
        let patch = parts[2].parse::<u32>().map_err(|_| {
            VersionError::ParseError(version_str.to_string(), "invalid patch version".to_string())
        })?;
        
        Ok(VersionInfo {
            major,
            minor,
            patch,
            build: None,
            pre_release,
            build_metadata,
        })
    }
}

impl VersionExtractor for HeaderExtractor {
    fn extract(&self) -> Result<VersionInfo, VersionError> {
        let content = self.read_header()?;
        
        // Strategy 1: Try to parse from version string macro if available
        if let Some(ref string_macro) = self.string_macro {
            if let Some(value) = self.extract_macro_value(&content, string_macro) {
                let version_str = self.parse_string(&value);
                if let Ok(mut version_info) = self.parse_semver(&version_str) {
                    // Try to supplement with individual fields if not present in string
                    if version_info.build.is_none() {
                        if let Some(ref build_macro) = self.build_macro {
                            if let Some(value) = self.extract_macro_value(&content, build_macro) {
                                if let Ok(build) = self.parse_integer(&value, build_macro) {
                                    version_info.build = Some(build);
                                }
                            }
                        }
                    }
                    return Ok(version_info);
                }
            }
        }
        
        // Strategy 2: Extract from individual macros
        let major = if let Some(ref major_macro) = self.major_macro {
            let major_value = self.extract_macro_value(&content, major_macro)
                .ok_or_else(|| VersionError::MacroNotFound(major_macro.clone()))?;
            self.parse_integer(&major_value, major_macro)?
        } else {
            return Err(VersionError::MissingConfig(
                "'string' or 'major'/'minor'/'patch'".to_string()
            ));
        };
        
        let minor = if let Some(ref minor_macro) = self.minor_macro {
            let minor_value = self.extract_macro_value(&content, minor_macro)
                .ok_or_else(|| VersionError::MacroNotFound(minor_macro.clone()))?;
            self.parse_integer(&minor_value, minor_macro)?
        } else {
            return Err(VersionError::MissingConfig(
                "'string' or 'major'/'minor'/'patch'".to_string()
            ));
        };
        
        let patch = if let Some(ref patch_macro) = self.patch_macro {
            let patch_value = self.extract_macro_value(&content, patch_macro)
                .ok_or_else(|| VersionError::MacroNotFound(patch_macro.clone()))?;
            self.parse_integer(&patch_value, patch_macro)?
        } else {
            return Err(VersionError::MissingConfig(
                "'string' or 'major'/'minor'/'patch'".to_string()
            ));
        };
        
        let build = if let Some(ref build_macro) = self.build_macro {
            self.extract_macro_value(&content, build_macro)
                .and_then(|v| self.parse_integer(&v, build_macro).ok())
        } else {
            None
        };
        
        let pre_release = if let Some(ref pre_macro) = self.pre_release_macro {
            self.extract_macro_value(&content, pre_macro)
                .map(|v| self.parse_string(&v))
        } else {
            None
        };
        
        Ok(VersionInfo {
            major,
            minor,
            patch,
            build,
            pre_release,
            build_metadata: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_header(content: &str) -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("version.h");
        fs::write(&file_path, content).unwrap();
        (dir, file_path)
    }

    #[test]
    fn test_extract_separate_fields() {
        let content = r#"
            #define VERSION_MAJOR 1
            #define VERSION_MINOR 2
            #define VERSION_PATCH 3
            #define BUILD_NUMBER 100
        "#;
        
        let (_dir, path) = create_test_header(content);
        let extractor = HeaderExtractor::new(
            path,
            Some("VERSION_MAJOR".to_string()),
            Some("VERSION_MINOR".to_string()),
            Some("VERSION_PATCH".to_string()),
        ).with_build(Some("BUILD_NUMBER".to_string()));
        
        let version = extractor.extract().unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.build, Some(100));
    }

    #[test]
    fn test_extract_from_string() {
        let content = r#"
            #define VERSION_STRING "1.2.3-beta.2+20260125"
        "#;
        
        let (_dir, path) = create_test_header(content);
        let extractor = HeaderExtractor::new(
            path,
            None,
            None,
            None,
        ).with_string(Some("VERSION_STRING".to_string()));
        
        let version = extractor.extract().unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.pre_release, Some("beta.2".to_string()));
        assert_eq!(version.build_metadata, Some("20260125".to_string()));
    }

    #[test]
    fn test_hex_values() {
        let content = r#"
            #define VER_MAJOR 0x01
            #define VER_MINOR 0x0A
            #define VER_PATCH 0xFF
        "#;
        
        let (_dir, path) = create_test_header(content);
        let extractor = HeaderExtractor::new(
            path,
            Some("VER_MAJOR".to_string()),
            Some("VER_MINOR".to_string()),
            Some("VER_PATCH".to_string()),
        );
        
        let version = extractor.extract().unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 10);
        assert_eq!(version.patch, 255);
    }

    #[test]
    fn test_version_with_v_prefix() {
        let content = r#"
            #define VERSION_STRING "v1.2.3"
        "#;
        
        let (_dir, path) = create_test_header(content);
        let extractor = HeaderExtractor::new(
            path,
            None,
            None,
            None,
        ).with_string(Some("VERSION_STRING".to_string()));
        
        let version = extractor.extract().unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }

    #[test]
    fn test_version_capital_v_prefix() {
        let content = r#"
            #define FW_VERSION "V2.5.10-rc.1"
        "#;
        
        let (_dir, path) = create_test_header(content);
        let extractor = HeaderExtractor::new(
            path,
            None,
            None,
            None,
        ).with_string(Some("FW_VERSION".to_string()));
        
        let version = extractor.extract().unwrap();
        assert_eq!(version.major, 2);
        assert_eq!(version.minor, 5);
        assert_eq!(version.patch, 10);
        assert_eq!(version.pre_release, Some("rc.1".to_string()));
    }

    #[test]
    fn test_string_only_config() {
        let content = r#"
            #define VERSION "1.0.0"
        "#;
        
        let (_dir, path) = create_test_header(content);
        let extractor = HeaderExtractor::new(
            path,
            None,  // No major field
            None,  // No minor field
            None,  // No patch field
        ).with_string(Some("VERSION".to_string()));
        
        let version = extractor.extract().unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 0);
        assert_eq!(version.patch, 0);
    }
}
