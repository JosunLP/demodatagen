//! XML file generator.
//!
//! Produces valid XML files with a configurable root element containing child
//! records, each with fields defined by the user-specified schema. Null values
//! are emitted as self-closing elements.
//!
//! Element *names* (root, row tag, field names) are sanitized into valid XML
//! names — invalid characters are replaced rather than entity-escaped, since
//! entity references are not permitted inside tag names. Element *text* is
//! escaped normally.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::data::{FieldValue, Schema};
use crate::error::{GenResult, GenerationError};

/// Generator for XML files.
pub struct XmlGenerator;

impl XmlGenerator {
    /// Escapes special XML characters in text content.
    fn escape_xml(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }
}

/// Sanitizes an arbitrary string into a valid XML element name.
///
/// Keeps ASCII letters, digits, `_`, `-`, and `.`; replaces anything else with
/// `_`. Ensures the name does not start with a digit, `-`, or `.` (XML names
/// must begin with a letter or underscore). Falls back to `fallback` if empty.
fn sanitize_tag(name: &str, fallback: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    let needs_prefix = out
        .chars()
        .next()
        .map(|c| !(c.is_ascii_alphabetic() || c == '_'))
        .unwrap_or(true);
    if out.is_empty() {
        return fallback.to_string();
    }
    if needs_prefix {
        out.insert(0, '_');
    }
    out
}

impl Generator for XmlGenerator {
    fn format_name(&self) -> &str {
        "XML"
    }

    fn file_extension(&self) -> &str {
        "xml"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let (rows, schema_str, pretty, root, row_tag) = match &config.format_options {
            FormatOptions::Xml {
                rows,
                schema,
                pretty,
                root,
                row_tag,
            } => (
                *rows,
                schema.clone(),
                *pretty,
                root.clone(),
                row_tag.clone(),
            ),
            FormatOptions::StructuredData {
                rows,
                schema,
                pretty,
            } => (
                *rows,
                schema.clone(),
                *pretty,
                "records".to_string(),
                "record".to_string(),
            ),
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "XML generator requires Xml options".to_string(),
                ));
            }
        };

        let schema = Schema::parse(&schema_str).map_err(GenerationError::InvalidConfig)?;
        if schema.is_empty() {
            return Err(GenerationError::InvalidConfig(
                "Schema must contain at least one field".to_string(),
            ));
        }

        let root = sanitize_tag(&root, "records");
        let row_tag = sanitize_tag(&row_tag, "record");

        let indent = if pretty { "  " } else { "" };
        let newline = if pretty { "\n" } else { "" };

        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
        xml.push_str(newline);
        xml.push_str(&format!("<{root}>"));
        xml.push_str(newline);

        for record in schema.generate_records(&mut config.rng, config.locale, rows) {
            xml.push_str(indent);
            xml.push_str(&format!("<{row_tag}>"));
            xml.push_str(newline);
            for (name, value) in &record {
                let tag = sanitize_tag(name, "field");
                if value.is_null() {
                    if pretty {
                        xml.push_str(&format!("{indent}{indent}<{tag}/>{newline}"));
                    } else {
                        xml.push_str(&format!("<{tag}/>"));
                    }
                    continue;
                }
                let escaped = Self::escape_xml(&value_text(value));
                if pretty {
                    xml.push_str(&format!(
                        "{indent}{indent}<{tag}>{escaped}</{tag}>{newline}"
                    ));
                } else {
                    xml.push_str(&format!("<{tag}>{escaped}</{tag}>"));
                }
            }
            xml.push_str(indent);
            xml.push_str(&format!("</{row_tag}>"));
            xml.push_str(newline);
        }

        xml.push_str(&format!("</{root}>"));
        if pretty {
            xml.push('\n');
        }

        Ok(xml.into_bytes())
    }
}

/// Renders a [`FieldValue`] as XML element text.
fn value_text(value: &FieldValue) -> String {
    match value {
        FieldValue::Array(items) => items
            .iter()
            .map(|v| v.to_flat_string())
            .collect::<Vec<_>>()
            .join(", "),
        other => other.to_flat_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::test_support::xml_config;

    #[test]
    fn test_xml_valid_structure() {
        let mut config = xml_config(3, "name:string,age:int", true, "records", "record");
        let result = XmlGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        assert!(text.starts_with("<?xml"));
        assert!(text.contains("<records>"));
        assert!(text.contains("</records>"));
        assert!(text.contains("<record>"));
        assert!(text.contains("<name>"));
        assert!(text.contains("<age>"));
    }

    #[test]
    fn test_xml_custom_tags() {
        let mut config = xml_config(2, "id:int", false, "users", "user");
        let result = XmlGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        assert!(text.contains("<users>"));
        assert!(text.contains("<user>"));
        assert!(text.contains("</users>"));
    }

    #[test]
    fn test_xml_compact() {
        let mut config = xml_config(2, "id:int", false, "records", "record");
        let result = XmlGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        assert!(!text.contains("  <"));
    }

    #[test]
    fn test_xml_null_self_closing() {
        let mut config = xml_config(1, "x:int?1.0", false, "records", "record");
        let result = XmlGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        assert!(text.contains("<x/>"));
    }

    #[test]
    fn test_xml_escaping() {
        let escaped = XmlGenerator::escape_xml("<test>&\"'value</test>");
        assert!(escaped.contains("&lt;"));
        assert!(escaped.contains("&amp;"));
        assert!(escaped.contains("&quot;"));
        assert!(escaped.contains("&apos;"));
    }

    #[test]
    fn test_sanitize_tag_produces_valid_names() {
        assert_eq!(sanitize_tag("user<id>", "field"), "user_id_");
        assert_eq!(sanitize_tag("has space", "field"), "has_space");
        assert_eq!(sanitize_tag("123start", "field"), "_123start");
        assert_eq!(sanitize_tag("", "field"), "field");
        assert_eq!(sanitize_tag("ok_name-1.2", "field"), "ok_name-1.2");
    }

    #[test]
    fn test_xml_invalid_field_names_stay_well_formed() {
        // Field/tag names with XML-illegal characters must still yield parseable XML.
        let mut config = xml_config(2, "user<id>:int,has space:name", false, "da ta", "re<c>");
        let result = XmlGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        // No raw '<' or '>' should appear inside tag names (only as delimiters).
        assert!(text.contains("<user_id_>"));
        assert!(text.contains("<has_space>"));
        assert!(text.contains("<da_ta>"));
        assert!(text.contains("<re_c_>"));
        assert!(!text.contains("&lt;user"));
    }

    #[test]
    fn test_xml_empty_schema_error() {
        let mut config = xml_config(5, "", false, "records", "record");
        assert!(XmlGenerator.generate(&mut config).is_err());
    }
}
