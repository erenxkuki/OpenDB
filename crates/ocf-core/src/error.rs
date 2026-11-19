use thiserror::Error;

#[derive(Debug, Error)]
pub enum OcfError {
    #[error("unsupported codec: {0}")]
    UnsupportedCodec(u32),
    #[error("unsupported transform: {0}")]
    UnsupportedTransform(u32),
    #[error("corrupt data: {0}")]
    CorruptData(String),
    #[error("checksum mismatch")]
    ChecksumMismatch,
    #[error("content hash mismatch")]
    ContentHashMismatch,
    #[error("dependency missing: {0}")]
    DependencyMissing(String),
    #[error("dependency cycle detected")]
    DependencyCycle,
    #[error("dependency depth exceeded: {0}")]
    DepthExceeded(u8),
    #[error("resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    #[error("fidelity violation: {0}")]
    FidelityViolation(String),
    #[error("version incompatible: {0}")]
    VersionIncompatible(String),
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, OcfError>;
