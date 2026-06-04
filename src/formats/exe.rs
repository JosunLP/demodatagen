/// Windows EXE (PE format) stub generator.
///
/// Produces a valid Portable Executable file with correct MZ and PE headers.
/// The generated file is recognized by Windows as an executable, though it
/// contains no meaningful code (just a minimal DOS stub that prints
/// "This program cannot be run in DOS mode").
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::error::{GenResult, GenerationError};
use rand::Rng;

/// Generator for Windows EXE stub files.
pub struct ExeGenerator;

/// Builds a minimal PE (Portable Executable) with the specified characteristics.
///
/// # Arguments
/// * `rng` - Random number generator for padding data
/// * `target_size` - Desired file size in bytes
/// * `is_dll` - If true, sets the DLL characteristic flag
pub fn build_pe_stub<R: Rng>(rng: &mut R, target_size: usize, is_dll: bool) -> Vec<u8> {
    let min_size = 512;
    let size = target_size.max(min_size);
    let mut data = Vec::with_capacity(size);

    // === DOS Header (MZ) ===
    // e_magic: "MZ"
    data.extend_from_slice(&[0x4D, 0x5A]);
    // e_cblp through e_ovno (28 bytes of DOS header fields)
    data.extend_from_slice(&[0x90, 0x00]); // e_cblp
    data.extend_from_slice(&[0x03, 0x00]); // e_cp
    data.extend_from_slice(&[0x00, 0x00]); // e_crlc
    data.extend_from_slice(&[0x04, 0x00]); // e_cparhdr
    data.extend_from_slice(&[0x00, 0x00]); // e_minalloc
    data.extend_from_slice(&[0xFF, 0xFF]); // e_maxalloc
    data.extend_from_slice(&[0x00, 0x00]); // e_ss
    data.extend_from_slice(&[0xB8, 0x00]); // e_sp
    data.extend_from_slice(&[0x00, 0x00]); // e_csum
    data.extend_from_slice(&[0x00, 0x00]); // e_ip
    data.extend_from_slice(&[0x00, 0x00]); // e_cs
    data.extend_from_slice(&[0x40, 0x00]); // e_lfarlc
    data.extend_from_slice(&[0x00, 0x00]); // e_ovno
    data.extend_from_slice(&[0x00; 8]); // e_res (8 bytes)
    data.extend_from_slice(&[0x00; 4]); // e_oemid, e_oeminfo
    data.extend_from_slice(&[0x00; 20]); // e_res2 (20 bytes)
    data.extend_from_slice(&[0x80, 0x00, 0x00, 0x00]); // e_lfanew: offset to PE header (at 0x80)

    // DOS stub program (prints message and exits)
    let dos_stub = b"\x0E\x1F\xBA\x0E\x00\xB4\x09\xCD\x21\xB8\x01\x4C\xCD\x21\
This program cannot be run in DOS mode.\r\r\n$";
    data.extend_from_slice(dos_stub);

    // Pad to PE header offset (0x80)
    while data.len() < 0x80 {
        data.push(0x00);
    }

    // === PE Signature ===
    data.extend_from_slice(&[0x50, 0x45, 0x00, 0x00]); // "PE\0\0"

    // === COFF Header (20 bytes) ===
    data.extend_from_slice(&[0x4C, 0x01]); // Machine: IMAGE_FILE_MACHINE_I386
    data.extend_from_slice(&[0x01, 0x00]); // NumberOfSections: 1
                                           // TimeDateStamp (random)
    let timestamp: u32 = rng.gen();
    data.extend_from_slice(&timestamp.to_le_bytes());
    data.extend_from_slice(&[0x00; 4]); // PointerToSymbolTable
    data.extend_from_slice(&[0x00; 4]); // NumberOfSymbols
    data.extend_from_slice(&[0xE0, 0x00]); // SizeOfOptionalHeader (224 for PE32)

    // Characteristics
    let mut characteristics: u16 = 0x0102; // EXECUTABLE_IMAGE | 32BIT_MACHINE
    if is_dll {
        characteristics |= 0x2000; // IMAGE_FILE_DLL
    }
    data.extend_from_slice(&characteristics.to_le_bytes());

    // === Optional Header (PE32, 224 bytes) ===
    data.extend_from_slice(&[0x0B, 0x01]); // Magic: PE32
    data.push(14); // MajorLinkerVersion
    data.push(0); // MinorLinkerVersion
    data.extend_from_slice(&[0x00, 0x02, 0x00, 0x00]); // SizeOfCode
    data.extend_from_slice(&[0x00; 4]); // SizeOfInitializedData
    data.extend_from_slice(&[0x00; 4]); // SizeOfUninitializedData
    data.extend_from_slice(&[0x00, 0x10, 0x00, 0x00]); // AddressOfEntryPoint
    data.extend_from_slice(&[0x00, 0x10, 0x00, 0x00]); // BaseOfCode
    data.extend_from_slice(&[0x00, 0x20, 0x00, 0x00]); // BaseOfData

    // NT-specific fields
    data.extend_from_slice(&[0x00, 0x00, 0x40, 0x00]); // ImageBase (0x400000)
    data.extend_from_slice(&[0x00, 0x10, 0x00, 0x00]); // SectionAlignment
    data.extend_from_slice(&[0x00, 0x02, 0x00, 0x00]); // FileAlignment (512)
    data.extend_from_slice(&[0x06, 0x00]); // MajorOSVersion
    data.extend_from_slice(&[0x00, 0x00]); // MinorOSVersion
    data.extend_from_slice(&[0x00; 4]); // MajorImageVersion, MinorImageVersion
    data.extend_from_slice(&[0x06, 0x00]); // MajorSubsystemVersion
    data.extend_from_slice(&[0x00, 0x00]); // MinorSubsystemVersion
    data.extend_from_slice(&[0x00; 4]); // Win32VersionValue
    data.extend_from_slice(&[0x00, 0x30, 0x00, 0x00]); // SizeOfImage
    data.extend_from_slice(&[0x00, 0x02, 0x00, 0x00]); // SizeOfHeaders (512)
    data.extend_from_slice(&[0x00; 4]); // CheckSum
    data.extend_from_slice(&[0x03, 0x00]); // Subsystem: CONSOLE
    data.extend_from_slice(&[0x40, 0x81]); // DllCharacteristics: DYNAMIC_BASE | NX_COMPAT | TERMINAL_SERVER_AWARE
    data.extend_from_slice(&[0x00, 0x00, 0x10, 0x00]); // SizeOfStackReserve
    data.extend_from_slice(&[0x00, 0x10, 0x00, 0x00]); // SizeOfStackCommit
    data.extend_from_slice(&[0x00, 0x00, 0x10, 0x00]); // SizeOfHeapReserve
    data.extend_from_slice(&[0x00, 0x10, 0x00, 0x00]); // SizeOfHeapCommit
    data.extend_from_slice(&[0x00; 4]); // LoaderFlags
    data.extend_from_slice(&[0x10, 0x00, 0x00, 0x00]); // NumberOfRvaAndSizes (16)

    // Data directories (16 entries, 8 bytes each = 128 bytes)
    for _ in 0..16 {
        data.extend_from_slice(&[0x00; 8]);
    }

    // === Section Header (.text) ===
    data.extend_from_slice(b".text\0\0\0"); // Name
    data.extend_from_slice(&[0x00, 0x02, 0x00, 0x00]); // VirtualSize
    data.extend_from_slice(&[0x00, 0x10, 0x00, 0x00]); // VirtualAddress
    data.extend_from_slice(&[0x00, 0x02, 0x00, 0x00]); // SizeOfRawData
    data.extend_from_slice(&[0x00, 0x02, 0x00, 0x00]); // PointerToRawData
    data.extend_from_slice(&[0x00; 4]); // PointerToRelocations
    data.extend_from_slice(&[0x00; 4]); // PointerToLinenumbers
    data.extend_from_slice(&[0x00; 2]); // NumberOfRelocations
    data.extend_from_slice(&[0x00; 2]); // NumberOfLinenumbers
    data.extend_from_slice(&[0x20, 0x00, 0x00, 0x60]); // Characteristics: CODE | EXECUTE | READ

    // Pad the rest with random data to reach target size
    while data.len() < size {
        data.push(rng.gen());
    }

    // Truncate if somehow over
    data.truncate(size);

    data
}

