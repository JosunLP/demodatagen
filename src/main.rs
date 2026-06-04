//! `demodatagen` — CLI entry point.
//!
//! All logic lives in the [`demodatagen`] library crate; this binary is a thin
//! wrapper that parses arguments and runs the application.
fn main() {
    std::process::exit(demodatagen::app::run());
}
