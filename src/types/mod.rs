//! Type definitions for Color Buddy
//!
//! This module contains all the data types used throughout the application,
//! including color representations, configuration enums, and error types.

pub mod color;
pub mod config;
pub mod error;

// Re-export commonly used types
pub use color::{ColorInfo, ImageDimensions, PaletteMetadata, PaletteOutput};
pub use config::{OutputType, PaletteHeight, QuantisationMethod};
pub use error::{ColorBuddyError, Result};
