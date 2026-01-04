mod error;
mod image;
mod formats;

pub use error::FirmwareError;
pub use image::FirmwareImage;
pub use formats::ihex;
pub use formats::binary;

use std::path::Path;

/// Read firmware image from file, automatically detecting format by extension
pub fn read(path: &Path, base_addr_for_bin: Option<u32>) -> Result<FirmwareImage, FirmwareError> {
    let extension = path.extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    
    match extension.to_lowercase().as_str() {
        "hex" => ihex::read(path),
        "bin" => {
            let base_addr = base_addr_for_bin
                .ok_or_else(|| FirmwareError::InvalidFormat("Binary file requires base address".to_string()))?;
            binary::read(path, base_addr)
        }
        "srec" | "s19" | "s28" | "s37" => {
            Err(FirmwareError::InvalidFormat("SREC format not yet supported".to_string()))
        }
        "elf" | "axf" => {
            Err(FirmwareError::InvalidFormat("ELF format not yet supported".to_string()))
        }
        _ => {
            // Try hex format as default
            ihex::read(path)
        }
    }
}
