use serde::{Deserialize, Serialize};
use exoquant::Color;

/// Represents a single color with RGB, alpha, and hex values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorInfo {
    /// Red component (0-255)
    pub r: u8,
    /// Green component (0-255)
    pub g: u8,
    /// Blue component (0-255)
    pub b: u8,
    /// Alpha component (0-255)
    pub a: u8,
    /// Hexadecimal representation (e.g., "#ff8040")
    pub hex: String,
}

impl ColorInfo {
    /// Creates a new ColorInfo from an exoquant Color
    pub fn from_color(color: &Color) -> Self {
        Self {
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
            hex: crate::utils::color_conversion::rgb_to_hex(color.r, color.g, color.b),
        }
    }
}

/// Represents the complete palette output structure
#[derive(Debug, Serialize, Deserialize)]
pub struct PaletteOutput {
    /// Metadata about the palette extraction
    pub metadata: PaletteMetadata,
    /// The extracted colors
    pub colors: Vec<ColorInfo>,
}

/// Metadata about how the palette was generated
#[derive(Debug, Serialize, Deserialize)]
pub struct PaletteMetadata {
    /// Number of colors requested
    pub requested_colors: u16,
    /// Number of colors actually extracted
    pub extracted_colors: u16,
    /// Quantization method used
    pub quantization_method: String,
    /// Source image dimensions
    pub image_dimensions: ImageDimensions,
    /// Timestamp when palette was generated
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

impl PaletteMetadata {
    pub fn new(
        requested_colors: u16,
        extracted_colors: u16,
        quantization_method: String,
        image_dimensions: ImageDimensions,
    ) -> Self {
        Self {
            requested_colors,
            extracted_colors,
            quantization_method,
            image_dimensions,
            generated_at: chrono::Utc::now(),
        }
    }
}

/// Image dimensions for metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
}
