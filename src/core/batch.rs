/// Batch processing orchestrator for generating multiple files in parallel.
///
/// Uses `rayon` for parallel execution and `indicatif` for progress reporting.
use crate::core::generator::{create_rng, resolve_filename, Generator, GeneratorConfig};
use crate::error::{AppError, AppResult, GenerationError};
use log::debug;
use rayon::prelude::*;
use std::fs;
use std::path::{Component, Path, PathBuf};

/// Configuration for a batch generation run.
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Output directory for all generated files.
    pub output_dir: PathBuf,
    /// Number of files to generate.
    pub count: usize,
    /// Name pattern for files (may contain `{n}`).
    pub name_pattern: String,
    /// File extension.
    pub extension: String,
    /// Whether to overwrite existing files.
    pub overwrite: bool,
    /// Optional RNG seed for reproducibility.
    pub seed: Option<u64>,
    /// Whether to suppress progress and summary output.
    pub quiet: bool,
    /// Locale for region-specific fake data (generated content).
    pub locale: crate::data::Locale,
    /// Interface language for progress and summary messages.
    pub lang: crate::i18n::Language,
    /// Format-specific options (cloned per file).
    pub format_options: crate::core::generator::FormatOptions,
}

/// Validates that the output path does not escape the output directory.
///
/// Prevents path traversal attacks via malicious name patterns.
fn validate_path(output_dir: &Path, filename: &str) -> AppResult<PathBuf> {
    let relative_path = Path::new(filename);
    if relative_path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(AppError::Generation(GenerationError::PathTraversal {
            path: output_dir.join(relative_path),
        }));
    }

    let joined = output_dir.join(relative_path);

    // Additional protection against escaping `output_dir` via symlinks inside it.
    // This mirrors the previous behavior that canonicalized the nearest existing
    // parent and ensured it stayed within the (canonicalized) output directory.
    if let Ok(output_canon) = fs::canonicalize(output_dir) {
        // Find the nearest existing ancestor of the target path (if any).
        let mut current = joined.as_path();
        let mut existing_parent: Option<PathBuf> = None;
        while let Some(parent) = current.parent() {
            if parent.exists() {
                existing_parent = Some(parent.to_path_buf());
                break;
            }
            current = parent;
        }

        if let Some(parent) = existing_parent {
            if let Ok(parent_canon) = fs::canonicalize(&parent) {
                if !parent_canon.starts_with(&output_canon) {
                    return Err(AppError::Generation(GenerationError::PathTraversal {
                        path: joined,
                    }));
                }
            }
        }
    }

    Ok(joined)
}

/// Runs a batch generation using the provided generator and configuration.
///
/// Files are generated in parallel using `rayon`. A progress bar is displayed
/// unless `quiet` mode is enabled.
///
/// # Arguments
/// * `generator` - The format-specific generator to use.
/// * `config` - Batch configuration specifying output dir, count, etc.
///
/// # Returns
/// A vector of paths to successfully generated files, or an error.
pub fn run_batch(generator: &dyn Generator, config: &BatchConfig) -> AppResult<Vec<PathBuf>> {
    use crate::i18n::tr;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    use std::time::Instant;

    // Create output directory if it doesn't exist.
    fs::create_dir_all(&config.output_dir)?;

    let start = Instant::now();
    let format_name = generator.format_name();
    debug!(
        "Generating {} {} file(s) in {:?}",
        config.count, format_name, config.output_dir
    );

    if !config.quiet {
        crate::ui::run_header(config.lang, config.count, format_name, &config.output_dir);
    }

    let progress = crate::ui::progress(config.count, format_name, config.lang, config.quiet);
    let progress_message = tr!(config.lang, progress_message, "format" => format_name);
    let bytes_written = Arc::new(AtomicU64::new(0));

    // Generate files in parallel.
    let results: Vec<AppResult<PathBuf>> = (0..config.count)
        .into_par_iter()
        .map(|i| {
            let filename = resolve_filename(&config.name_pattern, i, &config.extension);
            let file_path = validate_path(&config.output_dir, &filename)?;

            if file_path.exists() && !config.overwrite {
                return Err(AppError::Generation(GenerationError::FileExists {
                    path: file_path,
                }));
            }

            let mut gen_config = GeneratorConfig {
                output_dir: config.output_dir.clone(),
                name_pattern: config.name_pattern.clone(),
                extension: config.extension.clone(),
                index: i,
                overwrite: config.overwrite,
                rng: create_rng(config.seed, i),
                locale: config.locale,
                format_options: config.format_options.clone(),
            };

            let content = generator
                .generate(&mut gen_config)
                .map_err(AppError::Generation)?;

            debug!("Writing {} bytes to {:?}", content.len(), file_path);
            fs::write(&file_path, &content)?;
            bytes_written.fetch_add(content.len() as u64, Ordering::Relaxed);

            crate::ui::tick_progress(&progress, &progress_message, &filename);
            Ok(file_path)
        })
        .collect();

    crate::ui::finish_progress(&progress, config.lang);

    // Partition into successes and errors.
    let mut paths = Vec::new();
    let mut errors = Vec::new();
    for result in results {
        match result {
            Ok(path) => paths.push(path),
            Err(e) => {
                debug!("Generation error: {e}");
                errors.push(e);
            }
        }
    }

    // Total failure: surface a localized message via the error path.
    if paths.is_empty() && !errors.is_empty() {
        return Err(AppError::Batch(tr!(
            config.lang,
            summary_failed,
            "total" => config.count,
            "error" => &errors[0],
        )));
    }

    let elapsed = start.elapsed();
    let total_bytes = bytes_written.load(Ordering::Relaxed);

    if !config.quiet {
        if !errors.is_empty() {
            crate::ui::partial(config.lang, paths.len(), config.count, errors.len());
        }
        crate::ui::summary(
            config.lang,
            paths.len(),
            total_bytes,
            elapsed,
            &config.output_dir,
        );
    }

    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_path_normal() {
        let dir = PathBuf::from("/tmp");
        let result = validate_path(&dir, "test.txt");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("/tmp/test.txt"));
    }

    #[test]
    fn test_validate_path_traversal() {
        let dir = std::env::temp_dir().join("demodatagen_validate_path_traversal");
        fs::create_dir_all(&dir).expect("failed to create test output directory");
        let result = validate_path(&dir, "../../etc/passwd");
        assert!(matches!(
            result,
            Err(AppError::Generation(GenerationError::PathTraversal { .. }))
        ));
    }

    #[test]
    fn test_validate_path_traversal_with_nonexistent_parent() {
        let dir = std::env::temp_dir().join("demodatagen_validate_path");
        fs::create_dir_all(&dir).expect("failed to create test output directory");
        let result = validate_path(&dir, "../missing-parent/test.txt");
        assert!(matches!(
            result,
            Err(AppError::Generation(GenerationError::PathTraversal { .. }))
        ));
    }
}
