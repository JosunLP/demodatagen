/// Plain text file generator.
///
/// Produces `.txt` files with lorem ipsum paragraphs.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::data::lorem;
use crate::error::{GenResult, GenerationError};

/// Generator for plain text files.
pub struct TxtGenerator;

impl Generator for TxtGenerator {
    fn format_name(&self) -> &str {
        "TXT"
    }

    fn file_extension(&self) -> &str {
        "txt"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let (paragraph_count, word_count) = match &config.format_options {
            FormatOptions::Text { paragraphs, words } => (*paragraphs, *words),
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "TXT generator requires Text options".to_string(),
                ));
            }
        };

        let text = if word_count > 0 {
            // Generate approximate word count across paragraphs
            let paras = (word_count / 50).max(1); // ~50 words per paragraph
            lorem::plain_text(&mut config.rng, paras)
        } else {
            lorem::plain_text(&mut config.rng, paragraph_count)
        };

        Ok(text.into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::FormatOptions;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use std::path::PathBuf;

    fn make_config(paragraphs: usize, words: usize) -> GeneratorConfig {
        GeneratorConfig {
            output_dir: PathBuf::from("/tmp"),
            name_pattern: "test_{n}".to_string(),
            extension: "txt".to_string(),
            index: 0,
            overwrite: false,
            rng: ChaCha8Rng::seed_from_u64(42),
            locale: crate::data::Locale::EnUs,
            format_options: FormatOptions::Text { paragraphs, words },
        }
    }

    #[test]
    fn test_txt_generates_content() {
        let generator = TxtGenerator;
        let mut config = make_config(3, 0);
        let result = generator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        assert!(!text.is_empty());
        assert!(text.contains('.'));
    }

    #[test]
    fn test_txt_paragraph_count() {
        let generator = TxtGenerator;
        let mut config = make_config(5, 0);
        let result = generator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        let paras: Vec<&str> = text.split("\n\n").collect();
        assert_eq!(paras.len(), 5);
    }

    #[test]
    fn test_txt_word_count_mode() {
        let generator = TxtGenerator;
        let mut config = make_config(0, 200);
        let result = generator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        assert!(!text.is_empty());
    }
}
