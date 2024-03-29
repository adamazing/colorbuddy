use std::fmt;
use std::path::*;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use console::style;
use console::Color as ConsoleColor;
use exoquant::{generate_palette, optimizer, Color, Histogram, SimpleColorSpace};
use image::{DynamicImage, RgbImage};
use mcq::ColorNode;
use mcq::MMCQ;

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

fn examples() -> String {
    let examples = vec![
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
        .map(|ex| {
            format!(
                "  {}\n     {}\n\n",
                style(ex.description.to_owned()).italic(),
                style(ex.example.to_owned()).white()
            )
        })
        .collect::<String>();

    format!(
        "{}\n{}",
        style("Examples:").underlined(),
        formatted_examples
    )
}

/**
 * A helper function that returns a styled rainbow string for display.
 **/
fn rainbow(s: &str) -> String {
    let mut colored_string = String::new();
    let colors = vec![
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

    #[arg(short = 'n', long = "number-of-colors", default_value = "8")]
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
          default_value = "256")]
    palette_height: PaletteHeight,

    #[arg(short = 'w',
          long = "palette-width",
          help = "Used only when generating a standalone palette. Provide a width in pixels. (e.g. 100, 500)",
          default_value = None)]
    palette_width: Option<u32>,

    #[arg(help = "Any number of images to process.")]
    images: Vec<PathBuf>,
}

fn main() -> Result<()> {
    let matches = Args::parse();

    for image in &matches.images {
        let output_file_name =
            output_file_name(image, matches.output.as_ref(), matches.output_type);

        process_image(
            image,
            matches.number_of_colors,
            matches.quantisation_method,
            matches.palette_height,
            matches.palette_width,
            matches.output_type,
            &output_file_name,
        );
    }

    Ok(())
}

/**
 * Internally we deal with a Vector<Color> (`Color` provided by the exoquant crate).
 * This helper function converts a Vector of MCQ `ColorNode`s into a Vector of exoquant `Color`s.
 */
fn mcq_color_nodes_to_exoquant_colors(mcq_color_nodes: Vec<ColorNode>) -> Vec<Color> {
    mcq_color_nodes
        .iter()
        .map(|c| Color {
            r: c.red,
            g: c.grn,
            b: c.blu,
            a: 0xff,
        })
        .collect()
}

/**
 * This function abstracts the extraction of the Vector of `Color`s depending on the chosen
 * quantisation method.
 *
 * [&RgbImage] The image to be processed.
 * [usize] The number of colors required for the palette.
 * [QuantisationMethod] The quantisation method to be used.
 **/
fn extract_palette(
    input_image: &RgbImage,
    number_of_colors: usize,
    quantisation_method: QuantisationMethod,
) -> Vec<Color> {
    match quantisation_method {
        QuantisationMethod::MedianCut => {
            let data = input_image.clone().into_vec();
            let mcq =
                MMCQ::from_pixels_u8_rgba(data.as_slice(), number_of_colors.try_into().unwrap());

            mcq_color_nodes_to_exoquant_colors(mcq.get_quantized_colors().to_vec())
        }
        QuantisationMethod::KMeans => {
            let histogram: Histogram = input_image
                .pixels()
                .map(|p| Color {
                    r: p[0],
                    g: p[1],
                    b: p[2],
                    a: 0xff,
                })
                .collect();
            generate_palette(
                &histogram,
                &SimpleColorSpace::default(),
                &optimizer::KMeans,
                number_of_colors,
            )
        }
    }
}

/**
 * This is the meat of the tool. Opens the image, gets the palette of colors, and outputs the
 * requested artifact (either a copy of the original image with the palette along the bottom, or a
 * JSON file with the palette details.)
 *
 * [&PathBuf] file, the image to process.
 * [usize] Number of colors to pick for the palette.
 * [QuantisationMethod] The quantisation method to use.
 * [PaletteHeight] The height of the palette.
 * [OutputType] The type of output requested.
 * [&PathBuf] The output file name.
 */
