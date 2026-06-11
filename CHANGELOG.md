# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2026-06-11

A major usability-and-reach release: the interface is now translated into nine
languages, the schema engine ships with ready-made presets and typo-aware
suggestions, the CLI gains `info`, `presets`, `--dry-run`, and `--jobs`, the
terminal UI gets a gradient banner and a boxed info panel, and thirteen new
schema field types land — all behind a fully DRY, compile-checked design.

### Added

- **Five more interface languages (9 total).** `--lang` now speaks **Italian,
  Portuguese, Dutch, Polish, and Swedish** in addition to English, German,
  French, and Spanish — one for every language family covered by the data
  locales. The `catalog!` macro generates all nine immutable catalogs from a
  single table, and a new `Catalog::fields()` accessor powers completeness and
  `{placeholder}`-consistency tests that fail the build on any gap or mismatch.
- **Built-in schema presets** (`src/presets.rs`). Twelve ready-made schemas —
  `users`, `employees`, `customers`, `products`, `orders`, `transactions`,
  `events`, `servers`, `geo`, `posts`, `payments`, `sensors` — usable on any
  structured format via `--preset <name>` (mutually exclusive with `--schema`),
  each with a localized description. The new `presets` subcommand lists them and
  the schema each expands to.
- **`info` subcommand** — a boxed panel of version, build target, profile, format
  / locale / language / preset counts, worker-thread count, self-update status,
  license, and repository.
- **`--dry-run`** — plan a run and print the file paths that *would* be written
  (to stdout, capped and greppable) without touching the filesystem.
- **`--jobs` / `-j`** — cap the Rayon worker-thread count (default: all cores).
- **Thirteen new schema field types**: `bic`, `ean`, `imei`, `card_network`,
  `company_email`, `job_level`, `http_method`, `http_status`, `os`, `browser`,
  `device`, `file_size`, and `coordinates` (with aliases) — bringing the faker
  library past 70 generators.
- **"Did you mean …?" schema hints.** A Levenshtein-based `suggest_type()` turns
  a typo like `emial` into a localized hint pointing at `email`; generation
  still proceeds (unknown types degrade to a generic word).
- **Terminal UI upgrades** — a per-character gradient banner, an attended-only
  animated banner reveal, a reusable `Spinner` (now driving the update check), a
  rounded **boxed** panel renderer, per-group icons in `list`, and a richer
  progress style. All motion is gated on `animations_enabled()`, so pipes, CI,
  `--quiet`, `--color never`, and tests stay perfectly static.

### Changed

- The schema type catalogue moved into `src/data/schema.rs` as the single source
  of truth (`FIELD_TYPE_GROUPS` + `KNOWN_TYPE_NAMES`), consumed by `list`,
  suggestions, and a drift-guard test — removing the duplicate table that lived
  in the CLI module.
- `DataArgs` builders are now fallible and `Language`-aware so `--preset`
  resolution can surface a localized error; the Luhn check-digit logic shared by
  `credit_card` and `imei` was extracted into one helper.
- CI now runs doctests explicitly (`cargo test --doc`), checks the MSRV (1.86),
  builds the docs with `-D warnings`, and cancels superseded runs.

### Fixed

- The bug-report template's reproduction example used a non-existent
  `generate --format` syntax; it now shows the real `demodatagen <format>
  --schema …` invocation.

## [0.4.0] - 2026-06-11

A major quality-and-reach release: a fully internationalized interface, eight
new data locales, an animated terminal UI, eleven new schema field types, and a
substantial DRY/OOP refactor of the CLI and locale subsystems.

### Added

- **Internationalized interface** (`src/i18n`). All progress, summary, update,
  and `list` output is now localized to **English, German, French, and Spanish**
  via `--lang`, auto-detected from `DEMODATAGEN_LANG` / `LC_ALL` / `LC_MESSAGES`
  / `LANG` / `LANGUAGE` and defaulting to English. A single declarative
  `catalog!` table is the source of truth, so a missing translation is a compile
  error; the `tr!` macro fills `{placeholder}` templates at call sites.
