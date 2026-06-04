/// JSON file generator.
///
/// Produces valid JSON files containing arrays of objects with fields
/// defined by the user-specified schema.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::data::faker;
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

        let schema = faker::parse_schema(&schema_str);
        if schema.is_empty() {
            return Err(GenerationError::InvalidConfig(
                "Schema must contain at least one field".to_string(),
            ));
        }

        let mut records: Vec<Value> = Vec::with_capacity(rows);
        for _ in 0..rows {
            let mut obj = Map::new();
            for (field_name, field_type) in &schema {
                let val = faker::value_for_type(&mut config.rng, field_type);
                // Try to parse numeric types as numbers
                let json_val = match field_type.as_str() {
                    "int" | "integer" => val
                        .parse::<i64>()
                        .map(Value::from)
                        .unwrap_or(Value::String(val)),
                    "float" | "decimal" => val
                        .parse::<f64>()
                        .map(Value::from)
                        .unwrap_or(Value::String(val)),
                    "bool" | "boolean" => val
                        .parse::<bool>()
                        .map(Value::from)
                        .unwrap_or(Value::String(val)),
                    _ => Value::String(val),
                };
                obj.insert(field_name.clone(), json_val);
            }
            records.push(Value::Object(obj));
        }

        let json_bytes = if pretty {
            serde_json::to_vec_pretty(&records)
        } else {
            serde_json::to_vec(&records)
        }
        .map_err(|e| GenerationError::Serialization(e.to_string()))?;

        Ok(json_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::FormatOptions;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use std::path::PathBuf;

    fn make_config(rows: usize, schema: &str, pretty: bool) -> GeneratorConfig {
        GeneratorConfig {
            output_dir: PathBuf::from("/tmp"),
            name_pattern: "test_{n}".to_string(),
            extension: "json".to_string(),
            index: 0,
            overwrite: false,
            rng: ChaCha8Rng::seed_from_u64(42),
            format_options: FormatOptions::StructuredData {
                rows,
                schema: schema.to_string(),
                pretty,
            },
        }
    }

    #[test]
    fn test_json_generate_valid() {
        let gen = JsonGenerator;
        let mut config = make_config(5, "name:string,age:int,email:email", false);
        let result = gen.generate(&mut config).unwrap();
        let parsed: Vec<Value> = serde_json::from_slice(&result).unwrap();
        assert_eq!(parsed.len(), 5);
    }

    #[test]
    fn test_json_pretty_print() {
        let gen = JsonGenerator;
        let mut config = make_config(2, "name:string", true);
        let result = gen.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        assert!(text.contains('\n'));
        assert!(text.contains("  "));
    }

    #[test]
    fn test_json_numeric_types() {
        let gen = JsonGenerator;
        let mut config = make_config(1, "age:int,score:float,active:bool", false);
        let result = gen.generate(&mut config).unwrap();
        let parsed: Vec<Value> = serde_json::from_slice(&result).unwrap();
        assert!(parsed[0]["age"].is_number());
        assert!(parsed[0]["score"].is_number());
        assert!(parsed[0]["active"].is_boolean());
    }

    #[test]
    fn test_json_empty_schema_error() {
        let gen = JsonGenerator;
        let mut config = make_config(5, "", false);
        let result = gen.generate(&mut config);
        assert!(result.is_err());
    }

    #[test]
    fn test_json_deterministic() {
        let gen = JsonGenerator;
        let mut c1 = make_config(3, "name:string,age:int", false);
        let mut c2 = make_config(3, "name:string,age:int", false);
        let r1 = gen.generate(&mut c1).unwrap();
        let r2 = gen.generate(&mut c2).unwrap();
        assert_eq!(r1, r2);
    }
}
