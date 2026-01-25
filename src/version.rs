mod error;
mod extractor;
mod header;

pub use error::VersionError;
pub use extractor::VersionExtractor;
pub use header::HeaderExtractor;

use std::fmt;

/// Version information extracted from various sources
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionInfo {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub build: Option<u32>,
    pub pre_release: Option<String>,
    pub build_metadata: Option<String>,
}

impl VersionInfo {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            build: None,
            pre_release: None,
            build_metadata: None,
        }
    }

    /// Get version string in "major.minor.patch" format
    pub fn version_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }

    /// Get full semver string with pre-release and build metadata
    pub fn full_string(&self) -> String {
        let mut result = self.version_string();
        
        if let Some(ref pre) = self.pre_release {
            result.push('-');
            result.push_str(pre);
        }
        
        if let Some(ref meta) = self.build_metadata {
            result.push('+');
            result.push_str(meta);
        }
        
        result
    }
}

impl fmt::Display for VersionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.full_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_string() {
        let ver = VersionInfo::new(1, 2, 3);
        assert_eq!(ver.version_string(), "1.2.3");
        assert_eq!(ver.full_string(), "1.2.3");
    }

    #[test]
    fn test_version_with_build() {
        let mut ver = VersionInfo::new(1, 2, 3);
        ver.build = Some(100);
        assert_eq!(ver.version_string(), "1.2.3");
    }

    #[test]
    fn test_version_with_pre_release() {
        let mut ver = VersionInfo::new(1, 2, 3);
        ver.pre_release = Some("beta.2".to_string());
        assert_eq!(ver.full_string(), "1.2.3-beta.2");
    }

    #[test]
    fn test_version_full() {
        let mut ver = VersionInfo::new(1, 2, 3);
        ver.pre_release = Some("beta.2".to_string());
        ver.build_metadata = Some("20260125".to_string());
        assert_eq!(ver.full_string(), "1.2.3-beta.2+20260125");
    }
}
