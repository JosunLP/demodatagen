//! Application orchestration: turns parsed CLI arguments into generation runs.
//!
//! This is the glue between [`crate::cli`] and the [`crate::core`] batch engine.
//! Keeping it in the library (rather than `main.rs`) lets integration tests and
//! embedders drive the whole pipeline programmatically.
use crate::cli::{Cli, FormatCommand};
use crate::core::batch::{run_batch, BatchConfig};
use crate::core::generator::{create_rng, FormatOptions, GeneratorConfig, ImagePattern, ToneType};
use crate::data::Locale;
use crate::error::{AppError, AppResult};
use clap::Parser;
use log::{error, info};
use std::io::Write;

/// Error shown when update support is disabled at compile time.
#[cfg(not(feature = "update"))]
const UPDATE_DISABLED_MESSAGE: &str =
    "Self-update support is disabled in this build. Rebuild with --features update.";

/// Checks for updates when the feature is enabled.
#[cfg(feature = "update")]
fn check_for_update() -> AppResult<bool> {
    crate::update::check_for_update()
}

/// Reports that update support is unavailable in this build.
#[cfg(not(feature = "update"))]
fn check_for_update() -> AppResult<bool> {
    Err(AppError::Update(UPDATE_DISABLED_MESSAGE.to_string()))
}

/// Performs a self-update when the feature is enabled, optionally to a tag.
#[cfg(feature = "update")]
fn perform_update(tag: Option<&str>) -> AppResult<()> {
    crate::update::update_to(tag)
}

/// Reports that self-update is unavailable in this build.
#[cfg(not(feature = "update"))]
fn perform_update(_tag: Option<&str>) -> AppResult<()> {
    Err(AppError::Update(UPDATE_DISABLED_MESSAGE.to_string()))
}

/// Parses the CLI from `std::env::args` and runs the program.
///
/// Returns the process exit code.
pub fn run() -> i32 {
    let cli = Cli::parse();
    init_logging(&cli);
    match dispatch(cli) {
        Ok(code) => code,
        Err(e) => {
            error!("{e}");
            1
        }
    }
}

/// Initializes the logger from the verbosity flags.
fn init_logging(cli: &Cli) {
    let log_level = if cli.verbose {
        "debug"
    } else if cli.quiet {
        "error"
    } else {
        "info"
    };
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
        .format_timestamp(None)
        .try_init();
}

/// Dispatches a parsed CLI to the right action, returning an exit code.
fn dispatch(cli: Cli) -> AppResult<i32> {
    // Update-only flows.
    if cli.check_update {
        return match check_for_update() {
            Ok(_) => Ok(0),
            Err(e) => {
                error!("Update check failed: {e}");
                Ok(1)
            }
        };
    }

    if let FormatCommand::Update { tag } = &cli.command {
        return match perform_update(tag.as_deref()) {
            Ok(()) => Ok(0),
            Err(e) => {
                error!("Update failed: {e}");
                Ok(1)
            }
        };
    }

    // Informational subcommands that don't generate files.
    match &cli.command {
        FormatCommand::List => {
            crate::cli::print_format_list();
            return Ok(0);
        }
        FormatCommand::Completions { shell } => {
            crate::cli::print_completions(*shell);
            return Ok(0);
        }
        _ => {}
    }

    #[cfg(feature = "update")]
    if !cli.skip_update && !cli.quiet && !cli.stdout {
        // Best-effort, non-blocking update notice.
        let _ = check_for_update();
    }

    let locale: Locale = cli.locale.parse().map_err(|e: String| AppError::Cli(e))?;

    let (format_options, format_key) = resolve_format(&cli.command)?;

    let generator = crate::formats::get_generator(format_key)
        .ok_or_else(|| AppError::Cli(format!("Unknown format: {format_key}")))?;
    let extension = generator.file_extension().to_string();

    // Stream a single artifact to stdout when requested.
    if cli.stdout {
        let mut gen_config = GeneratorConfig {
            output_dir: cli.output_dir.clone(),
            name_pattern: cli.name_pattern.clone(),
            extension,
            index: 0,
            overwrite: cli.overwrite,
            rng: create_rng(cli.seed, 0),
            locale,
            format_options,
        };
        let bytes = generator
            .generate(&mut gen_config)
            .map_err(AppError::Generation)?;
        std::io::stdout().write_all(&bytes)?;
        return Ok(0);
    }

    let batch_config = BatchConfig {
        output_dir: cli.output_dir,
        count: cli.count,
        name_pattern: cli.name_pattern,
        extension,
        overwrite: cli.overwrite,
        seed: cli.seed,
        quiet: cli.quiet,
        locale,
        format_options,
    };

    match run_batch(generator.as_ref(), &batch_config) {
        Ok(paths) => {
            if !cli.quiet {
                info!(
                    "Generated {} file(s) in {:?}",
                    paths.len(),
                    batch_config.output_dir
                );
            }
            Ok(0)
        }
        Err(e) => {
            error!("Generation failed: {e}");
            Ok(1)
        }
    }
}

