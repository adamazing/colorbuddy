use crate::types::error::{ColorBuddyError, Result};
use crate::types::config::PaletteHeight;

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
///
pub fn rgb_to_hex(red: u8, green: u8, blue: u8) -> String {
    format!("#{red:02x}{green:02x}{blue:02x}")
}

/// Parses a string representation of palette height into a `PaletteHeight` enum.
///
/// This function accepts three formats for specifying palette height:
/// - Percentage values ending with '%' (e.g., "50%", "100%")
/// - Absolute pixel values ending with "px" (e.g., "200px", "300px")
/// - Plain numeric values interpreted as pixels (e.g., "200", "300")
///
/// # Arguments
///
/// * `s` - A string slice containing the height specification
///
/// # Returns
///
/// * `Ok(PaletteHeight::Percentage(f32))` - For percentage values between 0-100%
/// * `Ok(PaletteHeight::Absolute(u32))` - For pixel values (with or without "px" suffix)
/// * `Err(ColorBuddyError::InvalidPaletteHeight)` - For invalid input formats or out-of-range values
///
/// # Examples
///
/// ```rust
/// # use your_crate::{palette_height_parser, PaletteHeight};
/// // Percentage format
/// assert_eq!(palette_height_parser("75%"), Ok(PaletteHeight::Percentage(75.0)));
///
/// // Pixel format with suffix
/// assert_eq!(palette_height_parser("250px"), Ok(PaletteHeight::Absolute(250)));
///
/// // Plain numeric format (interpreted as pixels)
/// assert_eq!(palette_height_parser("300"), Ok(PaletteHeight::Absolute(300)));
///
/// // Invalid formats return errors
/// assert!(palette_height_parser("150%").is_err()); // Over 100%
/// assert!(palette_height_parser("invalid").is_err());
/// ```
///
/// # Errors
///
/// Returns `ColorBuddyError::InvalidPaletteHeight` in the following cases:
/// - Percentage values greater than 100%
/// - Non-numeric values that cannot be parsed
/// - Negative values or invalid number formats
pub fn palette_height_parser(s: &str) -> Result<PaletteHeight> {
    if s.ends_with('%') {
        let percentage = &s[0..s.len() - 1];
        match percentage.parse::<f32>() {
            Ok(n) if n <= 100.0 => Ok(PaletteHeight::Percentage(n)),
            _ => Err(ColorBuddyError::InvalidPaletteHeight(
                "Percentage must be between 0 and 100".to_owned()
            )),
        }
    } else if s.ends_with("px") {
        let pixels = &s[0..s.len() - 2];
        match pixels.parse::<u32>() {
            Ok(n) => Ok(PaletteHeight::Absolute(n)),
            _ => Err(ColorBuddyError::InvalidPaletteHeight(
                "Pixels must be a positive integer".to_owned()
            )),
        }
    } else {
        match s.parse::<u32>() {
            Ok(n) => Ok(PaletteHeight::Absolute(n)),
            _ => Err(ColorBuddyError::InvalidPaletteHeight(
                "Pixels must be a positive integer".to_owned()
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_palette_height_parser() {
        assert_eq!(palette_height_parser("235").unwrap(), PaletteHeight::Absolute(235));
        assert_eq!(palette_height_parser("130px").unwrap(), PaletteHeight::Absolute(130));
        assert_eq!(palette_height_parser("50%").unwrap(), PaletteHeight::Percentage(50.0));
        assert!(palette_height_parser("150%").is_err());
        assert!(palette_height_parser("foo").is_err());
    }
}
