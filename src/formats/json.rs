//! JSON file generator.
//!
//! Produces valid JSON files containing arrays of objects with fields defined
//! by the user-specified schema. Values are emitted with their natural JSON
//! type (numbers, booleans, null, arrays, strings) via the schema engine.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::data::Schema;
use crate::error::{GenResult, GenerationError};
use serde_json::{Map, Value};

/// Generator for JSON files.
pub struct JsonGenerator;

impl Generator for JsonGenerator {
    fn format_name(&self) -> &str {
        "JSON"
    }

    fn file_extension(&self) -> &str {
        "json"
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
                    "JSON generator requires StructuredData options".to_string(),
                ))
            }
        };

        let schema = Schema::parse(&schema_str).map_err(GenerationError::InvalidConfig)?;
        if schema.is_empty() {
            return Err(GenerationError::InvalidConfig(
                "Schema must contain at least one field".to_string(),
            ));
        }

        let records = schema.generate_records(&mut config.rng, config.locale, rows);
        let array: Vec<Value> = records
            .iter()
            .map(|record| {
                let mut obj = Map::new();
                for (name, value) in record {
                    obj.insert(name.clone(), value.to_json());
                }
                Value::Object(obj)
            })
            .collect();

        let bytes = if pretty {
            serde_json::to_vec_pretty(&array)
        } else {
            serde_json::to_vec(&array)
        }
        .map_err(|e| GenerationError::Serialization(e.to_string()))?;

        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::test_support::structured_config;
    use serde_json::Value;

    #[test]
    fn test_json_generate_valid() {
        let mut config = structured_config(5, "name:string,age:int,email:email", false);
        let result = JsonGenerator.generate(&mut config).unwrap();
        let parsed: Vec<Value> = serde_json::from_slice(&result).unwrap();
        assert_eq!(parsed.len(), 5);
    }

    #[test]
    fn test_json_pretty_print() {
        let mut config = structured_config(2, "name:string", true);
        let result = JsonGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        assert!(text.contains('\n'));
        assert!(text.contains("  "));
    }

    #[test]
    fn test_json_numeric_types() {
        let mut config = structured_config(1, "age:int,score:float,active:bool", false);
        let result = JsonGenerator.generate(&mut config).unwrap();
        let parsed: Vec<Value> = serde_json::from_slice(&result).unwrap();
        assert!(parsed[0]["age"].is_number());
        assert!(parsed[0]["score"].is_number());
        assert!(parsed[0]["active"].is_boolean());
    }

    #[test]
    fn test_json_nullable_and_array() {
        let mut config = structured_config(1, "x:int?1.0,tags:array(word,2)", false);
        let result = JsonGenerator.generate(&mut config).unwrap();
        let parsed: Vec<Value> = serde_json::from_slice(&result).unwrap();
        assert!(parsed[0]["x"].is_null());
        assert_eq!(parsed[0]["tags"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_json_empty_schema_error() {
        let mut config = structured_config(5, "", false);
        assert!(JsonGenerator.generate(&mut config).is_err());
    }

    #[test]
    fn test_json_deterministic() {
        let mut c1 = structured_config(3, "id:sequence,name:string,age:int", false);
        let mut c2 = structured_config(3, "id:sequence,name:string,age:int", false);
        assert_eq!(
            JsonGenerator.generate(&mut c1).unwrap(),
            JsonGenerator.generate(&mut c2).unwrap()
        );
    }
}
