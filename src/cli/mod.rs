//! CLI argument definitions using `clap`.
//!
//! Defines the top-level [`Cli`] with global options and the per-format
//! subcommands ([`FormatCommand`]), plus the informational `list` and
//! `completions` subcommands. Repeated parameter sets live in
//! [`args`] and are pulled in via `#[command(flatten)]`.
pub mod args;

use crate::i18n::{tr, Language};
use crate::ui::{self, ColorChoice};
use args::{AudioArgs, DataArgs, DocArgs, ImageArgs, TextArgs, VideoArgs};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use console::style;
use std::path::PathBuf;

/// When to colorize terminal output.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, ValueEnum)]
pub enum ColorWhen {
    /// Colorize when stderr is a terminal and `NO_COLOR` is unset (default).
    #[default]
    Auto,
    /// Always colorize.
    Always,
    /// Never colorize.
    Never,
}

impl ColorWhen {
    /// Maps to the presentation layer's [`ColorChoice`].
    pub fn to_choice(self) -> ColorChoice {
        match self {
            ColorWhen::Auto => ColorChoice::Auto,
            ColorWhen::Always => ColorChoice::Always,
            ColorWhen::Never => ColorChoice::Never,
        }
    }
}

/// A fast, offline, fully internationalized CLI for generating realistic demo
/// files in many formats.
///
/// Supports structured data (JSON, JSONL, YAML, TOML, XML, CSV, TSV, SQL),
/// text (TXT, Markdown, HTML, LOG, INI, ENV), images (PNG, JPG, WebP, BMP,
/// TIFF, ICO, GIF, SVG), audio (MP3, WAV), video (MP4, WebM), documents
/// (PDF, XLSX), binary stubs (EXE, DLL), and archives (ZIP, TAR, GZIP) — across
/// ten data locales and four interface languages.
#[derive(Parser, Debug)]
#[command(name = "demodatagen")]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Target output directory for generated files.
    #[arg(short = 'o', long, default_value = "./output", global = true)]
    pub output_dir: PathBuf,

    /// Number of files to generate.
    #[arg(short = 'c', long, default_value_t = 1, global = true)]
    pub count: usize,

    /// RNG seed for reproducible output.
    #[arg(short = 's', long, global = true)]
    pub seed: Option<u64>,

    /// Data locale for region-specific fake data (e.g. `en_us`, `de_de`, `fr_fr`).
    #[arg(short = 'l', long, default_value = "en_us", global = true)]
    pub locale: String,

    /// Interface language for messages (`en`, `de`, `fr`, `es`).
    /// Defaults to the system locale, then English.
    #[arg(long, global = true)]
    pub lang: Option<String>,

    /// When to use colored output.
    #[arg(long, value_enum, default_value_t = ColorWhen::Auto, global = true)]
    pub color: ColorWhen,

    /// Write a single generated file to stdout instead of disk
    /// (forces `--count 1` and suppresses the progress bar).
    #[arg(long, default_value_t = false, global = true)]
    pub stdout: bool,

    /// Filename pattern. Use `{n}` as a placeholder for the file index.
    #[arg(short = 'n', long, default_value = "demo_{n}", global = true)]
    pub name_pattern: String,

    /// Overwrite existing files without prompting.
    #[arg(long, default_value_t = false, global = true)]
    pub overwrite: bool,

    /// Suppress all output except errors.
    #[arg(short = 'q', long, default_value_t = false, global = true)]
    pub quiet: bool,

    /// Enable verbose (debug) logging.
    #[arg(short = 'v', long, default_value_t = false, global = true)]
    pub verbose: bool,

    /// Skip the auto-update check on startup.
    #[arg(long, default_value_t = false, global = true)]
    pub skip_update: bool,

    /// Only check for updates without running the main command.
    #[arg(long, default_value_t = false, global = true)]
    pub check_update: bool,

    /// The command to run.
    #[command(subcommand)]
    pub command: FormatCommand,
}

/// Subcommands for supported file formats and maintenance operations.
#[derive(Subcommand, Debug)]
pub enum FormatCommand {
    /// Generate JSON files (array of objects) with structured fake data.
    Json {
        #[command(flatten)]
        data: DataArgs,
        /// Pretty-print the JSON output.
        #[arg(long, default_value_t = false)]
        pretty: bool,
    },

    /// Generate newline-delimited JSON (JSONL / NDJSON).
    Jsonl {
        #[command(flatten)]
        data: DataArgs,
    },

    /// Generate YAML files with structured fake data.
    Yaml {
        #[command(flatten)]
        data: DataArgs,
    },

    /// Generate TOML files with an array of tables.
    Toml {
        #[command(flatten)]
        data: DataArgs,
    },

