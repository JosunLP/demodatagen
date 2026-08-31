//! PDF document generator.
//!
//! Produces genuinely valid PDF 1.4 documents with a title, headings, and
//! paragraphs of lorem-ipsum text, laid out across as many pages as needed.
//!
//! The PDF is written by hand (no external PDF dependency): a small object graph
//! — catalog, page tree, a Helvetica font, and one content stream per page —
//! followed by a correct cross-reference table. This keeps the dependency tree
//! light while emitting files that open in any standard PDF viewer.
use crate::core::generator::{FormatOptions, Generator, GeneratorConfig};
use crate::data::lorem;
use crate::error::{GenResult, GenerationError};
use rand::Rng;

/// Generator for PDF documents.
pub struct PdfGenerator;

// US-Letter page geometry, in PDF points (1/72 inch).
const PAGE_W: f32 = 612.0;
const PAGE_H: f32 = 792.0;
const MARGIN: f32 = 72.0;
const BODY_SIZE: f32 = 11.0;
const HEADING_SIZE: f32 = 16.0;
const TITLE_SIZE: f32 = 22.0;
const LINE_GAP: f32 = 4.0;
/// Characters per line at body size (approximate, for word wrapping).
const WRAP_COLS: usize = 90;

/// A single laid-out text line: its font size and content.
struct Line {
    size: f32,
    text: String,
}

impl Generator for PdfGenerator {
    fn format_name(&self) -> &str {
        "PDF"
    }

    fn file_extension(&self) -> &str {
        "pdf"
    }

    fn generate(&self, config: &mut GeneratorConfig) -> GenResult<Vec<u8>> {
        let (paragraphs, headings) = match &config.format_options {
            FormatOptions::Markdown {
                paragraphs,
                headings,
            } => (*paragraphs, *headings),
            _ => {
                return Err(GenerationError::InvalidConfig(
                    "PDF generator requires Markdown options".to_string(),
                ));
            }
        };

        let lines = build_lines(&mut config.rng, paragraphs, headings);
        let pages = paginate(lines);
        Ok(render_pdf(&pages))
    }
}

/// Builds the flat list of lines (title, headings, wrapped paragraphs).
fn build_lines<R: Rng>(rng: &mut R, paragraphs: usize, headings: usize) -> Vec<Line> {
    let mut lines = Vec::new();
    lines.push(Line {
        size: TITLE_SIZE,
        text: lorem::heading(rng),
    });
    lines.push(Line {
        size: 0.0,
        text: String::new(),
    }); // spacer

    let per_section = if headings > 0 {
        paragraphs.max(1) / headings.max(1)
    } else {
        paragraphs
    };

    let emit_paragraph = |lines: &mut Vec<Line>, rng: &mut R| {
        for chunk in wrap(&lorem::paragraph(rng), WRAP_COLS) {
            lines.push(Line {
                size: BODY_SIZE,
                text: chunk,
            });
        }
        lines.push(Line {
            size: 0.0,
            text: String::new(),
        });
    };

    if headings == 0 {
        for _ in 0..paragraphs {
            emit_paragraph(&mut lines, rng);
        }
    } else {
        for _ in 0..headings {
            lines.push(Line {
                size: HEADING_SIZE,
                text: lorem::heading(rng),
            });
            for _ in 0..per_section.max(1) {
                emit_paragraph(&mut lines, rng);
            }
        }
    }
    lines
}

/// Greedy word-wrap to roughly `cols` characters per line.
fn wrap(text: &str, cols: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if !current.is_empty() && current.len() + 1 + word.len() > cols {
            out.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        out.push(current);
    }
    if out.is_empty() {
        out.push(String::new());
    }
    out
}

/// Splits the line list into pages based on vertical space.
fn paginate(lines: Vec<Line>) -> Vec<Vec<Line>> {
    let mut pages = Vec::new();
    let mut page: Vec<Line> = Vec::new();
    let mut y = PAGE_H - MARGIN;
    for line in lines {
        let advance = if line.size == 0.0 {
            BODY_SIZE * 0.6
        } else {
            line.size + LINE_GAP
        };
        if y - advance < MARGIN && !page.is_empty() {
            pages.push(std::mem::take(&mut page));
            y = PAGE_H - MARGIN;
        }
        y -= advance;
        page.push(line);
    }
    if !page.is_empty() {
        pages.push(page);
    }
    if pages.is_empty() {
        pages.push(Vec::new());
    }
    pages
}

/// Escapes a string for inclusion in a PDF literal `(...)` and drops
/// non-printable / non-ASCII bytes (Helvetica/WinAnsi text stays simple).
fn escape_pdf(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '(' => out.push_str("\\("),
            ')' => out.push_str("\\)"),
            c if c.is_ascii_graphic() || c == ' ' => out.push(c),
            _ => {}
        }
    }
    out
}

