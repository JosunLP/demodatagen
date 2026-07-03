//! INI configuration-file generator.
//!
//! Produces an INI file with the requested number of `[section]` blocks, each
//! containing realistic `key = value` config pairs. The `config_pair` helper
//! is shared with the `.env` generator.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::data::faker;
use crate::error::{GenResult, GenerationError};
use rand::{Rng, RngExt};

/// Generator for INI files.
pub struct IniGenerator;

const SECTION_NAMES: &[&str] = &[
    "server", "database", "cache", "logging", "auth", "network", "storage", "email", "worker",
    "features", "limits", "metrics",
];

const CONFIG_KEYS: &[&str] = &[
    "host",
    "port",
    "timeout",
    "max_connections",
    "debug",
    "log_level",
    "retries",
    "username",
    "base_url",
    "enabled",
    "region",
    "bucket",
    "cache_ttl",
    "workers",
    "api_key",
    "pool_size",
];

const LOG_LEVELS: &[&str] = &["trace", "debug", "info", "warn", "error"];
const REGIONS: &[&str] = &["us-east-1", "us-west-2", "eu-central-1", "ap-southeast-1"];

/// Generates one realistic `(key, value)` configuration pair.
pub(crate) fn config_pair<R: Rng>(rng: &mut R) -> (String, String) {
    let key = CONFIG_KEYS[rng.random_range(0..CONFIG_KEYS.len())];
    let value = match key {
        "host" | "base_url" => faker::domain(rng),
        "port" => rng.random_range(1024..65535u32).to_string(),
        "timeout" | "cache_ttl" => format!("{}", rng.random_range(1..600u32)),
        "max_connections" | "retries" | "workers" | "pool_size" => {
            rng.random_range(1..256u32).to_string()
        }
        "debug" | "enabled" => faker::boolean(rng).to_string(),
        "log_level" => LOG_LEVELS[rng.random_range(0..LOG_LEVELS.len())].to_string(),
        "username" => faker::username(rng, crate::data::Locale::EnUs),
        "region" => REGIONS[rng.random_range(0..REGIONS.len())].to_string(),
        "bucket" => faker::slug(rng),
        "api_key" => faker::uuid(rng).replace('-', ""),
        _ => faker::slug(rng),
    };
    (key.to_string(), value)
}

impl Generator for IniGenerator {
    fn format_name(&self) -> &str {
        "INI"
    }

    fn file_extension(&self) -> &str {
        "ini"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let (sections, keys) = match &config.format_options {
            FormatOptions::KeyValue { sections, keys, .. } => (*sections, *keys),
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "INI generator requires KeyValue options".to_string(),
                ))
            }
        };

        let rng = &mut config.rng;
        let mut out = String::new();
        for i in 0..sections.max(1) {
            // Keep section names unique by suffixing repeats.
            let base = SECTION_NAMES[i % SECTION_NAMES.len()];
            let name = if i < SECTION_NAMES.len() {
                base.to_string()
            } else {
                format!("{base}{i}")
            };
            out.push_str(&format!("[{name}]\n"));
            for _ in 0..keys {
                let (k, v) = config_pair(rng);
                out.push_str(&format!("{k} = {v}\n"));
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
    fn test_ini_has_sections() {
        let mut config = keyvalue_config(3, 4, false);
        let result = IniGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        let section_count = text
            .lines()
            .filter(|l| l.starts_with('[') && l.ends_with(']'))
            .count();
        assert_eq!(section_count, 3);
        let kv_count = text.lines().filter(|l| l.contains(" = ")).count();
        assert_eq!(kv_count, 12);
    }

    #[test]
    fn test_ini_deterministic() {
        let mut a = keyvalue_config(2, 3, false);
        let mut b = keyvalue_config(2, 3, false);
        assert_eq!(
            IniGenerator.generate(&mut a).unwrap(),
            IniGenerator.generate(&mut b).unwrap()
        );
    }
}
