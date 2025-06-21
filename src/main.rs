use std::{fmt, fmt::Write};
use std::path::*;

use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use console::style;
use console::Color as ConsoleColor;
use exoquant::{generate_palette, optimizer, Color, Histogram, SimpleColorSpace};
use image::RgbImage;
use mcq::ColorNode;
use mcq::MMCQ;

const DEFAULT_PALETTE_HEIGHT: &str = "256";
const DEFAULT_NUMBER_OF_COLORS: &str = "8";
const DEFAULT_ALPHA_COLOR: u8 = 0xff;

/// Represents a single color with RGB, alpha, and hex values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorInfo {
    /// Red component (0-255)
    pub r: u8,
    /// Green component (0-255)
    pub g: u8,
    /// Blue component (0-255)
    pub b: u8,
    /// Alpha component (0-255)
    pub a: u8,
    /// Hexadecimal representation (e.g., "#ff8040")
    pub hex: String,
}

impl ColorInfo {
    /// Creates a new ColorInfo from an exoquant Color
    pub fn from_color(color: &Color) -> Self {
        Self {
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
            hex: rgb_to_hex(color.r, color.g, color.b),
        }
    }
}

/// Represents the complete palette output structure
#[derive(Debug, Serialize, Deserialize)]
pub struct PaletteOutput {
    /// Metadata about the palette extraction
    pub metadata: PaletteMetadata,
    /// The extracted colors
    pub colors: Vec<ColorInfo>,
}

// Even simpler approach - let chrono handle the serialization automatically:
impl PaletteMetadata {
    pub fn new(
        requested_colors: usize,
        extracted_colors: usize,
        quantization_method: String,
        image_dimensions: ImageDimensions,
    ) -> Self {
        Self {
            requested_colors,
            extracted_colors,
            quantization_method,
            image_dimensions,
            generated_at: chrono::Utc::now(),
        }
    }
}