impl Generator for ExeGenerator {
    fn format_name(&self) -> &str {
        "EXE"
    }

    fn file_extension(&self) -> &str {
        "exe"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let size = match &config.format_options {
            FormatOptions::Binary { size } => *size,
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "EXE generator requires Binary options".to_string(),
                ))
            }
        };

        if size < 512 {
            return Err(GenerationError::InvalidConfig(
                "EXE size must be at least 512 bytes".to_string(),
            ));
        }

        Ok(build_pe_stub(&mut config.rng, size, false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::test_support::binary_config as make_config;

    #[test]
    fn test_exe_mz_header() {
        let gen = ExeGenerator;
        let mut config = make_config(4096);
        let result = gen.generate(&mut config).unwrap();
        assert_eq!(&result[0..2], b"MZ");
    }

    #[test]
    fn test_exe_pe_signature() {
        let gen = ExeGenerator;
        let mut config = make_config(4096);
        let result = gen.generate(&mut config).unwrap();
        // PE signature at offset 0x80
        assert_eq!(&result[0x80..0x84], &[0x50, 0x45, 0x00, 0x00]);
    }

    #[test]
    fn test_exe_correct_size() {
        let gen = ExeGenerator;
        let mut config = make_config(8192);
        let result = gen.generate(&mut config).unwrap();
        assert_eq!(result.len(), 8192);
    }

    #[test]
    fn test_exe_not_dll() {
        let gen = ExeGenerator;
        let mut config = make_config(4096);
        let result = gen.generate(&mut config).unwrap();
        // Characteristics at offset 0x80 + 4 (PE sig) + 18 = 0x96
        let characteristics = u16::from_le_bytes([result[0x96], result[0x97]]);
        assert_eq!(characteristics & 0x2000, 0, "EXE should not have DLL flag");
    }

    #[test]
    fn test_exe_too_small_error() {
        let gen = ExeGenerator;
        let mut config = make_config(100);
        assert!(gen.generate(&mut config).is_err());
    }
}
