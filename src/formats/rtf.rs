//! Rich Text Format (`.rtf`) generator.
//!
//! Produces a minimal but valid RTF 1.5 document with a title, bold section
//! headings, and justified lorem paragraphs — enough structure for word
//! processors to open and render it faithfully.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::data::lorem;
use crate::error::{GenResult, GenerationError};

/// Generator for RTF documents.
pub struct RtfGenerator;

/// Escapes RTF control characters and non-ASCII characters (as `\uN?`).
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '{' => out.push_str("\\{"),
            '}' => out.push_str("\\}"),
            c if c.is_ascii() => out.push(c),
            c => out.push_str(&format!("\\u{}?", c as u32 as i32)),
        }
    }
    out
}

impl Generator for RtfGenerator {
    fn format_name(&self) -> &str {
        "RTF"
    }

    fn file_extension(&self) -> &str {
        "rtf"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let (paragraphs, headings) = match &config.format_options {
            FormatOptions::Markdown {
                paragraphs,
                headings,
            } => ((*paragraphs).max(1), (*headings).max(1)),
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "RTF generator requires Markdown options".to_string(),
                ))
            }
        };

        let rng = &mut config.rng;
        let mut out = String::new();
        out.push_str("{\\rtf1\\ansi\\deff0{\\fonttbl{\\f0\\fswiss Helvetica;}}\n");
        out.push_str("\\f0\\fs22\n");

        // Document title.
        out.push_str(&format!(
            "{{\\b\\fs36 {}}}\\par\\par\n",
            escape(&lorem::heading(rng))
        ));

        // Spread the paragraphs across the requested number of sections.
        let per_section = paragraphs.div_ceil(headings);
        for _ in 0..headings {
            out.push_str(&format!(
                "{{\\b\\fs28 {}}}\\par\\par\n",
                escape(&lorem::heading(rng))
            ));
            for _ in 0..per_section {
                out.push_str(&format!("{}\\par\\par\n", escape(&lorem::paragraph(rng))));
            }
        }

        out.push('}');
        Ok(out.into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::test_support::markdown_config;

    #[test]
    fn test_rtf_structure() {
        let mut config = markdown_config(4, 2);
        let result = RtfGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        assert!(text.starts_with("{\\rtf1\\ansi"));
        assert!(text.ends_with('}'));
        // Title + 2 section headings.
        assert_eq!(text.matches("{\\b\\fs28 ").count(), 2);
        assert!(text.matches("\\par").count() >= 4);
    }

    #[test]
    fn test_rtf_braces_balanced() {
        let mut config = markdown_config(3, 3);
        let result = RtfGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        let opens = text.matches('{').count() - text.matches("\\{").count();
        let closes = text.matches('}').count() - text.matches("\\}").count();
        assert_eq!(opens, closes);
    }

    #[test]
    fn test_rtf_escape() {
        assert_eq!(escape("a{b}c\\"), "a\\{b\\}c\\\\");
        assert_eq!(escape("café"), "caf\\u233?");
    }

    #[test]
    fn test_rtf_deterministic() {
        let mut a = markdown_config(2, 1);
        let mut b = markdown_config(2, 1);
        assert_eq!(
            RtfGenerator.generate(&mut a).unwrap(),
            RtfGenerator.generate(&mut b).unwrap()
        );
    }
}
