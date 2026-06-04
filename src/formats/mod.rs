//! Format-specific file generators.
//!
//! Each sub-module implements the [`Generator`](crate::core::generator::Generator)
//! trait for a specific output format. [`get_generator`] acts as a registry,
//! returning the appropriate generator for a given format key.
pub mod bmp;
pub mod csv;
pub mod dll;
pub mod env;
pub mod exe;
pub mod gif;
pub mod gzip;
pub mod html;
pub mod ico;
pub mod ini;
pub mod jpg;
pub mod json;
pub mod jsonl;
pub mod log;
pub mod markdown;
pub mod mp3;
pub mod mp4;
pub mod pdf;
pub mod png;
pub mod sql;
pub mod svg;
pub mod tar;
pub mod tiff;
pub mod toml;
pub mod tsv;
pub mod txt;
pub mod wav;
pub mod webm;
pub mod webp;
pub mod xlsx;
pub mod xml;
pub mod yaml;
pub mod zip;

use crate::core::generator::Generator;

/// Returns the appropriate generator for the given format key.
///
/// Keys are case-insensitive. Returns `None` for unknown formats.
pub fn get_generator(format: &str) -> Option<Box<dyn Generator>> {
    match format.to_lowercase().as_str() {
        // Structured data
        "json" => Some(Box::new(json::JsonGenerator)),
        "jsonl" | "ndjson" => Some(Box::new(jsonl::JsonlGenerator)),
        "yaml" | "yml" => Some(Box::new(yaml::YamlGenerator)),
        "toml" => Some(Box::new(toml::TomlGenerator)),
        "xml" => Some(Box::new(xml::XmlGenerator)),
        "csv" => Some(Box::new(csv::CsvGenerator)),
        "tsv" => Some(Box::new(tsv::TsvGenerator)),
        "sql" => Some(Box::new(sql::SqlGenerator)),
        // Text & config
        "markdown" | "md" => Some(Box::new(markdown::MarkdownGenerator)),
        "html" | "htm" => Some(Box::new(html::HtmlGenerator)),
        "txt" | "text" => Some(Box::new(txt::TxtGenerator)),
        "log" => Some(Box::new(log::LogGenerator)),
        "ini" => Some(Box::new(ini::IniGenerator)),
        "env" => Some(Box::new(env::EnvGenerator)),
        // Images
        "png" => Some(Box::new(png::PngGenerator)),
        "jpg" | "jpeg" => Some(Box::new(jpg::JpgGenerator)),
        "webp" => Some(Box::new(webp::WebpGenerator)),
        "bmp" => Some(Box::new(bmp::BmpGenerator)),
        "tiff" | "tif" => Some(Box::new(tiff::TiffGenerator)),
        "ico" => Some(Box::new(ico::IcoGenerator)),
        "gif" => Some(Box::new(gif::GifGenerator)),
        "svg" => Some(Box::new(svg::SvgGenerator)),
        // Audio & video
        "mp3" => Some(Box::new(mp3::Mp3Generator)),
        "wav" => Some(Box::new(wav::WavGenerator)),
        "mp4" => Some(Box::new(mp4::Mp4Generator)),
        "webm" => Some(Box::new(webm::WebmGenerator)),
        // Documents
        "pdf" => Some(Box::new(pdf::PdfGenerator)),
        "xlsx" => Some(Box::new(xlsx::XlsxGenerator)),
        // Binary & archives
        "exe" => Some(Box::new(exe::ExeGenerator)),
        "dll" => Some(Box::new(dll::DllGenerator)),
        "zip" => Some(Box::new(zip::ZipGenerator)),
        "tar" => Some(Box::new(tar::TarGenerator)),
        "gz" | "gzip" => Some(Box::new(gzip::GzipGenerator)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_formats_registered() {
        let formats = [
            "json", "jsonl", "ndjson", "yaml", "yml", "toml", "xml", "csv", "tsv", "sql",
            "markdown", "md", "html", "htm", "txt", "text", "log", "ini", "env", "png", "jpg",
            "jpeg", "webp", "bmp", "tiff", "tif", "ico", "gif", "svg", "mp3", "wav", "mp4", "webm",
            "pdf", "xlsx", "exe", "dll", "zip", "tar", "gz", "gzip",
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
        assert!(get_generator("YAML").is_some());
    }
}
