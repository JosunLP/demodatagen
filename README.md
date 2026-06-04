# demodatagen

A fast, offline CLI **and library** for generating realistic demo files in **33 formats**.

Built in Rust for maximum performance, with a typed schema engine, locale-aware
fake data, parallel batch generation, deterministic seeding, and zero external
service dependencies.

## Features

| Category              | Formats                                     |
| --------------------- | ------------------------------------------- |
| **Structured data**   | JSON, JSONL, YAML, TOML, XML, CSV, TSV, SQL |
| **Text & config**     | TXT, Markdown, HTML, LOG, INI, ENV          |
| **Images**            | PNG, JPG, WebP, BMP, TIFF, ICO, GIF, SVG    |
| **Audio & video**     | MP3, WAV, MP4, WebM                         |
| **Documents**         | PDF, XLSX                                   |
| **Binary & archives** | EXE, DLL, ZIP, TAR, GZIP                    |

- **Typed schema engine** – ranges, enums, sequences, arrays, nullable fields
- **~50 fake-data types** – names, emails, UUIDs, IBANs, credit cards, geo, and more
- **Locale-aware** – `en_us` and `de_de` for region-appropriate data
- **Deterministic output** – pass `--seed` for reproducible results
- **Parallel batch generation** – uses all CPU cores via Rayon with progress bars
- **Real, valid files** – WAV plays, PDF opens, XLSX loads in Excel, archives extract
- **Library or CLI** – embed the engine in your own Rust code
- **Shell completions & `list`** – first-class CLI ergonomics
- **Self-update** – checks GitHub Releases for new versions

## Installation

### From source

```bash
cargo install --path .
```

### Pre-built binaries

The install scripts auto-detect your OS/arch (incl. musl vs glibc on Linux),
**verify the release SHA-256 checksum**, install the binary, and add it to your
`PATH`.

**Linux / macOS:**

```bash
curl -fsSL https://raw.githubusercontent.com/j-pfalzgraf/demodatagen/main/install.sh | bash
```

Options (run the downloaded script directly): `--version vX.Y.Z`, `--bin-dir DIR`,
`--prefix DIR`, `--no-modify-path`, `--force`, `--quiet`. Without root it installs
to `~/.local/bin`. Remove with `./uninstall.sh` (`--purge` also clears config/cache).

**Windows (PowerShell):**

```powershell
iwr -useb https://raw.githubusercontent.com/j-pfalzgraf/demodatagen/main/install.ps1 | iex
```

Options: `-Version v0.3.0`, `-InstallDir DIR`, `-NoModifyPath`, `-Force`, `-Quiet`.
Remove with `.\uninstall.ps1` (`-Purge` to also clear data).

