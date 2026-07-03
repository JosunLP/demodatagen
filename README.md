# demodatagen

A fast, offline, **fully internationalized** CLI **and library** for generating
realistic demo files in **40 formats**, **16 data locales**, and **15 interface
languages**.

Built in Rust for maximum performance, with a typed schema engine, built-in
schema presets, locale-aware fake data, parallel batch generation, deterministic
seeding, an animated terminal UI, a fifteen-language interface, and zero external
service dependencies.

## Features

| Category              | Formats                                     |
| --------------------- | ------------------------------------------- |
| **Structured data**   | JSON, JSONL, YAML, TOML, XML, CSV, TSV, SQL, GeoJSON |
| **Text & config**     | TXT, Markdown, HTML, LOG, INI, ENV, Properties, SRT  |
| **Images**            | PNG, JPG, WebP, BMP, TIFF, ICO, GIF, SVG             |
| **Audio & video**     | MP3, WAV, MP4, WebM                                  |
| **Documents**         | PDF, XLSX, RTF, vCard, iCalendar, EML                |
| **Binary & archives** | EXE, DLL, ZIP, TAR, GZIP                             |

- **Typed schema engine** – ranges, enums, sequences, arrays, nullable fields,
  and a "did you mean …?" hint for mistyped field types
- **70+ fake-data types** – names, emails, UUIDs, IBANs, BICs, credit cards, geo
  coordinates, SSNs, MIME types, semver, IMEIs, EAN barcodes, HTTP methods &
  statuses, and more
- **16 built-in schema presets** – `users`, `products`, `orders`, `invoices`,
  `logins`, … via `--preset`, so the common cases need no hand-written schema
- **16 data locales** – `en_us`, `en_gb`, `de_de`, `fr_fr`, `es_es`, `it_it`,
  `pt_br`, `nl_nl`, `pl_pl`, `sv_se`, `da_dk`, `nb_no`, `fi_fi`, `cs_cz`,
  `tr_tr`, `ja_jp` for region-appropriate names, addresses, postal-code
  formats, and company forms
- **15-language interface** – messages in English, German, French, Spanish,
  Italian, Portuguese, Dutch, Polish, Swedish, Danish, Norwegian, Finnish,
  Czech, Turkish, and Japanese (`--lang`), auto-detected from your system
  locale, defaulting to English
- **Instant terminal preview** – `demodatagen preview --preset users` renders
  sample records as an animated, type-colored table without writing a file
- **Polished, animated CLI** – a gradient banner, spinners, a live progress bar
  with throughput & ETA, colorized summaries, a boxed `info` panel, and `list` /
  `presets` overviews; honors `NO_COLOR` and non-TTY pipes
- **Plan before you write** – `--dry-run` reports exactly what would be generated
- **Deterministic output** – pass `--seed` for byte-identical, reproducible results
- **Parallel batch generation** – uses all CPU cores via Rayon (cap with `--jobs`)
- **Real, valid files** – WAV plays, PDF opens, XLSX loads in Excel, archives extract
- **Library or CLI** – embed the engine in your own Rust code
- **Shell completions, self-update, `list`, `presets` & `info`** – first-class
  CLI ergonomics

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
curl -fsSL https://raw.githubusercontent.com/josunlp/demodatagen/main/install.sh | bash
```

Options (run the downloaded script directly): `--version vX.Y.Z`, `--bin-dir DIR`,
`--prefix DIR`, `--no-modify-path`, `--force`, `--quiet`. Without root it installs
to `~/.local/bin`. Remove with `./uninstall.sh` (`--purge` also clears config/cache).

**Windows (PowerShell):**

```powershell
iwr -useb https://raw.githubusercontent.com/josunlp/demodatagen/main/install.ps1 | iex
```

Options: `-Version v0.5.0`, `-InstallDir DIR`, `-NoModifyPath`, `-Force`, `-Quiet`.
Remove with `.\uninstall.ps1` (`-Purge` to also clear data).

Prebuilt binaries are published for Linux (x86_64/aarch64, gnu + musl), macOS
(x86_64/aarch64), and Windows (x86_64) on [GitHub Releases](https://github.com/josunlp/demodatagen/releases).

### Self-update

```bash
demodatagen update                # update to the latest release
demodatagen update --tag v0.5.0   # update (or downgrade) to a specific tag
demodatagen --check-update        # report whether an update is available
```

### Docker

```bash
docker run --rm -v "$PWD/output:/output" ghcr.io/josunlp/demodatagen json --schema "name:name,age:int" --rows 10
```

## Quick start

```bash
# 100 JSON files of fake users
demodatagen -c 100 -o ./data json --schema "id:sequence,name:name,email:email" --rows 50

# Skip the schema entirely with a built-in preset
demodatagen csv --preset products --rows 200

# A SQL seed script
demodatagen sql --table users --rows 1000 --schema "id:sequence,name:name,age:int(18..90),active:bool"

# German test data, with a German interface, streamed to stdout
demodatagen --locale de_de --lang de --stdout csv --rows 20 --schema "name:name,city:city,iban:iban"

