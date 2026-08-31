/// Markdown file generator.
///
/// Produces `.md` files with headings, paragraphs, bullet lists,
/// and code blocks.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::data::lorem;
use crate::error::{GenResult, GenerationError};

/// Generator for Markdown files.
pub struct MarkdownGenerator;

impl Generator for MarkdownGenerator {
    fn format_name(&self) -> &str {
        "Markdown"
    }

    fn file_extension(&self) -> &str {
        "md"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let (paragraph_count, heading_count) = match &config.format_options {
            FormatOptions::Markdown {
                paragraphs,
                headings,
            } => (*paragraphs, *headings),
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "Markdown generator requires Markdown options".to_string(),
                ));
            }
        };

        let doc = lorem::markdown_document(&mut config.rng, heading_count, paragraph_count);
        Ok(doc.into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::FormatOptions;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use std::path::PathBuf;

    fn make_config(paragraphs: usize, headings: usize) -> GeneratorConfig {
        GeneratorConfig {
            output_dir: PathBuf::from("/tmp"),
            name_pattern: "test_{n}".to_string(),
            extension: "md".to_string(),
            index: 0,
            overwrite: false,
            rng: ChaCha8Rng::seed_from_u64(42),
            locale: crate::data::Locale::EnUs,
            format_options: FormatOptions::Markdown {
                paragraphs,
                headings,
            },
        }
    }

    #[test]
    fn test_markdown_has_title() {
        let generator = MarkdownGenerator;
        let mut config = make_config(3, 2);
        let result = generator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        assert!(text.starts_with("# "));
    }

    #[test]
    fn test_markdown_has_headings() {
        let generator = MarkdownGenerator;
        let mut config = make_config(6, 3);
        let result = generator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        assert!(text.contains("## ") || text.contains("### "));
    }

    #[test]
    fn test_markdown_not_empty() {
        let generator = MarkdownGenerator;
        let mut config = make_config(1, 1);
        let result = generator.generate(&mut config).unwrap();
        assert!(!result.is_empty());
    }
}
