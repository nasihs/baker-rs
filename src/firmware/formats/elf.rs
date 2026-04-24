use std::path::PathBuf;
use object::elf::{FileHeader32, FileHeader64, PT_LOAD};
use object::read::elf::{FileHeader as ElfFileHeader, ProgramHeader as ElfProgramHeader};
use object::{BigEndian, LittleEndian};
use crate::firmware::error::FirmwareError;
use crate::firmware::image::{Image, ImageReader};

pub struct ElfReader {
    file: PathBuf,
}

impl ElfReader {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { file: path.into() }
    }
}

impl ImageReader for ElfReader {
    fn read(&self) -> Result<Image, FirmwareError> {
        let data = std::fs::read(&self.file)?;
        parse_elf_bytes(&data)
    }
}

fn parse_elf_bytes(data: &[u8]) -> Result<Image, FirmwareError> {
    if data.len() < 6 {
        return Err(FirmwareError::ElfParse("file too small to be ELF".to_string()));
    }
    if &data[0..4] != b"\x7fELF" {
        return Err(FirmwareError::ElfParse("not an ELF file (invalid magic)".to_string()));
    }
    match data[4] {
        1 => parse_elf32(data),
        2 => parse_elf64(data),
        c => Err(FirmwareError::ElfParse(format!("unknown ELF class: {}", c))),
    }
}

fn parse_elf32(data: &[u8]) -> Result<Image, FirmwareError> {
    match data[5] {
        1 => parse_elf32_endian::<LittleEndian>(data),
        2 => parse_elf32_endian::<BigEndian>(data),
        e => Err(FirmwareError::ElfParse(format!("unknown ELF data encoding: {}", e))),
    }
}

fn parse_elf32_endian<E: object::Endian>(data: &[u8]) -> Result<Image, FirmwareError> {
    let header = FileHeader32::<E>::parse(data)
        .map_err(|e| FirmwareError::ElfParse(e.to_string()))?;
    let endian = header.endian()
        .map_err(|e| FirmwareError::ElfParse(e.to_string()))?;
    let segments = header.program_headers(endian, data)
        .map_err(|e| FirmwareError::ElfParse(e.to_string()))?;

    let mut image = Image::new();
    for seg in segments {
        if seg.p_type(endian) != PT_LOAD {
            continue;
        }
        let filesz = seg.p_filesz(endian) as usize;
        if filesz == 0 {
            continue;
        }
        let lma = seg.p_paddr(endian);  // u32 for ELF32
        let offset = seg.p_offset(endian) as usize;
        if offset.checked_add(filesz).map_or(true, |end| end > data.len()) {
            return Err(FirmwareError::ElfParse(
                format!("PT_LOAD segment at LMA 0x{:08X} extends beyond file", lma)
            ));
        }
        image.add_data(lma, data[offset..offset + filesz].to_vec());
    }
    Ok(image)
}

fn parse_elf64(data: &[u8]) -> Result<Image, FirmwareError> {
    match data[5] {
        1 => parse_elf64_endian::<LittleEndian>(data),
        2 => parse_elf64_endian::<BigEndian>(data),
        e => Err(FirmwareError::ElfParse(format!("unknown ELF data encoding: {}", e))),
    }
}

