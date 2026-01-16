use std::path::{Path, PathBuf};
use super::super::error::FirmwareError;
use super::super::image::{Image, ImageReader, ImageWriter};

pub struct BinWriter {
    file: PathBuf,
    fill_byte: u8,
}

impl BinWriter {
    pub fn new(path: impl Into<PathBuf>, fill_byte: u8) -> Self {
        Self { file: path.into(), fill_byte }
    }
}

pub struct BinReader {
    file: PathBuf,
    base_addr: u32,
}

impl BinReader {
    pub fn new(path: impl Into<PathBuf>, base_addr: u32) -> Self {
        Self { file: path.into(), base_addr }
    }
}

impl ImageReader for BinReader {
    fn read(&self) -> Result<Image, FirmwareError> {
        read(&self.file, self.base_addr)
    }
}

impl ImageWriter for BinWriter {
    fn write(&self, image: &Image) -> Result<(), FirmwareError> {
        write(image, &self.file, self.fill_byte)
    }
}

fn read(path: &Path, base_addr: u32) -> Result<Image, FirmwareError> {
    let content = std::fs::read(path)?;
    let mut image = Image::new();
    image.add_data(base_addr, content.to_vec());
    Ok(image)
}

fn write(image: &Image, path: &Path, fill_byte: u8) -> Result<(), FirmwareError> {
    let (start, end) = image.address_range().ok_or(FirmwareError::EmptyImage)?;
    let size = (end - start + 1) as usize;
    let mut buffer = vec![fill_byte; size];

    for (addr, data) in image.segments() {
        let offset = (addr - start) as usize;
        buffer[offset..offset + data.len()].copy_from_slice(data);
    }

    std::fs::write(path, buffer)?;
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_read_binary() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(&[0x01, 0x02, 0x03, 0x04]).unwrap();

        let image = read(file.path(), 0x8000).unwrap();

        assert_eq!(image.data_size(), 4);
        assert_eq!(image.address_range(), Some((0x8000, 0x8003)));
    }

    #[test]
    fn test_write_binary_single_segment() {
        let mut image = Image::new();
        image.add_data(0x1000, vec![0xAA, 0xBB, 0xCC]);

        let file = NamedTempFile::new().unwrap();
        write(&image, file.path(), 0xFF).unwrap();

        let data = std::fs::read(file.path()).unwrap();
        assert_eq!(data, vec![0xAA, 0xBB, 0xCC]);
    }

    #[test]
    fn test_write_binary_with_gap() {
        let mut image = Image::new();
        image.add_data(0x0000, vec![0xAA, 0xBB]);
        image.add_data(0x0004, vec![0xCC, 0xDD]);

        let file = NamedTempFile::new().unwrap();
        write(&image, file.path(), 0xFF).unwrap();

        let data = std::fs::read(file.path()).unwrap();
        // [AA, BB, FF, FF, CC, DD]
        assert_eq!(data, vec![0xAA, 0xBB, 0xFF, 0xFF, 0xCC, 0xDD]);
    }

    #[test]
    fn test_roundtrip() {
        let mut image = Image::new();
        image.add_data(0x2000, vec![0x01, 0x02, 0x03, 0x04]);

        let file = NamedTempFile::new().unwrap();
        write(&image, file.path(), 0xFF).unwrap();

        // 读回时用相同基地址
        let image2 = read(file.path(), 0x2000).unwrap();
        assert_eq!(image.data_size(), image2.data_size());
        assert_eq!(image.address_range(), image2.address_range());
    }
}