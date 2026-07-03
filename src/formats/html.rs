//! HTML document generator.
//!
//! Produces a small, valid HTML5 document with a title, headings, paragraphs,
//! and an occasional list — useful as demo web content.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::data::lorem;
use crate::error::{GenResult, GenerationError};
use rand::RngExt;

/// Generator for HTML documents.
pub struct HtmlGenerator;

/// Escapes the five HTML-significant characters.
fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

impl Generator for HtmlGenerator {
    fn format_name(&self) -> &str {
        "HTML"
    }

    fn file_extension(&self) -> &str {
        "html"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let (paragraphs, headings) = match &config.format_options {
            FormatOptions::Markdown {
                paragraphs,
                headings,
            } => (*paragraphs, *headings),
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "HTML generator requires Markdown options".to_string(),
                ))
            }
        };

        let rng = &mut config.rng;
        let title = lorem::heading(rng);
        let mut body = String::new();
        body.push_str(&format!("    <h1>{}</h1>\n", escape(&title)));
        body.push_str(&format!("    <p>{}</p>\n", escape(&lorem::paragraph(rng))));

        let per_section = if headings > 0 {
            paragraphs.max(1) / headings.max(1)
        } else {
            paragraphs
        };

        for i in 0..headings {
            let level = if i % 3 == 0 { 2 } else { 3 };
            body.push_str(&format!(
                "    <h{level}>{}</h{level}>\n",
                escape(&lorem::heading(rng))
            ));
            for _ in 0..per_section.max(1) {
                body.push_str(&format!("    <p>{}</p>\n", escape(&lorem::paragraph(rng))));
            }
            if rng.random_bool(0.3) {
                body.push_str("    <ul>\n");
                for _ in 0..rng.random_range(3..=6) {
                    let words = rng.random_range(4..10);
                    body.push_str(&format!(
                        "      <li>{}</li>\n",
                        escape(&lorem::sentence(rng, words))
                    ));
                }
                body.push_str("    </ul>\n");
            }
        }

        if headings == 0 {
            for _ in 0..paragraphs {
                body.push_str(&format!("    <p>{}</p>\n", escape(&lorem::paragraph(rng))));
            }
        }

        let html = format!(
            "<!DOCTYPE html>\n<html lang=\"en\">\n  <head>\n    <meta charset=\"utf-8\">\n    \
             <title>{}</title>\n  </head>\n  <body>\n{}  </body>\n</html>\n",
            escape(&title),
            body
        );
        Ok(html.into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::test_support::markdown_config;

    #[test]
    fn test_html_well_formed() {
        let mut config = markdown_config(6, 3);
        let result = HtmlGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        assert!(text.starts_with("<!DOCTYPE html>"));
        assert!(text.contains("<html"));
        assert!(text.contains("</html>"));
        assert!(text.contains("<h1>"));
        assert!(text.contains("<title>"));
    }

    #[test]
    fn test_html_no_headings() {
        let mut config = markdown_config(3, 0);
        let result = HtmlGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        assert!(text.contains("<p>"));
    }
}
