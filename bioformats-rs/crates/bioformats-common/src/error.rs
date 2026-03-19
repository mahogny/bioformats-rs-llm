/// Shared error type for all bioformats crates.
#[derive(thiserror::Error, Debug)]
pub enum BioFormatsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Format error: {0}")]
    Format(String),
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    #[error("Invalid data: {0}")]
    InvalidData(String),
    #[error("Codec error: {0}")]
    Codec(String),
    #[error("Reader not initialized — call set_id first")]
    NotInitialized,
    #[error("Series index {0} out of range")]
    SeriesOutOfRange(usize),
    #[error("Plane index {0} out of range")]
    PlaneOutOfRange(u32),
}

pub type Result<T> = std::result::Result<T, BioFormatsError>;