/// Resolves a format subcommand into its [`FormatOptions`] and registry key.
fn resolve_format(command: &FormatCommand) -> AppResult<(FormatOptions, &'static str)> {
    let resolved = match command {
        FormatCommand::Json {
            rows,
            schema,
            pretty,
        } => (
            FormatOptions::StructuredData {
                rows: *rows,
                schema: schema.clone(),
                pretty: *pretty,
            },
            "json",
        ),
        FormatCommand::Jsonl { rows, schema } => (
            FormatOptions::StructuredData {
                rows: *rows,
                schema: schema.clone(),
                pretty: false,
            },
            "jsonl",
        ),
        FormatCommand::Yaml { rows, schema } => (
            FormatOptions::StructuredData {
                rows: *rows,
                schema: schema.clone(),
                pretty: true,
            },
            "yaml",
        ),
        FormatCommand::Toml { rows, schema } => (
            FormatOptions::StructuredData {
                rows: *rows,
                schema: schema.clone(),
                pretty: true,
            },
            "toml",
        ),
        FormatCommand::Xml {
            rows,
            schema,
            pretty,
            root,
            row_tag,
        } => (
            FormatOptions::Xml {
                rows: *rows,
                schema: schema.clone(),
                pretty: *pretty,
                root: root.clone(),
                row_tag: row_tag.clone(),
            },
            "xml",
        ),
        FormatCommand::Csv {
            rows,
            schema,
            delimiter,
        } => (
            FormatOptions::Delimited {
                rows: *rows,
                schema: schema.clone(),
                delimiter: parse_delimiter(delimiter)?,
            },
            "csv",
        ),
        FormatCommand::Tsv { rows, schema } => (
            FormatOptions::Delimited {
                rows: *rows,
                schema: schema.clone(),
                delimiter: b'\t',
            },
            "tsv",
        ),
        FormatCommand::Sql {
            rows,
            schema,
            table,
        } => (
            FormatOptions::Sql {
                rows: *rows,
                schema: schema.clone(),
                table: table.clone(),
            },
            "sql",
        ),
        FormatCommand::Markdown {
            paragraphs,
            headings,
        } => (
            FormatOptions::Markdown {
                paragraphs: *paragraphs,
                headings: *headings,
            },
            "md",
        ),
        FormatCommand::Html {
            paragraphs,
            headings,
        } => (
            FormatOptions::Markdown {
                paragraphs: *paragraphs,
                headings: *headings,
            },
            "html",
        ),
        FormatCommand::Txt { paragraphs, words } => (
            FormatOptions::Text {
                paragraphs: *paragraphs,
                words: *words,
            },
            "txt",
        ),
        FormatCommand::Log { lines, style } => (
            FormatOptions::Log {
                lines: *lines,
                style: style.clone(),
            },
            "log",
        ),
        FormatCommand::Ini { sections, keys } => (
            FormatOptions::KeyValue {
                sections: *sections,
                keys: *keys,
                env_style: false,
            },
            "ini",
        ),
        FormatCommand::Env { keys } => (
            FormatOptions::KeyValue {
                sections: 1,
                keys: *keys,
                env_style: true,
            },
            "env",
        ),
        FormatCommand::Png {
            width,
            height,
            pattern,
        } => (image_options(*width, *height, pattern, 1)?, "png"),
        FormatCommand::Jpg {
            width,
            height,
            pattern,
        } => (image_options(*width, *height, pattern, 1)?, "jpg"),
        FormatCommand::Webp {
            width,
            height,
            pattern,
        } => (image_options(*width, *height, pattern, 1)?, "webp"),
        FormatCommand::Bmp {
            width,
            height,
            pattern,
        } => (image_options(*width, *height, pattern, 1)?, "bmp"),
        FormatCommand::Tiff {
            width,
            height,
            pattern,
        } => (image_options(*width, *height, pattern, 1)?, "tiff"),
        FormatCommand::Ico { size, pattern } => (image_options(*size, *size, pattern, 1)?, "ico"),
        FormatCommand::Gif {
            width,
            height,
            pattern,
            frames,
        } => (image_options(*width, *height, pattern, *frames)?, "gif"),
        FormatCommand::Svg {
            width,
            height,
            shapes,
        } => (
            FormatOptions::Svg {
                width: *width,
                height: *height,
                shapes: *shapes,
            },
            "svg",
        ),
        FormatCommand::Mp3 {
            duration,
            sample_rate,
            tone,
        } => (audio_options(*duration, *sample_rate, tone)?, "mp3"),
        FormatCommand::Wav {
            duration,
            sample_rate,
            tone,
        } => (audio_options(*duration, *sample_rate, tone)?, "wav"),
        FormatCommand::Mp4 {
            duration,
            width,
            height,
            fps,
        } => (
            FormatOptions::Video {
                duration: *duration,
                width: *width,
                height: *height,
                fps: *fps,
            },
            "mp4",
        ),
        FormatCommand::Webm {
            duration,
            width,
            height,
            fps,
        } => (
            FormatOptions::Video {
                duration: *duration,
                width: *width,
                height: *height,
                fps: *fps,
            },
            "webm",
        ),
        FormatCommand::Exe { size } => (FormatOptions::Binary { size: *size }, "exe"),
        FormatCommand::Dll { size } => (FormatOptions::Binary { size: *size }, "dll"),
        FormatCommand::Zip {
            files,
            contained_format,
            compression_level,
        } => (
            FormatOptions::Zip {
                file_count: *files,
                contained_format: contained_format.clone(),
                compression_level: *compression_level,
            },
            "zip",
        ),
        FormatCommand::Tar {
            files,
            contained_format,
        } => (
            FormatOptions::Zip {
                file_count: *files,
                contained_format: contained_format.clone(),
                compression_level: 0,
            },
            "tar",
        ),
        FormatCommand::Pdf {
            paragraphs,
            headings,
        } => (
            FormatOptions::Markdown {
                paragraphs: *paragraphs,
                headings: *headings,
            },
            "pdf",
        ),
        FormatCommand::Xlsx {
            rows,
            schema,
            sheet,
        } => (
            FormatOptions::Sql {
                rows: *rows,
                schema: schema.clone(),
                table: sheet.clone(),
            },
            "xlsx",
        ),
        FormatCommand::Gzip { paragraphs, words } => (
            FormatOptions::Text {
                paragraphs: *paragraphs,
                words: *words,
            },
            "gz",
        ),
        FormatCommand::Update { .. } | FormatCommand::List | FormatCommand::Completions { .. } => {
            unreachable!("non-generating subcommands are handled before resolve_format")
        }
    };
    Ok(resolved)
}

