/// `demodatagen` – A CLI tool for generating realistic demo files.
///
/// Supports structured data (JSON, XML, CSV), text (TXT, Markdown),
/// images (PNG, JPG, WebP, GIF), audio (MP3), video (MP4, WebM),
/// binary stubs (EXE, DLL), and archives (ZIP).
mod cli;
mod core;
mod data;
mod error;
mod formats;
mod update;

use crate::cli::{Cli, FormatCommand};
use crate::core::batch::{run_batch, BatchConfig};
use crate::core::generator::{FormatOptions, ImagePattern, ToneType};
use clap::Parser;
use log::{error, info};
use std::process;

fn main() {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose {
        "debug"
    } else if cli.quiet {
        "error"
    } else {
        "info"
    };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
        .format_timestamp(None)
        .init();

    if matches!(&cli.command, FormatCommand::Update) {
        match update::perform_update() {
            Ok(()) => process::exit(0),
            Err(e) => {
                error!("Update failed: {e}");
                process::exit(1);
            }
        }
    }

    // Handle update check
    if cli.check_update {
        match update::check_for_update() {
            Ok(true) => process::exit(0),
            Ok(false) => {
                println!("No updates available.");
                process::exit(0);
            }
            Err(e) => {
                error!("Update check failed: {e}");
                process::exit(1);
            }
        }
    }

    if !cli.skip_update {
        // Non-blocking update check (just log, don't fail)
        let _ = update::check_for_update();
    }

    // Resolve format options and generator from the subcommand
    let (format_options, format_key) = match &cli.command {
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
        FormatCommand::Xml {
            rows,
            schema,
            pretty,
        } => (
            FormatOptions::StructuredData {
                rows: *rows,
                schema: schema.clone(),
                pretty: *pretty,
            },
            "xml",
        ),
        FormatCommand::Csv { rows, schema } => (
            FormatOptions::StructuredData {
                rows: *rows,
                schema: schema.clone(),
                pretty: false,
            },
            "csv",
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
        FormatCommand::Txt { paragraphs, words } => (
            FormatOptions::Text {
                paragraphs: *paragraphs,
                words: *words,
            },
            "txt",
        ),
        FormatCommand::Png {
            width,
            height,
            pattern,
        } => (
            FormatOptions::Image {
                width: *width,
                height: *height,
                pattern: parse_pattern(pattern),
                frames: 1,
            },
            "png",
        ),
        FormatCommand::Jpg {
            width,
            height,
            pattern,
        } => (
            FormatOptions::Image {
                width: *width,
                height: *height,
                pattern: parse_pattern(pattern),
                frames: 1,
            },
            "jpg",
        ),
        FormatCommand::Webp {
            width,
            height,
            pattern,
        } => (
            FormatOptions::Image {
                width: *width,
                height: *height,
                pattern: parse_pattern(pattern),
                frames: 1,
            },
            "webp",
        ),
        FormatCommand::Gif {
            width,
            height,
            pattern,
            frames,
        } => (
            FormatOptions::Image {
                width: *width,
                height: *height,
                pattern: parse_pattern(pattern),
                frames: *frames,
            },
            "gif",
        ),
        FormatCommand::Mp3 {
            duration,
            sample_rate,
            tone,
        } => (
            FormatOptions::Audio {
                duration: *duration,
                sample_rate: *sample_rate,
                tone: parse_tone(tone),
            },
            "mp3",
        ),
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
        FormatCommand::Update => unreachable!("update command handled before generation"),
    };

    // Get the generator for this format
    let generator = match formats::get_generator(format_key) {
        Some(gen) => gen,
        None => {
            error!("Unknown format: {format_key}");
            process::exit(1);
        }
    };

    // Build batch configuration
    let batch_config = BatchConfig {
        output_dir: cli.output_dir,
        count: cli.count,
        name_pattern: cli.name_pattern,
        extension: generator.file_extension().to_string(),
        overwrite: cli.overwrite,
        seed: cli.seed,
        quiet: cli.quiet,
        format_options,
    };

    // Run generation
    match run_batch(generator.as_ref(), &batch_config) {
        Ok(paths) => {
            if !cli.quiet {
                info!(
                    "Generated {} file(s) in {:?}",
                    paths.len(),
                    batch_config.output_dir
                );
            }
        }
        Err(e) => {
            error!("Generation failed: {e}");
            process::exit(1);
        }
    }
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
