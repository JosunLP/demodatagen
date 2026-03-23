/// CLI argument definitions using `clap`.
///
/// Defines the top-level CLI structure with global options and format-specific
/// subcommands, each carrying their own parameters.
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// A CLI tool for generating realistic demo files in various formats.
///
/// Supports structured data (JSON, XML, CSV), text (TXT, Markdown),
/// images (PNG, JPG, WebP, GIF), audio (MP3), video (MP4, WebM),
/// binary stubs (EXE, DLL), and archives (ZIP).
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
    /// Generate JSON files with structured fake data.
    Json {
        /// Number of data rows / records.
        #[arg(long, default_value_t = 10)]
        rows: usize,

        /// Schema definition, e.g. `"name:string,age:int,email:email"`.
        #[arg(long, default_value = "id:int,name:string,email:email,date:date")]
        schema: String,

        /// Pretty-print the JSON output.
        #[arg(long, default_value_t = false)]
        pretty: bool,
    },

    /// Generate XML files with structured fake data.
    Xml {
        /// Number of data rows / records.
        #[arg(long, default_value_t = 10)]
        rows: usize,

        /// Schema definition.
        #[arg(long, default_value = "id:int,name:string,email:email,date:date")]
        schema: String,

        /// Pretty-print the XML output.
        #[arg(long, default_value_t = false)]
        pretty: bool,
    },

    /// Generate CSV files with structured fake data.
    Csv {
        /// Number of data rows.
        #[arg(long, default_value_t = 10)]
        rows: usize,

        /// Schema definition.
        #[arg(long, default_value = "id:int,name:string,email:email,date:date")]
        schema: String,
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

    /// Generate plain text files.
    Txt {
        /// Number of paragraphs.
        #[arg(long, default_value_t = 5)]
        paragraphs: usize,

        /// Approximate total word count (0 = auto based on paragraphs).
        #[arg(long, default_value_t = 0)]
        words: usize,
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

    /// Update the installed `demodatagen` binary to the latest GitHub release.
    Update,
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
            "--overwrite",
            "txt",
        ]);
        assert_eq!(cli.output_dir, PathBuf::from("/tmp/test"));
        assert_eq!(cli.count, 10);
        assert_eq!(cli.seed, Some(42));
        assert!(cli.overwrite);
    }

    #[test]
    fn test_cli_defaults() {
        let cli = Cli::parse_from(["demodatagen", "txt"]);
        assert_eq!(cli.output_dir, PathBuf::from("./output"));
        assert_eq!(cli.count, 1);
        assert_eq!(cli.seed, None);
        assert!(!cli.overwrite);
        assert!(!cli.quiet);
        assert!(!cli.verbose);
    }

    #[test]
    fn test_cli_parse_update() {
        let cli = Cli::parse_from(["demodatagen", "update"]);
        assert!(matches!(cli.command, FormatCommand::Update));
    }
}
