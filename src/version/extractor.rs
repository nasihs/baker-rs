use super::{VersionInfo, VersionError};

/// Trait for extracting version information from different sources
pub trait VersionExtractor {
    /// Extract version information
    fn extract(&self) -> Result<VersionInfo, VersionError>;
}
