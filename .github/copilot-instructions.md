# Copilot Instructions for demodatagen

## Project Overview

`demodatagen` is a Rust **CLI and library** that generates realistic demo files in 33 formats (JSON, JSONL, YAML, TOML, XML, CSV, TSV, SQL, PDF, XLSX, PNG, WAV, MP4, ZIP, TAR, …) across **10 data locales**, with a **fully internationalized interface** (English/German/French/Spanish) and an animated terminal UI. It uses **no external services** — all data is generated procedurally with deterministic seeding via `ChaCha8Rng`.

## Architecture

The codebase builds as both a library (`src/lib.rs`) and a thin binary (`src/main.rs`, which just calls `demodatagen::app::run()`). It follows a **trait-based plugin architecture**:

1. **`src/cli/`** — CLI argument parsing via `clap` derive macros. `FormatCommand` enum maps each format to its subcommand and parameters. Also hosts `print_format_list()` (the `list` command) and `print_completions()` (shell completions via `clap_complete`).
2. **`src/app.rs`** — Orchestration: parses CLI → resolves a `FormatOptions` + format key → runs the batch (or streams to stdout with `--stdout`).
3. **`src/core/`**:
   - `generator.rs` — The `Generator` trait (`format_name()`, `file_extension()`, `generate() -> GenResult<Vec<u8>>`), the `FormatOptions` enum, `GeneratorConfig` (carries `rng` + `locale`), RNG helpers, and `test_support` constructors for tests.
   - `batch.rs` — Parallel batch execution via `rayon` with `indicatif` progress bars. Includes path-traversal validation.
4. **`src/data/`** — Format-agnostic building blocks:
   - `schema.rs` — **The typed schema engine.** `Schema::parse()` → `Vec<FieldSpec>`; `generate_records()` → `Vec<Record>` of typed `FieldValue`s. Supports ranges `int(1..9)`, `enum(...)`, `const(...)`, `sequence(n)`, `array(t,n)`, and nullable `type?p`.
   - `faker.rs` — 60+ procedural fake-data generators, locale-aware.
   - `locale/` — `Locale` enum + `LocaleData` struct, generated from a `define_locales!` table in `mod.rs`; one data module per locale (`en_us.rs`, `de_de.rs`, …, 10 total).
   - `lorem.rs` — Lorem ipsum text generation.
5. **`src/formats/`** — One module per format, each implementing `Generator`. Registry in `mod.rs::get_generator()` maps format keys to boxed generators; `FORMAT_GROUPS` is the canonical catalogue used by `list`, the banner, and tests.
6. **`src/i18n/`** — Interface translations. One `catalog!` table holds every user-facing string in `en`/`de`/`fr`/`es`; `Language::detect()` resolves `--lang`/env/default; `tr!(lang, key, "name" => val)` fills `{placeholder}` templates. A missing translation is a **compile error**.
7. **`src/ui/`** — The only module that touches `console` styling and `indicatif` progress. Banner, animated progress (spinner for one file, bar for many), and styled summaries. **All status output goes to stderr** so `--stdout` stays clean. Honors `NO_COLOR` / `--color`.
8. **`src/cli/args.rs`** — Reusable argument groups (`DataArgs`, `ImageArgs`, `AudioArgs`, `VideoArgs`, `DocArgs`, `TextArgs`) pulled into subcommands via `#[command(flatten)]`; each owns its `FormatOptions` mapping.

**Data flow:** `main.rs` → `app::run()` parses CLI → `resolve_format()` builds `FormatOptions` + key → `get_generator()` → `run_batch()` → rayon parallel iter → per-file `Generator::generate()` returns `Vec<u8>` → written to disk.

## Adding a New Format

1. Create `src/formats/<name>.rs` with a struct implementing `Generator`. Structured formats use `Schema::parse()` + `generate_records()`; reuse `FieldValue::to_json()` / `to_flat_string()` / `to_sql_literal()`.
2. Register it in `src/formats/mod.rs`: add `pub mod <name>;` and a match arm in `get_generator()` (plus the format key in the `test_all_formats_registered` test).
3. Add a variant to `FormatCommand` in `src/cli/mod.rs` with `clap` attributes.
4. Add a match arm in `src/app.rs::resolve_format()` mapping the subcommand to `FormatOptions` + format key. Reuse an existing `FormatOptions` variant where possible.
5. If the format needs a new shape of parameters, add a `FormatOptions` variant in `generator.rs` and a matching `test_support` constructor.
6. Add a `#[cfg(test)] mod tests` (use `crate::core::generator::test_support`) and an integration test in `tests/new_formats.rs`.
7. Add the format to the `list` catalogue in `cli/mod.rs::print_format_list()`.

