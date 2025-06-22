//! Color Buddy - A command line tool to extract color palettes from images
//!
//! This library provides functionality for extracting color palettes from images
//! using various quantization algorithms and outputting results in multiple formats.

pub mod cli;
pub mod palette;
pub mod output;
pub mod types;
pub mod utils;

// Re-export main types for easier usage
pub use types::{
    color::{ColorInfo, PaletteOutput, PaletteMetadata, ImageDimensions},
    config::{OutputType, QuantisationMethod, PaletteHeight},
    error::ColorBuddyError,
};

// Re-export main functions
pub use palette::extractor::extract_palette;
pub use output::{json::output_json_palette, image::save_original_with_palette, standalone::save_standalone_palette};
pub use cli::args::Args;

