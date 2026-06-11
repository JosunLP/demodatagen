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
//!
//! # Animations
//!
//! Motion (spinner ticks, the banner reveal, the success flourish) is gated on
//! [`animations_enabled`], which is true only on an attended terminal with
//! colors on. In pipes, CI, `--quiet`, `--color never`, or under tests the gate
//! is false, so output is static and deterministic.

use crate::i18n::{tr, Language};
use console::{style, Emoji, Style};
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

/// 256-color palette for the title gradient (cyan → blue → periwinkle).
const GRADIENT: &[u8] = &[51, 45, 39, 33, 69, 75, 111, 147];

/// Smooth braille spinner frames, ending on a check once finished.
const TICK_CHARS: &str = "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏✓";

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
/// decision for both stdout and stderr.
pub fn set_colors(choice: ColorChoice) {
    match choice {
        ColorChoice::Auto => {}
        ColorChoice::Always => {
            console::set_colors_enabled(true);
            console::set_colors_enabled_stderr(true);
        }
        ColorChoice::Never => {
            console::set_colors_enabled(false);
            console::set_colors_enabled_stderr(false);
        }
    }
}

/// Returns `true` when it is safe and desirable to animate.
///
/// Requires an attended stderr terminal with colors enabled, and honors the
/// `DEMODATAGEN_NO_ANIMATION` and `CI` environment variables as explicit
/// opt-outs. Everything motion-related funnels through this gate, so pipes, CI,
/// and tests stay perfectly static.
pub fn animations_enabled() -> bool {
    console::colors_enabled()
        && console::user_attended_stderr()
        && std::env::var_os("DEMODATAGEN_NO_ANIMATION").is_none()
        && std::env::var_os("CI").is_none()
}

/// Renders `text` with a per-character cyan→blue gradient (bold).
///
/// When colors are disabled the styling collapses to the plain characters, so
/// the result is always safe to print or compare.
pub fn gradient(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len().max(1);
    chars
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let idx = i * GRADIENT.len() / n;
            let color = GRADIENT[idx.min(GRADIENT.len() - 1)];
            Style::new()
                .color256(color)
                .bold()
                .apply_to(c.to_string())
                .to_string()
        })
        .collect()
}

/// Builds the multi-line startup banner (product name, version, tagline).
///
/// Returned as a `String` (rather than printed) so callers decide the stream
/// and whether to animate via [`show_banner`].
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
        style(SPARK).cyan(),
        gradient("demodatagen"),
        style(format!("v{version}")).dim(),
    );
    format!("{title}\n{}\n{}", style(tagline).italic(), rule())
}

/// Prints the banner to stdout, revealing it with a brief animation when the
/// terminal is attended (otherwise printing it at once).
///
/// Used by the informational subcommands (`list`, `presets`, `info`).
pub fn show_banner(lang: Language) {
    let banner = banner(lang);
    if !animations_enabled() {
        println!("{banner}");
        return;
    }
    // Gentle line-by-line cascade for a polished reveal.
    use std::io::Write;
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    for line in banner.lines() {
        let _ = writeln!(handle, "{line}");
        let _ = handle.flush();
        std::thread::sleep(Duration::from_millis(45));
    }
}

/// Returns a full-width horizontal rule sized to the terminal (capped at 72).
pub fn rule() -> String {
    let width = console::Term::stderr().size().1.clamp(20, 72) as usize;
    style("─".repeat(width)).dim().to_string()
}

/// Draws a rounded box around `title` and `rows`, sized to the widest line.
///
/// Falls back to ASCII corners when the terminal lacks Unicode. Used by the
/// `info` panel; returns a `String` so the caller picks the stream.
pub fn boxed(title: &str, rows: &[String]) -> String {
    let unicode = console::Term::stdout().features().wants_emoji() || cfg!(not(windows));
    let (tl, tr_c, bl, br, h, v) = if unicode {
        ("╭", "╮", "╰", "╯", "─", "│")
    } else {
        ("+", "+", "+", "+", "-", "|")
    };
    // Width is computed on the *visible* text (ANSI codes stripped).
    let visible_len = |s: &str| console::measure_text_width(s);
    let title_w = visible_len(title);
    let content_w = rows.iter().map(|r| visible_len(r)).max().unwrap_or(0);
    let inner = content_w.max(title_w + 2).max(10);

    let mut out = String::new();
    // Top border with embedded title: ╭─ Title ───╮ (sized to match the rows,
    // whose visible width is `inner + 4`: "│ " + inner + " │").
    let title_styled = style(title).cyan().bold();
    let dashes = inner.saturating_sub(title_w + 1);
    out.push_str(&style(format!("{tl}{h} ")).dim().to_string());
    out.push_str(&title_styled.to_string());
    out.push(' ');
    out.push_str(
        &style(format!("{}{tr_c}", h.repeat(dashes)))
            .dim()
            .to_string(),
    );
    out.push('\n');
    for row in rows {
        let pad = inner.saturating_sub(visible_len(row));
        out.push_str(&style(format!("{v} ")).dim().to_string());
        out.push_str(row);
        out.push_str(&" ".repeat(pad));
        out.push_str(&style(format!(" {v}")).dim().to_string());
        out.push('\n');
    }
    out.push_str(
        &style(format!("{bl}{}{br}", h.repeat(inner + 2)))
            .dim()
            .to_string(),
    );
    out
}

