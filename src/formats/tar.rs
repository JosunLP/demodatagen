//! TAR archive file generator.
//!
//! Produces valid (uncompressed) TAR archives containing multiple generated
//! files, reusing [`generate_contained_file`](crate::formats::zip::generate_contained_file).
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::error::{GenResult, GenerationError};
use crate::formats::zip::generate_contained_file;

/// Generator for TAR archive files.
pub struct TarGenerator;

impl Generator for TarGenerator {
    fn format_name(&self) -> &str {
        "TAR"
    }

    fn file_extension(&self) -> &str {
        "tar"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let (file_count, contained_format) = match &config.format_options {
            FormatOptions::Zip {
                file_count,
                contained_format,
                ..
            } => (*file_count, contained_format.clone()),
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "TAR generator requires Zip options".to_string(),
                ))
            }
        };
        if file_count == 0 {
            return Err(GenerationError::InvalidConfig(
                "TAR must contain at least one file".to_string(),
            ));
        }

        let mut builder = tar::Builder::new(Vec::new());
        for i in 0..file_count {
            let filename = format!("file_{i}.{contained_format}");
            let content =
                generate_contained_file(&mut config.rng, config.locale, &contained_format)?;
            let mut header = tar::Header::new_gnu();
            header.set_size(content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append_data(&mut header, &filename, content.as_slice())
                .map_err(|e| GenerationError::Archive(e.to_string()))?;
        }
        builder
            .into_inner()
            .map_err(|e| GenerationError::Archive(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::test_support::zip_config;
    use std::io::Read;

    #[test]
    fn test_tar_contains_files() {
        let mut config = zip_config(4, "txt", 0);
        let result = TarGenerator.generate(&mut config).unwrap();
        let mut archive = tar::Archive::new(std::io::Cursor::new(result));
        let mut count = 0;
        for entry in archive.entries().unwrap() {
            let mut entry = entry.unwrap();
            let mut content = Vec::new();
            entry.read_to_end(&mut content).unwrap();
            assert!(!content.is_empty());
            count += 1;
        }
        assert_eq!(count, 4);
    }

    #[test]
    fn test_tar_csv_entry_name() {
        let mut config = zip_config(1, "csv", 0);
        let result = TarGenerator.generate(&mut config).unwrap();
        let mut archive = tar::Archive::new(std::io::Cursor::new(result));
        let entry = archive.entries().unwrap().next().unwrap().unwrap();
        let path = entry.path().unwrap().to_string_lossy().into_owned();
        assert!(path.ends_with(".csv"));
    }

    #[test]
    fn test_tar_zero_files_error() {
        let mut config = zip_config(0, "txt", 0);
        assert!(TarGenerator.generate(&mut config).is_err());
    }
}
