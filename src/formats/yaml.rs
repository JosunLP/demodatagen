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
                ));
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

        let yaml = serde_saphyr::to_string(&Value::Array(array))
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
        let parsed: Value = serde_saphyr::from_str(&text).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_yaml_round_trips_generated_values() {
        // Ints, floats, bools, timestamp strings, nested arrays, and nulls.
        const SCHEMA: &str = "id:sequence,name:name,price:float(0..100),active:bool,\
                              created:datetime,tags:array(int,3),note:word?0.5";
        let mut config = structured_config(25, SCHEMA, false);
        let text = String::from_utf8(YamlGenerator.generate(&mut config).unwrap()).unwrap();
        let parsed: Value = serde_saphyr::from_str(&text).unwrap();

        // Rebuild the exact source records from an identically seeded config.
        let mut replay = structured_config(25, SCHEMA, false);
        let expected: Vec<Value> = Schema::parse(SCHEMA)
            .unwrap()
            .generate_records(&mut replay.rng, replay.locale, 25)
            .into_iter()
            .map(|record| {
                Value::Object(record.into_iter().map(|(k, v)| (k, v.to_json())).collect())
            })
            .collect();
        assert!(expected.iter().any(|r| r["note"].is_null()));
        assert_eq!(parsed, Value::Array(expected));
    }

    #[test]
    fn test_yaml_quotes_ambiguous_strings() {
        // Strings that look like other YAML scalars must come back as strings.
        let tricky = [
            "2015-05-08T01:15:50Z",
            "true",
            "null",
            "~",
            "42",
            "3.14",
            "yes",
            "",
        ];
        let source = Value::Array(tricky.iter().map(|s| Value::String((*s).into())).collect());
        let text = serde_saphyr::to_string(&source).unwrap();
        let parsed: Value = serde_saphyr::from_str(&text).unwrap();
        assert_eq!(parsed, source);

        let nested = serde_json::json!([{ "a": null, "b": [1, 2.5, [true]], "c": "1e3" }]);
        let text = serde_saphyr::to_string(&nested).unwrap();
        assert_eq!(serde_saphyr::from_str::<Value>(&text).unwrap(), nested);
    }

    #[test]
    fn test_yaml_empty_schema_error() {
        let mut config = structured_config(3, "", false);
        assert!(YamlGenerator.generate(&mut config).is_err());
    }
}
