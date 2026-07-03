//! vCard (`.vcf`) contact-card generator.
//!
//! Produces RFC 6350-compatible vCard 3.0 cards — one `BEGIN:VCARD` …
//! `END:VCARD` block per contact — with locale-aware names, emails, phone
//! numbers, postal addresses, and employers. vCard 3.0 is emitted (rather
//! than 4.0) because it is what address books import most reliably.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::data::faker;
use crate::error::{GenResult, GenerationError};

/// Generator for vCard contact files.
pub struct VcfGenerator;

/// Escapes a vCard text value (RFC 6350 §3.4: backslash, comma, semicolon,
/// and newline must be backslash-escaped).
fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace(',', "\\,")
        .replace(';', "\\;")
        .replace('\n', "\\n")
}

impl Generator for VcfGenerator {
    fn format_name(&self) -> &str {
        "vCard"
    }

    fn file_extension(&self) -> &str {
        "vcf"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let count = match &config.format_options {
            FormatOptions::Contacts { count } => *count,
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "vCard generator requires Contacts options".to_string(),
                ))
            }
        };
        if count == 0 {
            return Err(GenerationError::InvalidConfig(
                "Contact count must be at least 1".to_string(),
            ));
        }

        let locale = config.locale;
        let rng = &mut config.rng;
        let mut out = String::new();
        for _ in 0..count {
            let first = faker::first_name(rng, locale);
            let last = faker::last_name(rng, locale);
            let data = locale.data();
            // vCard requires CRLF line endings (RFC 6350 §3.2).
            out.push_str("BEGIN:VCARD\r\n");
            out.push_str("VERSION:3.0\r\n");
            out.push_str(&format!("FN:{}\r\n", escape(&format!("{first} {last}"))));
            out.push_str(&format!("N:{};{};;;\r\n", escape(last), escape(first)));
            out.push_str(&format!(
                "EMAIL;TYPE=INTERNET:{}\r\n",
                faker::email(rng, locale)
            ));
            out.push_str(&format!("TEL;TYPE=CELL:{}\r\n", faker::phone(rng, locale)));
            out.push_str(&format!("ORG:{}\r\n", escape(&faker::company(rng, locale))));
            out.push_str(&format!("TITLE:{}\r\n", escape(faker::job_title(rng))));
            out.push_str(&format!(
                "ADR;TYPE=HOME:;;{};{};{};{};{}\r\n",
                escape(&faker::street(rng, locale)),
                escape(faker::city(rng, locale)),
                escape(faker::state(rng, locale)),
                escape(&faker::zipcode(rng, locale)),
                escape(data.country)
            ));
            out.push_str(&format!("URL:{}\r\n", faker::url(rng, locale)));
            out.push_str(&format!("UID:urn:uuid:{}\r\n", faker::uuid(rng)));
            out.push_str("END:VCARD\r\n");
        }
        Ok(out.into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::test_support::contacts_config;

    #[test]
    fn test_vcf_card_count_and_structure() {
        let mut config = contacts_config(4);
        let result = VcfGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        assert_eq!(text.matches("BEGIN:VCARD").count(), 4);
        assert_eq!(text.matches("END:VCARD").count(), 4);
        assert_eq!(text.matches("VERSION:3.0").count(), 4);
        assert!(text.contains("FN:"));
        assert!(text.contains("EMAIL;TYPE=INTERNET:"));
        // vCard requires CRLF line endings.
        assert!(text.contains("\r\n"));
    }

    #[test]
    fn test_vcf_zero_contacts_errors() {
        let mut config = contacts_config(0);
        assert!(VcfGenerator.generate(&mut config).is_err());
    }

    #[test]
    fn test_vcf_deterministic() {
        let mut a = contacts_config(3);
        let mut b = contacts_config(3);
        assert_eq!(
            VcfGenerator.generate(&mut a).unwrap(),
            VcfGenerator.generate(&mut b).unwrap()
        );
    }

    #[test]
    fn test_vcf_escaping() {
        assert_eq!(escape("a,b;c\\d"), "a\\,b\\;c\\\\d");
    }
}
