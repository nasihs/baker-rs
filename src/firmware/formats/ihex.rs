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
            ihex::Record::Data { offset, value } => {  // TODO merge data in continued addr to optimize
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

pub fn write(image: &FirmwareImage, path: &Path) -> Result<(), FirmwareError> {
    let records = build_records(image)?;
    let hex_str = ihex::create_object_file_representation(&records)?;
    std::fs::write(path, hex_str)?;
    Ok(())
}

fn build_records(image: &FirmwareImage) -> Result<Vec<ihex::Record>, FirmwareError> {
    let mut records = Vec::new();
    let mut current_high_addr: Option<u16> = None;
    const BYTES_PER_LINE: usize = 32;

    for (start_addr, data) in image.segments() {
        let mut addr = *start_addr;

        for chunk in data.chunks(BYTES_PER_LINE) {  // TODO or 32 bytes?
            let high_addr = (addr >> 16) as u16;
            if current_high_addr != Some(high_addr) {
                current_high_addr = Some(high_addr);
                records.push(ihex::Record::ExtendedLinearAddress(high_addr));
            }
            records.push(ihex::Record::Data { offset: (addr & 0xFFFF) as u16, value: chunk.to_vec() });
            addr += chunk.len() as u32;
        }
    }

    records.push(ihex::Record::EndOfFile);
    Ok(records)

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

    #[test]
    fn test_write_and_read_back() {
        let mut image = FirmwareImage::new();
        image.add_data(0x1000, vec![0x01, 0x02, 0x03, 0x04]);
        
        let file = NamedTempFile::new().unwrap();
        write(&image, file.path()).unwrap();
        
        // 读回来验证
        let image2 = read(file.path()).unwrap();
        assert_eq!(image.data_size(), image2.data_size());
        assert_eq!(image.address_range(), image2.address_range());
    }

    #[test]
    fn test_write_simple() {
        let mut image = FirmwareImage::new();
        image.add_data(0x0000, vec![0x01, 0x02, 0x03, 0x04]);

        let file = NamedTempFile::new().unwrap();
        write(&image, file.path()).unwrap();

        // 读回验证
        let image2 = read(file.path()).unwrap();
        assert_eq!(image.data_size(), image2.data_size());
        assert_eq!(image.address_range(), image2.address_range());
    }

    #[test]
    fn test_write_large_data() {
        let mut image = FirmwareImage::new();
        image.add_data(0x1000, vec![0xAA; 100]);  // 超过 16 字节

        let file = NamedTempFile::new().unwrap();
        write(&image, file.path()).unwrap();

        let image2 = read(file.path()).unwrap();
        assert_eq!(image.data_size(), image2.data_size());
    }

    #[test]
    fn test_write_high_address() {
        let mut image = FirmwareImage::new();
        image.add_data(0x08000000, vec![0x01, 0x02, 0x03, 0x04]);

        let file = NamedTempFile::new().unwrap();
        write(&image, file.path()).unwrap();

        assert_eq!(image.address_range(), Some((0x08000000, 0x08000003)));
    }

    #[test]
    fn test_roundtrip_multiple_segments() {
        let mut image = FirmwareImage::new();
        image.add_data(0x00000000, vec![0x11; 20]);
        image.add_data(0x00001000, vec![0x22; 20]);
        image.add_data(0x08000000, vec![0x33; 20]);  // 高地址段

        let file = NamedTempFile::new().unwrap();
        write(&image, file.path()).unwrap();

        let image2 = read(file.path()).unwrap();
        assert_eq!(image.data_size(), image2.data_size());
    }

}
