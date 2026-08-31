//! BMP image file generator.
//!
//! Produces valid Windows Bitmap (BMP) images, reusing the shared image-buffer
//! patterns from the PNG generator.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::error::{GenResult, GenerationError};
use crate::formats::png::generate_image_buffer;
use image::ImageFormat;
use std::io::Cursor;

/// Generator for BMP image files.
pub struct BmpGenerator;

impl Generator for BmpGenerator {
    fn format_name(&self) -> &str {
        "BMP"
    }

    fn file_extension(&self) -> &str {
        "bmp"
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
                    "BMP generator requires Image options".to_string(),
                ));
            }
        };
        if width == 0 || height == 0 {
            return Err(GenerationError::InvalidConfig(
                "Image dimensions must be greater than 0".to_string(),
            ));
        }

        let rgba = generate_image_buffer(&mut config.rng, width, height, pattern);
        let rgb = image::DynamicImage::ImageRgba8(rgba).to_rgb8();
        let mut buffer = Cursor::new(Vec::new());
        rgb.write_to(&mut buffer, ImageFormat::Bmp)
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
    fn test_bmp_valid_header() {
        let mut config = image_config(32, 32, ImagePattern::Gradient, 1);
        let result = BmpGenerator.generate(&mut config).unwrap();
        assert_eq!(&result[0..2], b"BM");
    }

    #[test]
    fn test_bmp_roundtrip() {
        let mut config = image_config(16, 16, ImagePattern::Checkerboard, 1);
        let result = BmpGenerator.generate(&mut config).unwrap();
        let img = image::load_from_memory_with_format(&result, ImageFormat::Bmp).unwrap();
        assert_eq!(img.width(), 16);
        assert_eq!(img.height(), 16);
    }

    #[test]
    fn test_bmp_zero_dimension_error() {
        let mut config = image_config(0, 32, ImagePattern::Noise, 1);
        assert!(BmpGenerator.generate(&mut config).is_err());
    }
}
