use crate::types::config::DEFAULT_ALPHA_COLOR;
use exoquant::Color;
use mcq::ColorNode;

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
/// use color_buddy::palette::converter::mcq_color_nodes_to_exoquant_colors;
/// use color_buddy::types::config::DEFAULT_ALPHA_COLOR;
/// use mcq::ColorNode;
/// use exoquant::Color;
///
/// let mcq_colors = vec![ColorNode { red: 255, grn: 0, blu: 0, rgb: 0, cnt: 1 }];
/// let exoquant_colors = mcq_color_nodes_to_exoquant_colors(mcq_colors);
/// assert_eq!(exoquant_colors[0].r, 255);
/// assert_eq!(exoquant_colors[0].a, DEFAULT_ALPHA_COLOR);
/// ```
pub fn mcq_color_nodes_to_exoquant_colors(mcq_color_nodes: Vec<ColorNode>) -> Vec<Color> {
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

#[cfg(test)]
mod tests {
    use super::*;

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
