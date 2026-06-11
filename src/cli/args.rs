//! Reusable, composable argument groups shared across format subcommands.
//!
//! Many formats take the same parameters — every structured format wants
//! `--rows`/`--schema`, every raster image wants `--width`/`--height`/
//! `--pattern`, and so on. Rather than repeat those fields (and their defaults,
//! help text, and [`FormatOptions`] wiring) on each subcommand, the groups here
//! are declared once and pulled into subcommands via `#[command(flatten)]`.
//!
//! Each group also owns the logic that turns its fields into a
//! [`FormatOptions`] value, keeping that mapping next to the data it describes
//! instead of in one sprawling match.
use crate::core::generator::{FormatOptions, ImagePattern, ToneType};
use crate::error::{AppError, AppResult};
use crate::i18n::{tr, Language};
use clap::Args;

/// Default schema used by structured-data subcommands.
pub const DEFAULT_SCHEMA: &str = "id:sequence,name:name,email:email,created:datetime";

/// Shared `--rows` / `--schema` options for structured and tabular formats.
#[derive(Args, Debug, Clone)]
pub struct DataArgs {
    /// Number of data rows / records.
    #[arg(long, default_value_t = 10)]
    pub rows: usize,
    /// Schema definition, e.g. `"id:sequence,name:name,age:int(18..65)"`.
    #[arg(long, default_value = DEFAULT_SCHEMA)]
    pub schema: String,
}

impl DataArgs {
    /// Builds object-stream options (JSON/JSONL/YAML/TOML).
    pub fn structured(&self, pretty: bool) -> FormatOptions {
        FormatOptions::StructuredData {
            rows: self.rows,
            schema: self.schema.clone(),
            pretty,
        }
    }

    /// Builds XML options with configurable element names.
    pub fn xml(&self, pretty: bool, root: String, row_tag: String) -> FormatOptions {
        FormatOptions::Xml {
            rows: self.rows,
            schema: self.schema.clone(),
            pretty,
            root,
            row_tag,
        }
    }

    /// Builds delimited options (CSV/TSV) for an already-parsed delimiter byte.
    pub fn delimited(&self, delimiter: u8) -> FormatOptions {
        FormatOptions::Delimited {
            rows: self.rows,
            schema: self.schema.clone(),
            delimiter,
        }
    }

    /// Builds SQL / spreadsheet options targeting `table`.
    pub fn sql(&self, table: String) -> FormatOptions {
        FormatOptions::Sql {
            rows: self.rows,
            schema: self.schema.clone(),
            table,
        }
    }
}

/// Shared `--width` / `--height` / `--pattern` options for raster images.
#[derive(Args, Debug, Clone)]
pub struct ImageArgs {
    /// Image width in pixels.
    #[arg(long, default_value_t = 800)]
    pub width: u32,
    /// Image height in pixels.
    #[arg(long, default_value_t = 600)]
    pub height: u32,
    /// Pattern type: noise, gradient, shapes, checkerboard.
    #[arg(long, default_value = "gradient")]
    pub pattern: String,
}

impl ImageArgs {
    /// Builds image options, parsing the pattern and falling back to
    /// [`ImagePattern::Gradient`] (with a localized warning) on a bad value.
    pub fn options(&self, frames: u32, lang: Language) -> FormatOptions {
        FormatOptions::Image {
            width: self.width,
            height: self.height,
            pattern: parse_pattern(&self.pattern, lang),
            frames,
        }
    }
}

/// Shared `--duration` / `--sample-rate` / `--tone` options for audio.
#[derive(Args, Debug, Clone)]
pub struct AudioArgs {
    /// Duration in seconds.
    #[arg(long, default_value_t = 5.0)]
    pub duration: f32,
    /// Sample rate in Hz.
    #[arg(long, default_value_t = 44100)]
    pub sample_rate: u32,
    /// Tone type: sine, noise, sweep.
    #[arg(long, default_value = "sine")]
    pub tone: String,
}

impl AudioArgs {
    /// Builds audio options, parsing the tone and falling back to
    /// [`ToneType::Sine`] (with a localized warning) on a bad value.
    pub fn options(&self, lang: Language) -> FormatOptions {
        FormatOptions::Audio {
            duration: self.duration,
            sample_rate: self.sample_rate,
            tone: parse_tone(&self.tone, lang),
        }
    }
}

/// Shared `--duration` / `--width` / `--height` / `--fps` options for video.
#[derive(Args, Debug, Clone)]
pub struct VideoArgs {
    /// Duration in seconds.
    #[arg(long, default_value_t = 5.0)]
    pub duration: f32,
    /// Video width in pixels.
    #[arg(long, default_value_t = 640)]
    pub width: u32,
    /// Video height in pixels.
    #[arg(long, default_value_t = 480)]
    pub height: u32,
    /// Frames per second.
    #[arg(long, default_value_t = 24)]
    pub fps: u32,
}

