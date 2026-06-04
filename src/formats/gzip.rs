//! Gzip-compressed text file generator (`.gz`).
//!
//! Generates lorem-ipsum text and compresses it with gzip (DEFLATE), producing
//! a valid `.gz` file that decompresses back to the original text.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::data::lorem;
use crate::error::{GenResult, GenerationError};
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;

/// Generator for gzip-compressed text files.
pub struct GzipGenerator;

impl Generator for GzipGenerator {
    fn format_name(&self) -> &str {
        "GZIP"
    }

    fn file_extension(&self) -> &str {
        "txt.gz"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let (paragraphs, words) = match &config.format_options {
            FormatOptions::Text { paragraphs, words } => (*paragraphs, *words),
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "GZIP generator requires Text options".to_string(),
                ))
            }
        };

        let paragraph_count = if words > 0 {
            (words / 50).max(1)
        } else {
            paragraphs
        };
        let text = lorem::plain_text(&mut config.rng, paragraph_count.max(1));

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(text.as_bytes())
            .map_err(GenerationError::Io)?;
        encoder.finish().map_err(GenerationError::Io)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::test_support::text_config;
    use flate2::read::GzDecoder;
    use std::io::Read;

    #[test]
    fn test_gzip_magic_bytes() {
        let mut config = text_config(5, 0);
        let result = GzipGenerator.generate(&mut config).unwrap();
        assert_eq!(&result[0..2], &[0x1f, 0x8b]);
    }

    #[test]
    fn test_gzip_decompresses() {
        let mut config = text_config(5, 0);
        let result = GzipGenerator.generate(&mut config).unwrap();
        let mut decoder = GzDecoder::new(&result[..]);
        let mut text = String::new();
        decoder.read_to_string(&mut text).unwrap();
        assert!(!text.is_empty());
        assert!(text.contains('.'));
    }
}
