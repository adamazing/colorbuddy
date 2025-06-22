use crate::cli::help::{about, examples, long_about};
use crate::types::config::{
    OutputType, PaletteHeight, QuantisationMethod, DEFAULT_NUMBER_OF_COLORS, DEFAULT_PALETTE_HEIGHT,
};
use crate::utils::color_conversion::palette_height_parser;
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(author, version, about = about(), long_about = long_about(), after_help = examples())]
pub struct Args {
    #[arg(short = 'm', long = "quantisation-method", default_value_t = QuantisationMethod::KMeans)]
    pub quantisation_method: QuantisationMethod,

    #[arg(short = 'n', long = "number-of-colors", default_value = DEFAULT_NUMBER_OF_COLORS
        , value_parser = clap::value_parser!(u16).range(1..=256))]
    pub number_of_colors: u16,

    #[arg(short = 'o', long = "output", default_value = None)]
    pub output: Option<PathBuf>,

    #[arg(short = 't', long = "output-type", default_value_t = OutputType::OriginalImage)]
    pub output_type: OutputType,

    #[arg(short = 'p',
          long = "palette-height",
          help = "e.g. 100, 100px, 50%",
          long_help = "Specify the height in pixels or as a percentage of the image height (e.g. 100, 100px, 50%)",
          value_parser = palette_height_parser,
          default_value = DEFAULT_PALETTE_HEIGHT)]
    pub palette_height: PaletteHeight,

    #[arg(short = 'w',
          long = "palette-width",
          help = "Used only when generating a standalone palette. Provide a width in pixels. (e.g. 100, 500)",
          default_value = None)]
    pub palette_width: Option<u32>,

    #[arg(help = "Any number of images to process.")]
    pub images: Vec<PathBuf>,
}