impl VideoArgs {
    /// Builds video options from the fields verbatim.
    pub fn options(&self) -> FormatOptions {
        FormatOptions::Video {
            duration: self.duration,
            width: self.width,
            height: self.height,
            fps: self.fps,
        }
    }
}

/// Shared `--paragraphs` / `--headings` options for document formats.
#[derive(Args, Debug, Clone)]
pub struct DocArgs {
    /// Number of paragraphs.
    #[arg(long, default_value_t = 5)]
    pub paragraphs: usize,
    /// Number of section headings.
    #[arg(long, default_value_t = 3)]
    pub headings: usize,
}

impl DocArgs {
    /// Builds Markdown-style options (used by Markdown, HTML, and PDF).
    pub fn options(&self) -> FormatOptions {
        FormatOptions::Markdown {
            paragraphs: self.paragraphs,
            headings: self.headings,
        }
    }
}

/// Shared `--paragraphs` / `--words` options for plain-text formats.
#[derive(Args, Debug, Clone)]
pub struct TextArgs {
    /// Number of paragraphs.
    #[arg(long, default_value_t = 5)]
    pub paragraphs: usize,
    /// Approximate total word count (0 = auto based on paragraphs).
    #[arg(long, default_value_t = 0)]
    pub words: usize,
}

impl TextArgs {
    /// Builds plain-text options from the fields verbatim.
    pub fn options(&self) -> FormatOptions {
        FormatOptions::Text {
            paragraphs: self.paragraphs,
            words: self.words,
        }
    }
}

/// Parses a one-character delimiter spec (`,`, `;`, `\t`, `|`, …).
pub fn parse_delimiter(s: &str) -> AppResult<u8> {
    let resolved = match s {
        "\\t" | "tab" => '\t',
        "" => ',',
        other => {
            let mut chars = other.chars();
            let c = chars.next().unwrap();
            if chars.next().is_some() {
                return Err(AppError::Cli(format!(
                    "Delimiter must be a single character, got '{other}'"
                )));
            }
            c
        }
    };
    if !resolved.is_ascii() {
        return Err(AppError::Cli("Delimiter must be an ASCII character".into()));
    }
    Ok(resolved as u8)
}

/// Parses an image pattern, warning and falling back to `gradient` on error.
///
/// Public so the bespoke `ico` / `gif` subcommands (which carry their own
/// dimensions) can reuse the same lenient parsing as [`ImageArgs`].
pub fn parse_pattern(s: &str, lang: Language) -> ImagePattern {
    s.parse().unwrap_or_else(|e: String| {
        crate::ui::warn_line(&tr!(lang, warn_fallback, "error" => e, "default" => "gradient"));
        ImagePattern::Gradient
    })
}

/// Parses a tone type, warning and falling back to `sine` on error.
pub fn parse_tone(s: &str, lang: Language) -> ToneType {
    s.parse().unwrap_or_else(|e: String| {
        crate::ui::warn_line(&tr!(lang, warn_fallback, "error" => e, "default" => "sine"));
        ToneType::Sine
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_delimiter_variants() {
        assert_eq!(parse_delimiter(",").unwrap(), b',');
        assert_eq!(parse_delimiter(";").unwrap(), b';');
        assert_eq!(parse_delimiter("\\t").unwrap(), b'\t');
        assert_eq!(parse_delimiter("tab").unwrap(), b'\t');
        assert_eq!(parse_delimiter("|").unwrap(), b'|');
        assert_eq!(parse_delimiter("").unwrap(), b',');
    }

    #[test]
    fn test_parse_delimiter_rejects_multichar() {
        assert!(parse_delimiter("ab").is_err());
    }

    #[test]
    fn test_parse_pattern_fallback() {
        crate::ui::set_colors(crate::ui::ColorChoice::Never);
        assert_eq!(parse_pattern("noise", Language::En), ImagePattern::Noise);
        // Unknown pattern falls back to gradient instead of failing.
        assert_eq!(parse_pattern("bogus", Language::En), ImagePattern::Gradient);
    }

    #[test]
    fn test_data_args_builds_options() {
        let data = DataArgs {
            rows: 7,
            schema: "a:int".into(),
        };
        match data.structured(true) {
            FormatOptions::StructuredData { rows, pretty, .. } => {
                assert_eq!(rows, 7);
                assert!(pretty);
            }
            _ => panic!("expected StructuredData"),
        }
    }
}
