use std::fmt;
use std::path::*;

use mcq::MMCQ;
use anyhow::Result;
use clap::{Parser, ValueEnum};

// Height for the palette that we construct along the bottom of the image
const COLOR_HEIGHT: u32 = 256;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None,trailing_var_arg=true)]
struct Args {
    #[arg(short='m', long="quantisation-method", default_value_t=QuantisationMethod::MedianCut)]
    quantisation_method: QuantisationMethod,

    #[arg(short='n', long="number-of-colours", default_value="8")]
    number_of_colours: u32,

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

fn main() -> Result<()>  {
    let matches = Args::parse();

    println!("{:?}", matches.quantisation_method);
    println!("{:?}", matches.number_of_colours);

    let image = matches.image;
    process_image(&image, &matches.number_of_colours);

    Ok(())
}


/**
 * Process Image function
 * Writes an output image consisting of the original image, with a palette of colours shown
 * along the bottom.
 *
 * [String] filename of the image to process
 * [u32] Size of the palette to generate
 */
fn process_image(file: &String, number_of_colours: &u32) {
    println!("Reading image {}", file);

    let dynamic_image = image::open(file).unwrap();
    let input_image = dynamic_image.to_rgba8();
    let data = input_image.clone().into_vec();
    let mcq = MMCQ::from_pixels_u8_rgba(data.as_slice(), *number_of_colours);

    // A `Vec` of colors, sorted by usage frequency descending
    let qc = mcq.get_quantized_colors();

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
    let color_width = input_image_width / number_of_colours;

    for y in (input_image_height + 1)..(input_image_height + COLOR_HEIGHT) {
        for x0 in 0..*number_of_colours {
            let x1 = x0 * color_width;
            let q = qc[x0 as usize];

            for x2 in 0..color_width {
                imgbuf.put_pixel(x1 + x2, y, image::Rgba([q.red, q.grn, q.blu, 0xff]));
            }
        }
    }

    let original_file_stem = Path::new(file).file_stem().unwrap().to_str().unwrap();
    // Get a file buffer using the original filename, appending the `.png` extension
    let output_file_name = format!("./target/{}.png", original_file_stem);

    // Save the output image
    let _ = imgbuf.save(output_file_name);
}
