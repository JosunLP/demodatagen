//! Java properties (`.properties`) file generator.
//!
//! Produces a `key=value` configuration file in the flat Java properties
//! dialect: keys are lowercase and dot-namespaced by section (e.g.
//! `database.pool_size=32`), with a comment header per namespace. Reuses the
//! same realistic config-pair pool as the INI and `.env` generators.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::error::{GenResult, GenerationError};

/// Generator for Java `.properties` files.
pub struct PropertiesGenerator;

/// Namespace prefixes used to group keys, mirroring the INI section names.
const NAMESPACES: &[&str] = &[
    "server", "database", "cache", "logging", "auth", "network", "storage", "email", "worker",
    "features", "limits", "metrics",
];

impl Generator for PropertiesGenerator {
    fn format_name(&self) -> &str {
        "Properties"
    }

    fn file_extension(&self) -> &str {
        "properties"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let (sections, keys) = match &config.format_options {
            FormatOptions::KeyValue { sections, keys, .. } => ((*sections).max(1), *keys),
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "Properties generator requires KeyValue options".to_string(),
                ))
            }
        };

        let rng = &mut config.rng;
        let mut out = String::new();
        for i in 0..sections {
            // Keep namespaces unique by suffixing repeats, like the INI sections.
            let base = NAMESPACES[i % NAMESPACES.len()];
            let namespace = if i < NAMESPACES.len() {
                base.to_string()
            } else {
                format!("{base}{i}")
            };
            out.push_str(&format!("# {namespace}\n"));
            for j in 0..keys {
                let (k, v) = crate::formats::ini::config_pair(rng);
                // Dedupe key collisions within a namespace with an index suffix.
                out.push_str(&format!("{namespace}.{k}_{j}={v}\n"));
            }
            out.push('\n');
        }
        Ok(out.into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::test_support::keyvalue_config;

    #[test]
    fn test_properties_structure() {
        let mut config = keyvalue_config(3, 4, false);
        let result = PropertiesGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        let comments = text.lines().filter(|l| l.starts_with('#')).count();
        assert_eq!(comments, 3);
        let pairs: Vec<&str> = text
            .lines()
            .filter(|l| !l.starts_with('#') && !l.is_empty())
            .collect();
        assert_eq!(pairs.len(), 12);
        for line in pairs {
            let (key, _value) = line.split_once('=').expect("key=value line");
            assert!(key.contains('.'), "key not namespaced: {key}");
            assert_eq!(key, key.to_lowercase());
        }
    }

    #[test]
    fn test_properties_keys_unique() {
        let mut config = keyvalue_config(2, 10, false);
        let result = PropertiesGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        let keys: Vec<&str> = text
            .lines()
            .filter_map(|l| l.split_once('=').map(|(k, _)| k))
            .collect();
        let unique: std::collections::HashSet<&&str> = keys.iter().collect();
        assert_eq!(keys.len(), unique.len());
    }

    #[test]
    fn test_properties_deterministic() {
        let mut a = keyvalue_config(2, 3, false);
        let mut b = keyvalue_config(2, 3, false);
        assert_eq!(
            PropertiesGenerator.generate(&mut a).unwrap(),
            PropertiesGenerator.generate(&mut b).unwrap()
        );
    }
}