/// Metadata about how the palette was generated
#[derive(Debug, Serialize, Deserialize)]
pub struct PaletteMetadata {
    /// Number of colors requested
    pub requested_colors: usize,
    /// Number of colors actually extracted
    pub extracted_colors: usize,
    /// Quantization method used
    pub quantization_method: String,
    /// Source image dimensions
    pub image_dimensions: ImageDimensions,
    /// Timestamp when palette was generated (using chrono's built-in serialization)
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

/// Image dimensions for metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
enum OutputType {
    Json,
    OriginalImage,
    StandalonePalette,
}

impl fmt::Display for OutputType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OutputType::Json => write!(f, "json"),
            OutputType::OriginalImage => write!(f, "original-image"),
            OutputType::StandalonePalette => write!(f, "standalone"),
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum QuantisationMethod {
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
enum PaletteHeight {
    Absolute(u32),
    Percentage(f32),
}

struct Example {
    description: String,
    example: String,
}

/// Generates example usage text for the CLI help system.
///
/// Creates formatted examples showing different ways to use the colorbuddy tool,
/// including JSON output, image output with palettes, and various sizing options.
///
/// # Returns
///
/// A formatted string containing styled examples for display in CLI help.
fn examples() -> String {
    let examples = [
        Example {
            description: "Generate JSON containing the 8 most prevalent colors in the image:".to_string(),
            example: "colorbuddy --output-type json original-image.jpg".to_string(),
        },
        Example {
            description: "Output the original images with a palette of the 5 most prevalent colors along the bottom:".to_string(),
            example: "colorbuddy --number-of-colors 5 --output-type original-image.jpg another-image.jpg".to_string()
        },
        Example {
            description: "Specify the height of the palette as a percentage of the original image's height:".to_string(),
            example: "colorbuddy --palette-height 20% original-image.jpg".to_string()
        },
        Example {
            description: "Specify a width, height, and the standalone-palette output height to create a standalone palette image:".to_string(),
            example: "colorbuddy --palette-height 50px --palette-width 500 original-image.jpg".to_string()
        }
    ];

    let formatted_examples = examples
        .iter()
        .fold(String::new(), |mut out, ex| {
            let _ = write!(
                out,
                "  {}\n     {}\n\n",
                style(ex.description.to_owned()).italic(),
                style(ex.example.to_owned()).white()
            );
            out
        });

    format!(
        "{}\n{}",
        style("Examples:").underlined(),
        formatted_examples
    )
}

/// Creates a rainbow-colored string for terminal display.
///
/// Takes a string and applies cycling colors to each alphabetic character,
/// creating a rainbow effect for terminal output. Non-alphabetic characters
/// remain unstyled.
///
/// # Arguments
///
/// * `s` - The string to apply rainbow coloring to
///
/// # Returns
///
/// A styled string with rainbow coloring applied to alphabetic characters.
///
/// # Examples
///
/// ```
/// # use crate::rainbow;
/// let colored = rainbow("Color Buddy");
/// println!("{}", colored); // Displays with rainbow colors
/// ```
fn rainbow(s: &str) -> String {
    let mut colored_string = String::new();
    let colors = [
        ConsoleColor::Red,
        ConsoleColor::Magenta,
        ConsoleColor::Blue,
        ConsoleColor::Cyan,
        ConsoleColor::Green,
        ConsoleColor::Yellow,
        ConsoleColor::Green,
        ConsoleColor::Cyan,
        ConsoleColor::Blue,
        ConsoleColor::Magenta,
    ];

    let mut color_index = 0;

    for c in s.chars() {
        let colored_char = if c.is_ascii_alphabetic() {
            let color = colors[color_index];
            color_index = (color_index + 1) % colors.len();
            style(c.to_string()).fg(color)
        } else {
            style(c.to_string())
        };
        colored_string.push_str(&colored_char.to_string());
    }

    colored_string
}

/// Generates the long description text for the CLI about section.
///
/// Creates a detailed description of the tool's capabilities, including
/// supported algorithms and output formats.
///
/// # Returns
///
/// A formatted string containing the complete tool description.
fn long_about() -> String {
    format!(
        "{}
It uses one of two algorithms to calculate the palette: K-Means, or Median Cut.\n
You can generate:
  - a standalone image containing the palette colors
  - a json file containing the color details in:
     - HEX notation (e.g. #1a6b3f); and
     - the individual R,G, and B components;
  - a copy of the original image with the palette of colors along the bottom of the image.",
        about()
    )
}

/// Generates the short about text for the CLI.
///
/// Creates a brief description with the tool name and basic purpose,
/// including a rainbow-styled title.
///
/// # Returns
///
/// A formatted string containing the short tool description.
fn about() -> String {
    format!(
        "\n{}\n\ncolorbuddy is a command line tool to extract a palette of colors from any image.",
        style(rainbow("Color Buddy 🎨"))
    )
}

#[derive(Debug, Parser)]
#[command(author, version, about = about(), long_about = long_about(), after_help = examples())]
struct Args {
    #[arg(short = 'm', long = "quantisation-method", default_value_t = QuantisationMethod::KMeans)]
    quantisation_method: QuantisationMethod,

    #[arg(short = 'n', long = "number-of-colors", default_value = DEFAULT_NUMBER_OF_COLORS)]
    number_of_colors: usize,

    #[arg(short = 'o', long = "output", default_value = None)]
    output: Option<PathBuf>,

    #[arg(short = 't', long = "output-type", default_value_t = OutputType::OriginalImage)]
    output_type: OutputType,

    #[arg(short = 'p',
          long = "palette-height",
          help = "e.g. 100, 100px, 50%",
          long_help = "Specify the height in pixels or as a percentage of the image height (e.g. 100, 100px, 50%)",
          value_parser = palette_height_parser,
          default_value = DEFAULT_PALETTE_HEIGHT)]
    palette_height: PaletteHeight,

    #[arg(short = 'w',
          long = "palette-width",
          help = "Used only when generating a standalone palette. Provide a width in pixels. (e.g. 100, 500)",
          default_value = None)]
    palette_width: Option<u32>,

    #[arg(help = "Any number of images to process.")]
    images: Vec<PathBuf>,
}

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

