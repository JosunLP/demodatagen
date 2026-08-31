//! SQL script generator.
//!
//! Emits a `CREATE TABLE` statement (with column types inferred from the
//! schema) followed by a multi-row `INSERT`, producing a script that loads
//! cleanly into SQLite, PostgreSQL, or MySQL.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::data::Schema;
use crate::error::{GenResult, GenerationError};

/// Generator for SQL scripts.
pub struct SqlGenerator;

impl Generator for SqlGenerator {
    fn format_name(&self) -> &str {
        "SQL"
    }

    fn file_extension(&self) -> &str {
        "sql"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let (rows, schema_str, table) = match &config.format_options {
            FormatOptions::Sql {
                rows,
                schema,
                table,
            } => (*rows, schema.clone(), table.clone()),
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "SQL generator requires Sql options".to_string(),
                ));
            }
        };

        let schema = Schema::parse(&schema_str).map_err(GenerationError::InvalidConfig)?;
        if schema.is_empty() {
            return Err(GenerationError::InvalidConfig(
                "Schema must contain at least one field".to_string(),
            ));
        }

        let table_ident = quote_ident(&table);
        let mut sql = String::new();

        // DDL.
        sql.push_str(&format!("CREATE TABLE {table_ident} (\n"));
        let cols: Vec<String> = schema
            .fields
            .iter()
            .map(|f| format!("  {} {}", quote_ident(&f.name), f.sql_type()))
            .collect();
        sql.push_str(&cols.join(",\n"));
        sql.push_str("\n);\n\n");

        // DML.
        let col_list: Vec<String> = schema.fields.iter().map(|f| quote_ident(&f.name)).collect();
        let records = schema.generate_records(&mut config.rng, config.locale, rows);
        if !records.is_empty() {
            sql.push_str(&format!(
                "INSERT INTO {table_ident} ({}) VALUES\n",
                col_list.join(", ")
            ));
            let value_rows: Vec<String> = records
                .iter()
                .map(|record| {
                    let vals: Vec<String> =
                        record.iter().map(|(_, v)| v.to_sql_literal()).collect();
                    format!("  ({})", vals.join(", "))
                })
                .collect();
            sql.push_str(&value_rows.join(",\n"));
            sql.push_str(";\n");
        }

        Ok(sql.into_bytes())
    }
}

/// Quotes a SQL identifier with double quotes, escaping embedded quotes.
fn quote_ident(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::test_support::sql_config;

    #[test]
    fn test_sql_has_create_and_insert() {
        let mut config = sql_config(5, "id:sequence,name:name,age:int(18..65)", "users");
        let result = SqlGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        assert!(text.contains("CREATE TABLE \"users\""));
        assert!(text.contains("INSERT INTO \"users\""));
        assert!(text.contains("\"id\" INTEGER"));
        assert!(text.contains("\"name\" TEXT"));
        assert!(text.contains("\"age\" INTEGER"));
    }

    #[test]
    fn test_sql_row_count() {
        let mut config = sql_config(7, "id:sequence", "t");
        let result = SqlGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        // 7 value tuples → 7 lines beginning with "  (".
        let count = text
            .lines()
            .filter(|l| l.trim_start().starts_with('('))
            .count();
        assert_eq!(count, 7);
    }

    #[test]
    fn test_sql_escapes_quotes() {
        let mut config = sql_config(3, "v:const(O'Brien)", "t");
        let result = SqlGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        assert!(text.contains("'O''Brien'"));
    }

    #[test]
    fn test_sql_empty_schema_error() {
        let mut config = sql_config(3, "", "t");
        assert!(SqlGenerator.generate(&mut config).is_err());
    }
}
