/// PNG image file generator.
///
/// Produces valid PNG images with various patterns (noise, gradient,
/// shapes, checkerboard) using the `image` crate.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig, ImagePattern};
use crate::error::{GenResult, GenerationError};
use image::{ImageBuffer, ImageFormat, Rgba};
use rand::Rng;
use std::io::Cursor;

/// Generator for PNG image files.
pub struct PngGenerator;

/// Generates pixel data for the given pattern type.
pub fn generate_image_buffer<R: Rng>(
    rng: &mut R,
    width: u32,
    height: u32,
    pattern: ImagePattern,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut img = ImageBuffer::new(width, height);

    match pattern {
        ImagePattern::Noise => {
            for pixel in img.pixels_mut() {
                *pixel = Rgba([rng.gen(), rng.gen(), rng.gen(), 255]);
            }
        }
        ImagePattern::Gradient => {
            let r_start: u8 = rng.gen();
            let g_start: u8 = rng.gen();
            let b_start: u8 = rng.gen();
            let r_end: u8 = rng.gen();
            let g_end: u8 = rng.gen();
            let b_end: u8 = rng.gen();

            for (x, y, pixel) in img.enumerate_pixels_mut() {
                let t_x = x as f32 / width.max(1) as f32;
                let t_y = y as f32 / height.max(1) as f32;
                let t = (t_x + t_y) / 2.0;
                let r = (r_start as f32 * (1.0 - t) + r_end as f32 * t) as u8;
                let g = (g_start as f32 * (1.0 - t) + g_end as f32 * t) as u8;
                let b = (b_start as f32 * (1.0 - t) + b_end as f32 * t) as u8;
                *pixel = Rgba([r, g, b, 255]);
            }
        }
        ImagePattern::Shapes => {
            // Fill with background color
            let bg = Rgba([rng.gen(), rng.gen(), rng.gen(), 255u8]);
            for pixel in img.pixels_mut() {
                *pixel = bg;
            }

            // Draw random rectangles and circles
            let shape_count = rng.gen_range(5..20);
            for _ in 0..shape_count {
                let color = Rgba([rng.gen(), rng.gen(), rng.gen(), 200u8]);
                let cx = rng.gen_range(0..width);
                let cy = rng.gen_range(0..height);
                let size = rng.gen_range(10..width.min(height) / 3 + 1);

                if rng.gen_bool(0.5) {
                    // Rectangle
                    let x1 = cx.saturating_sub(size / 2);
                    let y1 = cy.saturating_sub(size / 2);
                    let x2 = (cx + size / 2).min(width - 1);
                    let y2 = (cy + size / 2).min(height - 1);
                    for ry in y1..=y2 {
                        for rx in x1..=x2 {
                            img.put_pixel(rx, ry, color);
                        }
                    }
                } else {
                    // Circle
                    let radius = size / 2;
                    let r2 = (radius * radius) as i64;
                    let min_x = cx.saturating_sub(radius);
                    let max_x = (cx + radius).min(width - 1);
                    let min_y = cy.saturating_sub(radius);
                    let max_y = (cy + radius).min(height - 1);
                    for ry in min_y..=max_y {
                        for rx in min_x..=max_x {
                            let dx = rx as i64 - cx as i64;
                            let dy = ry as i64 - cy as i64;
                            if dx * dx + dy * dy <= r2 {
                                img.put_pixel(rx, ry, color);
                            }
                        }
                    }
                }
            }
        }
        ImagePattern::Checkerboard => {
            let tile_size = rng.gen_range(8..64u32);
            let color_a = Rgba([rng.gen(), rng.gen(), rng.gen(), 255u8]);
            let color_b = Rgba([rng.gen(), rng.gen(), rng.gen(), 255u8]);

            for (x, y, pixel) in img.enumerate_pixels_mut() {
                let checker = ((x / tile_size) + (y / tile_size)) % 2 == 0;
                *pixel = if checker { color_a } else { color_b };
            }
        }
    }

    img
}

impl Generator for PngGenerator {
    fn format_name(&self) -> &str {
        "PNG"
    }

    fn file_extension(&self) -> &str {
        "png"
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
                    "PNG generator requires Image options".to_string(),
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
        img.write_to(&mut buffer, ImageFormat::Png)
            .map_err(|e| GenerationError::Image(e.to_string()))?;

        Ok(buffer.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::FormatOptions;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use std::path::PathBuf;

    fn make_config(width: u32, height: u32, pattern: ImagePattern) -> GeneratorConfig {
        GeneratorConfig {
            output_dir: PathBuf::from("/tmp"),
            name_pattern: "test_{n}".to_string(),
            extension: "png".to_string(),
            index: 0,
            overwrite: false,
            rng: ChaCha8Rng::seed_from_u64(42),
            format_options: FormatOptions::Image {
                width,
                height,
                pattern,
                frames: 1,
            },
        }
    }

    #[test]
    fn test_png_valid_header() {
        let gen = PngGenerator;
        let mut config = make_config(64, 64, ImagePattern::Gradient);
        let result = gen.generate(&mut config).unwrap();
        // PNG magic bytes
        assert_eq!(&result[0..4], &[0x89, 0x50, 0x4E, 0x47]);
    }

    #[test]
    fn test_png_all_patterns() {
        let gen = PngGenerator;
        for pattern in [
            ImagePattern::Noise,
            ImagePattern::Gradient,
            ImagePattern::Shapes,
            ImagePattern::Checkerboard,
        ] {
            let mut config = make_config(32, 32, pattern);
            let result = gen.generate(&mut config);
            assert!(result.is_ok(), "Failed for pattern: {pattern}");
        }
    }

    #[test]
    fn test_png_zero_dimension_error() {
        let gen = PngGenerator;
        let mut config = make_config(0, 100, ImagePattern::Noise);
        assert!(gen.generate(&mut config).is_err());
    }
}
