use thiserror::Error;

#[derive(Error, Debug)]
pub enum FirmwareError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("empty firmware image")]
    EmptyImage,

    #[error("failed to parse Intel HEX: {0}")]
    IhexParse(#[from] ihex::ReaderError),

    #[error("failed to write Intel HEX: {0}")]
    IhexWrite(#[from] ihex::WriterError),

    #[error("address overlap at 0x{0:08X}")]
    AddressOverlap(u32),

}