/// Builds [`FormatOptions::Image`], validating the pattern string.
fn image_options(width: u32, height: u32, pattern: &str, frames: u32) -> AppResult<FormatOptions> {
    Ok(FormatOptions::Image {
        width,
        height,
        pattern: parse_pattern(pattern),
        frames,
    })
}

/// Builds [`FormatOptions::Audio`], validating the tone string.
fn audio_options(duration: f32, sample_rate: u32, tone: &str) -> AppResult<FormatOptions> {
    Ok(FormatOptions::Audio {
        duration,
        sample_rate,
        tone: parse_tone(tone),
    })
}

/// Parses a one-character delimiter spec (`,`, `;`, `\t`, `|`, …).
fn parse_delimiter(s: &str) -> AppResult<u8> {
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

/// Parses an image pattern string, falling back to Gradient on error.
fn parse_pattern(s: &str) -> ImagePattern {
    s.parse().unwrap_or_else(|e| {
        eprintln!("Warning: {e}. Using 'gradient' as default.");
        ImagePattern::Gradient
    })
}

/// Parses a tone type string, falling back to Sine on error.
fn parse_tone(s: &str) -> ToneType {
    s.parse().unwrap_or_else(|e| {
        eprintln!("Warning: {e}. Using 'sine' as default.");
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
}
