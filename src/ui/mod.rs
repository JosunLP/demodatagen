//! Terminal presentation layer: banner, colors, animated progress, summaries.
//!
//! This module owns everything the user *sees* (as opposed to the data they
//! get). It is deliberately the only place that touches [`console`] styling and
//! [`indicatif`] progress bars, so the rest of the codebase stays free of
//! presentation concerns and every string flows through [`crate::i18n`].
//!
//! All status output goes to **stderr** so that `--stdout` keeps a clean,
//! pipeable byte stream on stdout. Color is automatic: it is disabled when
//! output is not a TTY, when `NO_COLOR` is set, or via [`set_colors`].

use crate::i18n::{tr, Language};
use console::{style, Emoji};
use indicatif::{HumanBytes, HumanDuration, ProgressBar, ProgressStyle};
use std::path::Path;
use std::time::Duration;

/// Success marker (falls back to ASCII on terminals without Unicode).
const CHECK: Emoji<'_, '_> = Emoji("✓ ", "+ ");
/// Failure marker.
const CROSS: Emoji<'_, '_> = Emoji("✗ ", "x ");
/// Warning marker.
const WARN: Emoji<'_, '_> = Emoji("⚠ ", "! ");
/// Decorative sparkle used in the banner.
const SPARK: Emoji<'_, '_> = Emoji("✦ ", "* ");

/// How the user asked us to colorize output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorChoice {
    /// Color when stderr is a terminal and `NO_COLOR` is unset (the default).
    #[default]
    Auto,
    /// Always emit ANSI color codes.
    Always,
    /// Never emit color.
    Never,
}

/// Applies a [`ColorChoice`] to the global terminal state.
///
/// [`ColorChoice::Auto`] leaves `console`'s environment-aware detection intact
/// (honoring `NO_COLOR` / `CLICOLOR` / TTY status); the other variants force the
/// decision.
pub fn set_colors(choice: ColorChoice) {
    match choice {
        ColorChoice::Auto => {}
        ColorChoice::Always => console::set_colors_enabled(true),
        ColorChoice::Never => console::set_colors_enabled(false),
    }
}

/// Builds the multi-line startup banner (product name, version, tagline).
///
/// Returned as a `String` (rather than printed) so callers decide the stream.
pub fn banner(lang: Language) -> String {
    let version = env!("CARGO_PKG_VERSION");
    let formats = crate::formats::format_count();
    let locales = crate::data::Locale::all().len();
    let tagline = tr!(
        lang,
        tagline,
        "formats" => formats,
        "locales" => locales,
    );
    let title = format!(
        "{} {}  {}",
        SPARK,
        style("demodatagen").cyan().bold(),
        style(format!("v{version}")).dim(),
    );
    format!("{title}\n{}\n{}", style(tagline).italic(), rule())
}

/// Returns a full-width horizontal rule sized to the terminal (capped at 72).
pub fn rule() -> String {
    let width = console::Term::stderr().size().1.clamp(20, 72) as usize;
    style("─".repeat(width)).dim().to_string()
}

/// Prints the one-line header shown at the start of a generation run (stderr).
pub fn run_header(lang: Language, count: usize, format_name: &str, dir: &Path) {
    let header = tr!(
        lang,
        generating_header,
        "count" => count,
        "format" => style(format_name).cyan().bold(),
        "dir" => style(dir.display()).underlined(),
    );
    eprintln!("{header}");
}

/// Builds a configured, animated [`ProgressBar`] for a run of `count` files.
///
/// A single file gets an indeterminate spinner; multiple files get a bar with
/// throughput and ETA. When `quiet` is set, a hidden bar is returned so callers
/// need no branching. The bar animates on its own via a steady tick.
pub fn progress(count: usize, format_name: &str, lang: Language, quiet: bool) -> ProgressBar {
    if quiet {
        return ProgressBar::hidden();
    }

    let message = tr!(lang, progress_message, "format" => format_name);

    let pb = if count <= 1 {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::with_template("  {spinner:.green} {msg} ({elapsed})")
                .unwrap_or_else(|_| ProgressStyle::default_spinner())
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏✓"),
        );
        pb
    } else {
        let pb = ProgressBar::new(count as u64);
        pb.set_style(
            ProgressStyle::with_template(
                "  {spinner:.green} {wide_bar:.cyan/blue} {pos}/{len} · {percent:>3}% · {per_sec} · ETA {eta}  {msg}",
            )
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏✓")
            .progress_chars("━╸─"),
        );
        pb
    };

    pb.set_message(message);
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

/// Finishes a progress bar with the localized "done" word, clearing animation.
pub fn finish_progress(pb: &ProgressBar, lang: Language) {
    pb.set_message(tr!(lang, progress_done));
    pb.finish();
}

/// Prints the success summary: count, total bytes, elapsed, and output dir.
pub fn summary(lang: Language, count: usize, bytes: u64, elapsed: Duration, dir: &Path) {
    let line = tr!(
        lang,
        summary_success,
        "count" => count,
        "bytes" => HumanBytes(bytes),
        "elapsed" => HumanDuration(elapsed),
    );
    eprintln!("{}{}", style(CHECK).green().bold(), style(line).green());
    eprintln!(
        "  {}",
        style(tr!(lang, summary_location, "dir" => dir.display())).dim()
    );
}

/// Prints the partial-success summary (some files generated, some failed).
pub fn partial(lang: Language, ok: usize, total: usize, errors: usize) {
    let line = tr!(
        lang,
        summary_partial,
        "ok" => ok,
        "total" => total,
        "errors" => errors,
    );
    eprintln!("{}{}", style(WARN).yellow().bold(), style(line).yellow());
}

/// Prints a localized, styled error line to stderr (used on the failure path).
pub fn error_line(message: &str) {
    eprintln!("{}{}", style(CROSS).red().bold(), style(message).red());
}

/// Prints a localized warning line to stderr.
pub fn warn_line(message: &str) {
    eprintln!("{}{}", style(WARN).yellow().bold(), style(message).yellow());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_banner_contains_name_and_version() {
        set_colors(ColorChoice::Never);
        let b = banner(Language::En);
        assert!(b.contains("demodatagen"));
        assert!(b.contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn test_progress_quiet_is_hidden() {
        let pb = progress(10, "JSON", Language::En, true);
        assert!(pb.is_hidden());
    }

    #[test]
    fn test_rule_is_nonempty() {
        assert!(!rule().is_empty());
    }
}
