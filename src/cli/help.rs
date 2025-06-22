use console::style;
use console::Color as ConsoleColor;
use std::fmt::Write;

struct Example {
    description: String,
    example: String,
}

/// Creates a rainbow-colored string for terminal display.
///
/// Takes a string and applies cycling colors to each alphabetic character,
/// creating a rainbow effect for terminal output. Non-alphabetic characters
/// remain unstyled.
///
/// # Arguments
///
/// * `s` - The string to apply rainbow coloring to
///
/// # Returns
///
/// A styled string with rainbow coloring applied to alphabetic characters.
///
/// # Examples
///
/// ```
/// # use color_buddy::cli::help::rainbow;
/// let colored = rainbow("Color Buddy");
/// println!("{}", colored); // Displays with rainbow colors
/// ```
pub fn rainbow(s: &str) -> String {
    let mut colored_string = String::new();
    let colors = [
        ConsoleColor::Red,
        ConsoleColor::Magenta,
        ConsoleColor::Blue,
        ConsoleColor::Cyan,
        ConsoleColor::Green,
        ConsoleColor::Yellow,
        ConsoleColor::Green,
        ConsoleColor::Cyan,
        ConsoleColor::Blue,
        ConsoleColor::Magenta,
    ];

    let mut color_index = 0;

    for c in s.chars() {
        let colored_char = if c.is_ascii_alphabetic() {
            let color = colors[color_index];
            color_index = (color_index + 1) % colors.len();
            style(c.to_string()).fg(color)
        } else {
            style(c.to_string())
        };
        colored_string.push_str(&colored_char.to_string());
    }

    colored_string
}

/// Generates example usage text for the CLI help system.
///
/// Creates formatted examples showing different ways to use the colorbuddy tool,
/// including JSON output, image output with palettes, and various sizing options.
///
/// # Returns
///
/// A formatted string containing styled examples for display in CLI help.
pub fn examples() -> String {
    let examples = [
        Example {
            description: "Generate JSON containing the 8 most prevalent colors in the image:".to_string(),
            example: "colorbuddy --output-type json original-image.jpg".to_string(),
        },
        Example {
            description: "Output the original images with a palette of the 5 most prevalent colors along the bottom:".to_string(),
            example: "colorbuddy --number-of-colors 5 --output-type original-image.jpg another-image.jpg".to_string()
        },
        Example {
            description: "Specify the height of the palette as a percentage of the original image's height:".to_string(),
            example: "colorbuddy --palette-height 20% original-image.jpg".to_string()
        },
        Example {
            description: "Specify a width, height, and the standalone-palette output height to create a standalone palette image:".to_string(),
            example: "colorbuddy --palette-height 50px --palette-width 500 original-image.jpg".to_string()
        }
    ];

    let formatted_examples = examples
        .iter()
        .fold(String::new(), |mut out, ex| {
            let _ = write!(
                out,
                "  {}\n     {}\n\n",
                style(ex.description.to_owned()).italic(),
                style(ex.example.to_owned()).white()
            );
            out
        });

    format!(
        "{}\n{}",
        style("Examples:").underlined(),
        formatted_examples
    )
}

/// Generates the short about text for the CLI.
///
/// Creates a brief description with the tool name and basic purpose,
/// including a rainbow-styled title.
///
/// # Returns
///
/// A formatted string containing the short tool description.
pub fn about() -> String {
    format!(
        "\n{}\n\ncolorbuddy is a command line tool to extract a palette of colors from any image.",
        style(rainbow("Color Buddy 🎨"))
    )
}

/// Generates the long description text for the CLI about section.
///
/// Creates a detailed description of the tool's capabilities, including
/// supported algorithms and output formats.
///
/// # Returns
///
/// A formatted string containing the complete tool description.
pub fn long_about() -> String {
    format!(
        "{}
It uses one of two algorithms to calculate the palette: K-Means, or Median Cut.\n
You can generate:
  - a standalone image containing the palette colors
  - a json file containing the color details in:
     - HEX notation (e.g. #1a6b3f); and
     - the individual R,G, and B components;
  - a copy of the original image with the palette of colors along the bottom of the image.",
        about()
    )
}
