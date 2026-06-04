/// The core `Generator` trait and shared configuration types.
///
/// Every format-specific generator implements the `Generator` trait,
/// ensuring a uniform interface for file generation across all formats.
use crate::data::Locale;
use crate::error::GenResult;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::path::PathBuf;

/// Shared configuration for all generators.
///
/// This struct carries the common parameters that every generator may need,
/// plus format-specific parameters via the `format_options` field.
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// Output directory for generated files.
    #[allow(dead_code)]
    pub output_dir: PathBuf,
    /// Base name pattern for files (e.g., `"user_{n}"`).
    #[allow(dead_code)]
    pub name_pattern: String,
    /// File extension (without dot).
    #[allow(dead_code)]
    pub extension: String,
    /// Current file index in batch (0-based).
    #[allow(dead_code)]
    pub index: usize,
    /// Whether to overwrite existing files.
    #[allow(dead_code)]
    pub overwrite: bool,
    /// Seeded RNG for deterministic generation.
    pub rng: ChaCha8Rng,
    /// Locale for region-specific fake data.
    pub locale: Locale,
    /// Format-specific options.
    pub format_options: FormatOptions,
}

/// Format-specific options for generators.
///
/// Each variant carries parameters relevant to a particular output format.
#[derive(Debug, Clone)]
pub enum FormatOptions {
    /// Options for object-stream structured formats (JSON, JSONL, YAML, TOML).
    StructuredData {
        /// Number of data rows to generate.
        rows: usize,
        /// Schema definition (e.g., `"name:string,age:int"`).
        schema: String,
        /// Whether to pretty-print the output.
        pretty: bool,
    },
    /// Options for XML, with configurable element names.
    Xml {
        /// Number of data rows to generate.
        rows: usize,
        /// Schema definition.
        schema: String,
        /// Whether to pretty-print (indent) the output.
        pretty: bool,
        /// Name of the root element.
        root: String,
        /// Name of each record element.
        row_tag: String,
    },
    /// Options for delimiter-separated formats (CSV, TSV).
    Delimited {
        /// Number of data rows to generate.
        rows: usize,
        /// Schema definition.
        schema: String,
        /// Field delimiter byte (e.g. `b','` or `b'\t'`).
        delimiter: u8,
    },
    /// Options for SQL `INSERT` scripts and spreadsheet (XLSX) output.
    Sql {
        /// Number of data rows to generate.
        rows: usize,
        /// Schema definition.
        schema: String,
        /// Table / sheet name.
        table: String,
    },
    /// Options for synthetic log files.
    Log {
        /// Number of log lines to generate.
        lines: usize,
        /// Log style: `apache`, `syslog`, or `json`.
        style: String,
    },
    /// Options for key/value config formats (INI, .env).
    KeyValue {
        /// Number of sections (INI only).
        sections: usize,
        /// Number of keys per section.
        keys: usize,
        /// Whether to render dotenv (`KEY=value`) instead of INI.
        env_style: bool,
    },
    /// Options for SVG vector images.
    Svg {
        /// Canvas width.
        width: u32,
        /// Canvas height.
        height: u32,
        /// Number of random shapes to draw.
        shapes: usize,
    },
    /// Options for plain text files.
    Text {
        /// Number of paragraphs to generate.
        paragraphs: usize,
        /// Approximate total word count (0 = auto).
        words: usize,
    },
    /// Options for Markdown files.
    Markdown {
        /// Number of paragraphs to generate.
        paragraphs: usize,
        /// Number of headings / sections.
        headings: usize,
    },
    /// Options for image formats (PNG, JPG, WebP, GIF).
    Image {
        /// Image width in pixels.
        width: u32,
        /// Image height in pixels.
        height: u32,
        /// Pattern type: noise, gradient, shapes, checkerboard.
        pattern: ImagePattern,
        /// Number of frames (GIF only).
        frames: u32,
    },
    /// Options for audio formats (MP3).
    Audio {
        /// Duration in seconds.
        duration: f32,
        /// Sample rate in Hz.
        sample_rate: u32,
        /// Tone type: sine, noise, sweep.
        tone: ToneType,
    },
    /// Options for video formats (MP4, WebM).
    Video {
        /// Duration in seconds.
        duration: f32,
        /// Video width in pixels.
        width: u32,
        /// Video height in pixels.
        height: u32,
        /// Frames per second.
        fps: u32,
    },
    /// Options for binary PE stubs (EXE, DLL).
    Binary {
        /// Target file size in bytes.
        size: usize,
    },
    /// Options for ZIP archives.
    Zip {
        /// Number of files to include in the archive.
        file_count: usize,
        /// Format of contained files.
        contained_format: String,
        /// Compression level (0-9).
        compression_level: u32,
    },
}

