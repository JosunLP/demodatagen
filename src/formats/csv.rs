//! CSV / TSV file generator.
//!
//! Produces valid delimiter-separated files with a header row and data rows,
//! using fields defined by the user-specified schema. The delimiter byte is
//! configurable (comma for CSV, tab for TSV, or any single character).
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::data::Schema;
use crate::error::{GenResult, GenerationError};

/// Generator for delimiter-separated files (CSV and TSV).
pub struct CsvGenerator;

impl Generator for CsvGenerator {
    fn format_name(&self) -> &str {
        "CSV"
    }

    fn file_extension(&self) -> &str {
        "csv"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let (rows, schema_str, delimiter) = match &config.format_options {
            FormatOptions::Delimited {
                rows,
                schema,
                delimiter,
            } => (*rows, schema.clone(), *delimiter),
            // Tolerate the generic structured-data options too (delimiter = comma).
            FormatOptions::StructuredData { rows, schema, .. } => (*rows, schema.clone(), b','),
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "CSV generator requires Delimited options".to_string(),
                ))
            }
        };

        let schema = Schema::parse(&schema_str).map_err(GenerationError::InvalidConfig)?;
        if schema.is_empty() {
            return Err(GenerationError::InvalidConfig(
                "Schema must contain at least one field".to_string(),
            ));
        }

        let mut writer = csv::WriterBuilder::new()
            .delimiter(delimiter)
            .from_writer(Vec::new());

        writer
            .write_record(schema.field_names())
            .map_err(|e| GenerationError::Serialization(e.to_string()))?;

        for record in schema.generate_records(&mut config.rng, config.locale, rows) {
            let values: Vec<String> = record.iter().map(|(_, v)| v.to_flat_string()).collect();
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
    use crate::core::generator::test_support::delimited_config;

    #[test]
    fn test_csv_valid_structure() {
        let mut config = delimited_config(5, "name:string,age:int,email:email", b',');
        let result = CsvGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        let lines: Vec<&str> = text.trim().lines().collect();
        assert_eq!(lines.len(), 6); // 1 header + 5 data rows
        assert!(lines[0].contains("name"));
        assert!(lines[0].contains("age"));
        assert!(lines[0].contains("email"));
    }

    #[test]
    fn test_csv_tab_delimiter() {
        let mut config = delimited_config(2, "a:int,b:int", b'\t');
        let result = CsvGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        assert!(text.lines().next().unwrap().contains('\t'));
    }

    #[test]
    fn test_csv_semicolon_delimiter() {
        let mut config = delimited_config(2, "a:int,b:int", b';');
        let result = CsvGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        assert!(text.lines().next().unwrap().contains(';'));
    }

    #[test]
    fn test_csv_correct_column_count() {
        let mut config = delimited_config(3, "a:int,b:string,c:email", b',');
        let result = CsvGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        let mut rdr = csv::Reader::from_reader(text.as_bytes());
        for record in rdr.records() {
            assert_eq!(record.unwrap().len(), 3);
        }
    }

    #[test]
    fn test_csv_empty_schema_error() {
        let mut config = delimited_config(5, "", b',');
        assert!(CsvGenerator.generate(&mut config).is_err());
    }
}
