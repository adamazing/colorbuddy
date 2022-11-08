use std::fmt;
use std::path::*;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use exoquant::{generate_palette, optimizer, Color, Histogram, SimpleColorSpace};
use image::RgbImage;
use mcq::ColorNode;
use mcq::MMCQ;

// TODO: Make this configurable, either in absolute pixels, or as a percentage of the image?
// Height for the palette that we construct along the bottom of the image
const COLOR_HEIGHT: u32 = 256;

#[derive(ValueEnum, Clone, Debug)]
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

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short='m', long="quantisation-method", default_value_t=QuantisationMethod::KMeans)]
    quantisation_method: QuantisationMethod,

    #[arg(short = 'n', long = "number-of-colours", default_value = "8")]
    number_of_colours: usize,

    #[arg(short='t', long = "output-type", default_value_t=OutputType::OriginalImage)]
    output_type: OutputType,

    image: PathBuf,
}

fn main() -> Result<()> {
    let matches = Args::parse();

    process_image(
        &matches.image,
        matches.number_of_colours,
        matches.quantisation_method,
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
    number_of_colours: usize,
    quantisation_method: QuantisationMethod,
) -> Vec<Color> {
    match quantisation_method {
        QuantisationMethod::MedianCut => {
            let data = input_image.clone().into_vec();
            let mcq =
                MMCQ::from_pixels_u8_rgba(data.as_slice(), number_of_colours.try_into().unwrap());

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
                number_of_colours,
            )
        }
    }
}

/**
 * Writes an output image consisting of the original image, with a palette of colours shown
 * along the bottom.
 *
 * [String] filename of the image to process.
 * [u32] Size of the palette to generate.
 * [QuantisationMethod] The quantisation method to use.
 */
fn process_image(
    file: &PathBuf,
    number_of_colours: usize,
    quantisation_method: QuantisationMethod,
) {
    let dynamic_image = image::open(file).unwrap();
    let input_image = dynamic_image.to_rgb8();
    let (input_image_width, input_image_height) = input_image.dimensions();

    let color_palette: Vec<Color> =
        extract_palette(&input_image, number_of_colours, quantisation_method);

    // Create an image buffer big enough to hold the output image
    let mut imgbuf = image::ImageBuffer::new(input_image_width, input_image_height + COLOR_HEIGHT);

    // The width of each color in the palette strip
    let color_width = input_image_width / number_of_colours as u32;

    // This clones the image we're processing into the output buffer
    for x in 0..input_image_width {
        for y in 0..input_image_height {
            imgbuf.put_pixel(x, y, *input_image.get_pixel(x, y));
        }
    }

    for y in (input_image_height)..(input_image_height + COLOR_HEIGHT) {
        for x0 in 0..number_of_colours {
            let x1 = x0 as u32 * color_width;
            let q = &color_palette[x0 as usize];

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
}
