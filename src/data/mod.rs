//! Data generation modules providing realistic fake data and text content.
//!
//! These modules are format-agnostic building blocks used by the
//! format-specific generators:
//!
//! - [`faker`] — primitive generators (names, emails, identifiers, …)
//! - [`lorem`] — lorem-ipsum text, sentences, and Markdown documents
//! - [`locale`] — region-specific data pools ([`Locale`])
//! - [`schema`] — the typed schema engine ([`schema::Schema`] / [`schema::FieldValue`])
pub mod faker;
pub mod locale;
pub mod lorem;
pub mod schema;

pub use locale::Locale;
pub use schema::{FieldValue, Record, Schema};
