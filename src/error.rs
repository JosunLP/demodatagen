/// Central error types for the demodatagen application.
///
/// All errors are defined using `thiserror` for ergonomic error handling
/// with automatic `Display` and `Error` trait implementations.
use std::path::PathBuf;
use thiserror::Error;

/// Top-level error type for the application.
#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum AppError {
    /// Error during file generation.
    #[error("Generation error: {0}")]
    Generation(#[from] GenerationError),

    /// Error during I/O operations.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Error during CLI argument parsing.
    #[error("CLI error: {0}")]
    Cli(String),

    /// Error during batch processing.
    #[error("Batch error: {0}")]
    Batch(String),

    /// Error during update check.
    #[error("Update error: {0}")]
    Update(String),
}

/// Error type for file generation operations.
#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum GenerationError {
    /// Invalid configuration provided for the generator.
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Error encoding the output format.
    #[error("Encoding error: {0}")]
    Encoding(String),

    /// The output path already exists and `--overwrite` was not specified.
    #[error("File already exists: {path:?}. Use --overwrite to replace.")]
    FileExists { path: PathBuf },

    /// Path traversal attempt detected.
    #[error("Path traversal detected: {path:?} escapes the output directory")]
    PathTraversal { path: PathBuf },

    /// Generic I/O error during generation.
    #[error("I/O error during generation: {0}")]
    Io(#[from] std::io::Error),

    /// Image encoding error.
    #[error("Image error: {0}")]
    Image(String),

    /// Audio encoding error.
    #[error("Audio error: {0}")]
    Audio(String),

    /// Archive error.
    #[error("Archive error: {0}")]
    Archive(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Type alias for a `Result` with `AppError`.
pub type AppResult<T> = Result<T, AppError>;

/// Type alias for a `Result` with `GenerationError`.
pub type GenResult<T> = Result<T, GenerationError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_error_display() {
        let err = GenerationError::InvalidConfig("bad value".to_string());
        assert_eq!(format!("{err}"), "Invalid configuration: bad value");
    }

    #[test]
    fn test_file_exists_error_display() {
        let err = GenerationError::FileExists {
            path: PathBuf::from("/tmp/test.txt"),
        };
        let msg = format!("{err}");
        assert!(msg.contains("File already exists"));
        assert!(msg.contains("test.txt"));
    }

    #[test]
    fn test_path_traversal_error_display() {
        let err = GenerationError::PathTraversal {
            path: PathBuf::from("../../etc/passwd"),
        };
        let msg = format!("{err}");
        assert!(msg.contains("Path traversal"));
    }

    #[test]
    fn test_app_error_from_generation_error() {
        let gen_err = GenerationError::InvalidConfig("test".to_string());
        let app_err: AppError = gen_err.into();
        let msg = format!("{app_err}");
        assert!(msg.contains("Generation error"));
    }

    #[test]
    fn test_app_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let app_err: AppError = io_err.into();
        let msg = format!("{app_err}");
        assert!(msg.contains("I/O error"));
    }
}
