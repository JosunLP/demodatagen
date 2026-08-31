//! XLSX (Excel) spreadsheet generator.
//!
//! Produces genuinely valid `.xlsx` workbooks via `rust_xlsxwriter`, with a
//! bold header row and typed cells (numbers, booleans, strings) derived from the
//! schema. Reuses the SQL options (`rows`, `schema`, `table` → sheet name).
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::data::{FieldValue, Schema};
use crate::error::{GenResult, GenerationError};
use rust_xlsxwriter::{Format, Workbook};

/// Generator for XLSX spreadsheets.
pub struct XlsxGenerator;

impl Generator for XlsxGenerator {
    fn format_name(&self) -> &str {
        "XLSX"
    }

    fn file_extension(&self) -> &str {
        "xlsx"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let (rows, schema_str, sheet) = match &config.format_options {
            FormatOptions::Sql {
                rows,
                schema,
                table,
            } => (*rows, schema.clone(), table.clone()),
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "XLSX generator requires Sql options".to_string(),
                ));
            }
        };

        let schema = Schema::parse(&schema_str).map_err(GenerationError::InvalidConfig)?;
        if schema.is_empty() {
            return Err(GenerationError::InvalidConfig(
                "Schema must contain at least one field".to_string(),
            ));
        }

        let map_err = |e: rust_xlsxwriter::XlsxError| GenerationError::Serialization(e.to_string());

        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        worksheet
            .set_name(sanitize_sheet_name(&sheet))
            .map_err(map_err)?;

        // Header row.
        let bold = Format::new().set_bold();
        for (col, name) in schema.field_names().iter().enumerate() {
            worksheet
                .write_string_with_format(0, col as u16, *name, &bold)
                .map_err(map_err)?;
        }

        // Data rows.
        let records = schema.generate_records(&mut config.rng, config.locale, rows);
        for (r, record) in records.iter().enumerate() {
            let row = (r + 1) as u32;
            for (col, (_, value)) in record.iter().enumerate() {
                let col = col as u16;
                match value {
                    FieldValue::Int(i) => worksheet
                        .write_number(row, col, *i as f64)
                        .map_err(map_err)?,
                    FieldValue::Float(f) => {
                        worksheet.write_number(row, col, *f).map_err(map_err)?
                    }
                    FieldValue::Bool(b) => {
                        worksheet.write_boolean(row, col, *b).map_err(map_err)?
                    }
                    FieldValue::Null => worksheet.write_string(row, col, "").map_err(map_err)?,
                    other => worksheet
                        .write_string(row, col, other.to_flat_string())
                        .map_err(map_err)?,
                };
            }
        }

        workbook.save_to_buffer().map_err(map_err)
    }
}

/// Excel sheet names cannot exceed 31 chars or contain `[]:*?/\`.
fn sanitize_sheet_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| if "[]:*?/\\".contains(c) { '_' } else { c })
        .take(31)
        .collect();
    if cleaned.is_empty() {
        "Sheet1".to_string()
    } else {
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::test_support::sql_config;

    #[test]
    fn test_xlsx_valid_zip_magic() {
        // XLSX is a ZIP container.
        let mut config = sql_config(5, "id:sequence,name:name,age:int(1..9)", "Data");
        let result = XlsxGenerator.generate(&mut config).unwrap();
        assert_eq!(&result[0..2], b"PK");
    }

    #[test]
    fn test_xlsx_roundtrip_with_zip_reader() {
        let mut config = sql_config(3, "id:sequence,name:name", "Data");
        let result = XlsxGenerator.generate(&mut config).unwrap();
        // An XLSX must contain the workbook part.
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(result)).unwrap();
        assert!(archive.by_name("xl/workbook.xml").is_ok());
    }

    #[test]
    fn test_xlsx_sheet_name_sanitized() {
        assert_eq!(sanitize_sheet_name("a/b:c"), "a_b_c");
        assert_eq!(sanitize_sheet_name(""), "Sheet1");
        assert_eq!(sanitize_sheet_name(&"x".repeat(40)).len(), 31);
    }

    #[test]
    fn test_xlsx_empty_schema_error() {
        let mut config = sql_config(3, "", "Data");
        assert!(XlsxGenerator.generate(&mut config).is_err());
    }
}
