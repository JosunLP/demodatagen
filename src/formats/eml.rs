//! Email message (`.eml`) generator.
//!
//! Produces RFC 5322 plain-text email messages with realistic, locale-aware
//! sender/recipient headers, an internally consistent `Date` header (the
//! weekday matches the date via Zeller's congruence), and a lorem body.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::data::{faker, lorem};
use crate::error::{GenResult, GenerationError};
use rand::{Rng, RngExt};

/// Generator for `.eml` email messages.
pub struct EmlGenerator;

const WEEKDAYS: &[&str] = &["Sat", "Sun", "Mon", "Tue", "Wed", "Thu", "Fri"];
const MONTHS: &[&str] = &[
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Returns the RFC 5322 weekday name for a Gregorian date (Zeller's
/// congruence; index 0 = Saturday).
fn weekday_name(year: u32, month: u32, day: u32) -> &'static str {
    let (m, y) = if month < 3 {
        (month + 12, year - 1)
    } else {
        (month, year)
    };
    let k = y % 100;
    let j = y / 100;
    let h = (day + (13 * (m + 1)) / 5 + k + k / 4 + j / 4 + 5 * j) % 7;
    WEEKDAYS[h as usize]
}

/// Builds an RFC 5322 `Date` header value, e.g. `Tue, 03 Jun 2025 14:07:00 +0000`.
fn rfc5322_date<R: Rng>(rng: &mut R) -> String {
    let year = rng.random_range(2023..=2026u32);
    let month = rng.random_range(1..=12u32);
    let day = rng.random_range(1..=28u32);
    let hour = rng.random_range(0..24u32);
    let minute = rng.random_range(0..60u32);
    let second = rng.random_range(0..60u32);
    format!(
        "{}, {day:02} {} {year} {hour:02}:{minute:02}:{second:02} +0000",
        weekday_name(year, month, day),
        MONTHS[(month - 1) as usize]
    )
}

impl Generator for EmlGenerator {
    fn format_name(&self) -> &str {
        "EML"
    }

    fn file_extension(&self) -> &str {
        "eml"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let paragraphs = match &config.format_options {
            FormatOptions::Text { paragraphs, .. } => (*paragraphs).max(1),
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "EML generator requires Text options".to_string(),
                ))
            }
        };

        let locale = config.locale;
        let rng = &mut config.rng;
        let from_name = faker::full_name(rng, locale);
        let from_email = faker::email(rng, locale);
        let to_name = faker::full_name(rng, locale);
        let to_email = faker::email(rng, locale);
        let subject = lorem::sentence(rng, 6);
        let date = rfc5322_date(rng);
        let message_id = format!("<{}@{}>", faker::uuid(rng), faker::domain(rng));
        let body = lorem::paragraphs(rng, paragraphs);

        // RFC 5322 requires CRLF line endings; headers, blank line, then body.
        let mut out = String::new();
        out.push_str(&format!("From: {from_name} <{from_email}>\r\n"));
        out.push_str(&format!("To: {to_name} <{to_email}>\r\n"));
        out.push_str(&format!("Subject: {subject}\r\n"));
        out.push_str(&format!("Date: {date}\r\n"));
        out.push_str(&format!("Message-ID: {message_id}\r\n"));
        out.push_str("MIME-Version: 1.0\r\n");
        out.push_str("Content-Type: text/plain; charset=UTF-8\r\n");
        out.push_str("Content-Transfer-Encoding: 8bit\r\n");
        out.push_str("\r\n");
        out.push_str(&body.replace('\n', "\r\n"));
        out.push_str("\r\n");
        Ok(out.into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::test_support::text_config;

    #[test]
    fn test_eml_headers_and_body() {
        let mut config = text_config(3, 0);
        let result = EmlGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        assert!(text.starts_with("From: "));
        for header in ["To: ", "Subject: ", "Date: ", "Message-ID: <"] {
            assert!(text.contains(header), "missing header {header}");
        }
        // Headers and body are separated by an empty line.
        let (headers, body) = text.split_once("\r\n\r\n").expect("header/body split");
        assert!(headers.contains("Content-Type: text/plain"));
        assert!(!body.trim().is_empty());
    }

    #[test]
    fn test_eml_weekday_is_consistent() {
        // Known dates: 2025-06-03 was a Tuesday; 2024-02-29 was a Thursday.
        assert_eq!(weekday_name(2025, 6, 3), "Tue");
        assert_eq!(weekday_name(2024, 2, 29), "Thu");
        assert_eq!(weekday_name(2024, 1, 1), "Mon");
    }

    #[test]
    fn test_eml_deterministic() {
        let mut a = text_config(2, 0);
        let mut b = text_config(2, 0);
        assert_eq!(
            EmlGenerator.generate(&mut a).unwrap(),
            EmlGenerator.generate(&mut b).unwrap()
        );
    }
}
