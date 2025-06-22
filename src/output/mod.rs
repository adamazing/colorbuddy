//! Output generation functionality
//!
//! This module handles generating different types of output from extracted
//! color palettes, including JSON, images with palettes, and standalone palette images.

pub mod json;
pub mod image;
pub mod standalone;

// Re-export output functions
pub use json::output_json_palette;
pub use image::save_original_with_palette;
pub use standalone::save_standalone_palette;
