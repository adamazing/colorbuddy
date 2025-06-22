use std::path::Path;
use exoquant::Color;
// use anyhow::Context;
use anyhow::{Context, Result};
// use crate::types::error::Result;

/// Saves a standalone color palette as an image file.
///
/// Creates an image containing only the color palette, with each color
/// occupying an equal width vertical strip across the entire image height.
///
/// # Arguments
///
/// * `color_palette` - Slice of colors to display in the palette
/// * `palette_width` - Width of the palette image in pixels
/// * `palette_height` - Height of the palette image in pixels
/// * `number_of_colors` - Number of colors from the palette to display
/// * `output_file_name` - Path where the palette image should be saved
///
/// # Returns
///
/// * `Ok(())` - Palette image saved successfully
/// * `Err` - If image saving fails
///
/// # Errors
///
/// Returns an error if the palette image cannot be saved to the specified path,
/// with context about the failed operation.
pub fn save_standalone_palette(
    color_palette: &[Color],
    palette_width: u32,
    palette_height: u32,
    number_of_colors: u16,
    output_file_name: &Path,
) -> Result<()> {
    let mut imgbuf = image::ImageBuffer::new(palette_width, palette_height);
    let color_width = palette_width / number_of_colors as u32;

    for y in 0..palette_height {
        for (x0, q) in color_palette.iter().enumerate().take(number_of_colors.into()) {
            let x1 = x0 as u32 * color_width;
            for x2 in 0..color_width {
                imgbuf.put_pixel(x1 + x2, y, image::Rgb([q.r, q.g, q.b]));
            }
        }
    }

    imgbuf.save(output_file_name).with_context(|| format!("Failed to save palette to {}", output_file_name.display()))?;

    Ok(())
}
