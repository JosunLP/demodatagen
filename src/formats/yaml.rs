//! YAML file generator.
//!
//! Emits a YAML sequence of mappings, one per record, using the typed schema
//! engine so numbers, booleans, nulls, and arrays render natively.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::data::Schema;
use crate::error::{GenResult, GenerationError};
use serde_json::{Map, Value};

/// Generator for YAML files.
pub struct YamlGenerator;

impl Generator for YamlGenerator {
    fn format_name(&self) -> &str {
        "YAML"
    }

    fn file_extension(&self) -> &str {
        "yaml"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let (rows, schema_str) = match &config.format_options {
            FormatOptions::StructuredData { rows, schema, .. } => (*rows, schema.clone()),
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "YAML generator requires StructuredData options".to_string(),
                ))
            }
        };

        let schema = Schema::parse(&schema_str).map_err(GenerationError::InvalidConfig)?;
        if schema.is_empty() {
            return Err(GenerationError::InvalidConfig(
                "Schema must contain at least one field".to_string(),
            ));
        }

        let array: Vec<Value> = schema
            .generate_records(&mut config.rng, config.locale, rows)
            .into_iter()
            .map(|record| {
                let mut obj = Map::new();
                for (name, value) in record {
                    obj.insert(name, value.to_json());
                }
                Value::Object(obj)
            })
            .collect();

        let yaml = serde_yaml_ng::to_string(&Value::Array(array))
            .map_err(|e| GenerationError::Serialization(e.to_string()))?;
        Ok(yaml.into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::test_support::structured_config;

    #[test]
    fn test_yaml_parses_back() {
        let mut config = structured_config(3, "id:sequence,name:name,age:int(1..9)", false);
        let result = YamlGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        let parsed: serde_yaml_ng::Value = serde_yaml_ng::from_str(&text).unwrap();
        assert!(parsed.is_sequence());
        assert_eq!(parsed.as_sequence().unwrap().len(), 3);
    }

    #[test]
    fn test_yaml_empty_schema_error() {
        let mut config = structured_config(3, "", false);
        assert!(YamlGenerator.generate(&mut config).is_err());
    }
}
