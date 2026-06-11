//! TSV (tab-separated values) generator.
//!
//! TSV is CSV with a tab delimiter, so this generator delegates to the
//! [`CsvGenerator`] — the app layer supplies a tab delimiter — and only differs
//! in its reported name and extension.
use crate::core::generator::{Generator, GeneratorConfig};
use crate::error::GenResult;
use crate::formats::csv::CsvGenerator;

/// Generator for TSV files.
pub struct TsvGenerator;

impl Generator for TsvGenerator {
    fn format_name(&self) -> &str {
        "TSV"
    }

    fn file_extension(&self) -> &str {
        "tsv"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        CsvGenerator.generate(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::test_support::delimited_config;

    #[test]
    fn test_tsv_is_tab_separated() {
        let mut config = delimited_config(3, "a:int,b:int,c:int", b'\t');
        let result = TsvGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        let header = text.lines().next().unwrap();
        assert_eq!(header.split('\t').count(), 3);
    }
}
