use std::collections::BTreeMap;

// TODO 定义 地址类型？？

#[derive(Default)]
pub struct FirmwareImage {
    segments: BTreeMap<u32, Vec<u8>>,
}

impl FirmwareImage {
    pub fn new() -> Self {
        FirmwareImage::default()
    }

    // TODO 考虑 address 是否重复？
    pub fn add_data(&mut self, address: u32, data: Vec<u8>) {
        self.segments.insert(address, data);
    }

    pub fn segments(&self) -> &BTreeMap<u32, Vec<u8>> {
        &self.segments
    }

    pub fn address_range(&self) -> Option<(u32, u32)> {
        let (start, _) = self.segments.first_key_value()?;
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
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_image() {
        let image = FirmwareImage::new();
        assert!(image.is_empty());
        assert_eq!(image.data_size(), 0);
        assert_eq!(image.image_size(), None);
        assert_eq!(image.address_range(), None);
    }

    #[test]
    fn test_single_segment() {
        let mut image = FirmwareImage::new();
        image.add_data(0x1000, vec![0x01, 0x02, 0x03, 0x04]);

        assert_eq!(image.data_size(), 4);
        assert_eq!(image.image_size(), Some(4));
        assert_eq!(image.address_range(), Some((0x1000, 0x1003)));
    }

    #[test]
    fn test_multiple_segments_with_gap() {
        let mut image = FirmwareImage::new();
        image.add_data(0x0000, vec![0x01, 0x02]);  // 2 bytes
        image.add_data(0x1000, vec![0x03, 0x04]);  // 2 bytes, 有间隙

        assert_eq!(image.data_size(), 4);                        // 实际数据 4 字节
        assert_eq!(image.image_size(), Some(0x1002));            // 0x0000 到 0x1001 = 0x1002 字节
        assert_eq!(image.address_range(), Some((0x0000, 0x1001)));
    }
}
