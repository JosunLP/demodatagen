//! JSON Lines (JSONL / NDJSON) generator.
//!
//! Emits one compact JSON object per line — the format consumed by log
//! pipelines, BigQuery, and many streaming tools.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::data::Schema;
use crate::error::{GenResult, GenerationError};
use serde_json::{Map, Value};

/// Generator for newline-delimited JSON files.
pub struct JsonlGenerator;

impl Generator for JsonlGenerator {
    fn format_name(&self) -> &str {
        "JSONL"
    }

    fn file_extension(&self) -> &str {
        "jsonl"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let (rows, schema_str) = match &config.format_options {
            FormatOptions::StructuredData { rows, schema, .. } => (*rows, schema.clone()),
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "JSONL generator requires StructuredData options".to_string(),
                ))
            }
        };

        let schema = Schema::parse(&schema_str).map_err(GenerationError::InvalidConfig)?;
        if schema.is_empty() {
            return Err(GenerationError::InvalidConfig(
                "Schema must contain at least one field".to_string(),
            ));
        }

        let mut out = Vec::new();
        for record in schema.generate_records(&mut config.rng, config.locale, rows) {
            let mut obj = Map::new();
            for (name, value) in record {
                obj.insert(name, value.to_json());
            }
            let line = serde_json::to_vec(&Value::Object(obj))
                .map_err(|e| GenerationError::Serialization(e.to_string()))?;
            out.extend_from_slice(&line);
            out.push(b'\n');
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::test_support::structured_config;
    use serde_json::Value;

    #[test]
    fn test_jsonl_one_object_per_line() {
        let mut config = structured_config(4, "id:sequence,name:name", false);
        let result = JsonlGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 4);
        for line in lines {
            let v: Value = serde_json::from_str(line).unwrap();
            assert!(v.is_object());
        }
    }

    #[test]
    fn test_jsonl_empty_schema_error() {
        let mut config = structured_config(3, "", false);
        assert!(JsonlGenerator.generate(&mut config).is_err());
    }
}
