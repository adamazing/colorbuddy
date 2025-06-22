//! Command-line interface components
//!
//! This module handles all aspects of the CLI including argument parsing,
//! help text generation, and output path management.

pub mod args;
pub mod help;
pub mod output_path;

// Re-export the main CLI interface
pub use args::Args;
