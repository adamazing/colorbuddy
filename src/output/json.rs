use anyhow::{Result, Context};
use exoquant::Color;
use crate::types::{
    color::{ColorInfo, PaletteOutput, PaletteMetadata, ImageDimensions},
    config::QuantisationMethod,
};

/// Outputs the color palette as formatted JSON to stdout.
///
/// Prints a JSON object containing color information for each palette color,
/// including RGB values, alpha channel, and hexadecimal representation.
/// The JSON is formatted with indentation for readability.
///
/// # Arguments
///
/// * `color_palette` - Slice of colors to output as JSON
/// * `quantization_method` - The method used to extract the palette
/// * `requested_colors` - Number of colors originally requested
/// * `image_dimensions` - Dimensions of the source image
///
/// # Returns
///
/// * `Ok(())` - JSON output completed successfully
/// * `Err` - If JSON serialization fails
///
/// # Examples
///
/// ```json
/// {
///   "metadata": {
///     "requested_colors": 8,
///     "extracted_colors": 6,
///     "quantization_method": "k-means",
///     "image_dimensions": { "width": 1920, "height": 1080 },
///     "generated_at": "2024-01-15T10:30:00Z"
///   },
///   "colors": [
///     {
///       "r": 255,
///       "g": 128,
///       "b": 64,
///       "a": 255,
///       "hex": "#ff8040"
///     }
///   ]
/// }
/// ```
pub fn output_json_palette(
    color_palette: &[Color],
    quantization_method: QuantisationMethod,
    requested_colors: u16,
    image_dimensions: (u32, u32),
) -> Result<()> {
    let colors: Vec<ColorInfo> = color_palette
        .iter()
        .map(ColorInfo::from_color)
        .collect();

    let metadata = PaletteMetadata::new(
        requested_colors,
        colors.len() as u16,
        quantization_method.to_string(),
        ImageDimensions {
            width: image_dimensions.0,
            height: image_dimensions.1,
        },
    );

    let output = PaletteOutput { metadata, colors };
    let json = serde_json::to_string_pretty(&output).context("Failed to serialize palette to JSON")?;

    println!("{}", json);
    Ok(())
}