    /// Generate XML files with structured fake data.
    Xml {
        #[command(flatten)]
        data: DataArgs,
        /// Pretty-print (indent) the XML output.
        #[arg(long, default_value_t = false)]
        pretty: bool,
        /// Name of the root element.
        #[arg(long, default_value = "records")]
        root: String,
        /// Name of each record element.
        #[arg(long = "row-tag", default_value = "record")]
        row_tag: String,
    },

    /// Generate CSV files with structured fake data.
    Csv {
        #[command(flatten)]
        data: DataArgs,
        /// Field delimiter (single character, or `\t`/`tab` for tab).
        #[arg(long, default_value = ",")]
        delimiter: String,
    },

    /// Generate TSV (tab-separated) files.
    Tsv {
        #[command(flatten)]
        data: DataArgs,
    },

    /// Generate a SQL script with `CREATE TABLE` + `INSERT` statements.
    Sql {
        #[command(flatten)]
        data: DataArgs,
        /// Target table name.
        #[arg(long, default_value = "demo_data")]
        table: String,
    },

    /// Generate Markdown files with headings and paragraphs.
    Markdown {
        #[command(flatten)]
        doc: DocArgs,
    },

    /// Generate HTML documents with headings and paragraphs.
    Html {
        #[command(flatten)]
        doc: DocArgs,
    },

    /// Generate plain text files.
    Txt {
        #[command(flatten)]
        text: TextArgs,
    },

    /// Generate synthetic log files.
    Log {
        /// Number of log lines.
        #[arg(long, default_value_t = 100)]
        lines: usize,
        /// Log style: apache, syslog, or json.
        #[arg(long, default_value = "apache")]
        style: String,
    },

    /// Generate INI configuration files.
    Ini {
        /// Number of sections.
        #[arg(long, default_value_t = 3)]
        sections: usize,
        /// Number of keys per section.
        #[arg(long, default_value_t = 5)]
        keys: usize,
    },

    /// Generate a dotenv (`.env`) file.
    Env {
        /// Number of keys.
        #[arg(long, default_value_t = 10)]
        keys: usize,
    },

    /// Generate PNG images.
    Png {
        #[command(flatten)]
        image: ImageArgs,
    },

    /// Generate JPEG images.
    Jpg {
        #[command(flatten)]
        image: ImageArgs,
    },

    /// Generate WebP images.
    Webp {
        #[command(flatten)]
        image: ImageArgs,
    },

    /// Generate BMP images.
    Bmp {
        #[command(flatten)]
        image: ImageArgs,
    },

    /// Generate TIFF images.
    Tiff {
        #[command(flatten)]
        image: ImageArgs,
    },

    /// Generate ICO icon files.
    Ico {
        /// Icon size in pixels (square).
        #[arg(long, default_value_t = 64)]
        size: u32,
        /// Pattern type: noise, gradient, shapes, checkerboard.
        #[arg(long, default_value = "shapes")]
        pattern: String,
    },

    /// Generate GIF images (optionally animated).
    Gif {
        /// Image width in pixels.
        #[arg(long, default_value_t = 320)]
        width: u32,
        /// Image height in pixels.
        #[arg(long, default_value_t = 240)]
        height: u32,
        /// Pattern type: noise, gradient, shapes, checkerboard.
        #[arg(long, default_value = "gradient")]
        pattern: String,
        /// Number of animation frames (1 = static).
        #[arg(long, default_value_t = 10)]
        frames: u32,
    },

    /// Generate SVG vector images.
    Svg {
        /// Canvas width in pixels.
        #[arg(long, default_value_t = 400)]
        width: u32,
        /// Canvas height in pixels.
        #[arg(long, default_value_t = 300)]
        height: u32,
        /// Number of random shapes.
        #[arg(long, default_value_t = 20)]
        shapes: usize,
    },

    /// Generate MP3 audio files.
    Mp3 {
        #[command(flatten)]
        audio: AudioArgs,
    },

    /// Generate WAV audio files (uncompressed PCM).
    Wav {
        #[command(flatten)]
        audio: AudioArgs,
    },

    /// Generate MP4 video files.
    Mp4 {
        #[command(flatten)]
        video: VideoArgs,
    },

    /// Generate WebM video files.
    Webm {
        #[command(flatten)]
        video: VideoArgs,
    },

    /// Generate a Windows EXE stub (PE format).
    Exe {
        /// Desired file size in bytes (minimum 512).
        #[arg(long, default_value_t = 4096)]
        size: usize,
    },

    /// Generate a Windows DLL stub (PE format).
    Dll {
        /// Desired file size in bytes (minimum 512).
        #[arg(long, default_value_t = 4096)]
        size: usize,
    },

