//! Utility functions
//!
//! This module contains helper functions that are used across multiple
//! parts of the application.

pub mod color_conversion;

// Re-export utility functions
pub use color_conversion::{rgb_to_hex, palette_height_parser};
