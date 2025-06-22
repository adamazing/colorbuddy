//! Type definitions for Color Buddy
//!
//! This module contains all the data types used throughout the application,
//! including color representations, configuration enums, and error types.

pub mod color;
pub mod config;
pub mod error;

// Re-export commonly used types
pub use color::{ColorInfo, PaletteOutput, PaletteMetadata, ImageDimensions};
pub use config::{OutputType, QuantisationMethod, PaletteHeight};
pub use error::{ColorBuddyError, Result};