Prebuilt binaries are published for Linux (x86_64/aarch64, gnu + musl), macOS
(x86_64/aarch64), and Windows (x86_64) on [GitHub Releases](https://github.com/j-pfalzgraf/demodatagen/releases).

### Self-update

```bash
demodatagen update            # update to the latest release
demodatagen update --tag v0.3.0   # update (or downgrade) to a specific tag
demodatagen --check-update    # report whether an update is available
```

### Docker

```bash
docker run --rm -v "$PWD/output:/output" ghcr.io/j-pfalzgraf/demodatagen json --schema "name:name,age:int" --rows 10
```

## Quick start

```bash
# 100 JSON files of fake users
demodatagen -c 100 -o ./data json --schema "id:sequence,name:name,email:email" --rows 50

# A SQL seed script
demodatagen sql --table users --rows 1000 --schema "id:sequence,name:name,age:int(18..90),active:bool"

# German test data, streamed to stdout
demodatagen --locale de_de --stdout csv --rows 20 --schema "name:name,city:city,iban:iban"

# See everything on offer
demodatagen list
```

## Usage

```text
demodatagen [OPTIONS] <COMMAND>
```

### Global options

| Flag                   | Short | Description                              | Default    |
| ---------------------- | ----- | ---------------------------------------- | ---------- |
| `--output-dir <DIR>`   | `-o`  | Output directory                         | `./output` |
| `--count <N>`          | `-c`  | Number of files to generate              | `1`        |
| `--seed <N>`           | `-s`  | RNG seed for reproducibility             | random     |
| `--locale <LOCALE>`    | `-l`  | Data locale (`en_us`, `de_de`)           | `en_us`    |
| `--name-pattern <PAT>` | `-n`  | Filename pattern (`{n}` = index)         | `demo_{n}` |
| `--stdout`             |       | Write one file to stdout instead of disk | `false`    |
| `--overwrite`          |       | Overwrite existing files                 | `false`    |
| `--quiet`              |       | Suppress all output except errors        | `false`    |
| `--verbose`            |       | Enable debug logging                     | `false`    |

### Structured data

```bash
demodatagen json  --schema "name:name,email:email,age:int(18..65)" --rows 100 --pretty
demodatagen jsonl --schema "id:sequence,event:enum(click,view,buy)" --rows 1000
demodatagen yaml  --schema "id:sequence,name:name" --rows 20
demodatagen toml  --schema "host:domain,port:int(1024..65535)" --rows 5
demodatagen xml   --schema "user:name,score:float" --rows 50 --root users --row-tag user --pretty
demodatagen csv   --schema "first:first_name,last:last_name,email:email" --rows 200 --delimiter ";"
demodatagen tsv   --schema "a:int,b:float" --rows 50
demodatagen sql   --schema "id:sequence,name:name,price:price(1..999)" --rows 100 --table products
```

### Text & config

```bash
demodatagen txt      --paragraphs 5
demodatagen markdown --sections 4 --paragraphs 3
demodatagen html     --headings 4 --paragraphs 3
demodatagen log      --lines 500 --style apache    # apache | syslog | json
demodatagen ini      --sections 3 --keys 5
demodatagen env      --keys 10
```

### Images

```bash
demodatagen png  --width 800 --height 600 --pattern gradient   # noise|gradient|shapes|checkerboard
demodatagen jpg  --width 1024 --height 768
demodatagen webp --width 512 --height 512 --pattern shapes
demodatagen bmp  --width 256 --height 256
demodatagen tiff --width 256 --height 256
demodatagen ico  --size 64                                     # max 256
demodatagen gif  --width 128 --height 128 --frames 10
demodatagen svg  --width 400 --height 300 --shapes 30
```

### Audio & video

```bash
demodatagen mp3  --duration 5 --tone sine --sample-rate 44100  # sine|noise|sweep
demodatagen wav  --duration 5 --tone sweep                     # real, playable PCM
demodatagen mp4  --width 320 --height 240 --duration 3 --fps 24
demodatagen webm --width 320 --height 240 --duration 2 --fps 30
```

### Documents

```bash
demodatagen pdf  --headings 4 --paragraphs 8                   # valid, multi-page PDF
demodatagen xlsx --schema "id:sequence,name:name,total:price(1..999)" --rows 50 --sheet Sales
```

### Binary & archives

```bash
demodatagen exe  --size 8192
demodatagen dll  --size 8192
demodatagen zip  --files 10 --contained-format csv --compression-level 6
demodatagen tar  --files 10 --contained-format json
demodatagen gzip --paragraphs 20
```

## Schema syntax

The `--schema` option accepts `field:type` pairs separated by commas. A type may
carry arguments in parentheses and a trailing `?` for nullability:

```text
id:sequence              # 1, 2, 3, … per row
id:sequence(100)         # start at 100
age:int(18..65)          # bounded integer
score:float(0..1)        # bounded float
price:price(0..999)      # money, 2 decimals
status:enum(new,paid)    # pick one at random
country:const(DE)        # fixed value
tags:array(word,3)       # array of 3 words
phone:phone?             # ~10% chance of null
note:sentence?0.5        # 50% chance of null
```

### Field types

| Group         | Types                                                                                        |
| ------------- | -------------------------------------------------------------------------------------------- |
| **Numeric**   | `int`, `float`, `price`, `age`, `year`, `latitude`, `longitude`, `timestamp`                 |
| **Boolean**   | `bool`                                                                                       |
| **People**    | `name`, `first_name`, `last_name`, `username`, `gender`, `password`                          |
| **Contact**   | `email`, `phone`, `address`, `street`, `city`, `state`, `zipcode`, `country`                 |
| **Business**  | `company`, `job`, `department`, `product`, `sku`, `currency`, `iban`, `credit_card`, `isbn`  |
| **Internet**  | `url`, `domain`, `slug`, `ipv4`, `ipv6`, `mac`, `uuid`, `user_agent`                         |
| **Misc**      | `color`, `hex_color`, `language`, `timezone`, `emoji`                                        |
| **Temporal**  | `date`, `time`, `datetime`, `weekday`, `month`                                               |
| **Text**      | `word`, `words(n)`, `sentence`, `paragraph`                                                  |
| **Modifiers** | `enum(...)`, `const(...)`, `sequence(start)`, `array(type,n)`, `type?` / `type?p` (nullable) |

Run `demodatagen list` for the full catalogue. Unknown types degrade gracefully
to a generic word rather than failing.

## Locales

`--locale de_de` switches names, addresses, cities, regions, and company forms to
German equivalents (emails and usernames are transliterated to ASCII):

```bash
demodatagen --locale de_de json --schema "name:name,city:city,company:company" --rows 3
```

## Batch generation

Generate multiple files at once with `-c`:

```bash
demodatagen -c 100 -o ./data -n "user_{n}" json --schema "id:uuid,name:name" --rows 50
```

This creates 100 JSON files (`user_0.json` … `user_99.json`) in `./data/`, all
generated in parallel across CPU cores.

## Shell completions

```bash
demodatagen completions bash > /etc/bash_completion.d/demodatagen
demodatagen completions zsh  > ~/.zfunc/_demodatagen
```

Supported shells: `bash`, `zsh`, `fish`, `powershell`, `elvish`.

## Library usage

`demodatagen` is also a library:

```rust
use demodatagen::core::batch::{run_batch, BatchConfig};
use demodatagen::core::generator::FormatOptions;
use demodatagen::data::Locale;
use demodatagen::formats::get_generator;
use std::path::PathBuf;

let generator = get_generator("json").unwrap();
let config = BatchConfig {
    output_dir: PathBuf::from("./output"),
    count: 3,
    name_pattern: "demo_{n}".into(),
    extension: generator.file_extension().to_string(),
    overwrite: true,
    seed: Some(42),
    quiet: true,
    locale: Locale::EnUs,
    format_options: FormatOptions::StructuredData {
        rows: 10,
        schema: "id:sequence,name:name,email:email".into(),
        pretty: true,
    },
};
let paths = run_batch(generator.as_ref(), &config).unwrap();
```

You can also drive the schema engine directly:

```rust
use demodatagen::data::{Locale, Schema};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

let schema = Schema::parse("id:sequence,name:name,age:int(18..65)").unwrap();
let mut rng = ChaCha8Rng::seed_from_u64(42);
let records = schema.generate_records(&mut rng, Locale::EnUs, 100);
```

## Project structure

```text
src/
├── main.rs            # Thin binary entry point
├── lib.rs             # Library root (public API)
├── app.rs             # CLI orchestration (parse → generate)
├── error.rs           # Error types (AppError, GenerationError)
├── cli/               # clap argument definitions, `list`, completions
├── core/
│   ├── generator.rs   # Generator trait, FormatOptions, config
│   └── batch.rs       # Parallel batch execution
├── data/
│   ├── schema.rs      # Typed schema engine (FieldValue, Schema)
│   ├── faker.rs       # Fake-data generators
│   ├── locale.rs      # Locale data pools (en_us, de_de)
│   └── lorem.rs       # Lorem ipsum text
├── formats/           # One module per format (33 generators)
└── update/            # Self-update via GitHub Releases
```

## Development

```bash
cargo build              # Build
cargo test               # Run all tests (unit + integration + property)
cargo clippy --all-targets   # Lint
cargo fmt                # Format
RUST_LOG=debug cargo run -- json --schema "name:name" --rows 5
```

## License

MIT – see [LICENSE](LICENSE) for details.
