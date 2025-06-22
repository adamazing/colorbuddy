use clap::ValueEnum;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub enum OutputType {
    Json,
    JsonFile,
    OriginalImage,
    StandalonePalette,
}

impl fmt::Display for OutputType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OutputType::Json => write!(f, "json"),
            OutputType::JsonFile => write!(f, "json-file"),
            OutputType::OriginalImage => write!(f, "original-image"),
            OutputType::StandalonePalette => write!(f, "standalone"),
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum QuantisationMethod {
    KMeans,
    MedianCut,
}

impl fmt::Display for QuantisationMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            QuantisationMethod::MedianCut => write!(f, "median-cut"),
            QuantisationMethod::KMeans => write!(f, "k-means"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PaletteHeight {
    Absolute(u32),
    Percentage(f32),
}

// Constants
pub const DEFAULT_PALETTE_HEIGHT: &str = "256";
pub const DEFAULT_NUMBER_OF_COLORS: &str = "8";
pub const DEFAULT_ALPHA_COLOR: u8 = 0xff;
