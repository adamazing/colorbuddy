use exoquant::{generate_palette, optimizer, Color, Histogram, SimpleColorSpace};
use image::RgbImage;
use mcq::MMCQ;
use crate::types::{
    config::{QuantisationMethod, DEFAULT_ALPHA_COLOR},
    error::Result,
};
use crate::palette::converter::mcq_color_nodes_to_exoquant_colors;

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
pub fn extract_palette(
    input_image: &RgbImage,
    number_of_colors: u16,
    quantisation_method: QuantisationMethod,
) -> Result<Vec<Color>> {
    match quantisation_method {
        QuantisationMethod::MedianCut => {
            // Convert pixels directly to RGBA format without cloning the entire image
            let rgba_data: Vec<u8> = input_image
                .pixels()
                .flat_map(|pixel| [pixel[0], pixel[1], pixel[2], DEFAULT_ALPHA_COLOR])
                .collect();

            let mcq = MMCQ::from_pixels_u8_rgba(&rgba_data, number_of_colors.into());

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
                number_of_colors.into(),
            ))
        }
    }
}
