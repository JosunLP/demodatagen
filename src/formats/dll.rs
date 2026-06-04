/// Windows DLL (PE format) stub generator.
///
/// Produces a valid Portable Executable file with the DLL characteristic flag set.
/// Reuses the PE building logic from the EXE generator.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::error::{GenResult, GenerationError};
use crate::formats::exe::build_pe_stub;

/// Generator for Windows DLL stub files.
pub struct DllGenerator;

impl Generator for DllGenerator {
    fn format_name(&self) -> &str {
        "DLL"
    }

    fn file_extension(&self) -> &str {
        "dll"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let size = match &config.format_options {
            FormatOptions::Binary { size } => *size,
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "DLL generator requires Binary options".to_string(),
                ))
            }
        };

        if size < 512 {
            return Err(GenerationError::InvalidConfig(
                "DLL size must be at least 512 bytes".to_string(),
            ));
        }

        Ok(build_pe_stub(&mut config.rng, size, true))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::test_support::binary_config as make_config;

    #[test]
    fn test_dll_mz_header() {
        let gen = DllGenerator;
        let mut config = make_config(4096);
        let result = gen.generate(&mut config).unwrap();
        assert_eq!(&result[0..2], b"MZ");
    }

    #[test]
    fn test_dll_has_dll_flag() {
        let gen = DllGenerator;
        let mut config = make_config(4096);
        let result = gen.generate(&mut config).unwrap();
        // Characteristics at offset 0x96
        let characteristics = u16::from_le_bytes([result[0x96], result[0x97]]);
        assert_ne!(characteristics & 0x2000, 0, "DLL should have DLL flag set");
    }

    #[test]
    fn test_dll_correct_size() {
        let gen = DllGenerator;
        let mut config = make_config(2048);
        let result = gen.generate(&mut config).unwrap();
        assert_eq!(result.len(), 2048);
    }
}
