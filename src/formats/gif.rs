/// GIF image file generator.
///
/// Produces valid GIF files, optionally animated with multiple frames.
/// Uses the `image` crate's GIF encoder for proper GIF89a output.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::error::{GenResult, GenerationError};
use crate::formats::png::generate_image_buffer;
use image::codecs::gif::{GifEncoder, Repeat};
use image::{Delay, Frame};
use std::io::Cursor;

/// Generator for GIF image files.
pub struct GifGenerator;

impl Generator for GifGenerator {
    fn format_name(&self) -> &str {
        "GIF"
    }

    fn file_extension(&self) -> &str {
        "gif"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let (width, height, pattern, frames) = match &config.format_options {
            FormatOptions::Image {
                width,
                height,
                pattern,
                frames,
            } => (*width, *height, *pattern, *frames),
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "GIF generator requires Image options".to_string(),
                ))
            }
        };

        if width == 0 || height == 0 {
            return Err(GenerationError::InvalidConfig(
                "Image dimensions must be greater than 0".to_string(),
            ));
        }

        let frame_count = frames.max(1);

        let mut buffer = Cursor::new(Vec::new());
        {
            let mut encoder = GifEncoder::new_with_speed(&mut buffer, 10);
            if frame_count > 1 {
                encoder.set_repeat(Repeat::Infinite).map_err(|e| {
                    GenerationError::Image(format!("Failed to set GIF repeat: {e}"))
                })?;
            }

            for _ in 0..frame_count {
                let img = generate_image_buffer(&mut config.rng, width, height, pattern);
                let delay = Delay::from_numer_denom_ms(100, 1);
                let frame = Frame::from_parts(img, 0, 0, delay);
                encoder.encode_frame(frame).map_err(|e| {
                    GenerationError::Image(format!("Failed to encode GIF frame: {e}"))
                })?;
            }
        }

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

    fn make_config(width: u32, height: u32, frames: u32) -> GeneratorConfig {
        GeneratorConfig {
            output_dir: PathBuf::from("/tmp"),
            name_pattern: "test_{n}".to_string(),
            extension: "gif".to_string(),
            index: 0,
            overwrite: false,
            rng: ChaCha8Rng::seed_from_u64(42),
            format_options: FormatOptions::Image {
                width,
                height,
                pattern: ImagePattern::Checkerboard,
                frames,
            },
        }
    }

    #[test]
    fn test_gif_valid_header() {
        let gen = GifGenerator;
        let mut config = make_config(32, 32, 1);
        let result = gen.generate(&mut config).unwrap();
        // GIF87a or GIF89a header
        let header = &result[0..3];
        assert_eq!(header, b"GIF");
    }

    #[test]
    fn test_gif_animated() {
        let gen = GifGenerator;
        let mut config = make_config(16, 16, 3);
        let result = gen.generate(&mut config).unwrap();
        assert!(result.len() > 100);
        assert_eq!(&result[0..3], b"GIF");
    }

    #[test]
    fn test_gif_zero_frames_defaults_to_one() {
        let gen = GifGenerator;
        let mut config = make_config(16, 16, 0);
        let result = gen.generate(&mut config);
        assert!(result.is_ok());
    }
}
