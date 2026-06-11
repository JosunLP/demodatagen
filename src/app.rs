//! Application orchestration: turns parsed CLI arguments into generation runs.
//!
//! This is the glue between [`crate::cli`] and the [`crate::core`] batch engine.
//! Keeping it in the library (rather than `main.rs`) lets integration tests and
//! embedders drive the whole pipeline programmatically.
//!
//! Responsibilities, in order: configure color, detect the interface language,
//! initialize logging, then dispatch the parsed command. All user-facing text
//! flows through [`crate::i18n`]; all status rendering through [`crate::ui`].
use crate::cli::args::{self, parse_delimiter};
use crate::cli::{Cli, FormatCommand};
use crate::core::batch::{run_batch, BatchConfig};
use crate::core::generator::{create_rng, FormatOptions, GeneratorConfig};
use crate::data::Locale;
use crate::error::{AppError, AppResult};
use crate::i18n::{tr, Language};
use clap::Parser;
use log::debug;
use std::io::Write;

/// Checks for updates when the feature is enabled.
#[cfg(feature = "update")]
fn check_for_update(lang: Language) -> AppResult<bool> {
    crate::update::check_for_update(lang)
}

/// Reports that update support is unavailable in this build.
#[cfg(not(feature = "update"))]
fn check_for_update(lang: Language) -> AppResult<bool> {
    Err(AppError::Update(tr!(lang, update_disabled)))
}

/// Performs a self-update when the feature is enabled, optionally to a tag.
#[cfg(feature = "update")]
fn perform_update(tag: Option<&str>, lang: Language) -> AppResult<()> {
    crate::update::update_to(tag, lang)
}

/// Reports that self-update is unavailable in this build.
#[cfg(not(feature = "update"))]
fn perform_update(_tag: Option<&str>, lang: Language) -> AppResult<()> {
    Err(AppError::Update(tr!(lang, update_disabled)))
}

/// Parses the CLI from `std::env::args` and runs the program.
///
/// Returns the process exit code.
pub fn run() -> i32 {
    let cli = Cli::parse();
    crate::ui::set_colors(cli.color.to_choice());
    let lang = Language::detect(cli.lang.as_deref());
    init_logging(&cli);
    configure_threads(cli.jobs);
    match dispatch(cli, lang) {
        Ok(code) => code,
        Err(e) => {
            crate::ui::error_line(&e.to_string());
            1
        }
    }
}

/// Sizes the global Rayon thread pool when `--jobs` is given.
///
/// Best-effort: building the global pool can only happen once, so a second call
/// (or an already-initialized pool) is silently ignored.
fn configure_threads(jobs: Option<usize>) {
    if let Some(n) = jobs {
        let _ = rayon::ThreadPoolBuilder::new()
            .num_threads(n.max(1))
            .build_global();
    }
}

/// Initializes the logger from the verbosity flags.
///
/// Logging is for *diagnostics* (opt-in via `--verbose` or `RUST_LOG`); the
/// user-facing status output is rendered separately by [`crate::ui`].
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
fn dispatch(cli: Cli, lang: Language) -> AppResult<i32> {
    // Update-only flows.
    if cli.check_update {
        return Ok(match check_for_update(lang) {
            Ok(_) => 0,
            Err(e) => {
                crate::ui::error_line(&tr!(lang, update_check_failed, "error" => e));
                1
            }
        });
    }

    if let FormatCommand::Update { tag } = &cli.command {
        return Ok(match perform_update(tag.as_deref(), lang) {
            Ok(()) => 0,
            Err(e) => {
                crate::ui::error_line(&tr!(lang, update_failed, "error" => e));
                1
            }
        });
    }

    // Informational subcommands that don't generate files.
    match &cli.command {
        FormatCommand::List => {
            crate::cli::print_format_list(lang);
            return Ok(0);
        }
        FormatCommand::Presets => {
            crate::cli::print_presets(lang);
            return Ok(0);
        }
        FormatCommand::Info => {
            crate::cli::print_info(lang, cli.jobs);
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
        let _ = check_for_update(lang);
    }

    let locale: Locale = cli.locale.parse().map_err(AppError::Cli)?;
    let (format_options, format_key) = resolve_format(&cli.command, lang)?;

    let generator = crate::formats::get_generator(format_key)
        .ok_or_else(|| AppError::Cli(tr!(lang, err_unknown_format, "format" => format_key)))?;
    let extension = generator.file_extension().to_string();

    // Surface likely schema typos as a non-fatal, localized hint. Generation
    // still proceeds (unknown types degrade to a generic word).
    if !cli.quiet {
        if let Some(schema) = schema_of(&format_options) {
            warn_unknown_schema_types(schema, lang);
        }
    }

    // Plan-only mode: report what would be generated and stop.
    if cli.dry_run {
        return Ok(dry_run_report(&cli, generator.as_ref(), &extension, lang));
    }

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
        lang,
        format_options,
    };

    match run_batch(generator.as_ref(), &batch_config) {
        Ok(paths) => {
            debug!(
                "Wrote {} file(s) to {:?}",
                paths.len(),
                batch_config.output_dir
            );
            Ok(0)
        }
        Err(e) => {
            crate::ui::error_line(&tr!(lang, err_generation_failed, "error" => e));
            Ok(1)
        }
    }
}