# Preview sample records as a table in the terminal — no files written
demodatagen preview --preset orders --rows 5

# Plan a large run without writing anything
demodatagen -c 1000 --dry-run json --preset orders

# See everything on offer (formats, schema types, presets, locales, languages)
demodatagen list
```

## Usage

```text
demodatagen [OPTIONS] <COMMAND>
```

### Global options

| Flag                   | Short | Description                                 | Default    |
| ---------------------- | ----- | ------------------------------------------- | ---------- |
| `--output-dir <DIR>`   | `-o`  | Output directory                            | `./output` |
| `--count <N>`          | `-c`  | Number of files to generate                 | `1`        |
| `--seed <N>`           | `-s`  | RNG seed for reproducibility                | random     |
| `--locale <LOCALE>`    | `-l`  | Data locale (`en_us`, `de_de`, `fr_fr`, …)  | `en_us`    |
| `--lang <LANG>`        |       | Interface language (15; run `list` for all) | auto / en  |
| `--color <WHEN>`       |       | Colorize output (`auto`, `always`, `never`) | `auto`     |
| `--name-pattern <PAT>` | `-n`  | Filename pattern (`{n}` = index)            | `demo_{n}` |
| `--dry-run`            |       | Plan the run; print what would be written   | `false`    |
| `--jobs <N>`           | `-j`  | Worker threads                              | all cores  |
| `--stdout`             |       | Write one file to stdout instead of disk    | `false`    |
| `--overwrite`          |       | Overwrite existing files                    | `false`    |
| `--quiet`              | `-q`  | Suppress all output except errors           | `false`    |
| `--verbose`            | `-v`  | Enable debug logging                        | `false`    |

### Structured data

```bash
demodatagen json  --schema "name:name,email:email,age:int(18..65)" --rows 100 --pretty
demodatagen jsonl --schema "id:sequence,event:enum(click,view,buy)" --rows 1000
demodatagen yaml  --schema "id:sequence,name:name" --rows 20
demodatagen toml  --schema "host:domain,port:port" --rows 5
demodatagen xml   --schema "user:name,score:float" --rows 50 --root users --row-tag user --pretty
demodatagen csv   --schema "first:first_name,last:last_name,email:email" --rows 200 --delimiter ";"
demodatagen tsv   --schema "a:int,b:float" --rows 50
demodatagen sql   --schema "id:sequence,name:name,price:price(1..999)" --rows 100 --table products
demodatagen geojson --schema "name:city,population:int(1000..900000)" --rows 50 --pretty
```

### Text & config

```bash
demodatagen txt      --paragraphs 5
demodatagen markdown --headings 4 --paragraphs 3
demodatagen html     --headings 4 --paragraphs 3
demodatagen log      --lines 500 --style apache    # apache | syslog | json
demodatagen ini      --sections 3 --keys 5
demodatagen env      --keys 10
demodatagen properties --sections 3 --keys 5     # Java .properties
demodatagen srt      --cues 50                   # SubRip subtitles
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
demodatagen rtf  --headings 3 --paragraphs 6                   # Rich Text Format
demodatagen vcf  --contacts 25                                 # vCard 3.0 contact cards
demodatagen ics  --events 10                                   # iCalendar events
demodatagen eml  --paragraphs 3                                # RFC 5322 email messages
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

| Group         | Types                                                                                                          |
| ------------- | -------------------------------------------------------------------------------------------------------------- |
| **Numeric**   | `int`, `float`, `price`, `age`, `year`, `latitude`, `longitude`, `percent`, `rating`, `port`, `timestamp`      |
| **Boolean**   | `bool`                                                                                                         |
| **People**    | `name`, `first_name`, `last_name`, `username`, `gender`, `password`, `ssn`                                     |
| **Contact**   | `email`, `phone`, `address`, `street`, `city`, `state`, `zipcode`, `country`, `country_code`                   |
| **Business**  | `company`, `job`, `department`, `product`, `sku`, `currency`, `currency_symbol`, `iban`, `credit_card`, `isbn` |
| **Internet**  | `url`, `domain`, `slug`, `ipv4`, `ipv6`, `mac`, `uuid`, `user_agent`, `mime_type`, `filename`, `semver`        |
| **Misc**      | `color`, `hex_color`, `language`, `timezone`, `emoji`, `hashtag`, `base64`, `hex(n)`, `airport`, `flight`, `vin` |
| **Temporal**  | `date`, `time`, `datetime`, `weekday`, `month`                                                                 |
| **Text**      | `word`, `words(n)`, `sentence`, `paragraph`                                                                    |
| **Modifiers** | `enum(...)`, `const(...)`, `sequence(start)`, `array(type,n)`, `type?` / `type?p` (nullable)                   |

The catalogue above is representative; 0.5.0 also adds `http_status`,
`job_level`, `company_email`, `coordinates`, `bic`, `card_network`, `ean`,
`http_method`, `os`, `browser`, `device`, `imei`, and `file_size`. Run
`demodatagen list` for the complete, always-current reference.

