/// ZIP archive file generator.
///
/// Produces valid ZIP archives containing multiple generated files.
/// The contained files are generated using the appropriate format generator.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::data::{faker, lorem};
use crate::error::{GenResult, GenerationError};
use rand::Rng;
use std::io::{Cursor, Write};
use zip::write::FileOptions;
use zip::CompressionMethod;

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
                ))
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

        let options = FileOptions::default().compression_method(compression);

        for i in 0..file_count {
            let filename = format!("file_{i}.{contained_format}");
            let content = generate_contained_file(&mut config.rng, &contained_format)?;

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

/// Generates content for a file to be included in a ZIP archive.
///
/// Supports basic formats: txt, csv, json, xml, md.
fn generate_contained_file<R: Rng>(rng: &mut R, format: &str) -> GenResult<Vec<u8>> {
    match format {
        "txt" | "text" => {
            let text = lorem::plain_text(rng, 3);
            Ok(text.into_bytes())
        }
        "csv" => {
            let schema = faker::parse_schema("id:int,name:string,email:email,date:date");
            let mut output = String::new();
            // Header
            let headers: Vec<&str> = schema.iter().map(|(n, _)| n.as_str()).collect();
            output.push_str(&headers.join(","));
            output.push('\n');
            // Rows
            for _ in 0..10 {
                let values: Vec<String> = schema
                    .iter()
                    .map(|(_, t)| faker::value_for_type(rng, t))
                    .collect();
                output.push_str(&values.join(","));
                output.push('\n');
            }
            Ok(output.into_bytes())
        }
        "json" => {
            let schema = faker::parse_schema("id:int,name:string,email:email");
            let mut records = Vec::new();
            for _ in 0..10 {
                let mut obj = serde_json::Map::new();
                for (name, ftype) in &schema {
                    let val = faker::value_for_type(rng, ftype);
                    let json_val = match ftype.as_str() {
                        "int" => val
                            .parse::<i64>()
                            .map(serde_json::Value::from)
                            .unwrap_or(serde_json::Value::String(val)),
                        _ => serde_json::Value::String(val),
                    };
                    obj.insert(name.clone(), json_val);
                }
                records.push(serde_json::Value::Object(obj));
            }
            serde_json::to_vec_pretty(&records)
                .map_err(|e| GenerationError::Serialization(e.to_string()))
        }
        "xml" => {
            let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<records>\n");
            for _ in 0..10 {
                xml.push_str("  <record>\n");
                xml.push_str(&format!("    <name>{}</name>\n", faker::full_name(rng)));
                xml.push_str(&format!("    <email>{}</email>\n", faker::email(rng)));
                xml.push_str("  </record>\n");
            }
            xml.push_str("</records>\n");
            Ok(xml.into_bytes())
        }
        "md" | "markdown" => {
            let doc = lorem::markdown_document(rng, 3, 5);
            Ok(doc.into_bytes())
        }
        _ => {
            // Default: generate some text content
            let text = lorem::plain_text(rng, 2);
            Ok(text.into_bytes())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::FormatOptions;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use std::io::Read;
    use std::path::PathBuf;

    fn make_config(files: usize, format: &str, level: u32) -> GeneratorConfig {
        GeneratorConfig {
            output_dir: PathBuf::from("/tmp"),
            name_pattern: "test_{n}".to_string(),
            extension: "zip".to_string(),
            index: 0,
            overwrite: false,
            rng: ChaCha8Rng::seed_from_u64(42),
            format_options: FormatOptions::Zip {
                file_count: files,
                contained_format: format.to_string(),
                compression_level: level,
            },
        }
    }

    #[test]
    fn test_zip_valid_header() {
        let gen = ZipGenerator;
        let mut config = make_config(3, "txt", 6);
        let result = gen.generate(&mut config).unwrap();
        // ZIP magic bytes: PK\x03\x04
        assert_eq!(&result[0..4], &[0x50, 0x4B, 0x03, 0x04]);
    }

    #[test]
    fn test_zip_contains_files() {
        let gen = ZipGenerator;
        let mut config = make_config(5, "txt", 6);
        let result = gen.generate(&mut config).unwrap();

        let reader = Cursor::new(result);
        let mut archive = zip::ZipArchive::new(reader).unwrap();
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
        let gen = ZipGenerator;
        let mut config = make_config(1, "csv", 6);
        let result = gen.generate(&mut config).unwrap();

        let reader = Cursor::new(result);
        let mut archive = zip::ZipArchive::new(reader).unwrap();
        let mut file = archive.by_index(0).unwrap();
        assert!(file.name().ends_with(".csv"));
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        assert!(content.contains("id,name,email,date"));
    }

    #[test]
    fn test_zip_json_content() {
        let gen = ZipGenerator;
        let mut config = make_config(1, "json", 6);
        let result = gen.generate(&mut config).unwrap();

        let reader = Cursor::new(result);
        let mut archive = zip::ZipArchive::new(reader).unwrap();
        let mut file = archive.by_index(0).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(parsed.is_array());
    }

    #[test]
    fn test_zip_zero_files_error() {
        let gen = ZipGenerator;
        let mut config = make_config(0, "txt", 6);
        assert!(gen.generate(&mut config).is_err());
    }

    #[test]
    fn test_zip_stored_compression() {
        let gen = ZipGenerator;
        let mut config = make_config(2, "txt", 0);
        let result = gen.generate(&mut config);
        assert!(result.is_ok());
    }
}
