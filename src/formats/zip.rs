//! ZIP archive file generator.
//!
//! Produces valid ZIP archives containing multiple generated files. The
//! [`generate_contained_file`] helper is shared with the TAR generator.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::data::{Locale, Schema, lorem};
use crate::error::{GenResult, GenerationError};
use rand::Rng;
use std::io::{Cursor, Write};
use zip::CompressionMethod;
use zip::write::SimpleFileOptions;

/// Generator for ZIP archive files.
pub struct ZipGenerator;

impl Generator for ZipGenerator {
    fn format_name(&self) -> &str {
        "ZIP"
    }

    fn file_extension(&self) -> &str {
        "zip"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let (file_count, contained_format, compression_level) = match &config.format_options {
            FormatOptions::Zip {
                file_count,
                contained_format,
                compression_level,
            } => (*file_count, contained_format.clone(), *compression_level),
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "ZIP generator requires Zip options".to_string(),
                ));
            }
        };

        if file_count == 0 {
            return Err(GenerationError::InvalidConfig(
                "ZIP must contain at least one file".to_string(),
            ));
        }

        let buffer = Cursor::new(Vec::new());
        let mut zip_writer = zip::ZipWriter::new(buffer);

        let compression = if compression_level == 0 {
            CompressionMethod::Stored
        } else {
            CompressionMethod::Deflated
        };
        let options = SimpleFileOptions::default().compression_method(compression);

        for i in 0..file_count {
            let filename = format!("file_{i}.{contained_format}");
            let content =
                generate_contained_file(&mut config.rng, config.locale, &contained_format)?;
            zip_writer
                .start_file(&filename, options)
                .map_err(|e| GenerationError::Archive(e.to_string()))?;
            zip_writer
                .write_all(&content)
                .map_err(|e| GenerationError::Archive(e.to_string()))?;
        }

        let cursor = zip_writer
            .finish()
            .map_err(|e| GenerationError::Archive(e.to_string()))?;
        Ok(cursor.into_inner())
    }
}

/// Schema used for the structured files placed inside archives.
const CONTAINED_SCHEMA: &str = "id:sequence,name:name,email:email,date:date";

/// Generates content for a file to be included in an archive (ZIP or TAR).
///
/// Supports `txt`, `csv`, `json`, `xml`, and `md`; anything else falls back to
/// plain text.
pub fn generate_contained_file<R: Rng>(
    rng: &mut R,
    locale: Locale,
    format: &str,
) -> GenResult<Vec<u8>> {
    match format {
        "csv" => {
            let schema = Schema::parse(CONTAINED_SCHEMA).map_err(GenerationError::InvalidConfig)?;
            let mut out = String::new();
            out.push_str(&schema.field_names().join(","));
            out.push('\n');
            for record in schema.generate_records(rng, locale, 10) {
                let row: Vec<String> = record.iter().map(|(_, v)| v.to_flat_string()).collect();
                out.push_str(&row.join(","));
                out.push('\n');
            }
            Ok(out.into_bytes())
        }
        "json" => {
            let schema = Schema::parse(CONTAINED_SCHEMA).map_err(GenerationError::InvalidConfig)?;
            let array: Vec<serde_json::Value> = schema
                .generate_records(rng, locale, 10)
                .iter()
                .map(|record| {
                    let mut obj = serde_json::Map::new();
                    for (name, value) in record {
                        obj.insert(name.clone(), value.to_json());
                    }
                    serde_json::Value::Object(obj)
                })
                .collect();
            serde_json::to_vec_pretty(&array)
                .map_err(|e| GenerationError::Serialization(e.to_string()))
        }
        "xml" => {
            let schema = Schema::parse(CONTAINED_SCHEMA).map_err(GenerationError::InvalidConfig)?;
            let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<records>\n");
            for record in schema.generate_records(rng, locale, 10) {
                xml.push_str("  <record>\n");
                for (name, value) in &record {
                    xml.push_str(&format!(
                        "    <{name}>{}</{name}>\n",
                        value.to_flat_string()
                    ));
                }
                xml.push_str("  </record>\n");
            }
            xml.push_str("</records>\n");
            Ok(xml.into_bytes())
        }
        "md" | "markdown" => Ok(lorem::markdown_document(rng, 3, 5).into_bytes()),
        "txt" | "text" => Ok(lorem::plain_text(rng, 3).into_bytes()),
        _ => Ok(lorem::plain_text(rng, 2).into_bytes()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::test_support::zip_config;
    use std::io::Read;

    #[test]
    fn test_zip_valid_header() {
        let mut config = zip_config(3, "txt", 6);
        let result = ZipGenerator.generate(&mut config).unwrap();
        assert_eq!(&result[0..4], &[0x50, 0x4B, 0x03, 0x04]);
    }

    #[test]
    fn test_zip_contains_files() {
        let mut config = zip_config(5, "txt", 6);
        let result = ZipGenerator.generate(&mut config).unwrap();
        let mut archive = zip::ZipArchive::new(Cursor::new(result)).unwrap();
        assert_eq!(archive.len(), 5);
        for i in 0..5 {
            let mut file = archive.by_index(i).unwrap();
            let mut content = Vec::new();
            file.read_to_end(&mut content).unwrap();
            assert!(!content.is_empty());
        }
    }

    #[test]
    fn test_zip_csv_content() {
        let mut config = zip_config(1, "csv", 6);
        let result = ZipGenerator.generate(&mut config).unwrap();
        let mut archive = zip::ZipArchive::new(Cursor::new(result)).unwrap();
        let mut file = archive.by_index(0).unwrap();
        assert!(file.name().ends_with(".csv"));
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        assert!(content.contains("id,name,email,date"));
    }

    #[test]
    fn test_zip_json_content() {
        let mut config = zip_config(1, "json", 6);
        let result = ZipGenerator.generate(&mut config).unwrap();
        let mut archive = zip::ZipArchive::new(Cursor::new(result)).unwrap();
        let mut file = archive.by_index(0).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(parsed.is_array());
    }

    #[test]
    fn test_zip_zero_files_error() {
        let mut config = zip_config(0, "txt", 6);
        assert!(ZipGenerator.generate(&mut config).is_err());
    }

    #[test]
    fn test_zip_stored_compression() {
        let mut config = zip_config(2, "txt", 0);
        assert!(ZipGenerator.generate(&mut config).is_ok());
    }
}
