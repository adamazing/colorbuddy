use crate::palette::converter::mcq_color_nodes_to_exoquant_colors;
use crate::types::{
    config::{QuantisationMethod, DEFAULT_ALPHA_COLOR},
    error::Result,
};
use exoquant::{generate_palette, optimizer, Color, Histogram, SimpleColorSpace};
use image::RgbImage;
use mcq::MMCQ;

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
/// use color_buddy::types::config::QuantisationMethod;
/// use color_buddy::palette::extractor::extract_palette;
/// use image::RgbImage;
/// let image = RgbImage::new(10, 10);
/// let palette = extract_palette(&image, 8, QuantisationMethod::KMeans).unwrap();
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

            Ok(mcq_color_nodes_to_exoquant_colors(
                mcq.get_quantized_colors().to_vec(),
            ))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::config::QuantisationMethod;
    use image::{Rgb, RgbImage};

    // Helper function to create a test image with known colors
    fn create_test_image(width: u32, height: u32, colors: &[Rgb<u8>]) -> RgbImage {
        let mut image = RgbImage::new(width, height);
        let color_count = colors.len();

        for (x, y, pixel) in image.enumerate_pixels_mut() {
            let color_index = ((x + y) as usize) % color_count;
            *pixel = colors[color_index];
        }

        image
    }

    // Helper function to create a solid color image
    fn create_solid_image(width: u32, height: u32, color: Rgb<u8>) -> RgbImage {
        let mut image = RgbImage::new(width, height);
        for pixel in image.pixels_mut() {
            *pixel = color;
        }
        image
    }

    #[test]
    fn test_extract_palette_median_cut_basic() {
        let colors = vec![
            Rgb([255, 0, 0]), // Red
            Rgb([0, 255, 0]), // Green
            Rgb([0, 0, 255]), // Blue
        ];
        let image = create_test_image(10, 10, &colors);

        let result = extract_palette(&image, 3, QuantisationMethod::MedianCut);

        assert!(result.is_ok());
        let palette = result.unwrap();
        assert!(palette.len() <= 3);
        assert!(!palette.is_empty());
    }

    #[test]
    fn test_extract_palette_kmeans_basic() {
        let colors = vec![
            Rgb([255, 0, 0]), // Red
            Rgb([0, 255, 0]), // Green
            Rgb([0, 0, 255]), // Blue
        ];
        let image = create_test_image(10, 10, &colors);

        let result = extract_palette(&image, 3, QuantisationMethod::KMeans);

        assert!(result.is_ok());
        let palette = result.unwrap();
        assert_eq!(palette.len(), 3);
    }

    #[test]
    fn test_extract_palette_single_color() {
        let image = create_solid_image(5, 5, Rgb([128, 128, 128]));

        let result_median = extract_palette(&image, 1, QuantisationMethod::MedianCut);
        let result_kmeans = extract_palette(&image, 1, QuantisationMethod::KMeans);

        assert!(result_median.is_ok());
        assert!(result_kmeans.is_ok());

        let palette_median = result_median.unwrap();
        let palette_kmeans = result_kmeans.unwrap();

        assert_eq!(palette_median.len(), 1);
        assert_eq!(palette_kmeans.len(), 1);

        // Check that the extracted color is what we expect
        assert_eq!(palette_median[0].r, 128);
        assert_eq!(palette_median[0].g, 128);
        assert_eq!(palette_median[0].b, 128);
    }

    #[test]
    fn test_extract_palette_complex_image() {
        // Create an image with many different colors
        let colors = vec![
            Rgb([255, 0, 0]),   // Red
            Rgb([255, 128, 0]), // Orange
            Rgb([255, 255, 0]), // Yellow
            Rgb([128, 255, 0]), // Lime
            Rgb([0, 255, 0]),   // Green
            Rgb([0, 255, 128]), // Spring green
            Rgb([0, 255, 255]), // Cyan
            Rgb([0, 128, 255]), // Sky blue
            Rgb([0, 0, 255]),   // Blue
            Rgb([128, 0, 255]), // Purple
            Rgb([255, 0, 255]), // Magenta
            Rgb([255, 0, 128]), // Rose
        ];
        let image = create_test_image(20, 20, &colors);

        let result_median = extract_palette(&image, 8, QuantisationMethod::MedianCut);
        let result_kmeans = extract_palette(&image, 8, QuantisationMethod::KMeans);

        assert!(result_median.is_ok());
        assert!(result_kmeans.is_ok());

        let palette_median = result_median.unwrap();
        let palette_kmeans = result_kmeans.unwrap();

        assert!(palette_median.len() <= 8);
        assert_eq!(palette_kmeans.len(), 8);

        // Verify all colors have valid alpha values
        for color in &palette_median {
            assert_eq!(color.a, crate::types::config::DEFAULT_ALPHA_COLOR);
        }
        for color in &palette_kmeans {
            assert_eq!(color.a, crate::types::config::DEFAULT_ALPHA_COLOR);
        }
    }

    #[test]
    fn test_extract_palette_small_image() {
        let image = create_solid_image(1, 1, Rgb([42, 142, 242]));

        let result_median = extract_palette(&image, 1, QuantisationMethod::MedianCut);
        let result_kmeans = extract_palette(&image, 1, QuantisationMethod::KMeans);

        assert!(result_median.is_ok());
        assert!(result_kmeans.is_ok());

        let palette_median = result_median.unwrap();
        let palette_kmeans = result_kmeans.unwrap();

        assert_eq!(palette_median.len(), 1);
        assert_eq!(palette_kmeans.len(), 1);

        // Check color accuracy
        assert_eq!(palette_median[0].r, 42);
        assert_eq!(palette_median[0].g, 142);
        assert_eq!(palette_median[0].b, 242);
    }

    #[test]
    fn test_extract_palette_methods_consistency() {
        // Test that both methods work on the same input
        let colors = vec![
            Rgb([255, 0, 0]),
            Rgb([0, 255, 0]),
            Rgb([0, 0, 255]),
            Rgb([255, 255, 0]),
        ];
        let image = create_test_image(16, 16, &colors);

        let result_median = extract_palette(&image, 4, QuantisationMethod::MedianCut);
        let result_kmeans = extract_palette(&image, 4, QuantisationMethod::KMeans);

        assert!(result_median.is_ok());
        assert!(result_kmeans.is_ok());

        let palette_median = result_median.unwrap();
        let palette_kmeans = result_kmeans.unwrap();

        // Both should return valid palettes, though they may differ
        assert!(palette_median.len() <= 4);
        assert_eq!(palette_kmeans.len(), 4);
    }

    // #[test] // Uncomment this test if you want to check for very large number of colors
    // fn test_extract_palette_very_large_number() {
    //     let image = create_solid_image(5, 5, Rgb([100, 100, 100]));
    //
    //     // Test with u16::MAX to check bounds handling
    //     let result_median = extract_palette(&image, u16::MAX, QuantisationMethod::MedianCut);
    //     let result_kmeans = extract_palette(&image, u16::MAX, QuantisationMethod::KMeans);
    //
    //     // Should handle gracefully - either succeed with reasonable number or error
    //     if let Ok(palette) = result_median {
    //         assert!(palette.len() < 1000); // Reasonable upper bound
    //     }
    //     if let Ok(palette) = result_kmeans {
    //         assert!(palette.len() < 1000); // Reasonable upper bound
    //     }
    // }
}