/// Pattern types for image generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImagePattern {
    /// Perlin-like noise pattern.
    Noise,
    /// Linear or radial gradient.
    Gradient,
    /// Random geometric shapes.
    Shapes,
    /// Checkerboard pattern.
    Checkerboard,
}

impl std::fmt::Display for ImagePattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImagePattern::Noise => write!(f, "noise"),
            ImagePattern::Gradient => write!(f, "gradient"),
            ImagePattern::Shapes => write!(f, "shapes"),
            ImagePattern::Checkerboard => write!(f, "checkerboard"),
        }
    }
}

impl std::str::FromStr for ImagePattern {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "noise" => Ok(ImagePattern::Noise),
            "gradient" => Ok(ImagePattern::Gradient),
            "shapes" => Ok(ImagePattern::Shapes),
            "checkerboard" => Ok(ImagePattern::Checkerboard),
            _ => Err(format!(
                "Unknown image pattern: '{s}'. Valid: noise, gradient, shapes, checkerboard"
            )),
        }
    }
}

/// Tone types for audio generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToneType {
    /// Pure sine wave.
    Sine,
    /// White noise.
    Noise,
    /// Frequency sweep.
    Sweep,
}

impl std::fmt::Display for ToneType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToneType::Sine => write!(f, "sine"),
            ToneType::Noise => write!(f, "noise"),
            ToneType::Sweep => write!(f, "sweep"),
        }
    }
}

impl std::str::FromStr for ToneType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sine" => Ok(ToneType::Sine),
            "noise" => Ok(ToneType::Noise),
            "sweep" => Ok(ToneType::Sweep),
            _ => Err(format!(
                "Unknown tone type: '{s}'. Valid: sine, noise, sweep"
            )),
        }
    }
}

/// The central trait for all file generators.
///
/// Each format implements this trait to produce the raw bytes of a valid file.
pub trait Generator: Send + Sync {
    /// Returns the human-readable name of the format this generator produces.
    fn format_name(&self) -> &str;

    /// Returns the file extension (without leading dot).
    fn file_extension(&self) -> &str;

    /// Generates the file content as a byte vector.
    ///
    /// # Arguments
    /// * `config` - The generation configuration with format-specific options.
    ///
    /// # Returns
    /// A `GenResult<Vec<u8>>` containing the raw bytes of the generated file.
    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>>;
}

/// Resolves the output file name from a pattern and index.
///
/// Replaces `{n}` in the pattern with the file index, then appends the extension.
pub fn resolve_filename(pattern: &str, index: usize, extension: &str) -> String {
    let name = pattern.replace("{n}", &index.to_string());
    format!("{name}.{extension}")
}

/// Creates a seeded RNG. If seed is `None`, generates a random seed.
pub fn create_rng(seed: Option<u64>, index: usize) -> ChaCha8Rng {
    match seed {
        Some(s) => ChaCha8Rng::seed_from_u64(s.wrapping_add(index as u64)),
        None => ChaCha8Rng::from_entropy(),
    }
}

/// Test-only constructors for [`GeneratorConfig`], shared across format tests.
#[cfg(test)]
pub(crate) mod test_support {
    use super::*;

    /// Builds a deterministic config (seed 42, `en_us`, index 0) wrapping the
    /// given format options.
    pub fn config(format_options: FormatOptions) -> GeneratorConfig {
        GeneratorConfig {
            output_dir: PathBuf::from("/tmp"),
            name_pattern: "test_{n}".to_string(),
            extension: "out".to_string(),
            index: 0,
            overwrite: false,
            rng: ChaCha8Rng::seed_from_u64(42),
            locale: Locale::EnUs,
            format_options,
        }
    }

    pub fn structured_config(rows: usize, schema: &str, pretty: bool) -> GeneratorConfig {
        config(FormatOptions::StructuredData {
            rows,
            schema: schema.to_string(),
            pretty,
        })
    }

