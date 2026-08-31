/// Lorem ipsum and general text generation utilities.
///
/// Provides functions for generating paragraphs, sentences, and words
/// of placeholder text using a procedural algorithm with a seeded RNG.
use rand::{Rng, RngExt};

/// Common lorem ipsum words used as building blocks for text generation.
const LOREM_WORDS: &[&str] = &[
    "lorem",
    "ipsum",
    "dolor",
    "sit",
    "amet",
    "consectetur",
    "adipiscing",
    "elit",
    "sed",
    "do",
    "eiusmod",
    "tempor",
    "incididunt",
    "ut",
    "labore",
    "et",
    "dolore",
    "magna",
    "aliqua",
    "enim",
    "ad",
    "minim",
    "veniam",
    "quis",
    "nostrud",
    "exercitation",
    "ullamco",
    "laboris",
    "nisi",
    "aliquip",
    "ex",
    "ea",
    "commodo",
    "consequat",
    "duis",
    "aute",
    "irure",
    "in",
    "reprehenderit",
    "voluptate",
    "velit",
    "esse",
    "cillum",
    "fugiat",
    "nulla",
    "pariatur",
    "excepteur",
    "sint",
    "occaecat",
    "cupidatat",
    "non",
    "proident",
    "sunt",
    "culpa",
    "qui",
    "officia",
    "deserunt",
    "mollit",
    "anim",
    "id",
    "est",
    "laborum",
    "at",
    "vero",
    "eos",
    "accusamus",
    "iusto",
    "odio",
    "dignissimos",
    "ducimus",
    "blanditiis",
    "praesentium",
    "voluptatum",
    "deleniti",
    "atque",
    "corrupti",
    "quos",
    "dolores",
    "quas",
    "molestias",
    "excepturi",
    "obcaecati",
    "cupiditate",
    "provident",
    "similique",
    "mollitia",
    "animi",
    "sapiente",
    "totam",
    "rem",
    "aperiam",
    "inventore",
    "veritatis",
    "quasi",
    "architecto",
    "beatae",
    "vitae",
    "dicta",
    "explicabo",
    "nemo",
    "ipsam",
    "voluptatem",
    "quia",
    "voluptas",
    "aspernatur",
    "aut",
    "odit",
    "fugit",
    "consequuntur",
    "magni",
    "dolorem",
    "porro",
    "quisquam",
    "nihil",
    "impedit",
    "quo",
    "minus",
    "quod",
    "maxime",
    "placeat",
    "facere",
    "possimus",
    "omnis",
    "assumenda",
    "repellendus",
    "temporibus",
    "quibusdam",
    "illum",
    "soluta",
    "nobis",
    "eligendi",
    "optio",
    "cumque",
    "recusandae",
    "itaque",
    "earum",
    "rerum",
    "hic",
    "tenetur",
];

/// Heading words for Markdown heading generation.
const HEADING_WORDS: &[&str] = &[
    "Introduction",
    "Overview",
    "Getting",
    "Started",
    "Configuration",
    "Setup",
    "Installation",
    "Usage",
    "Examples",
    "Advanced",
    "Topics",
    "Performance",
    "Security",
    "Testing",
    "Deployment",
    "Architecture",
    "Design",
    "Patterns",
    "Best",
    "Practices",
    "Troubleshooting",
    "FAQ",
    "Reference",
    "API",
    "Documentation",
    "Guide",
    "Tutorial",
    "Quick",
    "Start",
    "Deep",
    "Dive",
    "Analysis",
    "Summary",
    "Conclusion",
    "Results",
    "Methodology",
    "Background",
    "Discussion",
    "Implementation",
    "Optimization",
    "Evaluation",
    "Comparison",
];

/// Generates a single random word.
pub fn word<R: Rng>(rng: &mut R) -> &'static str {
    LOREM_WORDS[rng.random_range(0..LOREM_WORDS.len())]
}

/// Generates a sequence of random words joined by spaces.
pub fn words<R: Rng>(rng: &mut R, count: usize) -> String {
    (0..count).map(|_| word(rng)).collect::<Vec<_>>().join(" ")
}

/// Generates a single sentence with the given approximate word count.
///
/// The first word is capitalized and the sentence ends with a period.
pub fn sentence<R: Rng>(rng: &mut R, word_count: usize) -> String {
    let count = if word_count == 0 {
        rng.random_range(5..15)
    } else {
        word_count
    };

    let mut s = words(rng, count);
    // Capitalize first letter
    if let Some(first) = s.get_mut(..1) {
        first.make_ascii_uppercase();
    }
    s.push('.');
    s
}

