# Copilot Instructions for demodatagen

## Project Overview

`demodatagen` is a Rust CLI tool that generates realistic demo files in 16+ formats (JSON, XML, CSV, PNG, MP3, MP4, ZIP, etc.). It uses **no external services** — all data is generated procedurally with deterministic seeding via `ChaCha8Rng`.

## Architecture

The codebase follows a **trait-based plugin architecture** with four layers:

1. **`src/cli/`** — CLI argument parsing via `clap` derive macros. `FormatCommand` enum maps each format to its subcommand and parameters.
2. **`src/core/`** — Orchestration layer:
   - `generator.rs` — The `Generator` trait (implement `format_name()`, `file_extension()`, `generate() -> GenResult<Vec<u8>>`), `FormatOptions` enum, `GeneratorConfig`, and RNG helpers.
   - `batch.rs` — Parallel batch execution via `rayon` with `indicatif` progress bars. Includes path-traversal validation.
3. **`src/data/`** — Format-agnostic data building blocks:
   - `faker.rs` — Procedural fake data (names, emails, UUIDs, etc.) using static `const` pools + `rand::Rng`. Schema parsing via `parse_schema("name:string,age:int")`.
   - `lorem.rs` — Lorem ipsum text generation (words, sentences, paragraphs).
4. **`src/formats/`** — One module per format, each implementing `Generator`. Registry in `mod.rs::get_generator()` maps format strings to boxed generators.

**Data flow:** `main.rs` parses CLI → builds `FormatOptions` + `BatchConfig` → calls `run_batch()` → rayon parallel iter → per-file `Generator::generate()` returns `Vec<u8>` → written to disk.

## Adding a New Format

1. Create `src/formats/<name>.rs` with a struct implementing `Generator` (return raw bytes from `generate()`).
2. Register it in `src/formats/mod.rs`: add `pub mod <name>;` and a match arm in `get_generator()`.
3. Add a variant to `FormatCommand` in `src/cli/mod.rs` with `clap` attributes.
4. Add a match arm in `main.rs` to map the subcommand to `FormatOptions` + extension string.
5. Add integration tests in `tests/cli_integration.rs`.

Follow existing patterns — e.g., [src/formats/json.rs](src/formats/json.rs) for structured data or [src/formats/png.rs](src/formats/png.rs) for binary formats.

## Build & Test Commands

```bash
cargo build                    # Debug build
cargo build --release          # Optimized build (LTO enabled, stripped)
cargo test                     # All unit + integration tests
cargo test --test cli_integration  # Integration tests only (spawns binary)
cargo clippy                   # Lint (should pass clean)
```

The release profile uses `lto = true`, `codegen-units = 1`, and `strip = true`.

## Error Handling

- Use `thiserror` types defined in `src/error.rs`: `AppError` (top-level) wraps `GenerationError` (format-specific).
- Type aliases: `AppResult<T>` and `GenResult<T>` — generators return `GenResult<Vec<u8>>`.
- Map format-specific errors to appropriate `GenerationError` variants (e.g., `Image(String)`, `Audio(String)`, `Serialization(String)`).

## Conventions

- **RNG:** All randomness goes through `&mut impl Rng` passed via `GeneratorConfig.rng` (seeded `ChaCha8Rng`). Never use `thread_rng()` in generators.
- **Schema types:** Supported faker types in `data/faker.rs::value_for_type()`: `string`, `name`, `int`/`integer`, `float`/`decimal`, `bool`/`boolean`, `email`, `date`, `datetime`, `phone`, `address`, `company`, `url`, `uuid`, `ipv4`/`ip`.
- **FormatOptions:** Always pattern-match the expected variant in `generate()` and return `GenerationError::InvalidConfig` for mismatches.
- **Tests:** Unit tests live in `#[cfg(test)] mod tests` at the bottom of each file. Integration tests in `tests/cli_integration.rs` use `assert_cmd` + `tempfile` to exercise the binary end-to-end.
- **Doc comments:** Every public item has a `///` doc comment. Module-level docs use `//!`-style or `///` at the top of the file.
- **Feature flags:** The `update` feature (default-enabled) gates self-update functionality.

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` (derive) | CLI parsing |
| `rayon` | Parallel batch generation |
| `rand` + `rand_chacha` | Deterministic RNG |
| `image` | PNG/JPG/WebP/GIF generation |
| `hound` | WAV audio writing (MP3 pipeline) |
| `serde_json`, `quick-xml`, `csv` | Structured data serialization |
| `thiserror` + `anyhow` | Error handling |
| `self_update` | GitHub Releases self-update |
