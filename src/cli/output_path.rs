use crate::types::config::OutputType;
use std::path::{Path, PathBuf};

/// Generates an output file path based on the original file and output settings.
///
/// Creates an appropriate output file path by appending "_palette" to the original
/// filename and adjusting the extension based on the output type. Handles both
/// explicit output paths and automatic path generation.
///
/// # Arguments
///
/// * `original_file` - Path to the original input image file
/// * `output` - Optional explicit output path specified by the user
/// * `output_type` - Type of output being generated (affects file extension)
///
/// # Returns
///
/// A `PathBuf` representing the complete output file path with appropriate
/// filename and extension.
///
/// # Behavior
///
/// - If `output` is a file path: Uses that filepath
/// - If `output` is a directory: Places generated filename in that directory
/// - If `output` is `None`: Uses original file's directory with generated filename
/// - Extensions: Preserves original for images, uses "json" for JSON output
///
/// # Examples
///
/// ```
/// use std::path::{Path, PathBuf};
/// use color_buddy::cli::output_path::output_file_name;
/// use color_buddy::types::config::OutputType;
///
/// let original = Path::new("photo.jpg");
///
/// // No output path specified, original image type
/// let result = output_file_name(original, None, OutputType::OriginalImage);
/// assert_eq!(result, PathBuf::from("photo_palette.jpg"));
///
/// // Output directory specified, JSON type
/// let output_dir = PathBuf::from("/tmp/");
/// let result = output_file_name(original, Some(&output_dir), OutputType::Json);
/// assert_eq!(result, PathBuf::from("/tmp/photo_palette.json"));
///
/// // Specific output file specified
/// let output_file = PathBuf::from("/output/custom.png");
/// let result = output_file_name(original, Some(&output_file), OutputType::StandalonePalette);
/// assert_eq!(result, PathBuf::from("/output/custom.png"));
/// ```
pub fn output_file_name(
    original_file: &Path,
    output: Option<&PathBuf>,
    output_type: OutputType,
) -> PathBuf {
    match output {
        Some(p) if p.is_dir() => {
            let original_image_stem = original_file.file_stem().unwrap().to_str().unwrap();
            let new_extension = match output_type {
                OutputType::OriginalImage | OutputType::StandalonePalette => {
                    match original_file.extension() {
                        Some(ext) => ext.to_str().unwrap(),
                        None => "png",
                    }
                }
                OutputType::Json | OutputType::JsonFile => "json",
            };
            let file_name = format!("{original_image_stem}_palette.{new_extension}");
            p.join(file_name)
        }
        Some(p) => p.clone(),
        None => {
            let original_image_stem = original_file.file_stem().unwrap().to_str().unwrap();
            let new_extension = match output_type {
                OutputType::OriginalImage | OutputType::StandalonePalette => {
                    match original_file.extension() {
                        Some(ext) => ext.to_str().unwrap(),
                        None => "png",
                    }
                }
                OutputType::Json | OutputType::JsonFile => "json",
            };
            let file_name = format!("{original_image_stem}_palette.{new_extension}");
            PathBuf::from(original_file).with_file_name(file_name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_file_name() {
        let original_file = Path::new("/tmp/some_file.png");

        // Test case 1: Output path provided
        let output_path = PathBuf::from("/tmp/something.jpg");
        let output_type = OutputType::OriginalImage;
        let result = output_file_name(&original_file, Some(&output_path), output_type);
        let expected_result = PathBuf::from("/tmp/something.jpg");
        assert_eq!(result, expected_result);

        // Test case 2: Output path not provided
        let output_type = OutputType::OriginalImage;
        let result = output_file_name(&original_file, None, output_type);
        let expected_result = PathBuf::from("/tmp/some_file_palette.png");
        assert_eq!(result, expected_result);

        // Test case 3: Output path provided and OutputType is json
        let output_path = PathBuf::from("/tmp/something.json");
        let output_type = OutputType::Json;
        let result = output_file_name(&original_file, Some(&output_path), output_type);
        let expected_result = PathBuf::from("/tmp/something.json");
        assert_eq!(result, expected_result);

        // Test case 4: Output path not provided and OutputType is json
        let output_type = OutputType::Json;
        let result = output_file_name(&original_file, None, output_type);
        let expected_result = PathBuf::from("/tmp/some_file_palette.json");
        assert_eq!(result, expected_result);
    }
}
