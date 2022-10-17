use std::fmt;
use std::path::*;

use mcq::ColorNode;
use mcq::MMCQ;
use exoquant::Color as ExoquantColor;
use exoquant::{generate_palette, Histogram, optimizer, SimpleColorSpace};
use anyhow::Result;
use clap::{Parser, ValueEnum};

// Height for the palette that we construct along the bottom of the image
const COLOR_HEIGHT: u32 = 256;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None,trailing_var_arg=true)]
struct Args {
    #[arg(short='m', long="quantisation-method", default_value_t=QuantisationMethod::KMeans)]
    quantisation_method: QuantisationMethod,

    #[arg(short='n', long="number-of-colours", default_value="8")]
    number_of_colours: usize,

    #[arg(long, short='i')]
    image: String
}

#[derive(ValueEnum,Clone, Debug)]
enum QuantisationMethod {
    MedianCut,
    KMeans
}

impl fmt::Display for QuantisationMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            QuantisationMethod::MedianCut => write!(f, "median-cut"),
            QuantisationMethod::KMeans => write!(f, "k-means")
        }
    }
}

struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

fn main() -> Result<()>  {
    let matches = Args::parse();

    println!("{:?}", matches.quantisation_method);
    println!("{:?}", matches.number_of_colours);

    let image = matches.image;
    process_image(&image, &matches.number_of_colours, &matches.quantisation_method);

    Ok(())
}

fn mcq_color_nodes_to_local_colors(mcq_color_nodes: Vec<ColorNode>) -> Vec<Color> {
    let mut local_colors: Vec<Color> = vec![];

    for c in mcq_color_nodes {
        local_colors.push(Color{
            r: c.red,
            g: c.grn,
            b: c.blu
        })
    }

    return local_colors;
}

fn exoquant_colors_to_local_colors(exoquant_colors: Vec<ExoquantColor>) -> Vec<Color> {
    let mut local_colors: Vec<Color> = vec![];

    for c in exoquant_colors {
        local_colors.push(Color{
            r: c.r,
            g: c.g,
            b: c.b
        })
    }

    return local_colors;
}

/**
 * Writes an output image consisting of the original image, with a palette of colours shown
 * along the bottom.
 *
 * [String] filename of the image to process.
 * [u32] Size of the palette to generate.
 * [QuantisationMethod] The quantisation method to use.
 */
fn process_image(file: &String, number_of_colours: &usize, quantisation_method: &QuantisationMethod) {
    println!("Reading image {}", file);

    let dynamic_image = image::open(file).unwrap();
    let input_image = dynamic_image.to_rgba8();
    let qc: Vec<Color>;

    match quantisation_method {
        QuantisationMethod::MedianCut => {
            let data = input_image.clone().into_vec();
            let mcq = MMCQ::from_pixels_u8_rgba(data.as_slice(), (*number_of_colours).try_into().unwrap());

            qc = mcq_color_nodes_to_local_colors(mcq.get_quantized_colors().to_vec());
        },
        QuantisationMethod::KMeans => {
            let histogram: Histogram = input_image.clone().pixels().map(|p| ExoquantColor {r: p[0], g: p[1], b: p[2], a: p[3] }).collect();
            qc = exoquant_colors_to_local_colors(generate_palette(
                    &histogram,
                    &SimpleColorSpace::default(),
                    &optimizer::KMeans,
                    *number_of_colours
                )
            )
        }
    }
    let (input_image_width, input_image_height) = input_image.dimensions();

    // Create an image buffer big enough to hold the output image
    let mut imgbuf = image::ImageBuffer::new(input_image_width, input_image_height + COLOR_HEIGHT);

    // This clones the image we're processing into the output buffer
    for x in 0..input_image_width {
        for y in 0..input_image_height {
            imgbuf.put_pixel(x, y, *input_image.get_pixel(x, y));
        }
    }

    // The width of each color in the palette strip
    let color_width = input_image_width / *number_of_colours as u32;

    for y in (input_image_height + 1)..(input_image_height + COLOR_HEIGHT) {
        for x0 in 0..*number_of_colours {
            let x1 = x0 as u32 * color_width;
            let q = &qc[x0 as usize];

            for x2 in 0..color_width {
                imgbuf.put_pixel(x1 + x2, y, image::Rgba([q.r, q.g, q.b, 0xff]));
            }
        }
    }

    let original_file_stem = Path::new(file).file_stem().unwrap().to_str().unwrap();
    // Get a file buffer using the original filename, appending the `.png` extension
    let output_file_name = format!("./target/{}.png", original_file_stem);

    // Save the output image
    let _ = imgbuf.save(output_file_name);
}