    /// Generate ZIP archive files.
    Zip {
        /// Number of files to include in the archive.
        #[arg(long, default_value_t = 5)]
        files: usize,
        /// Format of the contained files (e.g., csv, txt, json).
        #[arg(long, default_value = "txt")]
        contained_format: String,
        /// Compression level (0 = store, 1-9 = deflate).
        #[arg(long, default_value_t = 6)]
        compression_level: u32,
    },

    /// Generate TAR archive files.
    Tar {
        /// Number of files to include in the archive.
        #[arg(long, default_value_t = 5)]
        files: usize,
        /// Format of the contained files (e.g., csv, txt, json).
        #[arg(long, default_value = "txt")]
        contained_format: String,
    },

    /// Generate PDF documents.
    Pdf {
        #[command(flatten)]
        doc: DocArgs,
    },

    /// Generate XLSX (Excel) spreadsheets.
    Xlsx {
        #[command(flatten)]
        data: DataArgs,
        /// Worksheet name.
        #[arg(long, default_value = "Sheet1")]
        sheet: String,
    },

    /// Generate a gzip-compressed text file (`.gz`).
    Gzip {
        /// Number of paragraphs of text to compress.
        #[arg(long, default_value_t = 20)]
        paragraphs: usize,
        /// Approximate total word count (0 = auto based on paragraphs).
        #[arg(long, default_value_t = 0)]
        words: usize,
    },

    /// List all supported formats, schema types, locales, and languages.
    List,

    /// Print shell completion script for the given shell.
    Completions {
        /// Target shell: bash, zsh, fish, powershell, elvish.
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Update the installed `demodatagen` binary from GitHub releases.
    Update {
        /// Update to a specific release tag (e.g. `v0.2.0`) instead of latest.
        #[arg(long)]
        tag: Option<String>,
    },
}

/// Prints shell completions for the given shell to stdout.
pub fn print_completions(shell: Shell) {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    clap_complete::generate(shell, &mut cmd, name, &mut std::io::stdout());
}

/// Prints the catalogue of formats, schema field types, locales, and interface
/// languages, localized to `lang` and styled for the terminal.
pub fn print_format_list(lang: Language) {
    use crate::formats::{FormatGroup, FORMAT_GROUPS};

    println!("{}\n", ui::banner(lang));

    let group_title = |group: FormatGroup| -> &'static str {
        match group {
            FormatGroup::Structured => lang.catalog().group_structured,
            FormatGroup::Text => lang.catalog().group_text,
            FormatGroup::Images => lang.catalog().group_images,
            FormatGroup::AudioVideo => lang.catalog().group_av,
            FormatGroup::Documents => lang.catalog().group_docs,
            FormatGroup::Binary => lang.catalog().group_binary,
        }
    };

    println!("{}", style(tr!(lang, list_title)).bold().underlined());
    for (group, formats) in FORMAT_GROUPS {
        println!("  {}", style(group_title(*group)).cyan().bold());
        println!("    {}", formats.join(", "));
    }

    println!("\n{}", style(tr!(lang, list_schema_title)).bold());
    for (group, types) in SCHEMA_TYPE_GROUPS {
        println!("  {}", style(group).cyan());
        println!("    {}", types.join(", "));
    }

    println!("\n{}", style(tr!(lang, list_locales_title)).bold());
    for loc in crate::data::Locale::variants() {
        println!("  {:<7} {}", style(loc.as_str()).green(), loc.label());
    }

    println!("\n{}", style(tr!(lang, list_langs_title)).bold());
    for l in Language::variants() {
        println!("  {:<7} {}", style(l.as_str()).green(), l.label());
    }

    println!("\n{}", style(tr!(lang, list_hint)).dim());
}

