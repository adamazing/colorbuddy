use std::fmt;
use std::path::*;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use exoquant::{generate_palette, optimizer, Color, Histogram, SimpleColorSpace};
use image::RgbImage;
use mcq::ColorNode;
use mcq::MMCQ;

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq)]
enum OutputType {
    Json,
    OriginalImage,
    // StandalonePalette,
}

impl fmt::Display for OutputType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OutputType::Json => write!(f, "json"),
            OutputType::OriginalImage => write!(f, "original-image"),
            // OutputType::StandalonePalette => write!(f, "standalone"),
        }
    }
}

#[derive(ValueEnum, Clone, Debug)]
enum QuantisationMethod {
    MedianCut,
    KMeans,
}

impl fmt::Display for QuantisationMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            QuantisationMethod::MedianCut => write!(f, "median-cut"),
            QuantisationMethod::KMeans => write!(f, "k-means"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum PaletteHeight {
    Percentage(f32),
    Absolute(u32),
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short='m', long="quantisation-method", default_value_t=QuantisationMethod::KMeans)]
    quantisation_method: QuantisationMethod,

    #[arg(short = 'n', long = "number-of-colors", default_value = "8")]
    number_of_colors: usize,

    #[arg(short='t', long = "output-type", default_value_t=OutputType::OriginalImage)]
    output_type: OutputType,

    #[arg(short='p', long = "palette-height", value_parser = palette_height_parser, default_value = "256")]
    palette_height: PaletteHeight,

    image: PathBuf,
}

fn main() -> Result<()> {
    let matches = Args::parse();

    process_image(
        &matches.image,
        matches.number_of_colors,
        matches.quantisation_method,
        matches.palette_height,
        matches.output_type,
    );

    Ok(())
}

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
 * Writes an output image consisting of the original image, with a palette of colors shown
 * along the bottom.
 *
 * [String] filename of the image to process.
 * [u32] Size of the palette to generate.
 * [QuantisationMethod] The quantisation method to use.
 */
fn process_image(
    file: &PathBuf,
    number_of_colors: usize,
    quantisation_method: QuantisationMethod,
    palette_height: PaletteHeight,
    output_type: OutputType,
) {
    let dynamic_image = image::open(file).unwrap();
    let input_image = dynamic_image.to_rgb8();
    let (input_image_width, input_image_height) = input_image.dimensions();

    let total_height = match (output_type, palette_height) {
        (OutputType::OriginalImage, PaletteHeight::Absolute(a)) => a + input_image_height,
        (OutputType::OriginalImage, PaletteHeight::Percentage(a)) => {
            input_image_height + (a / 100.0 * input_image_height as f32).round() as u32
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

        // Get an output file name using the original filename, appending the `.png` extension
        let mut output_file_name = PathBuf::from(file.file_stem().unwrap());
        output_file_name.set_extension("png");

        // Save the output image
        let save_result = imgbuf.save(&output_file_name);

        assert!(
            save_result.is_ok(),
            "Failed to save: {:?}",
            output_file_name.canonicalize().unwrap()
        );
    } else {
        println!("{{");
        for (i, color) in color_palette.iter().enumerate() {
            println!("\t\"color_{}\": {{", i + 1);
            println!("\t\t\"r\":\t{},\n\t\t\"g\":\t{},\n\t\t\"b\":\t{},\n\t\t\"a\":\t{},\n\t\t\"hex\":\t\"{}\"", color.r, color.g, color.b, color.a, rgb_to_hex(color.r, color.g, color.b));
            println!("\n\t}}");
        }
        println!("}}");
    }
}

fn rgb_to_hex(red: u8, green: u8, blue: u8) -> String {
    format!("#{red:02x}{green:02x}{blue:02x}")
}

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