Unknown types degrade gracefully to a generic word rather than failing — and a
**"did you mean …?"** hint points at the closest known type, so a typo like
`emial` is caught immediately:

```console
$ demodatagen json --schema "mail:emial"
⚠ Unknown schema type 'emial'; did you mean 'email'? Generating a generic word instead.
```

## Schema presets

Don't want to write a schema at all? Use a **preset** — a named, ready-made
schema for a common shape — on any structured format:

```bash
demodatagen json --preset users --rows 100
demodatagen csv  --preset products --rows 500
demodatagen sql  --preset orders --table orders --rows 1000
```

`--preset` and `--schema` are mutually exclusive. Run `demodatagen presets` to
see every preset and the schema it expands to. Built in: `users`, `employees`,
`customers`, `products`, `orders`, `transactions`, `events`, `servers`, `geo`,
`posts`, `payments`, `sensors`, `invoices`, `logins`, `vehicles`, `books`.

## Discovering & planning

```bash
demodatagen list      # formats, schema types, presets, locales, languages
demodatagen presets   # built-in presets and the schema each expands to
demodatagen info      # version, build, threads, and capability counts
demodatagen preview --preset users --rows 5          # sample records as a table
demodatagen -c 1000 --dry-run json --preset orders   # plan without writing
```

## Data locales

`--locale` switches names, addresses, postal-code formats, cities, regions, and
company forms to region-appropriate equivalents (emails/usernames are always
transliterated to ASCII):

| Locale  | Region                   | Locale  | Region                    |
| ------- | ------------------------ | ------- | ------------------------- |
| `en_us` | English (United States)  | `sv_se` | Swedish (Sweden)          |
| `en_gb` | English (United Kingdom) | `da_dk` | Danish (Denmark)          |
| `de_de` | German (Germany)         | `nb_no` | Norwegian Bokmål (Norway) |
| `fr_fr` | French (France)          | `fi_fi` | Finnish (Finland)         |
| `es_es` | Spanish (Spain)          | `cs_cz` | Czech (Czechia)           |
| `it_it` | Italian (Italy)          | `tr_tr` | Turkish (Türkiye)         |
| `pt_br` | Portuguese (Brazil)      | `ja_jp` | Japanese (Japan)          |
| `nl_nl` | Dutch (Netherlands)      | `pl_pl` | Polish (Poland)           |

```bash
demodatagen --locale pt_br json --schema "name:name,city:city,company:company" --rows 3
```

## Interface language

Separately from the *data* locale, `--lang` selects the language of the
**program's own messages** (progress, summaries, errors, `list`, `presets`,
`info`). Supported: `en`, `de`, `fr`, `es`, `it`, `pt`, `nl`, `pl`, `sv`, `da`,
`nb`, `fi`, `cs`, `tr`, `ja` — one for every language family covered by the
data locales. When omitted, the
language is detected from `DEMODATAGEN_LANG` and the standard `LC_ALL` /
`LC_MESSAGES` / `LANG` / `LANGUAGE` variables, falling back to English.

```bash
demodatagen --lang fr -c 5 json          # French interface, English data
demodatagen --locale de_de --lang de sql # German data and German interface
```

> Note: clap-generated `--help` text remains in English (the lingua franca for
> flags); everything else is fully localized.

## Batch generation

Generate multiple files at once with `-c`:

```bash
demodatagen -c 100 -o ./data -n "user_{n}" json --schema "id:uuid,name:name" --rows 50
```

This creates 100 JSON files (`user_0.json` … `user_99.json`) in `./data/`, all
generated in parallel across CPU cores, with a live progress bar.

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
use demodatagen::i18n::Language;
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
    lang: Language::En,
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
├── app.rs             # CLI orchestration (parse → generate, dry-run, --jobs)
├── error.rs           # Error types (AppError, GenerationError)
├── presets.rs         # Built-in named schemas (--preset, `presets`)
├── i18n/              # Interface translations (15 languages) + tr! macro
├── ui/                # Gradient banner, boxed panel, spinner, animated progress
├── cli/
│   ├── mod.rs         # clap definitions, `list` / `presets` / `info` / completions
│   └── args.rs        # Reusable, flattened argument groups (DRY)
├── core/
│   ├── generator.rs   # Generator trait, FormatOptions, config
│   └── batch.rs       # Parallel batch execution
├── data/
│   ├── schema.rs      # Typed schema engine + type catalogue + suggestions
│   ├── faker.rs       # 70+ fake-data generators
│   ├── locale/        # Locale registry (macro) + 16 per-locale data modules
│   └── lorem.rs       # Lorem ipsum text
├── formats/           # One module per format (40 generators)
└── update/            # Self-update via GitHub Releases
```

## Development

```bash
cargo build              # Build
cargo test               # Run all tests (unit + integration + property + doc)
cargo clippy --all-targets   # Lint (CI uses -D warnings)
cargo fmt                # Format
RUST_LOG=debug cargo run -- json --schema "name:name" --rows 5
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for the architecture overview and the
step-by-step guide to adding a new format or locale.

## License

MIT – see [LICENSE](LICENSE) for details.
