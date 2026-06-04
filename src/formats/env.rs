//! Dotenv (`.env`) file generator.
//!
//! Produces `KEY=value` lines with uppercased keys, reusing the INI generator's
//! [`config_pair`](crate::formats::ini::config_pair) helper. Values containing
//! whitespace are quoted.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::error::{GenResult, GenerationError};

/// Generator for `.env` files.
pub struct EnvGenerator;

impl Generator for EnvGenerator {
    fn format_name(&self) -> &str {
        "ENV"
    }

    fn file_extension(&self) -> &str {
        "env"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let keys = match &config.format_options {
            FormatOptions::KeyValue { keys, .. } => *keys,
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "ENV generator requires KeyValue options".to_string(),
                ))
            }
        };

        let rng = &mut config.rng;
        let mut out = String::new();
        for i in 0..keys {
            let (k, v) = crate::formats::ini::config_pair(rng);
            // Uppercase, dedupe collisions with an index suffix.
            let key = format!("{}_{i}", k.to_uppercase());
            let value = if v.contains(char::is_whitespace) {
                format!("\"{v}\"")
            } else {
                v
            };
            out.push_str(&format!("{key}={value}\n"));
        }
        Ok(out.into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::test_support::keyvalue_config;

    #[test]
    fn test_env_line_count_and_format() {
        let mut config = keyvalue_config(1, 8, true);
        let result = EnvGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        assert_eq!(text.lines().count(), 8);
        for line in text.lines() {
            assert!(line.contains('='));
            let key = line.split('=').next().unwrap();
            assert_eq!(key, key.to_uppercase());
        }
    }
}
