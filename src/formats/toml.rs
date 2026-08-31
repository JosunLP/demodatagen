//! TOML file generator.
//!
//! Emits records as a TOML array of tables under a `records` key. Because TOML
//! has no null type, null fields are omitted.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::data::{FieldValue, Schema};
use crate::error::{GenResult, GenerationError};
use toml::value::{Array, Table, Value};

/// Generator for TOML files.
pub struct TomlGenerator;

impl Generator for TomlGenerator {
    fn format_name(&self) -> &str {
        "TOML"
    }

    fn file_extension(&self) -> &str {
        "toml"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let (rows, schema_str) = match &config.format_options {
            FormatOptions::StructuredData { rows, schema, .. } => (*rows, schema.clone()),
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "TOML generator requires StructuredData options".to_string(),
                ));
            }
        };

        let schema = Schema::parse(&schema_str).map_err(GenerationError::InvalidConfig)?;
        if schema.is_empty() {
            return Err(GenerationError::InvalidConfig(
                "Schema must contain at least one field".to_string(),
            ));
        }

        let mut records: Array = Array::new();
        for record in schema.generate_records(&mut config.rng, config.locale, rows) {
            let mut table = Table::new();
            for (name, value) in record {
                if let Some(tv) = to_toml(&value) {
                    table.insert(name, tv);
                }
            }
            records.push(Value::Table(table));
        }

        let mut root = Table::new();
        root.insert("records".to_string(), Value::Array(records));

        let text = toml::to_string_pretty(&Value::Table(root))
            .map_err(|e| GenerationError::Serialization(e.to_string()))?;
        Ok(text.into_bytes())
    }
}

/// Converts a [`FieldValue`] to a TOML value, dropping nulls (and nulls within
/// arrays) since TOML cannot represent them.
fn to_toml(value: &FieldValue) -> Option<Value> {
    match value {
        FieldValue::Null => None,
        FieldValue::Bool(b) => Some(Value::Boolean(*b)),
        FieldValue::Int(i) => Some(Value::Integer(*i)),
        FieldValue::Float(f) => Some(Value::Float(*f)),
        FieldValue::Str(s) => Some(Value::String(s.clone())),
        FieldValue::Array(items) => Some(Value::Array(items.iter().filter_map(to_toml).collect())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::test_support::structured_config;

    #[test]
    fn test_toml_parses_back() {
        let mut config = structured_config(3, "id:sequence,name:name,active:bool", false);
        let result = TomlGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        let parsed: toml::Value = toml::from_str(&text).unwrap();
        let records = parsed.get("records").unwrap().as_array().unwrap();
        assert_eq!(records.len(), 3);
    }

    #[test]
    fn test_toml_skips_nulls() {
        let mut config = structured_config(1, "x:int?1.0,y:int", false);
        let result = TomlGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        let parsed: toml::Value = toml::from_str(&text).unwrap();
        let rec = &parsed.get("records").unwrap().as_array().unwrap()[0];
        assert!(rec.get("x").is_none());
        assert!(rec.get("y").is_some());
    }

    #[test]
    fn test_toml_empty_schema_error() {
        let mut config = structured_config(3, "", false);
        assert!(TomlGenerator.generate(&mut config).is_err());
    }
}
