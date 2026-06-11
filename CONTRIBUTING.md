# Contributing to demodatagen

Thanks for your interest in contributing to **demodatagen** — a fast, offline Rust CLI and library
that generates realistic demo/test files in 33+ formats. Contributions of all kinds are welcome:
bug reports, new formats, locale data, documentation, and performance improvements.

This guide explains how to set up your environment, the conventions we follow, and the exact
checklist for adding a new output format.

- Repository: <https://github.com/j-pfalzgraf/demodatagen>
- Maintainer contact: <webadmins@leuchtturm.com>
- License: MIT

---

## Table of contents

- [Code of conduct](#code-of-conduct)
- [Development environment](#development-environment)
- [Build, test, lint, and format](#build-test-lint-and-format)
- [Project architecture](#project-architecture)
- [Adding a new format (7-step checklist)](#adding-a-new-format-7-step-checklist)
- [Coding conventions](#coding-conventions)
- [Commit and pull-request conventions](#commit-and-pull-request-conventions)

---

## Code of conduct

This project and everyone participating in it is governed by our
[Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold it.
Please report unacceptable behavior to <webadmins@leuchtturm.com>.

---

## Development environment

demodatagen targets **Rust edition 2021** with a **minimum supported Rust version (MSRV) of 1.86**.

1. Install Rust via [rustup](https://rustup.rs/):

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Ensure your toolchain meets the MSRV. Pinning the exact version is the most reliable way to
   reproduce CI results locally:

   ```bash
   rustup toolchain install 1.86
   rustup override set 1.86      # pins this directory to 1.86
   rustc --version               # should print 1.86.x
   ```

3. Add the components used by our lint and format checks:

   ```bash
   rustup component add rustfmt clippy
   ```

4. Clone your fork and build:

   ```bash
   git clone https://github.com/<your-username>/demodatagen.git
   cd demodatagen
   cargo build
   ```

---

## Build, test, lint, and format

Run all of the following before opening a pull request. CI
(`.github/workflows/ci.yml`) runs check, test, fmt, and clippy with warnings denied, so a green
local run means a green CI run.

```bash
# Build the project
cargo build

# Run the full test suite (unit + integration tests)
cargo test

# Lint with Clippy — warnings are treated as errors, exactly as CI does
cargo clippy --all-targets -- -D warnings

# Check formatting without modifying files (CI fails on any diff)
cargo fmt --all -- --check

# Auto-format your code before committing
cargo fmt --all
```

If any of these fail locally, CI will fail too. Please fix all warnings and formatting issues
before requesting review.

---

## Project architecture

demodatagen is built around a small, trait-based core that keeps every output format independent
and easy to add.

- **`Generator` trait** — Each output format implements a common `Generator` trait. A generator
  knows how to turn a typed schema plus a seeded random source into bytes for one file in its
  format. Keeping the surface area of this trait small is what makes the format set extensible.
- **Format registry (`src/formats/mod.rs`)** — A central `get_generator` function maps a
  `Format` value to the concrete generator that implements it. A registration test
  (`test_all_formats_registered`) guarantees that every known format resolves to a generator, so a
  format can never be silently forgotten.
- **Schema engine (`src/core/`)** — A typed schema describes the fields to generate. The schema
  engine drives generators with locale-aware fake data and deterministic, seeded randomness so that
  the same seed always reproduces the same output.
- **CLI (`src/cli/`)** — The command-line surface. Each format is exposed as a `FormatCommand`
  variant, and `app.rs` resolves CLI input to the corresponding `Format` via `resolve_format`.
- **Generator options (`src/core/generator.rs`)** — Per-format configuration is carried in
  `FormatOptions` variants, with `test_support` constructors used by the test suite.

Determinism, offline operation, and graceful degradation are first-class design goals: generation
must never reach out to the network (see [SECURITY.md](SECURITY.md)), and an unknown or unsupported
schema type should degrade gracefully rather than panic.

---

## Adding a new format (7-step checklist)

Adding a new output format is the most common kind of contribution. Follow these **seven steps in
order**. The format will not be complete (and CI will not pass) until every step is done.

1. **Create the generator module.** Add `src/formats/<name>.rs` implementing the `Generator`
   trait. This is where the format-specific encoding lives.

2. **Register the generator.** In `src/formats/mod.rs`, add an arm to `get_generator` that maps the
   new `Format` to your generator, and extend the `test_all_formats_registered` test so the new
   format is covered by the registration guarantee.

3. **Expose it on the CLI.** Add the corresponding `FormatCommand` variant in `src/cli/mod.rs`.

4. **Resolve it in the app.** Add a matching `resolve_format` arm in `src/app.rs` so CLI input maps
   to the new `Format`.

5. **Add options if needed.** If the format takes configuration, add a `FormatOptions` variant in
   `src/core/generator.rs`, along with its `test_support` constructor used by tests.

6. **Add tests.** Add unit tests inside your new module, and add integration coverage in
   `tests/new_formats.rs`.

7. **List it.** Add the format to `print_format_list` so it appears in the CLI's list of supported
   formats.

After completing the checklist, run the full
[build/test/lint/format](#build-test-lint-and-format) suite.

---

## Coding conventions

- **Seeded randomness only.** Always use the project's seeded `ChaCha8Rng`. **Never** use
  `thread_rng` or any other non-deterministic source — determinism under a given seed is a hard
  guarantee of this project.
- **Guard random ranges.** Before calling `gen_range`, guard against empty or reversed ranges
  (for example, `start >= end`). An unguarded `gen_range` on an invalid range will panic; handle
  the degenerate case explicitly instead.
- **Document every public item.** Every public type, function, trait, and module must carry a
  doc comment (`///`) describing its purpose and behavior.
- **Degrade gracefully.** When encountering an unknown or unsupported schema type, degrade
  gracefully (e.g., produce a sensible fallback value) rather than panicking or aborting the run.
- **Keep it offline.** Generation code must never make network calls. The only permitted network
  access anywhere in the project is the opt-in self-update path.
- **Match existing style.** Run `cargo fmt --all` and resolve every `cargo clippy` warning before
  committing.

---

## Commit and pull-request conventions

### Conventional Commits

We use [Conventional Commits](https://www.conventionalcommits.org/). Each commit message should
start with a type, an optional scope, and a short imperative description:

```text
<type>(<optional scope>): <description>
```

Common types:

- `feat` — a new feature (e.g., a new format)
- `fix` — a bug fix
- `docs` — documentation-only changes
- `test` — adding or correcting tests
- `refactor` — code changes that neither fix a bug nor add a feature
- `perf` — performance improvements
- `chore` — tooling, build, or maintenance changes
- `ci` — changes to CI configuration

Examples:

```text
feat(formats): add TOML generator
fix(cli): guard gen_range against empty ranges
docs: clarify the add-a-new-format checklist
```

### Pull requests

- Open pull requests against the `main` branch.
- Keep each PR focused on a single logical change.
- Ensure `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, and
  `cargo fmt --all -- --check` all pass before requesting review.
- Describe what changed and why. If you added a format, confirm in the PR description that you
  completed all seven steps of the [checklist](#adding-a-new-format-7-step-checklist).
- Link any related issues.

### Developer Certificate of Origin (DCO) / sign-off

By contributing, you certify that you wrote the contribution or otherwise have the right to submit
it under the project's MIT license, per the [Developer Certificate of Origin](https://developercertificate.org/).

Please sign off every commit. Git adds the required `Signed-off-by` trailer automatically with the
`-s` flag:

```bash
git commit -s -m "feat(formats): add TOML generator"
```

This appends a line of the form:

```text
Signed-off-by: Your Name <your.email@example.com>
```

Make sure your Git `user.name` and `user.email` are configured so the sign-off identifies you
correctly.

---

Thank you for helping make demodatagen better!
