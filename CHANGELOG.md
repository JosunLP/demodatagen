# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