Follow existing patterns — e.g., [src/formats/json.rs](src/formats/json.rs) for structured data, [src/formats/png.rs](src/formats/png.rs) for images, or [src/formats/pdf.rs](src/formats/pdf.rs) for a hand-written binary format.

## Build & Test Commands

```bash
cargo build                        # Debug build
cargo build --release              # Optimized build (LTO, stripped)
cargo test                         # All unit + integration + property tests
cargo test --test new_formats      # New-format integration tests only
cargo clippy --all-targets         # Lint (must pass clean; CI uses -D warnings)
cargo fmt                          # Format (CI enforces --check)
```

## Error Handling

- Use `thiserror` types in `src/error.rs`: `AppError` (top-level) wraps `GenerationError` (format-specific). `AppError::Cli` covers argument/locale errors.
- Type aliases: `AppResult<T>` and `GenResult<T>` — generators return `GenResult<Vec<u8>>`.
- Map schema-parse failures with `.map_err(GenerationError::InvalidConfig)`. Map format-specific errors to `Image`, `Audio`, `Archive`, or `Serialization` variants.

## Conventions

- **RNG:** All randomness goes through `&mut impl Rng` (or `config.rng`, a seeded `ChaCha8Rng`). Never use `thread_rng()`. Guard `gen_range` against empty/reversed ranges.
- **Locale:** Locale-sensitive faker functions take a `Locale` (from `config.locale`); locale-agnostic data (UUIDs, IPs, colors) ignores it.
- **Schema:** Prefer the typed engine over ad-hoc string generation so every format emits correctly-typed values. Unknown types degrade to a generic word (never panic).
- **FormatOptions:** Always pattern-match the expected variant in `generate()` and return `GenerationError::InvalidConfig` for mismatches.
- **Tests:** Unit tests in `#[cfg(test)] mod tests` using `test_support` constructors; property tests with `proptest`; integration tests in `tests/` use `assert_cmd` + `tempfile`. Validate real files by magic bytes / round-trip parsing.
- **Doc comments:** Every public item has a `///` doc comment; modules use `//!`.
- **i18n:** Never hard-code user-facing strings. Add a key to the `catalog!` in `src/i18n/mod.rs` (with all four translations — a gap is a compile error) and emit it via `tr!(lang, key, "name" => val)`. Thread `Language` through call sites; don't reach for a global.
- **UI/output:** Render status through `crate::ui` (which writes to **stderr**); keep stdout for `--stdout` data only. Don't `println!` status messages. Use `log::debug!` for diagnostics, never for user-facing output.
- **Feature flags:** The `update` feature (default-enabled) gates self-update.

## Adding a Locale or Interface Language

- **Data locale:** add `src/data/locale/<id>.rs` with a `pub static <ID>: LocaleData = …`, declare `mod <id>;`, and add one row to the `define_locales!` table in `src/data/locale/mod.rs`. The enum, parser, `data()`, `all()`, and `label()` are all generated from that table.
- **Interface language:** add a variant to `Language` (enum, `catalog()`, `as_str()`, `label()`, `variants()`, `all()`, `FromStr`) and a `<lang>:` arm to every entry of the `catalog!` table.

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` + `clap_complete` | CLI parsing & shell completions |
| `indicatif` + `console` | Animated progress bars & terminal styling/color |
| `rayon` | Parallel batch generation |
| `rand` + `rand_chacha` | Deterministic RNG |
| `image` | PNG/JPG/WebP/BMP/TIFF/ICO/GIF generation |
| `hound` | Real WAV audio writing |
| `serde_json`, `serde_yaml_ng`, `toml`, `quick-xml`, `csv` | Structured data |
| `rust_xlsxwriter` | XLSX (Excel) workbooks |
| `zip`, `tar`, `flate2` | Archives & compression |
| `thiserror` + `anyhow` | Error handling |
| `self_update` | GitHub Releases self-update |