- **8 new data locales** (10 total): `en_gb`, `fr_fr`, `es_es`, `it_it`,
  `pt_br`, `nl_nl`, `pl_pl`, `sv_se` — each with authentic names, streets,
  cities, regions, and legal-form company suffixes. Locale data now lives in
  per-locale modules under `src/data/locale/`, registered via a `define_locales!`
  macro (adding a locale is one file plus one table line).
- **Animated, colorized terminal UI** (`src/ui`): a startup banner, an
  indeterminate spinner for single files, a live progress bar with throughput
  and ETA for batches, and styled success/partial/error summaries with total
  bytes and elapsed time. Respects `NO_COLOR`; new `--color auto|always|never`.
- **11 new schema field types**: `percent`, `rating`, `port`, `ssn`,
  `currency_symbol`, `mime_type`, `filename`, `semver`, `hashtag`, `base64`,
  and `hex(n)`.
- **Locale-aware postal codes** — Dutch (`1234 AB`), Swedish (`123 45`), Polish
  (`12-345`), UK (`SW1 9AA`), and Brazilian (`12345-678`) formats — plus
  per-locale street-number ordering driven by locale data rather than hard-coded
  country checks.
- Short flags `-q` (`--quiet`) and `-v` (`--verbose`).

### Changed

- **DRY CLI refactor**: repeated subcommand parameters are now reusable argument
  groups (`DataArgs`, `ImageArgs`, `AudioArgs`, `VideoArgs`, `DocArgs`,
  `TextArgs`) flattened in via `#[command(flatten)]`, each owning its mapping to
  `FormatOptions`. `resolve_format` shrank to one-liners per format.
- ASCII transliteration broadened from German umlauts to all Latin diacritics
  used by the supported locales, so every locale yields clean ASCII emails.
- Status output and diagnostic logging are now cleanly separated: user-facing
  text goes through the styled UI on stderr; `--verbose`/`RUST_LOG` controls
  diagnostics only. `--stdout` keeps a pristine, pipeable byte stream.
- Self-update messages are localized and routed through the UI.

### Fixed

- README documented `markdown --sections`; the flag is `--headings`.
- Removed an unused `update::perform_update` helper.

## [0.3.0] - 2026-06-04

Distribution overhaul: hardened install/update/uninstall mechanisms, checksummed
releases for more platforms, and CI/build fixes.

### Added

- **`demodatagen update --tag <vX.Y.Z>`** to update (or downgrade) to a specific
  release; `update` now runs non-interactively with a download progress bar.
- **Checksum-verifying installers.** `install.sh` and `install.ps1` download the
  release `SHA256SUMS` and verify the archive's SHA-256 before installing.
- **`install.sh` rewrite**: musl-vs-glibc detection, `curl` *or* `wget`, non-root
  fallback to `~/.local/bin` (sudo only when needed), automatic `PATH` setup,
  post-install run check, and flags `--version/--bin-dir/--prefix/--repo/`
  `--no-modify-path/--force/--quiet`.
- **`uninstall.sh` rewrite**: locates installs across common dirs and `PATH`,
  removes installer-added `PATH` entries, and `--purge` for config/cache/data.
- **PowerShell scripts rewrite**: arch awareness, checksum verification, user +
  current-session `PATH` updates, TLS 1.2, and `-Version/-InstallDir/-Force/`
  `-Quiet/-NoModifyPath/-Purge` flags.
- **Release binaries for aarch64 Linux** (gnu + musl) on native ARM runners, and
  a published `SHA256SUMS` file. Builds now use `--locked`.
- Semver-aware update checks (`0.10.0` correctly newer than `0.9.0`) that degrade
  gracefully when offline or when no releases exist.

### Changed

- `self_update` now uses **rustls** instead of native-tls, so static musl builds
  no longer require OpenSSL.

### Fixed

- Corrected the placeholder repository (`youruser`/`user` → `josunlp`) in the
  self-update module, both install scripts, both uninstall scripts, and
  `Cargo.toml` — self-update and the install scripts now target the real repo.
- Pinned the transitive `time` crate to `0.3.36` so the `rust:1.86` Docker builder
  (and the declared MSRV) keeps compiling (newer `time` requires rustc 1.88).

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