    pub fn xml_config(
        rows: usize,
        schema: &str,
        pretty: bool,
        root: &str,
        row_tag: &str,
    ) -> GeneratorConfig {
        config(FormatOptions::Xml {
            rows,
            schema: schema.to_string(),
            pretty,
            root: root.to_string(),
            row_tag: row_tag.to_string(),
        })
    }

    pub fn delimited_config(rows: usize, schema: &str, delimiter: u8) -> GeneratorConfig {
        config(FormatOptions::Delimited {
            rows,
            schema: schema.to_string(),
            delimiter,
        })
    }

    pub fn sql_config(rows: usize, schema: &str, table: &str) -> GeneratorConfig {
        config(FormatOptions::Sql {
            rows,
            schema: schema.to_string(),
            table: table.to_string(),
        })
    }

    pub fn text_config(paragraphs: usize, words: usize) -> GeneratorConfig {
        config(FormatOptions::Text { paragraphs, words })
    }

    pub fn markdown_config(paragraphs: usize, headings: usize) -> GeneratorConfig {
        config(FormatOptions::Markdown {
            paragraphs,
            headings,
        })
    }

    pub fn log_config(lines: usize, style: &str) -> GeneratorConfig {
        config(FormatOptions::Log {
            lines,
            style: style.to_string(),
        })
    }

    pub fn keyvalue_config(sections: usize, keys: usize, env_style: bool) -> GeneratorConfig {
        config(FormatOptions::KeyValue {
            sections,
            keys,
            env_style,
        })
    }

    pub fn svg_config(width: u32, height: u32, shapes: usize) -> GeneratorConfig {
        config(FormatOptions::Svg {
            width,
            height,
            shapes,
        })
    }

    pub fn image_config(
        width: u32,
        height: u32,
        pattern: ImagePattern,
        frames: u32,
    ) -> GeneratorConfig {
        config(FormatOptions::Image {
            width,
            height,
            pattern,
            frames,
        })
    }

    pub fn audio_config(duration: f32, sample_rate: u32, tone: ToneType) -> GeneratorConfig {
        config(FormatOptions::Audio {
            duration,
            sample_rate,
            tone,
        })
    }

    pub fn video_config(duration: f32, width: u32, height: u32, fps: u32) -> GeneratorConfig {
        config(FormatOptions::Video {
            duration,
            width,
            height,
            fps,
        })
    }

    pub fn binary_config(size: usize) -> GeneratorConfig {
        config(FormatOptions::Binary { size })
    }

    pub fn zip_config(
        file_count: usize,
        contained_format: &str,
        compression_level: u32,
    ) -> GeneratorConfig {
        config(FormatOptions::Zip {
            file_count,
            contained_format: contained_format.to_string(),
            compression_level,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_filename_with_index() {
        let name = resolve_filename("user_{n}", 5, "json");
        assert_eq!(name, "user_5.json");
    }

    #[test]
    fn test_resolve_filename_no_placeholder() {
        let name = resolve_filename("data", 3, "csv");
        assert_eq!(name, "data.csv");
    }

    #[test]
    fn test_create_rng_deterministic() {
        let rng1 = create_rng(Some(42), 0);
        let rng2 = create_rng(Some(42), 0);
        // Both should produce same sequence
        let mut r1 = rng1;
        let mut r2 = rng2;
        use rand::Rng;
        assert_eq!(r1.gen::<u64>(), r2.gen::<u64>());
    }

    #[test]
    fn test_image_pattern_from_str() {
        assert_eq!(
            "noise".parse::<ImagePattern>().unwrap(),
            ImagePattern::Noise
        );
        assert_eq!(
            "Gradient".parse::<ImagePattern>().unwrap(),
            ImagePattern::Gradient
        );
        assert!("invalid".parse::<ImagePattern>().is_err());
    }

    #[test]
    fn test_tone_type_from_str() {
        assert_eq!("sine".parse::<ToneType>().unwrap(), ToneType::Sine);
        assert_eq!("NOISE".parse::<ToneType>().unwrap(), ToneType::Noise);
        assert!("invalid".parse::<ToneType>().is_err());
    }

    #[test]
    fn test_image_pattern_display() {
        assert_eq!(ImagePattern::Noise.to_string(), "noise");
        assert_eq!(ImagePattern::Checkerboard.to_string(), "checkerboard");
    }

    #[test]
    fn test_tone_type_display() {
        assert_eq!(ToneType::Sine.to_string(), "sine");
        assert_eq!(ToneType::Sweep.to_string(), "sweep");
    }
}