fn process_image(
    file: &PathBuf,
    number_of_colors: usize,
    quantisation_method: QuantisationMethod,
    palette_height: PaletteHeight,
    palette_width: Option<u32>,
    output_type: OutputType,
    output_file_name: &PathBuf,
) {
    let dynamic_image: DynamicImage;

    if let Ok(img) = image::open(file) {
        dynamic_image = img;
    } else {
        eprintln!("Error opening image: {}", file.to_str().unwrap());
        return;
    };

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

    let color_palette: Vec<Color> =
        extract_palette(&input_image, number_of_colors, quantisation_method);

    /*
     *  Output to the original image: */
    if OutputType::OriginalImage == output_type {
        // Create an image buffer big enough to hold the output image
        let mut imgbuf = image::ImageBuffer::new(input_image_width, total_height);

        // The width of each color in the palette strip
        let color_width = input_image_width / number_of_colors as u32;

        // This clones the image we're processing into the output buffer
        for x in 0..input_image_width {
            for y in 0..input_image_height {
                imgbuf.put_pixel(x, y, *input_image.get_pixel(x, y));
            }
        }

        for y in (input_image_height)..(total_height) {
            for (x0, q) in color_palette.iter().enumerate().take(number_of_colors) {
                let x1 = x0 as u32 * color_width;
                for x2 in 0..color_width {
                    imgbuf.put_pixel(x1 + x2, y, image::Rgb([q.r, q.g, q.b]));
                }
            }
        }

        let save_result = imgbuf.save(&output_file_name);

        assert!(
            save_result.is_ok(),
            "Failed to save: {:?}",
            output_file_name.canonicalize().unwrap()
        );
    } else if OutputType::StandalonePalette == output_type {
        let standalone_palette_width = match palette_width {
            Some(w) => w,
            None => input_image_width,
        };
        let mut imgbuf = image::ImageBuffer::new(standalone_palette_width, total_height);

        let color_width = standalone_palette_width / number_of_colors as u32;

        for y in 0..total_height {
            for (x0, q) in color_palette.iter().enumerate().take(number_of_colors) {
                let x1 = x0 as u32 * color_width;
                for x2 in 0..color_width {
                    imgbuf.put_pixel(x1 + x2, y, image::Rgb([q.r, q.g, q.b]));
                }
            }
        }

        let save_result = imgbuf.save(&output_file_name);

        assert!(
            save_result.is_ok(),
            "Failed to save: {:?}",
            output_file_name.canonicalize().unwrap()
        );
    } else if OutputType::Json == output_type {
        println!("{{");
        for (i, color) in color_palette.iter().enumerate() {
            println!("\t\"color_{}\": {{", i + 1);
            println!("\t\t\"r\":\t{},\n\t\t\"g\":\t{},\n\t\t\"b\":\t{},\n\t\t\"a\":\t{},\n\t\t\"hex\":\t\"{}\"", color.r, color.g, color.b, color.a, rgb_to_hex(color.r, color.g, color.b));
            if color_palette.len() - 1 != i {
                println!("\t}},");
            } else {
                println!("\t}}");
            }
        }
        println!("}}");
    }
}

/**
 * Given an original file path, an optional output path, and an output type,
 * returns a new file path for the output file. If an output path is provided,
 * the function uses that path. Otherwise, it constructs a new path based on the
 * original file path and the output type.
 *
 * Parameters:
 * - `original_file`: A reference to the original file path.
 * - `output`: An optional reference to the output file path.
 * - `output_type`: The type of output to generate.
 *
 * Returns:
 * - A `PathBuf` representing the new output file path.
 */
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

/**
 * This helper function just converts a color from RGB values to a hex string.
 */
fn rgb_to_hex(red: u8, green: u8, blue: u8) -> String {
    format!("#{red:02x}{green:02x}{blue:02x}")
}

/**
 * This helper function is used by clap when handling the palette-height option.
 * It parses a string and returns a palette height.
 *
 * The palette height can be provided:
 *  - as a percentage of the original image (a number followed by '%')
 *  - as a number of pixels (a number followed by the string 'px')
 *  - as a number of pixels (a number by itself)
 */
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
