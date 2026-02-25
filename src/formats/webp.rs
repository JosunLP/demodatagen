/// WebP image file generator.
///
/// Produces valid WebP images. Since the `image` crate supports WebP encoding,
/// we use it to write lossless WebP files.

use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::error::{GenResult, GenerationError};
use crate::formats::png::generate_image_buffer;
use image::ImageFormat;
use std::io::Cursor;

/// Generator for WebP image files.
pub struct WebpGenerator;

impl Generator for WebpGenerator {
    fn format_name(&self) -> &str {
        "WebP"
    }

    fn file_extension(&self) -> &str {
        "webp"
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
                    "WebP generator requires Image options".to_string(),
                ))
            }
        };

        if width == 0 || height == 0 {
            return Err(GenerationError::InvalidConfig(
                "Image dimensions must be greater than 0".to_string(),
            ));
        }

        let img = generate_image_buffer(&mut config.rng, width, height, pattern);

        let mut buffer = Cursor::new(Vec::new());
        img.write_to(&mut buffer, ImageFormat::WebP)
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
            extension: "webp".to_string(),
            index: 0,
            overwrite: false,
            rng: ChaCha8Rng::seed_from_u64(42),
            format_options: FormatOptions::Image {
                width,
                height,
                pattern: ImagePattern::Noise,
                frames: 1,
            },
        }
    }

    #[test]
    fn test_webp_valid_header() {
        let gen = WebpGenerator;
        let mut config = make_config(32, 32);
        let result = gen.generate(&mut config).unwrap();
        // WebP starts with "RIFF" then 4 bytes size, then "WEBP"
        assert_eq!(&result[0..4], b"RIFF");
        assert_eq!(&result[8..12], b"WEBP");
    }
}
