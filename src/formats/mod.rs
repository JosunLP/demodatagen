/// Format-specific file generators.
///
/// Each sub-module implements the `Generator` trait for a specific output format.
/// The `get_generator` function acts as a registry, returning the appropriate
/// generator for a given format name.

pub mod csv;
pub mod dll;
pub mod exe;
pub mod gif;
pub mod jpg;
pub mod json;
pub mod markdown;
pub mod mp3;
pub mod mp4;
pub mod png;
pub mod txt;
pub mod webm;
pub mod webp;
pub mod xml;
pub mod zip;

use crate::core::generator::Generator;

/// Returns the appropriate generator for the given format name.
///
/// Format names are case-insensitive. Returns `None` for unknown formats.
pub fn get_generator(format: &str) -> Option<Box<dyn Generator>> {
    match format.to_lowercase().as_str() {
        "json" => Some(Box::new(json::JsonGenerator)),
        "xml" => Some(Box::new(xml::XmlGenerator)),
        "csv" => Some(Box::new(csv::CsvGenerator)),
        "markdown" | "md" => Some(Box::new(markdown::MarkdownGenerator)),
        "txt" | "text" => Some(Box::new(txt::TxtGenerator)),
        "png" => Some(Box::new(png::PngGenerator)),
        "jpg" | "jpeg" => Some(Box::new(jpg::JpgGenerator)),
        "webp" => Some(Box::new(webp::WebpGenerator)),
        "gif" => Some(Box::new(gif::GifGenerator)),
        "mp3" => Some(Box::new(mp3::Mp3Generator)),
        "mp4" => Some(Box::new(mp4::Mp4Generator)),
        "webm" => Some(Box::new(webm::WebmGenerator)),
        "exe" => Some(Box::new(exe::ExeGenerator)),
        "dll" => Some(Box::new(dll::DllGenerator)),
        "zip" => Some(Box::new(zip::ZipGenerator)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_formats_registered() {
        let formats = [
            "json", "xml", "csv", "markdown", "md", "txt", "text",
            "png", "jpg", "jpeg", "webp", "gif", "mp3", "mp4", "webm",
            "exe", "dll", "zip",
        ];
        for fmt in formats {
            assert!(
                get_generator(fmt).is_some(),
                "Generator not found for format: {fmt}"
            );
        }
    }

    #[test]
    fn test_unknown_format() {
        assert!(get_generator("unknown").is_none());
    }

    #[test]
    fn test_format_case_insensitive() {
        assert!(get_generator("JSON").is_some());
        assert!(get_generator("Png").is_some());
    }
}
