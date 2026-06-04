# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-06-04

A major expansion: **33 format generators** (up from 15), a typed schema engine,
locale support, a reusable library API, and several new CLI capabilities.

### Added

- **18 new format generators**: JSONL/NDJSON, YAML, TOML, SQL, TSV, HTML, SVG,
  WAV (real PCM audio), BMP, TIFF, ICO, LOG (apache/syslog/json), INI, ENV,
  TAR, GZIP, PDF (hand-written, dependency-free), and XLSX (real Excel).
- **Typed schema engine** (`data::schema`) producing correctly-typed values
  across all structured formats. New schema syntax:
  - bounded ranges — `int(18..65)`, `float(0..1)`, `price(1..99)`
  - `enum(a,b,c)`, `const(value)`, `sequence(start)`, `array(type,n)`
  - nullable fields — `phone?` (10%) or `note?0.5` (50%)
- **~35 new fake-data types**: `first_name`, `last_name`, `username`, `password`,
  `gender`, `job`, `department`, `product`, `sku`, `currency`, `iban`,
  `credit_card` (Luhn-valid), `isbn`, `domain`, `slug`, `ipv6`, `mac`,
  `user_agent`, `color`, `hex_color`, `language`, `timezone`, `emoji`,
  `latitude`, `longitude`, `city`, `state`, `country`, `zipcode`, `time`,
  `weekday`, `month`, `word(s)`, `sentence`, `paragraph`, and more.
- **Locale support** via `--locale` (`en_us`, `de_de`) for region-appropriate
  names, addresses, cities, and company forms.
- **Library API**: the crate now builds as a library (`demodatagen`) in addition
  to the binary, exposing the `Generator` trait, schema engine, and batch runner.
- **`list`** subcommand — prints all formats, schema types, and locales.
- **`completions <shell>`** subcommand — shell completions for bash, zsh, fish,
  PowerShell, and elvish.
- **`--stdout`** flag — stream a single generated file to stdout for piping.
- **CSV `--delimiter`** and **XML `--root` / `--row-tag`** options (previously
  documented but not implemented).
- Property-based tests (proptest) for the schema engine.

### Changed

- JSON/CSV/XML now emit typed values from the shared schema engine.
- Default schema upgraded to `id:sequence,name:name,email:email,created:datetime`.
- Removed the unused `fake` dependency.

### Fixed

- Image `shapes` pattern no longer panics for small images (empty RNG range).
- SVG no longer panics on tiny canvases and keeps shapes within the viewBox.
- XML element names (root, row tag, field names) are sanitized to valid XML
  names instead of being entity-escaped, so unusual names still produce
  well-formed XML.
- Schema rejects out-of-range null probabilities (`type?2.5`) instead of
  silently clamping them.
- `latitude`/`longitude` ranges are now inclusive, matching their documentation.

## [0.1.0] - 2025-01-01

### Added

- Initial release of `demodatagen`.
- **16 format generators**: JSON, XML, CSV, TXT, Markdown, PNG, JPG, WebP, GIF, MP3, MP4, WebM, EXE, DLL, ZIP.
- Built-in fake data engine with 15 data types (name, email, uuid, date, etc.).
- Lorem ipsum generator for text and markdown content.
- Parallel batch file generation with Rayon and progress bars (indicatif).
- Deterministic seeding (`--seed`) for reproducible output via ChaCha8 RNG.
- Pattern-based filenames (`{n}` placeholder).
- Overwrite protection with `--overwrite` flag.
- Self-update mechanism via GitHub Releases.
- Image pattern modes: noise, gradient, shapes, checkerboard.
- Audio tone modes: sine, noise, sweep.
- Animated GIF support with configurable frame count.
- MP4/WebM video stubs with valid container headers.
- PE binary stubs (EXE/DLL) with full MZ/PE header structure.
- ZIP archive generation with selectable contained-file formats.
- Cross-platform support: Linux, macOS, Windows.
- CI/CD via GitHub Actions (test, lint, release, Docker).
- Docker image published to GHCR.
- Install/uninstall scripts for Bash and PowerShell.