/// Builds the content stream for one page.
fn page_content(lines: &[Line]) -> String {
    let mut s = String::new();
    let mut y = PAGE_H - MARGIN;
    for line in lines {
        let advance = if line.size == 0.0 {
            BODY_SIZE * 0.6
        } else {
            line.size + LINE_GAP
        };
        y -= advance;
        if line.size > 0.0 && !line.text.is_empty() {
            s.push_str(&format!(
                "BT /F1 {} Tf 1 0 0 1 {} {:.2} Tm ({}) Tj ET\n",
                line.size as i32,
                MARGIN as i32,
                y,
                escape_pdf(&line.text)
            ));
        }
    }
    s
}

/// Renders the full PDF byte stream from laid-out pages.
fn render_pdf(pages: &[Vec<Line>]) -> Vec<u8> {
    let page_count = pages.len();
    // Object numbering: 1=Catalog, 2=Pages, 3=Font, then per page a Page and a
    // Contents object.
    let first_page_obj = 4;
    let mut objects: Vec<String> = Vec::new();

    objects.push("<< /Type /Catalog /Pages 2 0 R >>".to_string());

    let kids: Vec<String> = (0..page_count)
        .map(|i| format!("{} 0 R", first_page_obj + i * 2))
        .collect();
    objects.push(format!(
        "<< /Type /Pages /Kids [{}] /Count {} >>",
        kids.join(" "),
        page_count
    ));

    objects.push(
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>"
            .to_string(),
    );

    let mut contents: Vec<String> = Vec::with_capacity(page_count);
    for (i, page) in pages.iter().enumerate() {
        let page_obj = first_page_obj + i * 2;
        let content_obj = page_obj + 1;
        objects.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] /Contents {} 0 R \
             /Resources << /Font << /F1 3 0 R >> >> >>",
            PAGE_W as i32, PAGE_H as i32, content_obj
        ));
        let stream = page_content(page);
        objects.push(format!(
            "<< /Length {} >>\nstream\n{}endstream",
            stream.len(),
            stream
        ));
        contents.push(stream);
    }

    // Serialize objects with byte-offset tracking for the xref table.
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");
    let mut offsets = Vec::with_capacity(objects.len());
    for (i, body) in objects.iter().enumerate() {
        offsets.push(buf.len());
        buf.extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", i + 1, body).as_bytes());
    }

    let xref_offset = buf.len();
    let total = objects.len() + 1;
    buf.extend_from_slice(format!("xref\n0 {total}\n").as_bytes());
    buf.extend_from_slice(b"0000000000 65535 f \n");
    for off in &offsets {
        buf.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
    }
    buf.extend_from_slice(
        format!("trailer\n<< /Size {total} /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n")
            .as_bytes(),
    );
    buf
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generator::test_support::markdown_config;

    #[test]
    fn test_pdf_header_and_trailer() {
        let mut config = markdown_config(6, 3);
        let result = PdfGenerator.generate(&mut config).unwrap();
        assert_eq!(&result[0..5], b"%PDF-");
        let tail = String::from_utf8_lossy(&result[result.len().saturating_sub(8)..]);
        assert!(tail.contains("%%EOF"));
    }

    #[test]
    fn test_pdf_structure() {
        let mut config = markdown_config(4, 2);
        let result = PdfGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8_lossy(&result);
        assert!(text.contains("/Type /Catalog"));
        assert!(text.contains("/Type /Pages"));
        assert!(text.contains("/BaseFont /Helvetica"));
        assert!(text.contains("startxref"));
        assert!(text.contains("xref"));
    }

    #[test]
    fn test_pdf_multipage() {
        // Lots of content should span multiple pages.
        let mut config = markdown_config(60, 8);
        let result = PdfGenerator.generate(&mut config).unwrap();
        let text = String::from_utf8_lossy(&result);
        let page_count = text.matches("/Type /Page ").count();
        assert!(page_count >= 2, "expected multiple pages, got {page_count}");
    }

    #[test]
    fn test_pdf_escaping() {
        assert_eq!(escape_pdf("a(b)c\\d"), "a\\(b\\)c\\\\d");
    }

    #[test]
    fn test_wrap_respects_width() {
        let long = "word ".repeat(50);
        for line in wrap(&long, 20) {
            assert!(line.len() <= 24, "line too long: {}", line.len());
        }
    }

    #[test]
    fn test_pdf_deterministic() {
        let mut a = markdown_config(5, 3);
        let mut b = markdown_config(5, 3);
        assert_eq!(
            PdfGenerator.generate(&mut a).unwrap(),
            PdfGenerator.generate(&mut b).unwrap()
        );
    }
}
