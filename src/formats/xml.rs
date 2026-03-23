/// XML file generator.
///
/// Produces valid XML files with a root element containing child records,
/// each with fields defined by the user-specified schema.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::data::faker;
use crate::error::{GenResult, GenerationError};

/// Generator for XML files.
pub struct XmlGenerator;

impl XmlGenerator {
    /// Escapes special XML characters in a string.
    fn escape_xml(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }
}

impl Generator for XmlGenerator {
    fn format_name(&self) -> &str {
        "XML"
    }

    fn file_extension(&self) -> &str {
        "xml"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let (rows, schema_str, pretty) = match &config.format_options {
            FormatOptions::StructuredData {
                rows,
                schema,
                pretty,
            } => (*rows, schema.clone(), *pretty),
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "XML generator requires StructuredData options".to_string(),
                ))
            }
        };

        let schema = faker::parse_schema(&schema_str);
        if schema.is_empty() {
            return Err(GenerationError::InvalidConfig(
                "Schema must contain at least one field".to_string(),
            ));
        }

        let indent = if pretty { "  " } else { "" };
        let newline = if pretty { "\n" } else { "" };

        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
        xml.push_str(newline);
        xml.push_str("<records>");
        xml.push_str(newline);

        for _ in 0..rows {
            xml.push_str(indent);
            xml.push_str("<record>");
            xml.push_str(newline);
            for (field_name, field_type) in &schema {
                let val = faker::value_for_type(&mut config.rng, field_type);
                let escaped = Self::escape_xml(&val);
                if pretty {
                    xml.push_str(&format!(
                        "{indent}{indent}<{field_name}>{escaped}</{field_name}>{newline}"
                    ));
                } else {
                    xml.push_str(&format!("<{field_name}>{escaped}</{field_name}>"));
                }
            }
            xml.push_str(indent);
            xml.push_str("</record>");
            xml.push_str(newline);
        }

        xml.push_str("</records>");
        if pretty {
            xml.push('\n');
        }

        Ok(xml.into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::FormatOptions;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use std::path::PathBuf;

    fn make_config(rows: usize, schema: &str, pretty: bool) -> GeneratorConfig {
        GeneratorConfig {
            output_dir: PathBuf::from("/tmp"),
            name_pattern: "test_{n}".to_string(),
            extension: "xml".to_string(),
            index: 0,
            overwrite: false,
            rng: ChaCha8Rng::seed_from_u64(42),
            format_options: FormatOptions::StructuredData {
                rows,
                schema: schema.to_string(),
                pretty,
            },
        }
    }

    #[test]
    fn test_xml_valid_structure() {
        let gen = XmlGenerator;
        let mut config = make_config(3, "name:string,age:int", true);
        let result = gen.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        assert!(text.starts_with("<?xml"));
        assert!(text.contains("<records>"));
        assert!(text.contains("</records>"));
        assert!(text.contains("<record>"));
        assert!(text.contains("</record>"));
        assert!(text.contains("<name>"));
        assert!(text.contains("<age>"));
    }

    #[test]
    fn test_xml_compact() {
        let gen = XmlGenerator;
        let mut config = make_config(2, "id:int", false);
        let result = gen.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        // Compact mode: no indentation
        assert!(!text.contains("  <"));
    }

    #[test]
    fn test_xml_escaping() {
        // The escape function should handle special chars
        let escaped = XmlGenerator::escape_xml("<test>&\"'value</test>");
        assert!(escaped.contains("&lt;"));
        assert!(escaped.contains("&amp;"));
        assert!(escaped.contains("&quot;"));
        assert!(escaped.contains("&apos;"));
    }

    #[test]
    fn test_xml_empty_schema_error() {
        let gen = XmlGenerator;
        let mut config = make_config(5, "", false);
        assert!(gen.generate(&mut config).is_err());
    }
}
