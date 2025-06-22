use anyhow::{Context, Result};
use clap::Parser;
use std::path::Path;

use color_buddy::{
    cli::{args::Args, output_path::output_file_name},
    output::{
        image::save_original_with_palette, json::output_json_palette,
        standalone::save_standalone_palette,
    },
    palette::extractor::extract_palette,
    types::config::{OutputType, PaletteHeight, QuantisationMethod},
};

/// Main entry point for the Color Buddy application.
///
/// Parses command line arguments and processes each specified image file.
/// Continues processing remaining images even if individual files fail,
/// logging errors to stderr.
///
/// # Returns
///
/// * `Ok(())` - All images processed successfully (or with handled errors)
/// * `Err` - Only if argument parsing fails
///
/// # Errors
///
/// Returns an error if command line argument parsing fails. Individual
/// image processing errors are logged but don't stop execution.
fn main() -> Result<()> {
    let matches = Args::parse();

    for image in &matches.images {
        let output_file_name =
            output_file_name(image, matches.output.as_ref(), matches.output_type);

        if let Err(e) = process_image(
            image,
            matches.number_of_colors,
            matches.quantisation_method,
            matches.palette_height,
            matches.palette_width,
            matches.output_type,
            &output_file_name,
        ) {
            eprintln!("Error processing image {}: {}", image.display(), e);
            // Continue processing other images instead of failing completely
            continue;
        };
    }

    Ok(())
}

/// Processes a single image file and generates the requested output.
///
/// This is the main processing function that coordinates image loading,
/// palette extraction, and output generation. It handles all three output
/// types: original image with palette, standalone palette, and JSON output.
///
/// # Arguments
///
/// * `file` - Path to the input image file to process
/// * `number_of_colors` - Number of colors to extract for the palette
/// * `quantisation_method` - Algorithm to use for color quantization
/// * `palette_height` - Height specification for the palette (absolute or percentage)
/// * `palette_width` - Optional width for standalone palette output
/// * `output_type` - Type of output to generate (image or JSON)
/// * `output_file_name` - Path where the output should be saved
///
/// # Returns
///
/// * `Ok(())` - Image processed and output generated successfully
/// * `Err` - If any step of the processing fails
///
/// # Errors
///
/// Returns an error if:
/// - The input image file cannot be opened or read
/// - Color palette extraction fails
/// - Output file cannot be written
/// - JSON output fails (though this is currently unlikely)
///
/// Each error includes context about which operation failed and the file path involved.
fn process_image(
    file: &Path,
    number_of_colors: u16,
    quantisation_method: QuantisationMethod,
    palette_height: PaletteHeight,
    palette_width: Option<u32>,
    output_type: OutputType,
    output_file_name: &Path,
) -> Result<()> {
    let dynamic_image =
        image::open(file).with_context(|| format!("Failed to open image: {}", file.display()))?;

    let input_image = dynamic_image.to_rgb8();
    let (input_image_width, input_image_height) = input_image.dimensions();

    let total_height = match (output_type, palette_height) {
        (OutputType::OriginalImage, PaletteHeight::Absolute(a)) => a + input_image_height,
        (OutputType::OriginalImage, PaletteHeight::Percentage(a)) => {
            input_image_height + (a / 100.0 * input_image_height as f32).round() as u32
        }
        (OutputType::StandalonePalette, PaletteHeight::Absolute(a)) => a,
        (OutputType::StandalonePalette, PaletteHeight::Percentage(a)) => {
            (a / 100.0 * input_image_height as f32).round() as u32
        }
        (OutputType::Json, _) => input_image_height,
    };

    let color_palette = extract_palette(&input_image, number_of_colors, quantisation_method)?;

    match output_type {
        OutputType::OriginalImage => {
            save_original_with_palette(
                &input_image,
                &color_palette,
                input_image_width,
                input_image_height,
                total_height,
                number_of_colors,
                output_file_name,
            )?;
        }
        OutputType::StandalonePalette => {
            let standalone_palette_width = palette_width.unwrap_or(input_image_width);
            save_standalone_palette(
                &color_palette,
                standalone_palette_width,
                total_height,
                number_of_colors,
                output_file_name,
            )?;
        }
        OutputType::Json => {
            output_json_palette(
                &color_palette,
                quantisation_method,
                number_of_colors,
                (input_image_width, input_image_height),
            )?;
        }
    }

    Ok(())
}