/// Converts MCQ ColorNode objects to exoquant Color objects.
///
/// The MCQ (Modified Median Cut Quantization) library uses `ColorNode` objects,
/// while the exoquant library uses `Color` objects. This function performs
/// the conversion between these two representations.
///
/// # Arguments
///
/// * `mcq_color_nodes` - Vector of ColorNode objects from the MCQ library
///
/// # Returns
///
/// Vector of Color objects compatible with the exoquant library, with
/// alpha channel set to the default value.
///
/// # Examples
///
/// ```
/// # use crate::{mcq_color_nodes_to_exoquant_colors, DEFAULT_ALPHA_COLOR};
/// # use mcq::ColorNode;
/// # use exoquant::Color;
/// let mcq_colors = vec![ColorNode { red: 255, grn: 0, blu: 0, rgb: 0, cnt: 1 }];
/// let exoquant_colors = mcq_color_nodes_to_exoquant_colors(mcq_colors);
/// assert_eq!(exoquant_colors[0].r, 255);
/// assert_eq!(exoquant_colors[0].a, DEFAULT_ALPHA_COLOR);
/// ```
fn mcq_color_nodes_to_exoquant_colors(mcq_color_nodes: Vec<ColorNode>) -> Vec<Color> {
    mcq_color_nodes
        .iter()
        .map(|c| Color {
            r: c.red,
            g: c.grn,
            b: c.blu,
            a: DEFAULT_ALPHA_COLOR,
        })
        .collect()
}

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
fn save_original_with_palette(
    input_image: &RgbImage,
    color_palette: &[Color],
    input_image_width: u32,
    input_image_height: u32,
    total_height: u32,
    number_of_colors: usize,
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
        for (x0, q) in color_palette.iter().enumerate().take(number_of_colors) {
            let x1 = x0 as u32 * color_width;
            for x2 in 0..color_width {
                imgbuf.put_pixel(x1 + x2, y, image::Rgb([q.r, q.g, q.b]));
            }
        }
    }

    imgbuf.save(output_file_name)
        .with_context(|| format!("Failed to save image to {}", output_file_name.display()))?;

    Ok(())
}

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
fn save_standalone_palette(
    color_palette: &[Color],
    palette_width: u32,
    palette_height: u32,
    number_of_colors: usize,
    output_file_name: &Path,
) -> Result<()> {
    let mut imgbuf = image::ImageBuffer::new(palette_width, palette_height);
    let color_width = palette_width / number_of_colors as u32;

    for y in 0..palette_height {
        for (x0, q) in color_palette.iter().enumerate().take(number_of_colors) {
            let x1 = x0 as u32 * color_width;
            for x2 in 0..color_width {
                imgbuf.put_pixel(x1 + x2, y, image::Rgb([q.r, q.g, q.b]));
            }
        }
    }

    imgbuf.save(output_file_name)
        .with_context(|| format!("Failed to save palette to {}", output_file_name.display()))?;

    Ok(())
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
fn output_json_palette(
    color_palette: &[Color],
    quantization_method: QuantisationMethod,
    requested_colors: usize,
    image_dimensions: (u32, u32),
) -> Result<()> {
    let colors: Vec<ColorInfo> = color_palette
        .iter()
        .map(ColorInfo::from_color)
        .collect();

    let metadata = PaletteMetadata {
        requested_colors,
        extracted_colors: colors.len(),
        quantization_method: quantization_method.to_string(),
        image_dimensions: ImageDimensions {
            width: image_dimensions.0,
            height: image_dimensions.1,
        },
        generated_at: chrono::Utc::now(),
    };

    let output = PaletteOutput { metadata, colors };
    let json = serde_json::to_string_pretty(&output)
        .context("Failed to serialize palette to JSON")?;

    println!("{}", json);
    Ok(())
}

/// Extracts a color palette from an RGB image using the specified quantization method.
///
/// Uses either K-Means clustering or Median Cut quantization to extract the most
/// representative colors from the input image. The function is optimized to avoid
/// unnecessary memory allocation and cloning.
///
/// # Arguments
///
/// * `input_image` - The RGB image to analyze for color extraction
/// * `number_of_colors` - Number of colors to include in the extracted palette
/// * `quantisation_method` - Algorithm to use for color quantization
///
/// # Returns
///
/// * `Ok(Vec<Color>)` - Vector of extracted palette colors
/// * `Err` - If quantization fails or number_of_colors is invalid
///
/// # Errors
///
/// Returns an error if:
/// - The number of colors is too large for the quantization algorithm
/// - The quantization process fails
///
/// # Examples
///
/// ```
/// # use crate::{extract_palette, QuantisationMethod};
/// # use image::RgbImage;
/// # let image = RgbImage::new(10, 10);
/// let palette = extract_palette(&image, 8, QuantisationMethod::KMeans)?;
/// assert!(palette.len() <= 8);
/// ```
fn extract_palette(
    input_image: &RgbImage,
    number_of_colors: usize,
    quantisation_method: QuantisationMethod,
) -> Result<Vec<Color>> {
    match quantisation_method {
        QuantisationMethod::MedianCut => {
            let color_count = number_of_colors.try_into()
                .with_context(|| "Number of colors is too large for median cut algorithm")?;

            // Convert pixels directly to RGBA format without cloning the entire image
            let rgba_data: Vec<u8> = input_image
                .pixels()
                .flat_map(|pixel| [pixel[0], pixel[1], pixel[2], DEFAULT_ALPHA_COLOR])
                .collect();

            let mcq = MMCQ::from_pixels_u8_rgba(&rgba_data, color_count);
            Ok(mcq_color_nodes_to_exoquant_colors(mcq.get_quantized_colors().to_vec()))
        }
        QuantisationMethod::KMeans => {
            let histogram: Histogram = input_image
                .pixels()
                .map(|p| Color {
                    r: p[0],
                    g: p[1],
                    b: p[2],
                    a: DEFAULT_ALPHA_COLOR,
                })
                .collect();
            Ok(generate_palette(
                &histogram,
                &SimpleColorSpace::default(),
                &optimizer::KMeans,
                number_of_colors,
            ))
        }
    }
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
    number_of_colors: usize,
    quantisation_method: QuantisationMethod,
    palette_height: PaletteHeight,
    palette_width: Option<u32>,
    output_type: OutputType,
    output_file_name: &Path,
) -> Result<()> {
    let dynamic_image = image::open(file)
        .with_context(|| format!("Failed to open image: {}", file.display()))?;

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

    let color_palette: Vec<Color> = extract_palette(&input_image, number_of_colors, quantisation_method)?;

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

/// Generates an output file path based on the original file and output settings.
///
/// Creates an appropriate output file path by appending "_palette" to the original
/// filename and adjusting the extension based on the output type. Handles both
/// explicit output paths and automatic path generation.
///
/// # Arguments
///
/// * `original_file` - Path to the original input image file
/// * `output` - Optional explicit output path specified by the user
/// * `output_type` - Type of output being generated (affects file extension)
///
/// # Returns
///
/// A `PathBuf` representing the complete output file path with appropriate
/// filename and extension.
///
/// # Behavior
///
/// - If `output` is a file path: Uses that directory with generated filename
/// - If `output` is a directory: Places generated filename in that directory
/// - If `output` is `None`: Uses original file's directory with generated filename
/// - Extensions: Preserves original for images, uses "json" for JSON output
///
/// # Examples
///
/// ```
/// use std::path::{Path, PathBuf};
/// # use crate::{output_file_name, OutputType};
///
/// let original = Path::new("photo.jpg");
///
/// // No output path specified, original image type
/// let result = output_file_name(original, None, OutputType::OriginalImage);
/// assert_eq!(result, PathBuf::from("photo_palette.jpg"));
///
/// // Output directory specified, JSON type
/// let output_dir = PathBuf::from("/tmp/");
/// let result = output_file_name(original, Some(&output_dir), OutputType::Json);
/// assert_eq!(result, PathBuf::from("/tmp/photo_palette.json"));
///
/// // Specific output file specified
/// let output_file = PathBuf::from("/output/custom.png");
/// let result = output_file_name(original, Some(&output_file), OutputType::StandalonePalette);
/// assert_eq!(result, PathBuf::from("/output/photo_palette.jpg"));
/// ```
fn output_file_name(
    original_file: &Path,
    output: Option<&PathBuf>,
    output_type: OutputType,
) -> PathBuf {
    let original_image_stem = original_file.file_stem().unwrap().to_str().unwrap();
    let new_extension = match output_type {
        OutputType::OriginalImage => match original_file.extension() {
            Some(ext) => ext.to_str().unwrap(),
            None => "png",
        },
        OutputType::StandalonePalette => match original_file.extension() {
            Some(ext) => ext.to_str().unwrap(),
            None => "png",
        },
        OutputType::Json => "json",
    };
    let file_name = format!("{original_image_stem}_palette.{new_extension}");

    match output {
        Some(p) if !p.is_dir() => PathBuf::from(p).with_file_name(file_name),
        Some(p) if p.is_dir() => PathBuf::from(p).join(file_name),
        _ => PathBuf::from(original_file).with_file_name(file_name),
    }
}

/// Converts RGB color values to a hexadecimal color string.
///
/// Takes individual red, green, and blue color components and formats them
/// as a lowercase hexadecimal color string with a leading hash symbol.
///
/// # Arguments
///
/// * `red` - Red color component (0-255)
/// * `green` - Green color component (0-255)
/// * `blue` - Blue color component (0-255)
///
/// # Returns
///
/// A hexadecimal color string in the format "#rrggbb" where each component
/// is represented as a two-digit lowercase hexadecimal value.
///
/// # Examples
///
/// ```
/// # use crate::rgb_to_hex;
/// assert_eq!(rgb_to_hex(255, 128, 64), "#ff8040");
/// assert_eq!(rgb_to_hex(0, 0, 0), "#000000");
/// assert_eq!(rgb_to_hex(255, 255, 255), "#ffffff");
/// ```
fn rgb_to_hex(red: u8, green: u8, blue: u8) -> String {
    format!("#{red:02x}{green:02x}{blue:02x}")
}

/// Parses a palette height specification string into a PaletteHeight enum.
///
/// Used by the clap argument parser to convert user input into the appropriate
/// palette height representation. Supports absolute pixel values and percentage
/// values relative to the original image height.
///
/// # Arguments
///
/// * `s` - String specification of palette height from command line
///
/// # Returns
///
/// * `Ok(PaletteHeight::Absolute(n))` - For pixel values (e.g., "100", "100px")
/// * `Ok(PaletteHeight::Percentage(n))` - For percentage values (e.g., "50%")
/// * `Err(String)` - If the input format is invalid
///
/// # Supported Formats
///
/// - `"123"` - Absolute pixels (no unit suffix)
/// - `"123px"` - Absolute pixels (with px suffix)
/// - `"50%"` - Percentage of original image height (0-100%)
///
/// # Errors
///
/// Returns an error string if:
/// - Percentage value is greater than 100%
/// - Pixel value is not a positive integer
/// - Input contains invalid characters or format
///
/// # Examples
///
/// ```
/// # use crate::{palette_height_parser, PaletteHeight};
/// assert_eq!(palette_height_parser("100"), Ok(PaletteHeight::Absolute(100)));
/// assert_eq!(palette_height_parser("50%"), Ok(PaletteHeight::Percentage(50.0)));
/// assert!(palette_height_parser("150%").is_err());
/// ```
fn palette_height_parser(s: &str) -> Result<PaletteHeight, String> {
    if s.ends_with('%') {
        let percentage = &s[0..s.len() - 1];
        match percentage.parse::<f32>() {
            Ok(n) if n <= 100.0 => Ok(PaletteHeight::Percentage(n)),
            _ => Err("Percentage must be between 0 and 100".to_owned()),
        }
    } else if s.ends_with("px") {
        let pixels = &s[0..s.len() - 2];
        match pixels.parse::<u32>() {
            Ok(n) => Ok(PaletteHeight::Absolute(n)),
            _ => Err("Pixels must be a positive integer".to_owned()),
        }
    } else {
        match s.parse::<u32>() {
            Ok(n) => Ok(PaletteHeight::Absolute(n)),
            _ => Err("Pixels must be a positive integer".to_owned()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_file_name() {
        let original_file = Path::new("path/to/original/some_file.png");

        // Test case 1: Output path provided
        let output_path = PathBuf::from("path/to/output/something.jpg");
        let output_type = OutputType::OriginalImage;
        let result = output_file_name(&original_file, Some(&output_path), output_type);
        let expected_result = PathBuf::from("path/to/output/some_file_palette.png");
        assert_eq!(result, expected_result);

        // Test case 2: Output path not provided
        let output_type = OutputType::OriginalImage;
        let result = output_file_name(&original_file, None, output_type);
        let expected_result = PathBuf::from("path/to/original/some_file_palette.png");
        assert_eq!(result, expected_result);

        // Test case 3: Output path provided and OutputType is json
        let output_path = PathBuf::from("path/to/output/something.jpg");
        let output_type = OutputType::Json;
        let result = output_file_name(&original_file, Some(&output_path), output_type);
        let expected_result = PathBuf::from("path/to/output/some_file_palette.json");
        assert_eq!(result, expected_result);

        // Test case 4: Output path not provided and OutputType is json
        let output_type = OutputType::Json;
        let result = output_file_name(&original_file, None, output_type);
        let expected_result = PathBuf::from("path/to/original/some_file_palette.json");
        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_palette_height_parser() {
        // Test case 0: Missing units (pixels assumed)
        let input = "235";
        let result = palette_height_parser(input);
        let expected_result = Ok(PaletteHeight::Absolute(235));
        assert_eq!(result, expected_result);

        // Test case 1: Valid absolute value (pixels specified)
        let input = "130px";
        let result = palette_height_parser(input);
        let expected_result = Ok(PaletteHeight::Absolute(130));
        assert_eq!(result, expected_result);

        // Test case 2: Valid percentage value
        let input = "50%";
        let result = palette_height_parser(input);
        let expected_result = Ok(PaletteHeight::Percentage(50.0));
        assert_eq!(result, expected_result);

        // Test case 3: Invalid percentage value
        let input = "150%";
        let result = palette_height_parser(input);
        let expected_result = Err(String::from("Percentage must be between 0 and 100"));
        assert_eq!(result, expected_result);

        // Test case 4: Invalid input
        let input = "foo";
        let result = palette_height_parser(input);
        let expected_result = Err(String::from("Pixels must be a positive integer"));
        assert_eq!(result, expected_result);

        // Test case 5: Invalid input
        let input = "-100";
        let result = palette_height_parser(input);
        let expected_result = Err(String::from("Pixels must be a positive integer"));
        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_rgb_to_hex() {
        // Test case 1: All zeros
        assert_eq!(rgb_to_hex(0, 0, 0), "#000000");

        // Test case 2: All max values
        assert_eq!(rgb_to_hex(255, 255, 255), "#ffffff");

        // Test case 3: Random values
        assert_eq!(rgb_to_hex(128, 64, 32), "#804020");
    }

    #[test]
    fn test_mcq_color_nodes_to_exoquant_colors() {
        let mcq_colors = vec![
            ColorNode {
                red: 32,
                grn: 64,
                blu: 128,
                rgb: 0,
                cnt: 0,
            },
            ColorNode {
                red: 133,
                grn: 78,
                blu: 232,
                rgb: 0,
                cnt: 0,
            },
        ];

        let result = mcq_color_nodes_to_exoquant_colors(mcq_colors);

        assert_eq!(result.len(), 2);

        assert_eq!(result.get(0).unwrap().r, 32);
        assert_eq!(result.get(0).unwrap().g, 64);
        assert_eq!(result.get(0).unwrap().b, 128);

        assert_eq!(result.get(1).unwrap().r, 133);
        assert_eq!(result.get(1).unwrap().g, 78);
        assert_eq!(result.get(1).unwrap().b, 232);
    }
}
