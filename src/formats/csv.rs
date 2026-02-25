/// CSV file generator.
///
/// Produces valid CSV files with a header row and data rows,
/// using fields defined by the user-specified schema.

use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::data::faker;
use crate::error::{GenResult, GenerationError};

/// Generator for CSV files.
pub struct CsvGenerator;

impl Generator for CsvGenerator {
    fn format_name(&self) -> &str {
        "CSV"
    }

    fn file_extension(&self) -> &str {
        "csv"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let (rows, schema_str) = match &config.format_options {
            FormatOptions::StructuredData { rows, schema, .. } => (*rows, schema.clone()),
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "CSV generator requires StructuredData options".to_string(),
                ))
            }
        };

        let schema = faker::parse_schema(&schema_str);
        if schema.is_empty() {
            return Err(GenerationError::InvalidConfig(
                "Schema must contain at least one field".to_string(),
            ));
        }

        let mut writer = csv::Writer::from_writer(Vec::new());

        // Write header row
        let headers: Vec<&str> = schema.iter().map(|(name, _)| name.as_str()).collect();
        writer
            .write_record(&headers)
            .map_err(|e| GenerationError::Serialization(e.to_string()))?;

        // Write data rows
        for _ in 0..rows {
            let values: Vec<String> = schema
                .iter()
                .map(|(_, field_type)| faker::value_for_type(&mut config.rng, field_type))
                .collect();
            writer
                .write_record(&values)
                .map_err(|e| GenerationError::Serialization(e.to_string()))?;
        }

        writer
            .flush()
            .map_err(|e| GenerationError::Serialization(e.to_string()))?;

        writer
            .into_inner()
            .map_err(|e| GenerationError::Serialization(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::FormatOptions;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use std::path::PathBuf;

    fn make_config(rows: usize, schema: &str) -> GeneratorConfig {
        GeneratorConfig {
            output_dir: PathBuf::from("/tmp"),
            name_pattern: "test_{n}".to_string(),
            extension: "csv".to_string(),
            index: 0,
            overwrite: false,
            rng: ChaCha8Rng::seed_from_u64(42),
            format_options: FormatOptions::StructuredData {
                rows,
                schema: schema.to_string(),
                pretty: false,
            },
        }
    }

    #[test]
    fn test_csv_valid_structure() {
        let gen = CsvGenerator;
        let mut config = make_config(5, "name:string,age:int,email:email");
        let result = gen.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        let lines: Vec<&str> = text.trim().lines().collect();
        // 1 header + 5 data rows
        assert_eq!(lines.len(), 6);
        assert!(lines[0].contains("name"));
        assert!(lines[0].contains("age"));
        assert!(lines[0].contains("email"));
    }

    #[test]
    fn test_csv_correct_column_count() {
        let gen = CsvGenerator;
        let mut config = make_config(3, "a:int,b:string,c:email");
        let result = gen.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        for line in text.trim().lines() {
            let cols = line.split(',').count();
            // At least 3 columns (some values may contain commas, hence quoted)
            assert!(cols >= 3, "Expected at least 3 columns, got {cols}");
        }
    }

    #[test]
    fn test_csv_empty_schema_error() {
        let gen = CsvGenerator;
        let mut config = make_config(5, "");
        assert!(gen.generate(&mut config).is_err());
    }
}
