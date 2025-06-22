use anyhow::{Context, Result};
use exoquant::Color;
use image::RgbImage;
use std::path::Path;
// use crate::types::error::Result;

/// Saves the original image with a color palette strip appended to the bottom.
///
/// Creates a new image containing the original image with a horizontal strip
/// of palette colors along the bottom. Each color in the palette occupies
/// an equal width portion of the strip.
///
/// # Arguments
///
/// * `input_image` - The original RGB image to process
/// * `color_palette` - Slice of colors to display in the palette strip
/// * `input_image_width` - Width of the original image in pixels
/// * `input_image_height` - Height of the original image in pixels
/// * `total_height` - Total height including the palette strip
/// * `number_of_colors` - Number of colors from the palette to display
/// * `output_file_name` - Path where the output image should be saved
///
/// # Returns
///
/// * `Ok(())` - Image saved successfully
/// * `Err` - If image saving fails
///
/// # Errors
///
/// Returns an error if the image cannot be saved to the specified path,
/// with context about the failed operation.
pub fn save_original_with_palette(
    input_image: &RgbImage,
    color_palette: &[Color],
    input_image_width: u32,
    input_image_height: u32,
    total_height: u32,
    number_of_colors: u16,
    output_file_name: &Path,
) -> Result<()> {
    // Create an image buffer big enough to hold the output image
    let mut imgbuf = image::ImageBuffer::new(input_image_width, total_height);

    // The width of each color in the palette strip
    let color_width = input_image_width / number_of_colors as u32;

    // Clone the original image into the output buffer
    for x in 0..input_image_width {
        for y in 0..input_image_height {
            imgbuf.put_pixel(x, y, *input_image.get_pixel(x, y));
        }
    }

    // Add the palette strip
    for y in input_image_height..total_height {
        for (x0, q) in color_palette
            .iter()
            .enumerate()
            .take(number_of_colors.into())
        {
            let x1 = x0 as u32 * color_width;
            for x2 in 0..color_width {
                imgbuf.put_pixel(x1 + x2, y, image::Rgb([q.r, q.g, q.b]));
            }
        }
    }

    imgbuf
        .save(output_file_name)
        .with_context(|| format!("Failed to save image to {}", output_file_name.display()))?;
    Ok(())
}
