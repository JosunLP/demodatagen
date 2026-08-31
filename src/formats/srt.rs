//! SubRip subtitle (`.srt`) generator.
//!
//! Produces a well-formed SubRip file: numbered cues with strictly increasing,
//! non-overlapping `HH:MM:SS,mmm --> HH:MM:SS,mmm` time ranges and one or two
//! lines of lorem text per cue.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::data::lorem;
use crate::error::{GenResult, GenerationError};
use rand::RngExt;

/// Generator for SubRip subtitle files.
pub struct SrtGenerator;

/// Formats a millisecond offset as an SRT timestamp (`HH:MM:SS,mmm`).
fn srt_timestamp(total_ms: u64) -> String {
    let ms = total_ms % 1000;
    let seconds = (total_ms / 1000) % 60;
    let minutes = (total_ms / 60_000) % 60;
    let hours = total_ms / 3_600_000;
    format!("{hours:02}:{minutes:02}:{seconds:02},{ms:03}")
}

impl Generator for SrtGenerator {
    fn format_name(&self) -> &str {
        "SRT"
    }

    fn file_extension(&self) -> &str {
        "srt"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let cues = match &config.format_options {
            FormatOptions::Subtitles { cues } => *cues,
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "SRT generator requires Subtitles options".to_string(),
                ))
            }
        };
        if cues == 0 {
            return Err(GenerationError::InvalidConfig(
                "Cue count must be at least 1".to_string(),
            ));
        }

        let rng = &mut config.rng;
        let mut out = String::new();
        let mut clock_ms: u64 = rng.random_range(200..1200);
        for i in 1..=cues {
            let duration = rng.random_range(1500..4500u64);
            let start = clock_ms;
            let end = start + duration;
            // A short gap before the next cue keeps ranges non-overlapping.
            clock_ms = end + rng.random_range(150..1200u64);

            out.push_str(&format!("{i}\n"));
            out.push_str(&format!(
                "{} --> {}\n",
                srt_timestamp(start),
                srt_timestamp(end)
            ));
            out.push_str(&lorem::sentence(rng, 0));
            if rng.random_bool(0.4) {
                out.push('\n');
                out.push_str(&lorem::sentence(rng, 4));
            }
            out.push_str("\n\n");
        }
        Ok(out.into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::test_support::subtitles_config;

    #[test]
    fn test_srt_cue_count_and_numbering() {
        let mut config = subtitles_config(6);
        let result = SrtGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        let blocks: Vec<&str> = text.trim_end().split("\n\n").collect();
        assert_eq!(blocks.len(), 6);
        for (i, block) in blocks.iter().enumerate() {
            let mut lines = block.lines();
            assert_eq!(lines.next().unwrap(), (i + 1).to_string());
            assert!(lines.next().unwrap().contains(" --> "));
        }
    }

    #[test]
    fn test_srt_timestamps_increase() {
        let mut config = subtitles_config(10);
        let result = SrtGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8(result).unwrap();
        let stamps: Vec<&str> = text.lines().filter(|l| l.contains(" --> ")).collect();
        assert_eq!(stamps.len(), 10);
        // Lexicographic comparison works because the format is fixed-width.
        let mut prev_end = String::new();
        for line in stamps {
            let (start, end) = line.split_once(" --> ").unwrap();
            assert!(start < end, "cue must end after it starts: {line}");
            assert!(prev_end.as_str() < start, "cues must not overlap: {line}");
            prev_end = end.to_string();
        }
    }

    #[test]
    fn test_srt_timestamp_format() {
        assert_eq!(srt_timestamp(0), "00:00:00,000");
        assert_eq!(srt_timestamp(3_723_456), "01:02:03,456");
    }

    #[test]
    fn test_srt_zero_cues_errors() {
        let mut config = subtitles_config(0);
        assert!(SrtGenerator.generate(&mut config).is_err());
    }

    #[test]
    fn test_srt_deterministic() {
        let mut a = subtitles_config(4);
        let mut b = subtitles_config(4);
        assert_eq!(
            SrtGenerator.generate(&mut a).unwrap(),
            SrtGenerator.generate(&mut b).unwrap()
        );
    }
}
