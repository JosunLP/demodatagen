//! ICO icon file generator.
//!
//! Produces valid Windows icon (ICO) files. ICO dimensions are limited to
//! 256×256, so larger requests are rejected.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::error::{GenResult, GenerationError};
use crate::formats::png::generate_image_buffer;
use image::ImageFormat;
use std::io::Cursor;

/// Generator for ICO icon files.
pub struct IcoGenerator;

impl Generator for IcoGenerator {
    fn format_name(&self) -> &str {
        "ICO"
    }

    fn file_extension(&self) -> &str {
        "ico"
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
                    "ICO generator requires Image options".to_string(),
                ));
            }
        };
        if width == 0 || height == 0 {
            return Err(GenerationError::InvalidConfig(
                "Icon dimensions must be greater than 0".to_string(),
            ));
        }
        if width > 256 || height > 256 {
            return Err(GenerationError::InvalidConfig(
                "ICO dimensions must not exceed 256x256".to_string(),
            ));
        }

        let img = generate_image_buffer(&mut config.rng, width, height, pattern);
        let mut buffer = Cursor::new(Vec::new());
        img.write_to(&mut buffer, ImageFormat::Ico)
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
    fn test_ico_valid_header() {
        let mut config = image_config(64, 64, ImagePattern::Shapes, 1);
        let result = IcoGenerator.generate(&mut config).unwrap();
        // ICO header: reserved(0,0) + type(1,0) for icons.
        assert_eq!(&result[0..4], &[0x00, 0x00, 0x01, 0x00]);
    }

    #[test]
    fn test_ico_roundtrip() {
        let mut config = image_config(32, 32, ImagePattern::Gradient, 1);
        let result = IcoGenerator.generate(&mut config).unwrap();
        let img = image::load_from_memory_with_format(&result, ImageFormat::Ico).unwrap();
        assert_eq!(img.width(), 32);
    }

    #[test]
    fn test_ico_too_large_error() {
        let mut config = image_config(512, 512, ImagePattern::Noise, 1);
        assert!(IcoGenerator.generate(&mut config).is_err());
    }
}
