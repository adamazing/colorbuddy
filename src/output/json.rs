use crate::types::{
    color::{ColorInfo, ImageDimensions, PaletteMetadata, PaletteOutput},
    config::QuantisationMethod,
};
use anyhow::{Context, Result};
use exoquant::Color;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Helper function to extract common JSON generation logic
/// Generates a JSON string representation of a color palette.
///
/// Creates a structured JSON object containing palette metadata and color information.
/// This is a helper function used by both stdout and file output methods to ensure
/// consistent JSON formatting.
///
/// # Arguments
///
/// * `color_palette` - Slice of colors to serialize
/// * `quantization_method` - Algorithm used for palette extraction
/// * `requested_colors` - Number of colors originally requested
/// * `image_dimensions` - Source image width and height as (width, height)
///
/// # Returns
///
/// * `Ok(String)` - Pretty-formatted JSON string
/// * `Err` - If JSON serialization fails
///
/// # Examples
///
/// ```
/// use exoquant::Color;
/// use color_buddy::output::json::generate_palette_json;
/// use color_buddy::types::config::QuantisationMethod;
/// let colors = vec![Color { r: 255, g: 0, b: 0, a: 255 }];
/// let json = generate_palette_json(
///     &colors,
///     QuantisationMethod::KMeans,
///     8,
///     (1920, 1080)
/// )?;
/// assert!(json.contains("\"requested_colors\": 8"));
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn generate_palette_json(
    color_palette: &[Color],
    quantization_method: QuantisationMethod,
    requested_colors: u16,
    image_dimensions: (u32, u32),
) -> Result<String> {
    let colors: Vec<ColorInfo> = color_palette.iter().map(ColorInfo::from_color).collect();

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
    serde_json::to_string_pretty(&output).context("Failed to serialize palette to JSON")
}

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
    let json = generate_palette_json(
        color_palette,
        quantization_method,
        requested_colors,
        image_dimensions,
    )?;
    println!("{}", json);
    Ok(())
}

/// Writes the color palette as formatted JSON to a file.
///
/// Creates a JSON file containing color information for each palette color,
/// including RGB values, alpha channel, and hexadecimal representation.
/// The JSON is formatted with indentation for readability.
///
/// # Arguments
///
/// * `color_palette` - Slice of colors to output as JSON
/// * `quantization_method` - The method used to extract the palette
/// * `requested_colors` - Number of colors originally requested
/// * `image_dimensions` - Dimensions of the source image
/// * `output_path` - Path where the JSON file should be written
///
/// # Returns
///
/// * `Ok(())` - JSON file written successfully
/// * `Err` - If JSON serialization or file writing fails
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use exoquant::Color;
/// use color_buddy::output::json::write_json_palette_to_file;
/// use color_buddy::types::config::QuantisationMethod;
/// let colors = vec![Color { r: 255, g: 128, b: 64, a: 255 }];
/// write_json_palette_to_file(
///     &colors,
///     QuantisationMethod::KMeans,
///     8,
///     (1920, 1080),
///     Path::new("palette.json")
/// )?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn write_json_palette_to_file(
    color_palette: &[Color],
    quantization_method: QuantisationMethod,
    requested_colors: u16,
    image_dimensions: (u32, u32),
    output_path: &Path,
) -> Result<()> {
    let colors: Vec<ColorInfo> = color_palette.iter().map(ColorInfo::from_color).collect();

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
    let json =
        serde_json::to_string_pretty(&output).context("Failed to serialize palette to JSON")?;

    let mut file = File::create(output_path)
        .with_context(|| format!("Failed to create file: {}", output_path.display()))?;

    file.write_all(json.as_bytes())
        .with_context(|| format!("Failed to write JSON to file: {}", output_path.display()))?;

    Ok(())
}
