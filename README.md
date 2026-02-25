# demodatagen

A fast, offline CLI tool for generating realistic demo files in **16+ formats**.

Built in Rust for maximum performance, with parallel batch generation, deterministic seeding, and zero external service dependencies.

## Features

| Category            | Formats                        |
| ------------------- | ------------------------------ |
| **Structured data** | JSON, XML, CSV                 |
| **Text**            | Plain text (TXT), Markdown     |
| **Images**          | PNG, JPG, WebP, GIF (animated) |
| **Audio**           | MP3                            |
| **Video**           | MP4, WebM                      |
| **Binary stubs**    | EXE (PE), DLL (PE)             |
| **Archives**        | ZIP                            |

- **Realistic fake data** – names, emails, addresses, UUIDs, dates, and more via a built-in faker engine
- **Deterministic output** – pass `--seed` for reproducible results
- **Parallel batch generation** – uses all CPU cores via Rayon with progress bars
- **Pattern-based filenames** – `{n}` placeholder for file index
- **Self-update** – checks GitHub Releases for new versions

## Installation

### From source

```bash
cargo install --path .
```

### Pre-built binaries

Download the latest release for your platform from [GitHub Releases](https://github.com/user/demodatagen/releases), then:

**Linux / macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/user/demodatagen/main/install.sh | bash
```

**Windows (PowerShell):**
```powershell
iwr -useb https://raw.githubusercontent.com/user/demodatagen/main/install.ps1 | iex
```

### Docker

```bash
docker run --rm -v "$PWD/output:/output" ghcr.io/user/demodatagen json --schema "name:name,age:integer" --rows 10
```

## Usage

```
demodatagen [OPTIONS] <COMMAND>
```

### Global options

| Flag                   | Short | Description                       | Default    |
| ---------------------- | ----- | --------------------------------- | ---------- |
| `--output-dir <DIR>`   | `-o`  | Output directory                  | `./output` |
| `--count <N>`          | `-c`  | Number of files to generate       | `1`        |
| `--seed <N>`           | `-s`  | RNG seed for reproducibility      | random     |
| `--name-pattern <PAT>` | `-n`  | Filename pattern (`{n}` = index)  | `demo_{n}` |
| `--overwrite`          |       | Overwrite existing files          | `false`    |
| `--quiet`              |       | Suppress all output except errors | `false`    |
| `--verbose`            |       | Enable debug logging              | `false`    |
| `--skip-update`        |       | Skip update check                 | `false`    |
| `--check-update`       |       | Only check for updates            | `false`    |

### Structured data

#### JSON
```bash
demodatagen json --schema "name:name,email:email,age:integer" --rows 100
```

Options: `--schema`, `--rows`, `--pretty`

#### XML
```bash
demodatagen xml --schema "user:name,score:float" --rows 50 --root "users" --row-tag "user"
```

Options: `--schema`, `--rows`, `--root`, `--row-tag`, `--compact`

#### CSV
```bash
demodatagen csv --schema "first:name,last:name,email:email" --rows 200
```

Options: `--schema`, `--rows`, `--delimiter`

### Text

#### Plain text
```bash
demodatagen txt --paragraphs 5
```

Options: `--paragraphs`, `--words`

#### Markdown
```bash
demodatagen markdown --sections 4 --paragraphs 3
```

Options: `--sections`, `--paragraphs`

### Images

#### PNG
```bash
demodatagen png --width 800 --height 600 --pattern gradient
```

Options: `--width`, `--height`, `--pattern` (noise | gradient | shapes | checkerboard)

#### JPG
```bash
demodatagen jpg --width 1024 --height 768
```

Options: `--width`, `--height`, `--pattern`

#### WebP
```bash
demodatagen webp --width 512 --height 512 --pattern shapes
```

Options: `--width`, `--height`, `--pattern`

#### GIF (animated)
```bash
demodatagen gif --width 128 --height 128 --frames 10
```

Options: `--width`, `--height`, `--pattern`, `--frames`

### Audio

#### MP3
```bash
demodatagen mp3 --duration 5 --tone sine --sample-rate 44100 --bitrate 128
```

Options: `--duration`, `--tone` (sine | noise | sweep), `--sample-rate`, `--bitrate`

### Video

#### MP4
```bash
demodatagen mp4 --width 320 --height 240 --duration 3 --fps 24
```

Options: `--width`, `--height`, `--duration`, `--fps`

#### WebM
```bash
demodatagen webm --width 320 --height 240 --duration 2 --fps 30
```

Options: `--width`, `--height`, `--duration`, `--fps`

### Binary stubs

#### EXE
```bash
demodatagen exe --size 8192
```

Options: `--size`

#### DLL
```bash
demodatagen dll --size 8192
```

Options: `--size`

### Archives

#### ZIP
```bash
demodatagen zip --files 10 --format csv --compression 6
```

Options: `--files`, `--format`, `--compression`

## Batch generation

Generate multiple files at once using `-c`:

```bash
demodatagen -c 100 -o ./data -n "user_{n}" json --schema "id:uuid,name:name" --rows 50
```

This creates 100 JSON files (`user_0.json` through `user_99.json`) in `./data/`, all generated in parallel.

## Schema types

The `--schema` option accepts `field:type` pairs separated by commas:

| Type         | Example output                            |
| ------------ | ----------------------------------------- |
| `name`       | `"Alice Johnson"`                         |
| `first_name` | `"Alice"`                                 |
| `last_name`  | `"Johnson"`                               |
| `email`      | `"alice@example.com"`                     |
| `integer`    | `42`                                      |
| `float`      | `3.14`                                    |
| `boolean`    | `true`                                    |
| `date`       | `"2024-03-15"`                            |
| `datetime`   | `"2024-03-15T14:30:00Z"`                  |
| `phone`      | `"+1-555-0123"`                           |
| `address`    | `"123 Oak Street, Springfield, IL 62701"` |
| `company`    | `"Tech Solutions Inc."`                   |
| `url`        | `"https://example.com/page"`              |
| `uuid`       | `"550e8400-e29b-41d4-a716-446655440000"`  |
| `ipv4`       | `"192.168.1.42"`                          |

## Project structure

```
src/
├── main.rs            # Entry point
├── error.rs           # Error types (AppError, GenerationError)
├── cli/
│   └── mod.rs         # CLI argument parsing (clap)
├── core/
│   ├── generator.rs   # Generator trait & config
│   └── batch.rs       # Parallel batch execution
├── data/
│   ├── faker.rs       # Fake data generation
│   └── lorem.rs       # Lorem ipsum text
├── formats/
│   ├── mod.rs         # Format registry
│   ├── json.rs        # JSON generator
│   ├── xml.rs         # XML generator
│   ├── csv.rs         # CSV generator
│   ├── txt.rs         # Plain text generator
│   ├── markdown.rs    # Markdown generator
│   ├── png.rs         # PNG generator (shared image buffer)
│   ├── jpg.rs         # JPG generator
│   ├── webp.rs        # WebP generator
│   ├── gif.rs         # Animated GIF generator
│   ├── mp3.rs         # MP3 generator
│   ├── mp4.rs         # MP4 generator
│   ├── webm.rs        # WebM generator
│   ├── exe.rs         # PE/EXE stub generator
│   ├── dll.rs         # PE/DLL stub generator
│   └── zip.rs         # ZIP archive generator
└── update/
    └── mod.rs         # Self-update via GitHub Releases
```

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run with verbose logging
RUST_LOG=debug cargo run -- json --schema "name:name" --rows 5

# Format code
cargo fmt

# Lint
cargo clippy
```

## License

MIT – see [LICENSE](LICENSE) for details.
