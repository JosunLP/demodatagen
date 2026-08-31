//! iCalendar (`.ics`) generator.
//!
//! Produces an RFC 5545 `VCALENDAR` containing the requested number of
//! `VEVENT` blocks with plausible meeting titles, locations, organizers, and
//! non-overlapping-looking start/end times.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::data::{faker, lorem};
use crate::error::{GenResult, GenerationError};
use rand::RngExt;

/// Generator for iCalendar files.
pub struct IcsGenerator;

/// Event status values allowed by RFC 5545.
const STATUSES: &[&str] = &["CONFIRMED", "TENTATIVE", "CANCELLED"];

/// Formats a date-time in the compact UTC form required by iCalendar
/// (`YYYYMMDDTHHMMSSZ`).
fn ics_datetime(year: u32, month: u32, day: u32, hour: u32, minute: u32) -> String {
    format!("{year:04}{month:02}{day:02}T{hour:02}{minute:02}00Z")
}

/// Escapes an iCalendar text value (RFC 5545 §3.3.11).
fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace(',', "\\,")
        .replace(';', "\\;")
        .replace('\n', "\\n")
}

impl Generator for IcsGenerator {
    fn format_name(&self) -> &str {
        "iCalendar"
    }

    fn file_extension(&self) -> &str {
        "ics"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let events = match &config.format_options {
            FormatOptions::Calendar { events } => *events,
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "iCalendar generator requires Calendar options".to_string(),
                ));
            }
        };
        if events == 0 {
            return Err(GenerationError::InvalidConfig(
                "Event count must be at least 1".to_string(),
            ));
        }

        let locale = config.locale;
        let rng = &mut config.rng;
        let mut out = String::new();
        // RFC 5545 requires CRLF line endings.
        out.push_str("BEGIN:VCALENDAR\r\n");
        out.push_str("VERSION:2.0\r\n");
        out.push_str("PRODID:-//demodatagen//demodatagen//EN\r\n");
        out.push_str("CALSCALE:GREGORIAN\r\n");
        for _ in 0..events {
            let year = rng.random_range(2024..=2026u32);
            let month = rng.random_range(1..=12u32);
            let day = rng.random_range(1..=28u32);
            // Start early enough that the 1–3 h duration never crosses midnight.
            let hour = rng.random_range(7..=18u32);
            let minute = *[0u32, 15, 30, 45]
                .get(rng.random_range(0..4usize))
                .unwrap_or(&0);
            let duration = rng.random_range(1..=3u32);
            out.push_str("BEGIN:VEVENT\r\n");
            out.push_str(&format!("UID:{}@demodatagen\r\n", faker::uuid(rng)));
            out.push_str(&format!(
                "DTSTAMP:{}\r\n",
                ics_datetime(year, month, day, 6, 0)
            ));
            out.push_str(&format!(
                "DTSTART:{}\r\n",
                ics_datetime(year, month, day, hour, minute)
            ));
            out.push_str(&format!(
                "DTEND:{}\r\n",
                ics_datetime(year, month, day, hour + duration, minute)
            ));
            out.push_str(&format!("SUMMARY:{}\r\n", escape(&lorem::sentence(rng, 5))));
            out.push_str(&format!(
                "LOCATION:{}\r\n",
                escape(faker::city(rng, locale))
            ));
            out.push_str(&format!(
                "ORGANIZER;CN={}:mailto:{}\r\n",
                escape(&faker::full_name(rng, locale)),
                faker::email(rng, locale)
            ));
            out.push_str(&format!(
                "DESCRIPTION:{}\r\n",
                escape(&lorem::sentence(rng, 0))
            ));
            out.push_str(&format!(
                "STATUS:{}\r\n",
                STATUSES[rng.random_range(0..STATUSES.len())]
            ));
            out.push_str("END:VEVENT\r\n");
        }
        out.push_str("END:VCALENDAR\r\n");
        Ok(out.into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::test_support::calendar_config;

    #[test]
    fn test_ics_structure_and_event_count() {
        let mut config = calendar_config(5);
        let result = IcsGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        assert!(text.starts_with("BEGIN:VCALENDAR"));
        assert!(text.trim_end().ends_with("END:VCALENDAR"));
        assert_eq!(text.matches("BEGIN:VEVENT").count(), 5);
        assert_eq!(text.matches("END:VEVENT").count(), 5);
        assert_eq!(text.matches("DTSTART:").count(), 5);
    }

    #[test]
    fn test_ics_datetimes_are_wellformed() {
        let mut config = calendar_config(10);
        let result = IcsGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        for line in text.lines().filter(|l| l.starts_with("DTSTART:")) {
            let stamp = line.trim_start_matches("DTSTART:").trim();
            assert_eq!(stamp.len(), 16, "bad stamp: {stamp}");
            assert!(stamp.ends_with('Z'));
            assert_eq!(stamp.as_bytes()[8], b'T');
        }
    }

    #[test]
    fn test_ics_zero_events_errors() {
        let mut config = calendar_config(0);
        assert!(IcsGenerator.generate(&mut config).is_err());
    }

    #[test]
    fn test_ics_deterministic() {
        let mut a = calendar_config(3);
        let mut b = calendar_config(3);
        assert_eq!(
            IcsGenerator.generate(&mut a).unwrap(),
            IcsGenerator.generate(&mut b).unwrap()
        );
    }
}