/// Prints the one-line header shown at the start of a generation run (stderr).
///
/// Styles are marked `.for_stderr()` so their color follows the *stderr* TTY /
/// color state, not stdout's — otherwise piping stdout (e.g. `… | less`) would
/// strip color from these status lines on a perfectly capable terminal.
pub fn run_header(lang: Language, count: usize, format_name: &str, dir: &Path) {
    let header = tr!(
        lang,
        generating_header,
        "count" => count,
        "format" => style(format_name).cyan().bold().for_stderr(),
        "dir" => style(dir.display()).underlined().for_stderr(),
    );
    eprintln!("{} {header}", style(SPARK).cyan().for_stderr());
}

/// Builds a configured, animated [`ProgressBar`] for a run of `count` files.
///
/// A single file gets an indeterminate spinner; multiple files get a bar with
/// throughput and ETA. When `quiet` is set, a hidden bar is returned so callers
/// need no branching. On an attended terminal the bar self-animates via a steady
/// tick; in pipes/CI (see [`animations_enabled`]) it only redraws on progress,
/// so it never spams non-interactive logs with spinner frames.
pub fn progress(count: usize, format_name: &str, lang: Language, quiet: bool) -> ProgressBar {
    if quiet {
        return ProgressBar::hidden();
    }

    let message = tr!(lang, progress_message, "format" => format_name);

    let pb = if count <= 1 {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::with_template("  {spinner:.cyan} {msg} ({elapsed})")
                .unwrap_or_else(|_| ProgressStyle::default_spinner())
                .tick_chars(TICK_CHARS),
        );
        pb
    } else {
        let pb = ProgressBar::new(count as u64);
        pb.set_style(
            ProgressStyle::with_template(
                "  {spinner:.cyan} {wide_bar:.cyan/blue} {pos}/{len} · {percent:>3}% · {per_sec} · ETA {eta}  {msg}",
            )
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .tick_chars(TICK_CHARS)
            .progress_chars("━╸─"),
        );
        pb
    };

    pb.set_message(message);
    if animations_enabled() {
        pb.enable_steady_tick(Duration::from_millis(80));
    }
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
    eprintln!(
        "{}{}",
        style(CHECK).green().bold().for_stderr(),
        style(line).green().for_stderr()
    );
    eprintln!(
        "  {}",
        style(tr!(lang, summary_location, "dir" => dir.display()))
            .dim()
            .for_stderr()
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
    eprintln!(
        "{}{}",
        style(WARN).yellow().bold().for_stderr(),
        style(line).yellow().for_stderr()
    );
}

/// Prints a localized, styled error line to stderr (used on the failure path).
pub fn error_line(message: &str) {
    eprintln!(
        "{}{}",
        style(CROSS).red().bold().for_stderr(),
        style(message).red().for_stderr()
    );
}

/// Prints a localized warning line to stderr.
pub fn warn_line(message: &str) {
    eprintln!(
        "{}{}",
        style(WARN).yellow().bold().for_stderr(),
        style(message).yellow().for_stderr()
    );
}

/// Prints a dimmed, indented hint/tip line to stderr.
pub fn hint_line(message: &str) {
    eprintln!("  {}", style(message).dim().for_stderr());
}

/// An indeterminate spinner for a single blocking operation (e.g. a network
/// call), with a localized-friendly message.
///
/// Hidden when animations are off, so it leaves no residue in pipes, CI, or
/// tests. Use [`Spinner::start`], optionally [`Spinner::set_message`], then one
/// of the finishers.
pub struct Spinner {
    pb: ProgressBar,
}

impl Spinner {
    /// Starts a spinner on stderr with the given message.
    pub fn start(message: impl Into<String>) -> Spinner {
        let pb = if animations_enabled() {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::with_template("  {spinner:.cyan} {msg}")
                    .unwrap_or_else(|_| ProgressStyle::default_spinner())
                    .tick_chars(TICK_CHARS),
            );
            pb.enable_steady_tick(Duration::from_millis(80));
            pb
        } else {
            ProgressBar::hidden()
        };
        pb.set_message(message.into());
        Spinner { pb }
    }

    /// Updates the spinner message in place.
    pub fn set_message(&self, message: impl Into<String>) {
        self.pb.set_message(message.into());
    }

    /// Clears the spinner, leaving no trace.
    pub fn finish_and_clear(&self) {
        self.pb.finish_and_clear();
    }
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

    #[test]
    fn test_gradient_preserves_text_when_uncolored() {
        set_colors(ColorChoice::Never);
        // With colors off, the gradient must collapse to the raw characters.
        assert_eq!(gradient("demodatagen"), "demodatagen");
    }

    #[test]
    fn test_boxed_contains_title_and_rows() {
        set_colors(ColorChoice::Never);
        let b = boxed(
            "Info",
            &["Version: 0.5.0".to_string(), "Formats: 33".to_string()],
        );
        assert!(b.contains("Info"));
        assert!(b.contains("Version: 0.5.0"));
        assert!(b.contains("Formats: 33"));
        // Four lines: top border, two rows, bottom border.
        assert_eq!(b.lines().count(), 4);
    }

    #[test]
    fn test_spinner_hidden_without_tty() {
        // In the test harness stderr is not attended, so the spinner is hidden.
        let sp = Spinner::start("working");
        assert!(sp.pb.is_hidden());
        sp.finish_and_clear();
    }
}
