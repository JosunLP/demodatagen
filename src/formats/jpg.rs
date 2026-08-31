/// JPEG image file generator.
///
/// Produces valid JPEG images using the `image` crate with various patterns.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::error::{GenResult, GenerationError};
use crate::formats::png::generate_image_buffer;
use image::ImageFormat;
use std::io::Cursor;

/// Generator for JPEG image files.
pub struct JpgGenerator;

impl Generator for JpgGenerator {
    fn format_name(&self) -> &str {
        "JPEG"
    }

    fn file_extension(&self) -> &str {
        "jpg"
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
                    "JPEG generator requires Image options".to_string(),
                ));
            }
        };

        if width == 0 || height == 0 {
            return Err(GenerationError::InvalidConfig(
                "Image dimensions must be greater than 0".to_string(),
            ));
        }

        let rgba_img = generate_image_buffer(&mut config.rng, width, height, pattern);
        // Convert RGBA to RGB for JPEG
        let rgb_img = image::DynamicImage::ImageRgba8(rgba_img).to_rgb8();

        let mut buffer = Cursor::new(Vec::new());
        rgb_img
            .write_to(&mut buffer, ImageFormat::Jpeg)
            .map_err(|e| GenerationError::Image(e.to_string()))?;

        Ok(buffer.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::{FormatOptions, ImagePattern};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use std::path::PathBuf;

    fn make_config(width: u32, height: u32) -> GeneratorConfig {
        GeneratorConfig {
            output_dir: PathBuf::from("/tmp"),
            name_pattern: "test_{n}".to_string(),
            extension: "jpg".to_string(),
            index: 0,
            overwrite: false,
            rng: ChaCha8Rng::seed_from_u64(42),
            locale: crate::data::Locale::EnUs,
            format_options: FormatOptions::Image {
                width,
                height,
                pattern: ImagePattern::Gradient,
                frames: 1,
            },
        }
    }

    #[test]
    fn test_jpg_valid_header() {
        let generator = JpgGenerator;
        let mut config = make_config(64, 64);
        let result = generator.generate(&mut config).unwrap();
        // JPEG magic bytes: FF D8 FF
        assert_eq!(&result[0..3], &[0xFF, 0xD8, 0xFF]);
    }

    #[test]
    fn test_jpg_produces_content() {
        let generator = JpgGenerator;
        let mut config = make_config(100, 100);
        let result = generator.generate(&mut config).unwrap();
        assert!(result.len() > 100);
    }
}
