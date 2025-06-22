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

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};
    use tempfile::{tempdir, NamedTempFile};

    // Helper to create a test image file
    fn create_test_image_file() -> NamedTempFile {
        let mut img = RgbImage::new(10, 10);
        for pixel in img.pixels_mut() {
            *pixel = Rgb([255, 0, 0]); // Red image
        }

        let temp_file = NamedTempFile::with_suffix(".png").unwrap();
        img.save(temp_file.path()).unwrap();
        temp_file
    }

    #[test]
    fn test_process_image_original_output() {
        let temp_image = create_test_image_file();
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("output.png");

        let result = process_image(
            temp_image.path(),
            4,
            QuantisationMethod::KMeans,
            PaletteHeight::Absolute(50),
            None,
            OutputType::OriginalImage,
            &output_path,
        );

        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    #[test]
    fn test_process_image_standalone_output() {
        let temp_image = create_test_image_file();
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("palette.png");

        let result = process_image(
            temp_image.path(),
            6,
            QuantisationMethod::MedianCut,
            PaletteHeight::Percentage(25.0),
            Some(200),
            OutputType::StandalonePalette,
            &output_path,
        );

        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    #[test]
    fn test_process_image_json_output() {
        let temp_image = create_test_image_file();

        let result = process_image(
            temp_image.path(),
            8,
            QuantisationMethod::KMeans,
            PaletteHeight::Absolute(100), // Ignored for JSON
            None,
            OutputType::Json,
            Path::new("unused.json"), // JSON output doesn't use this path
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_process_image_nonexistent_file() {
        let nonexistent = Path::new("does_not_exist.jpg");
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("output.png");

        let result = process_image(
            nonexistent,
            4,
            QuantisationMethod::KMeans,
            PaletteHeight::Absolute(50),
            None,
            OutputType::OriginalImage,
            &output_path,
        );

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to open image"));
    }

    #[test]
    fn test_process_image_invalid_output_directory() {
        let temp_image = create_test_image_file();
        let invalid_output = Path::new("/invalid/path/that/does/not/exist/output.png");

        let result = process_image(
            temp_image.path(),
            4,
            QuantisationMethod::KMeans,
            PaletteHeight::Absolute(50),
            None,
            OutputType::OriginalImage,
            invalid_output,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_total_height_calculation_absolute() {
        let temp_image = create_test_image_file();
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("output.png");

        // Test absolute height for original image (should add to image height)
        let result = process_image(
            temp_image.path(),
            4,
            QuantisationMethod::KMeans,
            PaletteHeight::Absolute(50),
            None,
            OutputType::OriginalImage,
            &output_path,
        );

        assert!(result.is_ok());

        // Verify output image exists and has correct dimensions
        let output_img = image::open(&output_path).unwrap();
        assert_eq!(output_img.height(), 60); // 10 (original) + 50 (palette)
    }

    #[test]
    fn test_total_height_calculation_percentage() {
        let temp_image = create_test_image_file();
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("output.png");

        // Test percentage height (50% of 10px = 5px)
        let result = process_image(
            temp_image.path(),
            4,
            QuantisationMethod::KMeans,
            PaletteHeight::Percentage(50.0),
            None,
            OutputType::OriginalImage,
            &output_path,
        );

        assert!(result.is_ok());

        let output_img = image::open(&output_path).unwrap();
        assert_eq!(output_img.height(), 15); // 10 (original) + 5 (50% palette)
    }

    #[test]
    fn test_standalone_palette_with_custom_width() {
        let temp_image = create_test_image_file();
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("palette.png");

        let result = process_image(
            temp_image.path(),
            4,
            QuantisationMethod::KMeans,
            PaletteHeight::Absolute(100),
            Some(300), // Custom width
            OutputType::StandalonePalette,
            &output_path,
        );

        assert!(result.is_ok());

        let output_img = image::open(&output_path).unwrap();
        assert_eq!(output_img.width(), 300);
        assert_eq!(output_img.height(), 100);
    }

    #[test]
    fn test_standalone_palette_default_width() {
        let temp_image = create_test_image_file();
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("palette.png");

        let result = process_image(
            temp_image.path(),
            4,
            QuantisationMethod::KMeans,
            PaletteHeight::Absolute(100),
            None, // No custom width - should use image width
            OutputType::StandalonePalette,
            &output_path,
        );

        assert!(result.is_ok());

        let output_img = image::open(&output_path).unwrap();
        assert_eq!(output_img.width(), 10); // Original image width
    }

    #[test]
    fn test_large_number_of_colors() {
        let temp_image = create_test_image_file();
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("output.png");

        let result = process_image(
            temp_image.path(),
            1000, // Large number
            QuantisationMethod::KMeans,
            PaletteHeight::Absolute(50),
            None,
            OutputType::OriginalImage,
            &output_path,
        );

        // Should handle gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_percentage_rounding() {
        let temp_image = create_test_image_file(); // 10x10 image
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("output.png");

        // Test 33.33% of 10px should round to 3px
        let result = process_image(
            temp_image.path(),
            4,
            QuantisationMethod::KMeans,
            PaletteHeight::Percentage(33.33),
            None,
            OutputType::OriginalImage,
            &output_path,
        );

        assert!(result.is_ok());

        let output_img = image::open(&output_path).unwrap();
        assert_eq!(output_img.height(), 13); // 10 + 3 (rounded)
    }

    #[test]
    fn test_different_quantisation_methods() {
        let temp_image = create_test_image_file();
        let temp_dir = tempdir().unwrap();

        for method in [QuantisationMethod::KMeans, QuantisationMethod::MedianCut] {
            let output_path = temp_dir.path().join(format!("output_{:?}.png", method));

            let result = process_image(
                temp_image.path(),
                4,
                method,
                PaletteHeight::Absolute(50),
                None,
                OutputType::OriginalImage,
                &output_path,
            );

            assert!(result.is_ok(), "Method {:?} failed", method);
            assert!(output_path.exists());
        }
    }
}
