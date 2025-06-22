//! Color palette extraction functionality
//!
//! This module contains the core algorithms for extracting color palettes
//! from images using various quantization methods.

pub mod converter;
pub mod extractor;

// Re-export the main extraction function
pub use extractor::extract_palette;
