use thiserror::Error;

#[derive(Error, Debug)]
pub enum ColorBuddyError {
    #[error("Image processing failed: {0}")]
    ImageProcessing(#[from] image::ImageError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization failed: {0}")]
    JsonSerialization(#[from] serde_json::Error),

    #[error("Invalid palette configuration: {message}")]
    InvalidPalette { message: String },

    #[error("Quantization failed: {0}")]
    Quantization(String),

    #[error("Invalid color count: {count} (must be 1-256)")]
    InvalidColorCount { count: usize },

    #[error("Invalid palette height: {0}")]
    InvalidPaletteHeight(String),
}

pub type Result<T> = std::result::Result<T, ColorBuddyError>;