/// Returns the schema string carried by a [`FormatOptions`], if any.
fn schema_of(options: &FormatOptions) -> Option<&str> {
    use FormatOptions as O;
    match options {
        O::StructuredData { schema, .. }
        | O::Xml { schema, .. }
        | O::Delimited { schema, .. }
        | O::Sql { schema, .. } => Some(schema.as_str()),
        _ => None,
    }
}

/// Prints a localized "did you mean …" hint for each unrecognized schema type.
///
/// Non-fatal: unknown types still generate (a generic word); this only helps
/// users notice typos like `emial` for `email`.
fn warn_unknown_schema_types(schema: &str, lang: Language) {
    let Ok(parsed) = crate::data::Schema::parse(schema) else {
        return;
    };
    for unknown in parsed.unknown_field_types() {
        match crate::data::schema::suggest_type(&unknown) {
            Some(s) => crate::ui::warn_line(
                &tr!(lang, warn_unknown_type, "type" => unknown, "suggestion" => s),
            ),
            None => crate::ui::warn_line(&tr!(lang, warn_unknown_type_plain, "type" => unknown)),
        }
    }
}

/// Prints the plan for a `--dry-run`: header, plan line, and the file paths that
/// *would* be written (capped), without touching the filesystem.
///
/// Status lines go to stderr; the planned paths go to stdout so they stay
/// greppable and pipeable. Returns the process exit code (always `0`).
fn dry_run_report(
    cli: &Cli,
    generator: &dyn crate::core::generator::Generator,
    extension: &str,
    lang: Language,
) -> i32 {
    use crate::core::generator::resolve_filename;
    let format_name = generator.format_name();

    crate::ui::warn_line(&tr!(lang, dryrun_header));

    if cli.stdout {
        eprintln!(
            "{}",
            tr!(lang, dryrun_plan, "count" => 1, "format" => format_name, "dir" => "stdout")
        );
        eprintln!("{}", tr!(lang, dryrun_done, "count" => 1));
        return 0;
    }

    eprintln!(
        "{}",
        tr!(lang, dryrun_plan,
            "count" => cli.count,
            "format" => format_name,
            "dir" => cli.output_dir.display())
    );
    eprintln!("{}", tr!(lang, dryrun_files_title));

    const MAX_LISTED: usize = 20;
    let show = cli.count.min(MAX_LISTED);
    for i in 0..show {
        let filename = resolve_filename(&cli.name_pattern, i, extension);
        println!("{}", cli.output_dir.join(filename).display());
    }
    if cli.count > show {
        crate::ui::hint_line(&tr!(lang, dryrun_more, "count" => cli.count - show));
    }

    eprintln!("{}", tr!(lang, dryrun_done, "count" => cli.count));
    0
}

