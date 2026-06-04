/// Batch processing orchestrator for generating multiple files in parallel.
///
/// Uses `rayon` for parallel execution and `indicatif` for progress reporting.
use crate::core::generator::{create_rng, resolve_filename, Generator, GeneratorConfig};
use crate::error::{AppError, AppResult, GenerationError};
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, error, info};
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
    /// Whether to show progress bar.
    pub quiet: bool,
    /// Locale for region-specific fake data.
    pub locale: crate::data::Locale,
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
    // Create output directory if it doesn't exist
    fs::create_dir_all(&config.output_dir)?;

    info!(
        "Generating {} {} file(s) in {:?}",
        config.count,
        generator.format_name(),
        config.output_dir
    );

    // Set up progress bar
    let progress = if config.quiet {
        ProgressBar::hidden()
    } else {
        let pb = ProgressBar::new(config.count as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
                .unwrap_or_else(|_| ProgressStyle::default_bar())
                .progress_chars("=>-"),
        );
        pb.set_message(format!("Generating {} files...", generator.format_name()));
        pb
    };

    // Generate files in parallel
    let results: Vec<AppResult<PathBuf>> = (0..config.count)
        .into_par_iter()
        .map(|i| {
            let filename = resolve_filename(&config.name_pattern, i, &config.extension);
            let file_path = validate_path(&config.output_dir, &filename)?;

            // Check if file exists
            if file_path.exists() && !config.overwrite {
                return Err(AppError::Generation(GenerationError::FileExists {
                    path: file_path,
                }));
            }

            let rng = create_rng(config.seed, i);

            let mut gen_config = GeneratorConfig {
                output_dir: config.output_dir.clone(),
                name_pattern: config.name_pattern.clone(),
                extension: config.extension.clone(),
                index: i,
                overwrite: config.overwrite,
                rng,
                locale: config.locale,
                format_options: config.format_options.clone(),
            };

            // Generate file content
            let content = generator
                .generate(&mut gen_config)
                .map_err(AppError::Generation)?;

            // Write to disk
            debug!("Writing {} bytes to {:?}", content.len(), file_path);
            fs::write(&file_path, &content)?;

            progress.inc(1);
            Ok(file_path)
        })
        .collect();

    progress.finish_with_message("Done!");

    // Collect results, reporting errors
    let mut paths = Vec::new();
    let mut errors = Vec::new();

    for result in results {
        match result {
            Ok(path) => paths.push(path),
            Err(e) => {
                error!("Generation error: {e}");
                errors.push(e);
            }
        }
    }

    if paths.is_empty() && !errors.is_empty() {
        return Err(AppError::Batch(format!(
            "All {} file(s) failed to generate. First error: {}",
            config.count, errors[0]
        )));
    }

    if !errors.is_empty() {
        info!(
            "Generated {}/{} files ({} errors)",
            paths.len(),
            config.count,
            errors.len()
        );
    } else {
        info!("Successfully generated {} file(s)", paths.len());
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
