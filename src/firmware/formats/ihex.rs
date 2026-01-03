use std::path::Path;
use ihex::Reader;
use crate::firmware::{FirmwareError, FirmwareImage};


pub fn read(path: &Path) -> Result<FirmwareImage, FirmwareError> {
    let content = std::fs::read_to_string(path)?;
    let reader = Reader::new(&content);

    let mut image = FirmwareImage::new();
    let mut base_addr: u32 = 0;


    for record in reader {
        let record = record?;

        match record {
            ihex::Record::ExtendedLinearAddress(addr) => {
                base_addr = (addr as u32) << 16;
            }
            ihex::Record::ExtendedSegmentAddress(addr) => {
                base_addr = (addr as u32) << 4;
            }
            ihex::Record::Data { offset, value } => {
                image.add_data(base_addr + offset as u32, value);
            }
            ihex::Record::StartLinearAddress(_) => {  // TODO 
            }
            ihex::Record::StartSegmentAddress { .. } => {
            }
            ihex::Record::EndOfFile => {
                break;
            }
        }
    }

    Ok(image)
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_hex(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    #[test]
    fn test_read_simple() {
        let hex = ":100000000102030405060708090A0B0C0D0E0F1068\n:00000001FF\n";
        let file = create_temp_hex(hex);
        
        let image = read(file.path()).unwrap();
        
        assert_eq!(image.data_size(), 16);
        assert_eq!(image.address_range(), Some((0x0000, 0x000F)));
    }

    #[test]
    fn test_read_with_extended_linear_address() {
        // Base address = 0x0001 << 16 = 0x00010000
        let hex = ":020000040001F9\n:100000000102030405060708090A0B0C0D0E0F1068\n:00000001FF\n";
        let file = create_temp_hex(hex);
        
        let image = read(file.path()).unwrap();
        
        assert_eq!(image.address_range(), Some((0x00010000, 0x0001000F)));
    }

    #[test]
    fn test_read_empty_hex() {
        let hex = ":00000001FF\n";
        let file = create_temp_hex(hex);
        
        let image = read(file.path()).unwrap();
        
        assert!(image.is_empty());
    }

    #[test]
    fn test_read_file_not_found() {
        let result = read(Path::new("nonexistent.hex"));
        assert!(result.is_err());
    }
}
