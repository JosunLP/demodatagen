//! TIFF image file generator.
//!
//! Produces valid TIFF images, reusing the shared image-buffer patterns from
//! the PNG generator.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::error::{GenResult, GenerationError};
use crate::formats::png::generate_image_buffer;
use image::ImageFormat;
use std::io::Cursor;

/// Generator for TIFF image files.
pub struct TiffGenerator;

impl Generator for TiffGenerator {
    fn format_name(&self) -> &str {
        "TIFF"
    }

    fn file_extension(&self) -> &str {
        "tiff"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let (width, height, pattern) = match &config.format_options {
            FormatOptions::Image {
                width,
                height,
                pattern,
                ..
            } => (*width, *height, *pattern),
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "TIFF generator requires Image options".to_string(),
                ));
            }
        };
        if width == 0 || height == 0 {
            return Err(GenerationError::InvalidConfig(
                "Image dimensions must be greater than 0".to_string(),
            ));
        }

        let img = generate_image_buffer(&mut config.rng, width, height, pattern);
        let mut buffer = Cursor::new(Vec::new());
        img.write_to(&mut buffer, ImageFormat::Tiff)
            .map_err(|e| GenerationError::Image(e.to_string()))?;
        Ok(buffer.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::ImagePattern;
    use crate::core::generator::test_support::image_config;

    #[test]
    fn test_tiff_valid_header() {
        let mut config = image_config(32, 32, ImagePattern::Gradient, 1);
        let result = TiffGenerator.generate(&mut config).unwrap();
        // TIFF magic: "II*\0" (little-endian) or "MM\0*" (big-endian).
        assert!(&result[0..4] == b"II\x2a\x00" || &result[0..4] == b"MM\x00\x2a");
    }

    #[test]
    fn test_tiff_roundtrip() {
        let mut config = image_config(24, 24, ImagePattern::Shapes, 1);
        let result = TiffGenerator.generate(&mut config).unwrap();
        let img = image::load_from_memory_with_format(&result, ImageFormat::Tiff).unwrap();
        assert_eq!(img.width(), 24);
    }

    #[test]
    fn test_tiff_zero_dimension_error() {
        let mut config = image_config(32, 0, ImagePattern::Noise, 1);
        assert!(TiffGenerator.generate(&mut config).is_err());
    }
}
