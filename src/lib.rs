//! Color Buddy - A command line tool to extract color palettes from images
//!
//! This library provides functionality for extracting color palettes from images
//! using various quantization algorithms and outputting results in multiple formats.

pub mod cli;
pub mod output;
pub mod palette;
pub mod types;
pub mod utils;

// Re-export main types for easier usage
pub use types::{
    color::{ColorInfo, ImageDimensions, PaletteMetadata, PaletteOutput},
    config::{OutputType, PaletteHeight, QuantisationMethod},
    error::ColorBuddyError,
};

// Re-export main functions
pub use cli::args::Args;
pub use output::{
    image::save_original_with_palette, json::output_json_palette,
    standalone::save_standalone_palette,
};
pub use palette::extractor::extract_palette;
