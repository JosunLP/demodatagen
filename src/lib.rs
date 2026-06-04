//! `demodatagen` — a fast, offline engine for generating realistic demo files
//! in many formats.
//!
//! This crate can be used both as a command-line tool (via the `demodatagen`
//! binary) and as a **library**. The library exposes the [`Generator`] trait,
//! the format registry ([`formats::get_generator`]), the batch orchestrator
//! ([`core::batch::run_batch`]), and the schema/faker engine ([`data`]).
//!
//! # Example
//!
//! ```no_run
//! use demodatagen::core::batch::{run_batch, BatchConfig};
//! use demodatagen::core::generator::FormatOptions;
//! use demodatagen::formats::get_generator;
//! use std::path::PathBuf;
//!
//! let generator = get_generator("json").expect("json generator exists");
//! let config = BatchConfig {
//!     output_dir: PathBuf::from("./output"),
//!     count: 3,
//!     name_pattern: "demo_{n}".into(),
//!     extension: generator.file_extension().to_string(),
//!     overwrite: true,
//!     seed: Some(42),
//!     quiet: true,
//!     locale: demodatagen::data::Locale::EnUs,
//!     format_options: FormatOptions::StructuredData {
//!         rows: 10,
//!         schema: "id:sequence,name:name,email:email".into(),
//!         pretty: true,
//!     },
//! };
//! let paths = run_batch(generator.as_ref(), &config).expect("generation succeeds");
//! assert_eq!(paths.len(), 3);
//! ```

pub mod app;
pub mod cli;
pub mod core;
pub mod data;
pub mod error;
pub mod formats;
#[cfg(feature = "update")]
pub mod update;

pub use crate::core::generator::{
    FormatOptions, Generator, GeneratorConfig, ImagePattern, ToneType,
};
pub use crate::data::Locale;
pub use crate::error::{AppError, AppResult, GenResult, GenerationError};