/// Resolves a format subcommand into its [`FormatOptions`] and registry key.
///
/// Each arm is a one-liner thanks to the [`args`] helper methods, which own the
/// field → [`FormatOptions`] mapping for their parameter group.
fn resolve_format(
    command: &FormatCommand,
    lang: Language,
) -> AppResult<(FormatOptions, &'static str)> {
    use FormatCommand as C;
    let resolved = match command {
        C::Json { data, pretty } => (data.structured(*pretty, lang)?, "json"),
        C::Jsonl { data } => (data.structured(false, lang)?, "jsonl"),
        C::Yaml { data } => (data.structured(true, lang)?, "yaml"),
        C::Toml { data } => (data.structured(true, lang)?, "toml"),
        C::Xml {
            data,
            pretty,
            root,
            row_tag,
        } => (
            data.xml(*pretty, root.clone(), row_tag.clone(), lang)?,
            "xml",
        ),
        C::Csv { data, delimiter } => (data.delimited(parse_delimiter(delimiter)?, lang)?, "csv"),
        C::Tsv { data } => (data.delimited(b'\t', lang)?, "tsv"),
        C::Sql { data, table } => (data.sql(table.clone(), lang)?, "sql"),
        C::Markdown { doc } => (doc.options(), "md"),
        C::Html { doc } => (doc.options(), "html"),
        C::Pdf { doc } => (doc.options(), "pdf"),
        C::Txt { text } => (text.options(), "txt"),
        C::Gzip { paragraphs, words } => (
            FormatOptions::Text {
                paragraphs: *paragraphs,
                words: *words,
            },
            "gz",
        ),
        C::Log { lines, style } => (
            FormatOptions::Log {
                lines: *lines,
                style: style.clone(),
            },
            "log",
        ),
        C::Ini { sections, keys } => (
            FormatOptions::KeyValue {
                sections: *sections,
                keys: *keys,
                env_style: false,
            },
            "ini",
        ),
        C::Env { keys } => (
            FormatOptions::KeyValue {
                sections: 1,
                keys: *keys,
                env_style: true,
            },
            "env",
        ),
        C::Png { image } => (image.options(1, lang), "png"),
        C::Jpg { image } => (image.options(1, lang), "jpg"),
        C::Webp { image } => (image.options(1, lang), "webp"),
        C::Bmp { image } => (image.options(1, lang), "bmp"),
        C::Tiff { image } => (image.options(1, lang), "tiff"),
        C::Ico { size, pattern } => (
            FormatOptions::Image {
                width: *size,
                height: *size,
                pattern: args::parse_pattern(pattern, lang),
                frames: 1,
            },
            "ico",
        ),
        C::Gif {
            width,
            height,
            pattern,
            frames,
        } => (
            FormatOptions::Image {
                width: *width,
                height: *height,
                pattern: args::parse_pattern(pattern, lang),
                frames: *frames,
            },
            "gif",
        ),
        C::Svg {
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
        C::Mp3 { audio } => (audio.options(lang), "mp3"),
        C::Wav { audio } => (audio.options(lang), "wav"),
        C::Mp4 { video } => (video.options(), "mp4"),
        C::Webm { video } => (video.options(), "webm"),
        C::Exe { size } => (FormatOptions::Binary { size: *size }, "exe"),
        C::Dll { size } => (FormatOptions::Binary { size: *size }, "dll"),
        C::Zip {
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
        C::Tar {
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
        C::Xlsx { data, sheet } => (data.sql(sheet.clone(), lang)?, "xlsx"),
        C::Update { .. } | C::List | C::Presets | C::Info | C::Completions { .. } => {
            unreachable!("non-generating subcommands are handled before resolve_format")
        }
    };
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_structured() {
        let cli = Cli::parse_from(["demodatagen", "json", "--rows", "5", "--pretty"]);
        let (opts, key) = resolve_format(&cli.command, Language::En).unwrap();
        assert_eq!(key, "json");
        assert!(matches!(
            opts,
            FormatOptions::StructuredData {
                rows: 5,
                pretty: true,
                ..
            }
        ));
    }

    #[test]
    fn test_resolve_csv_bad_delimiter_errors() {
        let cli = Cli::parse_from(["demodatagen", "csv", "--delimiter", "ab"]);
        assert!(resolve_format(&cli.command, Language::En).is_err());
    }

    #[test]
    fn test_resolve_image_and_audio() {
        crate::ui::set_colors(crate::ui::ColorChoice::Never);
        let cli = Cli::parse_from(["demodatagen", "png", "--width", "32", "--height", "16"]);
        let (opts, key) = resolve_format(&cli.command, Language::En).unwrap();
        assert_eq!(key, "png");
        assert!(matches!(
            opts,
            FormatOptions::Image {
                width: 32,
                height: 16,
                ..
            }
        ));

        let cli = Cli::parse_from(["demodatagen", "mp3", "--duration", "2"]);
        let (opts, key) = resolve_format(&cli.command, Language::En).unwrap();
        assert_eq!(key, "mp3");
        assert!(matches!(opts, FormatOptions::Audio { .. }));
    }
}
