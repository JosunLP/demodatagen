//! CLI argument definitions using `clap`.
//!
//! Defines the top-level CLI structure with global options and format-specific
//! subcommands, each carrying their own parameters, plus the `list` and
//! `completions` helper subcommands.
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use std::path::PathBuf;

/// Default schema used by structured-data subcommands.
const DEFAULT_SCHEMA: &str = "id:sequence,name:name,email:email,created:datetime";

/// A fast, offline CLI for generating realistic demo files in many formats.
///
/// Supports structured data (JSON, JSONL, YAML, TOML, XML, CSV, TSV, SQL),
/// text (TXT, Markdown, HTML, LOG, INI, ENV), images (PNG, JPG, WebP, BMP,
/// TIFF, ICO, GIF, SVG), audio (MP3, WAV), video (MP4, WebM), documents
/// (PDF, XLSX), binary stubs (EXE, DLL), and archives (ZIP, TAR, GZIP).
#[derive(Parser, Debug)]
#[command(name = "demodatagen")]
#[command(version, about, long_about = None)]
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

    /// Locale for region-specific fake data (e.g. `en_us`, `de_de`).
    #[arg(short = 'l', long, default_value = "en_us", global = true)]
    pub locale: String,

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
    #[arg(long, default_value_t = false, global = true)]
    pub quiet: bool,

    /// Enable verbose (debug) logging.
    #[arg(long, default_value_t = false, global = true)]
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
        /// Number of data rows / records.
        #[arg(long, default_value_t = 10)]
        rows: usize,
        /// Schema definition, e.g. `"id:sequence,name:name,age:int(18..65)"`.
        #[arg(long, default_value = DEFAULT_SCHEMA)]
        schema: String,
        /// Pretty-print the JSON output.
        #[arg(long, default_value_t = false)]
        pretty: bool,
    },

    /// Generate newline-delimited JSON (JSONL / NDJSON).
    Jsonl {
        /// Number of records (one JSON object per line).
        #[arg(long, default_value_t = 10)]
        rows: usize,
        /// Schema definition.
        #[arg(long, default_value = DEFAULT_SCHEMA)]
        schema: String,
    },

    /// Generate YAML files with structured fake data.
    Yaml {
        /// Number of data rows / records.
        #[arg(long, default_value_t = 10)]
        rows: usize,
        /// Schema definition.
        #[arg(long, default_value = DEFAULT_SCHEMA)]
        schema: String,
    },

    /// Generate TOML files with an array of tables.
    Toml {
        /// Number of data rows / records.
        #[arg(long, default_value_t = 10)]
        rows: usize,
        /// Schema definition.
        #[arg(long, default_value = DEFAULT_SCHEMA)]
        schema: String,
    },

    /// Generate XML files with structured fake data.
    Xml {
        /// Number of data rows / records.
        #[arg(long, default_value_t = 10)]
        rows: usize,
        /// Schema definition.
        #[arg(long, default_value = DEFAULT_SCHEMA)]
        schema: String,
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
        /// Number of data rows.
        #[arg(long, default_value_t = 10)]
        rows: usize,
        /// Schema definition.
        #[arg(long, default_value = DEFAULT_SCHEMA)]
        schema: String,
        /// Field delimiter (single character, or `\t`/`tab` for tab).
        #[arg(long, default_value = ",")]
        delimiter: String,
    },

    /// Generate TSV (tab-separated) files.
    Tsv {
        /// Number of data rows.
        #[arg(long, default_value_t = 10)]
        rows: usize,
        /// Schema definition.
        #[arg(long, default_value = DEFAULT_SCHEMA)]
        schema: String,
    },

    /// Generate a SQL script with `CREATE TABLE` + `INSERT` statements.
    Sql {
        /// Number of rows to insert.
        #[arg(long, default_value_t = 10)]
        rows: usize,
        /// Schema definition.
        #[arg(long, default_value = DEFAULT_SCHEMA)]
        schema: String,
        /// Target table name.
        #[arg(long, default_value = "demo_data")]
        table: String,
    },

    /// Generate Markdown files with headings and paragraphs.
    Markdown {
        /// Number of paragraphs.
        #[arg(long, default_value_t = 5)]
        paragraphs: usize,
        /// Number of section headings.
        #[arg(long, default_value_t = 3)]
        headings: usize,
    },

    /// Generate HTML documents with headings and paragraphs.
    Html {
        /// Number of paragraphs.
        #[arg(long, default_value_t = 5)]
        paragraphs: usize,
        /// Number of section headings.
        #[arg(long, default_value_t = 3)]
        headings: usize,
    },

    /// Generate plain text files.
    Txt {
        /// Number of paragraphs.
        #[arg(long, default_value_t = 5)]
        paragraphs: usize,
        /// Approximate total word count (0 = auto based on paragraphs).
        #[arg(long, default_value_t = 0)]
        words: usize,
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
        /// Image width in pixels.
        #[arg(long, default_value_t = 800)]
        width: u32,
        /// Image height in pixels.
        #[arg(long, default_value_t = 600)]
        height: u32,
        /// Pattern type: noise, gradient, shapes, checkerboard.
        #[arg(long, default_value = "gradient")]
        pattern: String,
    },

    /// Generate JPEG images.
    Jpg {
        /// Image width in pixels.
        #[arg(long, default_value_t = 800)]
        width: u32,
        /// Image height in pixels.
        #[arg(long, default_value_t = 600)]
        height: u32,
        /// Pattern type: noise, gradient, shapes, checkerboard.
        #[arg(long, default_value = "gradient")]
        pattern: String,
    },

    /// Generate WebP images.
    Webp {
        /// Image width in pixels.
        #[arg(long, default_value_t = 800)]
        width: u32,
        /// Image height in pixels.
        #[arg(long, default_value_t = 600)]
        height: u32,
        /// Pattern type: noise, gradient, shapes, checkerboard.
        #[arg(long, default_value = "gradient")]
        pattern: String,
    },

    /// Generate BMP images.
    Bmp {
        /// Image width in pixels.
        #[arg(long, default_value_t = 800)]
        width: u32,
        /// Image height in pixels.
        #[arg(long, default_value_t = 600)]
        height: u32,
        /// Pattern type: noise, gradient, shapes, checkerboard.
        #[arg(long, default_value = "gradient")]
        pattern: String,
    },

    /// Generate TIFF images.
    Tiff {
        /// Image width in pixels.
        #[arg(long, default_value_t = 800)]
        width: u32,
        /// Image height in pixels.
        #[arg(long, default_value_t = 600)]
        height: u32,
        /// Pattern type: noise, gradient, shapes, checkerboard.
        #[arg(long, default_value = "gradient")]
        pattern: String,
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
        /// Duration in seconds.
        #[arg(long, default_value_t = 5.0)]
        duration: f32,
        /// Sample rate in Hz.
        #[arg(long, default_value_t = 44100)]
        sample_rate: u32,
        /// Tone type: sine, noise, sweep.
        #[arg(long, default_value = "sine")]
        tone: String,
    },

    /// Generate WAV audio files (uncompressed PCM).
    Wav {
        /// Duration in seconds.
        #[arg(long, default_value_t = 5.0)]
        duration: f32,
        /// Sample rate in Hz.
        #[arg(long, default_value_t = 44100)]
        sample_rate: u32,
        /// Tone type: sine, noise, sweep.
        #[arg(long, default_value = "sine")]
        tone: String,
    },

    /// Generate MP4 video files.
    Mp4 {
        /// Duration in seconds.
        #[arg(long, default_value_t = 5.0)]
        duration: f32,
        /// Video width in pixels.
        #[arg(long, default_value_t = 640)]
        width: u32,
        /// Video height in pixels.
        #[arg(long, default_value_t = 480)]
        height: u32,
        /// Frames per second.
        #[arg(long, default_value_t = 24)]
        fps: u32,
    },

    /// Generate WebM video files.
    Webm {
        /// Duration in seconds.
        #[arg(long, default_value_t = 5.0)]
        duration: f32,
        /// Video width in pixels.
        #[arg(long, default_value_t = 640)]
        width: u32,
        /// Video height in pixels.
        #[arg(long, default_value_t = 480)]
        height: u32,
        /// Frames per second.
        #[arg(long, default_value_t = 24)]
        fps: u32,
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
        /// Number of paragraphs.
        #[arg(long, default_value_t = 5)]
        paragraphs: usize,
        /// Number of section headings.
        #[arg(long, default_value_t = 3)]
        headings: usize,
    },

    /// Generate XLSX (Excel) spreadsheets.
    Xlsx {
        /// Number of data rows.
        #[arg(long, default_value_t = 10)]
        rows: usize,
        /// Schema definition.
        #[arg(long, default_value = DEFAULT_SCHEMA)]
        schema: String,
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

    /// List all supported formats and schema types.
    List,

    /// Print shell completion script for the given shell.
    Completions {
        /// Target shell: bash, zsh, fish, powershell, elvish.
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Update the installed `demodatagen` binary to the latest GitHub release.
    Update,
}

/// Prints shell completions for the given shell to stdout.
pub fn print_completions(shell: Shell) {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    clap_complete::generate(shell, &mut cmd, name, &mut std::io::stdout());
}

/// Prints the catalogue of supported formats and schema field types.
pub fn print_format_list() {
    println!("demodatagen — supported formats\n");
    let groups: &[(&str, &[&str])] = &[
        (
            "Structured data",
            &["json", "jsonl", "yaml", "toml", "xml", "csv", "tsv", "sql"],
        ),
        (
            "Text & config",
            &["txt", "markdown", "html", "log", "ini", "env"],
        ),
        (
            "Images",
            &["png", "jpg", "webp", "bmp", "tiff", "ico", "gif", "svg"],
        ),
        ("Audio & video", &["mp3", "wav", "mp4", "webm"]),
        ("Documents", &["pdf", "xlsx"]),
        ("Binary & archives", &["exe", "dll", "zip", "tar", "gzip"]),
    ];
    for (group, formats) in groups {
        println!("  {group}:");
        println!("    {}", formats.join(", "));
    }

    println!("\nSchema field types (use as `field:type` in --schema):");
    let types: &[&str] = &[
        "int(min..max)",
        "float(min..max)",
        "bool",
        "price(min..max)",
        "age",
        "year",
        "sequence(start)",
        "enum(a,b,c)",
        "const(value)",
        "array(type,n)",
        "type? (nullable)",
        "name",
        "first_name",
        "last_name",
        "username",
        "email",
        "password",
        "phone",
        "gender",
        "address",
        "street",
        "city",
        "state",
        "zipcode",
        "country",
        "country_code",
        "company",
        "job",
        "department",
        "product",
        "sku",
        "currency",
        "iban",
        "credit_card",
        "isbn",
        "url",
        "domain",
        "slug",
        "ipv4",
        "ipv6",
        "mac",
        "uuid",
        "user_agent",
        "color",
        "hex_color",
        "language",
        "timezone",
        "emoji",
        "date",
        "time",
        "datetime",
        "timestamp",
        "weekday",
        "month",
        "word",
        "words(n)",
        "sentence",
        "paragraph",
    ];
    for chunk in types.chunks(4) {
        println!("    {}", chunk.join(", "));
    }

    println!("\nLocales: {}", crate::data::Locale::all().join(", "));
}

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
            FormatCommand::Json { rows, schema, .. } => {
                assert_eq!(rows, 100);
                assert_eq!(schema, "name:string");
            }
            _ => panic!("Expected Json command"),
        }
    }

    #[test]
    fn test_cli_parse_png() {
        let cli = Cli::parse_from(["demodatagen", "png", "--width", "1920", "--height", "1080"]);
        match cli.command {
            FormatCommand::Png { width, height, .. } => {
                assert_eq!(width, 1920);
                assert_eq!(height, 1080);
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
            "--overwrite",
            "txt",
        ]);
        assert_eq!(cli.output_dir, PathBuf::from("/tmp/test"));
        assert_eq!(cli.count, 10);
        assert_eq!(cli.seed, Some(42));
        assert_eq!(cli.locale, "de_de");
        assert!(cli.overwrite);
    }

    #[test]
    fn test_cli_defaults() {
        let cli = Cli::parse_from(["demodatagen", "txt"]);
        assert_eq!(cli.output_dir, PathBuf::from("./output"));
        assert_eq!(cli.count, 1);
        assert_eq!(cli.seed, None);
        assert_eq!(cli.locale, "en_us");
        assert!(!cli.overwrite);
        assert!(!cli.quiet);
        assert!(!cli.verbose);
        assert!(!cli.stdout);
    }

    #[test]
    fn test_cli_parse_update() {
        let cli = Cli::parse_from(["demodatagen", "update"]);
        assert!(matches!(cli.command, FormatCommand::Update));
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