fn parse_elf64_endian<E: object::Endian>(data: &[u8]) -> Result<Image, FirmwareError> {
    let header = FileHeader64::<E>::parse(data)
        .map_err(|e| FirmwareError::ElfParse(e.to_string()))?;
    let endian = header.endian()
        .map_err(|e| FirmwareError::ElfParse(e.to_string()))?;
    let segments = header.program_headers(endian, data)
        .map_err(|e| FirmwareError::ElfParse(e.to_string()))?;

    let mut image = Image::new();
    for seg in segments {
        if seg.p_type(endian) != PT_LOAD {
            continue;
        }
        let filesz = seg.p_filesz(endian) as usize;
        if filesz == 0 {
            continue;
        }
        let lma = seg.p_paddr(endian);  // u64 for ELF64
        let end_addr = lma.checked_add(filesz as u64).unwrap_or(u64::MAX);
        if end_addr > u32::MAX as u64 + 1 {
            return Err(FirmwareError::ElfAddressOverflow(lma));
        }
        let offset = seg.p_offset(endian) as usize;
        if offset.checked_add(filesz).map_or(true, |end| end > data.len()) {
            return Err(FirmwareError::ElfParse(
                format!("PT_LOAD segment at LMA 0x{:016X} extends beyond file", lma)
            ));
        }
        image.add_data(lma as u32, data[offset..offset + filesz].to_vec());
    }
    Ok(image)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal ELF32 LE binary: one PT_LOAD segment with VMA=0x20000000, LMA=0x08000000.
    fn make_elf32_le() -> Vec<u8> {
        vec![
            // e_ident (16 bytes)
            0x7f, 0x45, 0x4c, 0x46, 0x01, 0x01, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // e_type=2, e_machine=40(ARM), e_version=1
            0x02, 0x00, 0x28, 0x00, 0x01, 0x00, 0x00, 0x00,
            // e_entry=0x08000000
            0x00, 0x00, 0x00, 0x08,
            // e_phoff=52, e_shoff=0, e_flags=0
            0x34, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // e_ehsize=52, e_phentsize=32, e_phnum=1, e_shentsize=40, e_shnum=0, e_shstrndx=0
            0x34, 0x00, 0x20, 0x00, 0x01, 0x00, 0x28, 0x00, 0x00, 0x00, 0x00, 0x00,
            // PT_LOAD program header (32 bytes at offset 52)
            // p_type=1, p_offset=84, p_vaddr=0x20000000, p_paddr=0x08000000
            0x01, 0x00, 0x00, 0x00,
            0x54, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x20,
            0x00, 0x00, 0x00, 0x08,
            // p_filesz=4, p_memsz=4, p_flags=5, p_align=4
            0x04, 0x00, 0x00, 0x00,
            0x04, 0x00, 0x00, 0x00,
            0x05, 0x00, 0x00, 0x00,
            0x04, 0x00, 0x00, 0x00,
            // data at offset 84
            0xDE, 0xAD, 0xBE, 0xEF,
        ]
    }

    /// Minimal ELF32 BE binary: same structure, big-endian byte order.
    fn make_elf32_be() -> Vec<u8> {
        vec![
            // e_ident (16 bytes)
            0x7f, 0x45, 0x4c, 0x46, 0x01, 0x02, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // e_type=2 BE, e_machine=8(MIPS) BE, e_version=1 BE
            0x00, 0x02, 0x00, 0x08, 0x00, 0x00, 0x00, 0x01,
            // e_entry=0x08000000 BE
            0x08, 0x00, 0x00, 0x00,
            // e_phoff=52 BE, e_shoff=0, e_flags=0
            0x00, 0x00, 0x00, 0x34, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // e_ehsize=52 BE, e_phentsize=32, e_phnum=1, e_shentsize=40, e_shnum=0, e_shstrndx=0
            0x00, 0x34, 0x00, 0x20, 0x00, 0x01, 0x00, 0x28, 0x00, 0x00, 0x00, 0x00,
            // PT_LOAD program header (32 bytes, big-endian)
            // p_type=1, p_offset=84, p_vaddr=0x20000000, p_paddr=0x08000000
            0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x54,
            0x20, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x00, 0x00,
            // p_filesz=4, p_memsz=4, p_flags=5, p_align=4
            0x00, 0x00, 0x00, 0x04,
            0x00, 0x00, 0x00, 0x04,
            0x00, 0x00, 0x00, 0x05,
            0x00, 0x00, 0x00, 0x04,
            // data at offset 84
            0xDE, 0xAD, 0xBE, 0xEF,
        ]
    }

    #[test]
    fn test_elf32_le_uses_lma_not_vma() {
        // VMA=0x20000000 (RAM), LMA=0x08000000 (flash).
        // Reader must place data at LMA.
        let image = parse_elf_bytes(&make_elf32_le()).unwrap();
        assert_eq!(image.address_range(), Some((0x08000000, 0x08000003)));
        assert_eq!(image.data_size(), 4);
        let seg = &image.segments()[&0x08000000];
        assert_eq!(seg, &[0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn test_elf32_be_uses_lma_not_vma() {
        let image = parse_elf_bytes(&make_elf32_be()).unwrap();
        assert_eq!(image.address_range(), Some((0x08000000, 0x08000003)));
        assert_eq!(image.data_size(), 4);
        let seg = &image.segments()[&0x08000000];
        assert_eq!(seg, &[0xDE, 0xAD, 0xBE, 0xEF]);
    }

    /// ELF32 LE fixture where p_filesz = 0 (models a BSS segment). Bytes 68..72 are zeroed.
    fn make_elf32_le_bss_only() -> Vec<u8> {
        let mut data = make_elf32_le();
        // p_filesz is at absolute offset 68 (52 ELF header + 16 into phdr)
        data[68] = 0x00;
        data[69] = 0x00;
        data[70] = 0x00;
        data[71] = 0x00;
        data
    }

    /// ELF64 LE: one PT_LOAD with parameterised LMA. Header=64B, PhHdr=56B, data at offset 120.
    fn make_elf64_le(lma: u64) -> Vec<u8> {
        let lma = lma.to_le_bytes();
        vec![
            // e_ident (16 bytes): class=64, data=LE
            0x7f, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // e_type=2, e_machine=183(AARCH64), e_version=1
            0x02, 0x00, 0xB7, 0x00, 0x01, 0x00, 0x00, 0x00,
            // e_entry (u64 LE) = 0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // e_phoff (u64 LE) = 64
            0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // e_shoff (u64 LE) = 0
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // e_flags=0, e_ehsize=64, e_phentsize=56, e_phnum=1
            0x00, 0x00, 0x00, 0x00,
            0x40, 0x00, 0x38, 0x00, 0x01, 0x00,
            // e_shentsize=64, e_shnum=0, e_shstrndx=0
            0x40, 0x00, 0x00, 0x00, 0x00, 0x00,
            // PT_LOAD program header (56 bytes at offset 64)
            // p_type=1 (PT_LOAD)
            0x01, 0x00, 0x00, 0x00,
            // p_flags=5  (flags is the 2nd field in ELF64 program headers)
            0x05, 0x00, 0x00, 0x00,
            // p_offset=120 (0x78) as u64 LE
            0x78, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // p_vaddr=0x80000000 as u64 LE (RAM, should be ignored)
            0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00,
            // p_paddr = lma as u64 LE
            lma[0], lma[1], lma[2], lma[3], lma[4], lma[5], lma[6], lma[7],
            // p_filesz=4 as u64 LE
            0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // p_memsz=4, p_align=8
            0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // data at offset 120
            0xDE, 0xAD, 0xBE, 0xEF,
        ]
    }

    #[test]
    fn test_elf32_bss_segment_is_skipped() {
        let image = parse_elf_bytes(&make_elf32_le_bss_only()).unwrap();
        assert!(image.is_empty(), "BSS segment (p_filesz=0) must not add data to image");
    }

    #[test]
    fn test_bad_magic_returns_error() {
        let data = vec![0x00u8; 64];
        let result = parse_elf_bytes(&data);
        assert!(matches!(result, Err(FirmwareError::ElfParse(_))));
    }

    #[test]
    fn test_unknown_elf_class_returns_error() {
        let mut data = make_elf32_le();
        data[4] = 0x03;  // invalid EI_CLASS
        let result = parse_elf_bytes(&data);
        assert!(matches!(result, Err(FirmwareError::ElfParse(_))));
    }

    #[test]
    fn test_elf64_le_within_u32_range() {
        let image = parse_elf_bytes(&make_elf64_le(0x08000000)).unwrap();
        assert_eq!(image.address_range(), Some((0x08000000, 0x08000003)));
        assert_eq!(image.data_size(), 4);
        let seg = &image.segments()[&0x08000000];
        assert_eq!(seg, &[0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn test_elf64_lma_overflow_returns_error() {
        // LMA = 0x1_0000_0000 exceeds u32::MAX
        let result = parse_elf_bytes(&make_elf64_le(0x1_0000_0000u64));
        assert!(
            matches!(result, Err(FirmwareError::ElfAddressOverflow(0x1_0000_0000))),
            "expected ElfAddressOverflow(0x1_0000_0000)"
        );
    }
}
