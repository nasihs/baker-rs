use std::collections::BTreeMap;

use super::FirmwareError;

pub trait ImageReader {
    fn read(&self) -> Result<Image, FirmwareError>;
}

pub trait ImageWriter {
    fn write(&self, image: &Image) -> Result<(), FirmwareError>;
}

// TODO use Addr as u32 ? x64 support?

#[derive(Default)]
pub struct Image {
    segments: BTreeMap<u32, Vec<u8>>,
}

impl Image {
    pub fn new() -> Self {
        Image::default()
    }

    // TODO consider address is repeated? 
    pub fn add_data(&mut self, address: u32, data: Vec<u8>) {
        if !data.is_empty() {  // in case of data.len() == 0 
            self.segments.insert(address, data);
            // TODO there should be a warning
        }
    }

    pub fn segments(&self) -> &BTreeMap<u32, Vec<u8>> {
        &self.segments
    }

    pub fn address_range(&self) -> Option<(u32, u32)> {
        let (start, _) = self.segments.first_key_value()?;  // image is empty ()
        let (last_addr, last_data) =self.segments.last_key_value()?;

        let end = last_addr + last_data.len() as u32 - 1;
        Some((*start, end))
    }

    pub fn data_size(&self) -> usize {
        self.segments.values().map(|v| v.len()).sum()
    }

    pub fn image_size(&self) -> Option<usize> {
        let (start, end) = self.address_range()?;
        Some(end as usize - start as usize + 1)
    }

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    pub fn merge(&mut self, other: &Image) -> Result<(), FirmwareError> {
        let Some((other_start, other_end)) = other.address_range() else {
            return Ok(());
        };

        let other_start = other_start + 0;
        let other_end = other_end + 0;

        if let Some((start, end)) = self.address_range() {
            // if other_start <= end && start <= other_end {
            //     return Err(FirmwareError::AddressOverlap(start));
            // }
        }

        for (addr, data) in other.segments() {
            self.segments.insert(addr + 0, data.clone());
        }
        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_image() {
        let image = Image::new();
        assert!(image.is_empty());
        assert_eq!(image.data_size(), 0);
        assert_eq!(image.image_size(), None);
        assert_eq!(image.address_range(), None);
    }

    #[test]
    fn test_single_segment() {
        let mut image = Image::new();
        image.add_data(0x1000, vec![0x01, 0x02, 0x03, 0x04]);

        assert_eq!(image.data_size(), 4);
        assert_eq!(image.image_size(), Some(4));
        assert_eq!(image.address_range(), Some((0x1000, 0x1003)));
    }

    #[test]
    fn test_multiple_segments_with_gap() {
        let mut image = Image::new();
        image.add_data(0x0000, vec![0x01, 0x02]);  // 2 bytes
        image.add_data(0x1000, vec![0x03, 0x04]);  // 2 bytes, gap exists

        assert_eq!(image.data_size(), 4);                        // actual 4 bytes
        assert_eq!(image.image_size(), Some(0x1002));            // 0x0000 - 0x1001 = 0x1002 bytes
        assert_eq!(image.address_range(), Some((0x0000, 0x1001)));
    }

    #[test]
    fn test_merge_no_overlap() {
        let mut image1 = Image::new();
        image1.add_data(0x0000, vec![0x11; 16]);

        let mut image2 = Image::new();
        image2.add_data(0x1000, vec![0x22; 16]);

        image1.merge(&image2).unwrap();

        assert_eq!(image1.segments().len(), 2);
        assert_eq!(image1.address_range(), Some((0x0000, 0x100F)));
    }

    #[test]
    fn test_merge_with_offset() {
        let mut bootloader = Image::new();
        bootloader.add_data(0x0000, vec![0xAA; 0x100]);

        let mut app = Image::new();
        app.add_data(0x8000, vec![0xBB; 0x100]);

        // App offset to 0x8000
        bootloader.merge(&app).unwrap();

        assert!(bootloader.segments().contains_key(&0x0000));
        assert!(bootloader.segments().contains_key(&0x8000));
    }

    #[test]
    fn test_merge_overlap_error() {
        let mut image1 = Image::new();
        image1.add_data(0x0000, vec![0x11; 0x100]);  // 0x0000 - 0x00FF

        let mut image2 = Image::new();
        image2.add_data(0x0080, vec![0x22; 0x100]);

        // will  overlap
        let result = image1.merge(&image2);

        assert!(result.is_err());
    }

    #[test]
    fn test_merge_empty_image() {
        let mut image1 = Image::new();
        image1.add_data(0x0000, vec![0x11; 16]);

        let image2 = Image::new();  // empty image

        // should success
        image1.merge(&image2).unwrap();

        assert_eq!(image1.segments().len(), 1);
    }

}