/// Generates a paragraph consisting of multiple sentences.
///
/// Each paragraph has between 3 and 8 sentences with varying lengths.
pub fn paragraph<R: Rng>(rng: &mut R) -> String {
    let sentence_count = rng.random_range(3..=8);
    (0..sentence_count)
        .map(|_| {
            let word_count = rng.random_range(5..20);
            sentence(rng, word_count)
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Generates multiple paragraphs separated by double newlines.
pub fn paragraphs<R: Rng>(rng: &mut R, count: usize) -> String {
    (0..count)
        .map(|_| paragraph(rng))
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Generates a random heading suitable for Markdown.
///
/// Returns 2-4 words that look like a section heading.
pub fn heading<R: Rng>(rng: &mut R) -> String {
    let word_count = rng.random_range(2..=4);
    (0..word_count)
        .map(|_| HEADING_WORDS[rng.random_range(0..HEADING_WORDS.len())])
        .collect::<Vec<_>>()
        .join(" ")
}

/// Generates a complete plain text document.
///
/// Produces the specified number of paragraphs of lorem ipsum text.
pub fn plain_text<R: Rng>(rng: &mut R, paragraph_count: usize) -> String {
    paragraphs(rng, paragraph_count)
}

/// Generates a Markdown document with headings and paragraphs.
///
/// The document has a title (H1) followed by sections with headings (H2/H3)
/// and paragraphs of content.
pub fn markdown_document<R: Rng>(
    rng: &mut R,
    heading_count: usize,
    paragraph_count: usize,
) -> String {
    let mut doc = String::new();

    // Title
    doc.push_str(&format!("# {}\n\n", heading(rng)));

    // Introduction paragraph
    doc.push_str(&paragraph(rng));
    doc.push_str("\n\n");

    let paragraphs_per_section = if heading_count > 0 {
        (paragraph_count.max(1)) / heading_count.max(1)
    } else {
        paragraph_count
    };

    for i in 0..heading_count {
        // Alternate between H2 and H3
        let level = if i % 3 == 0 { "##" } else { "###" };
        doc.push_str(&format!("{} {}\n\n", level, heading(rng)));

        let para_count = paragraphs_per_section.max(1);
        for _ in 0..para_count {
            doc.push_str(&paragraph(rng));
            doc.push_str("\n\n");
        }

        // Occasionally add a bullet list
        if rng.random_bool(0.3) {
            let list_items = rng.random_range(3..=6);
            for _ in 0..list_items {
                let word_count = rng.random_range(4..10);
                doc.push_str(&format!("- {}\n", sentence(rng, word_count)));
            }
            doc.push('\n');
        }

        // Occasionally add a code block
        if rng.random_bool(0.2) {
            doc.push_str("```\n");
            doc.push_str(&format!("let data = {};\n", words(rng, 3)));
            doc.push_str(&format!("println!(\"{{}}\", {});\n", word(rng)));
            doc.push_str("```\n\n");
        }
    }

    // If no headings, just output paragraphs
    if heading_count == 0 {
        for _ in 0..paragraph_count {
            doc.push_str(&paragraph(rng));
            doc.push_str("\n\n");
        }
    }

    doc.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn test_rng() -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(42)
    }

    #[test]
    fn test_word_not_empty() {
        let mut rng = test_rng();
        let w = word(&mut rng);
        assert!(!w.is_empty());
    }

    #[test]
    fn test_words_count() {
        let mut rng = test_rng();
        let w = words(&mut rng, 5);
        let count = w.split_whitespace().count();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_sentence_ends_with_period() {
        let mut rng = test_rng();
        let s = sentence(&mut rng, 8);
        assert!(s.ends_with('.'));
    }

    #[test]
    fn test_sentence_starts_uppercase() {
        let mut rng = test_rng();
        let s = sentence(&mut rng, 8);
        let first_char = s.chars().next().unwrap();
        assert!(first_char.is_uppercase());
    }

    #[test]
    fn test_paragraph_non_empty() {
        let mut rng = test_rng();
        let p = paragraph(&mut rng);
        assert!(!p.is_empty());
        assert!(p.contains('.'));
    }

    #[test]
    fn test_paragraphs_separated() {
        let mut rng = test_rng();
        let p = paragraphs(&mut rng, 3);
        let count = p.split("\n\n").count();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_heading_non_empty() {
        let mut rng = test_rng();
        let h = heading(&mut rng);
        assert!(!h.is_empty());
        let word_count = h.split_whitespace().count();
        assert!((2..=4).contains(&word_count));
    }

    #[test]
    fn test_markdown_document_has_title() {
        let mut rng = test_rng();
        let doc = markdown_document(&mut rng, 3, 6);
        assert!(doc.starts_with("# "));
    }

    #[test]
    fn test_plain_text_paragraphs() {
        let mut rng = test_rng();
        let text = plain_text(&mut rng, 4);
        let para_count = text.split("\n\n").count();
        assert_eq!(para_count, 4);
    }

    #[test]
    fn test_deterministic_output() {
        let mut rng1 = test_rng();
        let mut rng2 = test_rng();
        let text1 = paragraph(&mut rng1);
        let text2 = paragraph(&mut rng2);
        assert_eq!(text1, text2);
    }
}
