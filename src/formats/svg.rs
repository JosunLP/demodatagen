//! SVG vector image generator.
//!
//! Produces a valid SVG document filled with random shapes (rectangles,
//! circles, lines) on a colored background.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::data::faker;
use crate::error::{GenResult, GenerationError};
use rand::RngExt;

/// Generator for SVG files.
pub struct SvgGenerator;

impl Generator for SvgGenerator {
    fn format_name(&self) -> &str {
        "SVG"
    }

    fn file_extension(&self) -> &str {
        "svg"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let (width, height, shapes) = match &config.format_options {
            FormatOptions::Svg {
                width,
                height,
                shapes,
            } => (*width, *height, *shapes),
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "SVG generator requires Svg options".to_string(),
                ))
            }
        };

        if width == 0 || height == 0 {
            return Err(GenerationError::InvalidConfig(
                "Canvas dimensions must be greater than 0".to_string(),
            ));
        }

        let rng = &mut config.rng;
        let mut svg = String::new();
        svg.push_str(&format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" \
             viewBox=\"0 0 {width} {height}\">\n"
        ));
        svg.push_str(&format!(
            "  <rect width=\"{width}\" height=\"{height}\" fill=\"{}\"/>\n",
            faker::hex_color(rng)
        ));

        for _ in 0..shapes {
            let fill = faker::hex_color(rng);
            let opacity = (rng.random_range(30..=95) as f32) / 100.0;
            match rng.random_range(0..3) {
                0 => {
                    // `..=.max(1)` keeps the range non-empty even on tiny canvases.
                    let w = rng.random_range(1..=(width / 2).max(1));
                    let h = rng.random_range(1..=(height / 2).max(1));
                    // Constrain position so the shape stays within the viewBox.
                    let x = rng.random_range(0..=width - w);
                    let y = rng.random_range(0..=height - h);
                    svg.push_str(&format!(
                        "  <rect x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" \
                         fill=\"{fill}\" opacity=\"{opacity}\"/>\n"
                    ));
                }
                1 => {
                    let r = rng.random_range(1..=(width.min(height) / 2).max(1));
                    // Keep the whole circle inside the canvas.
                    let cx = rng.random_range(r..=(width - r).max(r));
                    let cy = rng.random_range(r..=(height - r).max(r));
                    svg.push_str(&format!(
                        "  <circle cx=\"{cx}\" cy=\"{cy}\" r=\"{r}\" \
                         fill=\"{fill}\" opacity=\"{opacity}\"/>\n"
                    ));
                }
                _ => {
                    let x1 = rng.random_range(0..width);
                    let y1 = rng.random_range(0..height);
                    let x2 = rng.random_range(0..width);
                    let y2 = rng.random_range(0..height);
                    let sw = rng.random_range(1..=5);
                    svg.push_str(&format!(
                        "  <line x1=\"{x1}\" y1=\"{y1}\" x2=\"{x2}\" y2=\"{y2}\" \
                         stroke=\"{fill}\" stroke-width=\"{sw}\" opacity=\"{opacity}\"/>\n"
                    ));
                }
            }
        }

        svg.push_str("</svg>\n");
        Ok(svg.into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::test_support::svg_config;

    #[test]
    fn test_svg_valid_root() {
        let mut config = svg_config(200, 150, 10);
        let result = SvgGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        assert!(text.contains("<svg"));
        assert!(text.contains("</svg>"));
        assert!(text.contains("width=\"200\""));
    }

    #[test]
    fn test_svg_zero_dimension_error() {
        let mut config = svg_config(0, 100, 5);
        assert!(SvgGenerator.generate(&mut config).is_err());
    }

    #[test]
    fn test_svg_tiny_canvas_does_not_panic() {
        // Regression: small canvases previously produced empty RNG ranges.
        for dim in [1u32, 2, 3, 5, 11] {
            let mut config = svg_config(dim, dim, 30);
            assert!(
                SvgGenerator.generate(&mut config).is_ok(),
                "panicked at {dim}x{dim}"
            );
        }
    }

    #[test]
    fn test_svg_deterministic() {
        let mut a = svg_config(100, 100, 8);
        let mut b = svg_config(100, 100, 8);
        assert_eq!(
            SvgGenerator.generate(&mut a).unwrap(),
            SvgGenerator.generate(&mut b).unwrap()
        );
    }
}
