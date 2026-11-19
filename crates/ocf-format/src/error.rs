use thiserror::Error;

#[derive(Debug, Error)]
pub enum FormatError {
    #[error("invalid magic")]
    BadMagic,
    #[error("unsupported version: {0}.{1}")]
    BadVersion(u16, u16),
    #[error("crc mismatch")]
    CrcMismatch,
    #[error("truncated input")]
    Truncated,
    #[error("invalid field: {0}")]
    InvalidField(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, FormatError>;
