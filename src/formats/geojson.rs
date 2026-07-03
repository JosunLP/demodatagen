//! GeoJSON (`.geojson`) generator.
//!
//! Produces an RFC 7946 `FeatureCollection` of `Point` features. Each feature
//! gets random world coordinates and a `properties` object driven by the same
//! typed schema engine used by the other structured formats, so
//! `--schema`/`--preset` work here unchanged.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::data::{faker, Schema};
use crate::error::{GenResult, GenerationError};
use serde_json::{json, Map, Value};

/// Generator for GeoJSON files.
pub struct GeoJsonGenerator;

impl Generator for GeoJsonGenerator {
    fn format_name(&self) -> &str {
        "GeoJSON"
    }

    fn file_extension(&self) -> &str {
        "geojson"
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
                    "GeoJSON generator requires StructuredData options".to_string(),
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
        let features: Vec<Value> = records
            .iter()
            .map(|record| {
                let mut properties = Map::new();
                for (name, value) in record {
                    properties.insert(name.clone(), value.to_json());
                }
                // RFC 7946 orders positions [longitude, latitude].
                let lon = faker::longitude(&mut config.rng);
                let lat = faker::latitude(&mut config.rng);
                json!({
                    "type": "Feature",
                    "geometry": { "type": "Point", "coordinates": [lon, lat] },
                    "properties": Value::Object(properties),
                })
            })
            .collect();

        let collection = json!({
            "type": "FeatureCollection",
            "features": features,
        });

        let bytes = if pretty {
            serde_json::to_vec_pretty(&collection)
        } else {
            serde_json::to_vec(&collection)
        }
        .map_err(|e| GenerationError::Serialization(e.to_string()))?;
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::test_support::structured_config;

    #[test]
    fn test_geojson_valid_feature_collection() {
        let mut config = structured_config(5, "name:name,population:int", false);
        let result = GeoJsonGenerator.generate(&mut config).unwrap();
        let parsed: Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(parsed["type"], "FeatureCollection");
        let features = parsed["features"].as_array().unwrap();
        assert_eq!(features.len(), 5);
        for f in features {
            assert_eq!(f["type"], "Feature");
            assert_eq!(f["geometry"]["type"], "Point");
            let coords = f["geometry"]["coordinates"].as_array().unwrap();
            let lon = coords[0].as_f64().unwrap();
            let lat = coords[1].as_f64().unwrap();
            assert!((-180.0..=180.0).contains(&lon));
            assert!((-90.0..=90.0).contains(&lat));
            assert!(f["properties"]["name"].is_string());
        }
    }

    #[test]
    fn test_geojson_empty_schema_errors() {
        let mut config = structured_config(3, "", false);
        assert!(GeoJsonGenerator.generate(&mut config).is_err());
    }

    #[test]
    fn test_geojson_deterministic() {
        let mut a = structured_config(3, "id:sequence,city:city", true);
        let mut b = structured_config(3, "id:sequence,city:city", true);
        assert_eq!(
            GeoJsonGenerator.generate(&mut a).unwrap(),
            GeoJsonGenerator.generate(&mut b).unwrap()
        );
    }
}