/// Schema field types grouped by theme, for the `list` command. The category
/// labels are intentionally technical (kept in English across languages).
const SCHEMA_TYPE_GROUPS: &[(&str, &[&str])] = &[
    (
        "Numeric",
        &[
            "int(min..max)",
            "float(min..max)",
            "price(min..max)",
            "age",
            "year",
            "latitude",
            "longitude",
            "percent",
            "rating",
            "port",
            "timestamp",
        ],
    ),
    ("Boolean", &["bool"]),
    (
        "People",
        &[
            "name",
            "first_name",
            "last_name",
            "username",
            "gender",
            "password",
            "ssn",
        ],
    ),
    (
        "Contact",
        &[
            "email",
            "phone",
            "address",
            "street",
            "city",
            "state",
            "zipcode",
            "country",
            "country_code",
        ],
    ),
    (
        "Business",
        &[
            "company",
            "job",
            "department",
            "product",
            "sku",
            "currency",
            "currency_symbol",
            "iban",
            "credit_card",
            "isbn",
        ],
    ),
    (
        "Internet",
        &[
            "url",
            "domain",
            "slug",
            "ipv4",
            "ipv6",
            "mac",
            "uuid",
            "user_agent",
            "mime_type",
            "filename",
            "semver",
        ],
    ),
    (
        "Misc",
        &[
            "color",
            "hex_color",
            "language",
            "timezone",
            "emoji",
            "hashtag",
            "base64",
            "hex",
        ],
    ),
    (
        "Temporal",
        &["date", "time", "datetime", "weekday", "month"],
    ),
    ("Text", &["word", "words(n)", "sentence", "paragraph"]),
    (
        "Modifiers",
        &[
            "enum(a,b,c)",
            "const(value)",
            "sequence(start)",
            "array(type,n)",
            "type? / type?p (nullable)",
        ],
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse_json() {
        let cli = Cli::parse_from([
            "demodatagen",
            "json",
            "--rows",
            "100",
            "--schema",
            "name:string",
        ]);
        match cli.command {
            FormatCommand::Json { data, .. } => {
                assert_eq!(data.rows, 100);
                assert_eq!(data.schema, "name:string");
            }
            _ => panic!("Expected Json command"),
        }
    }

    #[test]
    fn test_cli_parse_png() {
        let cli = Cli::parse_from(["demodatagen", "png", "--width", "1920", "--height", "1080"]);
        match cli.command {
            FormatCommand::Png { image } => {
                assert_eq!(image.width, 1920);
                assert_eq!(image.height, 1080);
            }
            _ => panic!("Expected Png command"),
        }
    }

    #[test]
    fn test_cli_global_options() {
        let cli = Cli::parse_from([
            "demodatagen",
            "--output-dir",
            "/tmp/test",
            "--count",
            "10",
            "--seed",
            "42",
            "--locale",
            "de_de",
            "--lang",
            "de",
            "--color",
            "never",
            "--overwrite",
            "txt",
        ]);
        assert_eq!(cli.output_dir, PathBuf::from("/tmp/test"));
        assert_eq!(cli.count, 10);
        assert_eq!(cli.seed, Some(42));
        assert_eq!(cli.locale, "de_de");
        assert_eq!(cli.lang.as_deref(), Some("de"));
        assert_eq!(cli.color, ColorWhen::Never);
        assert!(cli.overwrite);
    }

    #[test]
    fn test_cli_defaults() {
        let cli = Cli::parse_from(["demodatagen", "txt"]);
        assert_eq!(cli.output_dir, PathBuf::from("./output"));
        assert_eq!(cli.count, 1);
        assert_eq!(cli.seed, None);
        assert_eq!(cli.locale, "en_us");
        assert_eq!(cli.lang, None);
        assert_eq!(cli.color, ColorWhen::Auto);
        assert!(!cli.overwrite);
        assert!(!cli.quiet);
        assert!(!cli.verbose);
        assert!(!cli.stdout);
    }

    #[test]
    fn test_cli_parse_update() {
        let cli = Cli::parse_from(["demodatagen", "update"]);
        assert!(matches!(cli.command, FormatCommand::Update { tag: None }));
    }

    #[test]
    fn test_cli_parse_update_tag() {
        let cli = Cli::parse_from(["demodatagen", "update", "--tag", "v0.2.0"]);
        match cli.command {
            FormatCommand::Update { tag } => assert_eq!(tag.as_deref(), Some("v0.2.0")),
            _ => panic!("Expected Update command"),
        }
    }

    #[test]
    fn test_cli_parse_csv_delimiter() {
        let cli = Cli::parse_from(["demodatagen", "csv", "--delimiter", ";"]);
        match cli.command {
            FormatCommand::Csv { delimiter, .. } => assert_eq!(delimiter, ";"),
            _ => panic!("Expected Csv command"),
        }
    }

    #[test]
    fn test_cli_parse_xml_tags() {
        let cli = Cli::parse_from(["demodatagen", "xml", "--root", "users", "--row-tag", "user"]);
        match cli.command {
            FormatCommand::Xml { root, row_tag, .. } => {
                assert_eq!(root, "users");
                assert_eq!(row_tag, "user");
            }
            _ => panic!("Expected Xml command"),
        }
    }

    #[test]
    fn test_cli_parse_completions() {
        let cli = Cli::parse_from(["demodatagen", "completions", "bash"]);
        assert!(matches!(cli.command, FormatCommand::Completions { .. }));
    }

    #[test]
    fn test_cli_parse_list() {
        let cli = Cli::parse_from(["demodatagen", "list"]);
        assert!(matches!(cli.command, FormatCommand::List));
    }

    #[test]
    fn test_verify_cli() {
        // Ensures the clap command tree is internally consistent.
        Cli::command().debug_assert();
    }
}
