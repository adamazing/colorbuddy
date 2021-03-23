use std::fs::*;
use std::io::BufReader;
use std::path::*;

use mcq::MMCQ;
use anyhow::{ Result};
use clap::{load_yaml, App};

// Height for the palette that we construct along the bottom of the image
const COLOR_HEIGHT: u32 = 256;

fn main() -> Result<()>  {
    let yaml = load_yaml!("cli.yml");
    let app = App::from(yaml);
    let matches = app.get_matches();

    let palette_size_str = matches.value_of("number_of_colours").unwrap();
    let palette_size: u32 = palette_size_str.parse::<u32>().unwrap();

    let input_images: Vec<&str> = matches.values_of("image").unwrap().collect();
    for img in input_images {
        process_image(img, &palette_size);
    }
    
    Ok(())
}


/**
 * Process Image function
 * Writes an output image consisting of the original image, with a palette of colours shown along the bottom.
 *
 * [String] filename of the image to process
 * [u32] Size of the palette to generate
 */
fn process_image(file: &str, palette_size: &u32) {
    //println!("Reading image {}", file);

    let mcq = {
        let img = image::load(BufReader::new(File::open(file).unwrap()), image::JPEG).unwrap().to_rgba();
        let data = img.into_vec();
        // Here we extract the quantized colors from the image.
        MMCQ::from_pixels_u8_rgba(data.as_slice(), *palette_size)
    };
    // A `Vec` of colors, sorted by usage frequency descending
    let qc = mcq.get_quantized_colors();

    // Open the image to be processed
    let img = image::load(BufReader::new(File::open(file).unwrap()), image::JPEG).unwrap().to_rgba();
    let (ix, iy) = img.dimensions(); 

    // Create an image buffer big enough to hold the output image
    let mut imgbuf = image::ImageBuffer::new(ix, iy + COLOR_HEIGHT);

    // This clones the image we're processing into the output buffer
    for x in 0..ix {
        for y in 0..iy {
            imgbuf.put_pixel(x, y, img.get_pixel(x, y).clone());
        }
    }

    // The width of each color in the palette strip
    let color_width = ix / palette_size;

    for y in (iy + 1)..(iy + COLOR_HEIGHT) {
        for x0 in 0..*palette_size {
            let x1 = x0 * color_width;
            let q = qc[x0 as usize];

            for x2 in 0..color_width {
                imgbuf.put_pixel(x1 + x2, y, image::Rgba([q.red, q.grn, q.blu, 0xff]));
            }
        }
    }

    // Get a file buffer using the original filename, appending the `.png`
    // extension
    let ref mut fout = File::create(format!("./target/{}.png",
                                    Path::new(file).file_name().unwrap().to_str().unwrap()).as_str()).unwrap();

    // Save the output image
    let _ = image::ImageRgba8(imgbuf).save(fout, image::PNG);
}
