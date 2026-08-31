//! Integration tests for the formats and features added in 0.2.0.
//!
//! Each test drives the real binary end-to-end and asserts on the produced
//! file's magic bytes or structure.
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Builds a [`Command`] pointing at our binary crate.
fn cmd() -> Command {
    #[allow(deprecated)]
    {
        Command::cargo_bin("demodatagen").expect("binary should be built")
    }
}

/// Runs the binary into a fresh temp dir with a fixed seed and returns the
/// bytes of the single produced file.
fn generate(args: &[&str]) -> Vec<u8> {
    let tmp = TempDir::new().unwrap();
    let mut full = vec![
        "-o",
        tmp.path().to_str().unwrap(),
        "-c",
        "1",
        "-s",
        "7",
        "--quiet",
    ];
    full.extend_from_slice(args);
    cmd().args(&full).assert().success();

    let files: Vec<PathBuf> = fs::read_dir(tmp.path())
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .collect();
    assert_eq!(files.len(), 1, "expected exactly one output file");
    fs::read(&files[0]).unwrap()
}

fn generate_text(args: &[&str]) -> String {
    String::from_utf8(generate(args)).unwrap()
}

// ── Structured formats ──────────────────────────────────────────────

#[test]
fn test_yaml() {
    let text = generate_text(&["yaml", "--rows", "3"]);
    let parsed: serde_json::Value = serde_yaml_ng::from_str(&text).unwrap();
    assert_eq!(parsed.as_array().unwrap().len(), 3);
}

#[test]
fn test_toml() {
    let text = generate_text(&["toml", "--rows", "3"]);
    let parsed: toml::Value = toml::from_str(&text).unwrap();
    assert_eq!(parsed.get("records").unwrap().as_array().unwrap().len(), 3);
}

#[test]
fn test_jsonl() {
    let text = generate_text(&["jsonl", "--rows", "4"]);
    assert_eq!(text.lines().count(), 4);
    for line in text.lines() {
        let v: serde_json::Value = serde_json::from_str(line).unwrap();
        assert!(v.is_object());
    }
}

#[test]
fn test_sql() {
    let text = generate_text(&["sql", "--rows", "3", "--table", "people"]);
    assert!(text.contains("CREATE TABLE \"people\""));
    assert!(text.contains("INSERT INTO \"people\""));
}

#[test]
fn test_tsv() {
    let text = generate_text(&["tsv", "--rows", "3", "--schema", "a:int,b:int,c:int"]);
    assert!(text.lines().next().unwrap().contains('\t'));
    assert_eq!(text.lines().count(), 4);
}

#[test]
fn test_csv_custom_delimiter() {
    let text = generate_text(&[
        "csv",
        "--rows",
        "2",
        "--schema",
        "a:int,b:int",
        "--delimiter",
        ";",
    ]);
    assert!(text.lines().next().unwrap().contains(';'));
}

#[test]
fn test_xml_custom_tags() {
    let text = generate_text(&[
        "xml",
        "--rows",
        "2",
        "--root",
        "people",
        "--row-tag",
        "person",
    ]);
    assert!(text.contains("<people>"));
    assert!(text.contains("<person>"));
}

// ── Text & config formats ───────────────────────────────────────────

#[test]
fn test_html() {
    let text = generate_text(&["html", "--paragraphs", "4", "--headings", "2"]);
    assert!(text.starts_with("<!DOCTYPE html>"));
    assert!(text.contains("</html>"));
}

#[test]
fn test_log_apache() {
    let text = generate_text(&["log", "--lines", "10", "--style", "apache"]);
    assert_eq!(text.lines().count(), 10);
    assert!(text.contains("HTTP/1.1"));
}

#[test]
fn test_log_json() {
    let text = generate_text(&["log", "--lines", "5", "--style", "json"]);
    for line in text.lines() {
        let _: serde_json::Value = serde_json::from_str(line).unwrap();
    }
}

#[test]
fn test_ini() {
    let text = generate_text(&["ini", "--sections", "2", "--keys", "3"]);
    assert_eq!(text.lines().filter(|l| l.starts_with('[')).count(), 2);
}

#[test]
fn test_env() {
    let text = generate_text(&["env", "--keys", "5"]);
    assert_eq!(text.lines().count(), 5);
    assert!(text.lines().all(|l| l.contains('=')));
}

// ── Image formats ───────────────────────────────────────────────────

#[test]
fn test_bmp() {
    let data = generate(&["bmp", "--width", "16", "--height", "16"]);
    assert_eq!(&data[0..2], b"BM");
}

#[test]
fn test_tiff() {
    let data = generate(&["tiff", "--width", "16", "--height", "16"]);
    assert!(&data[0..2] == b"II" || &data[0..2] == b"MM");
}

#[test]
fn test_ico() {
    let data = generate(&["ico", "--size", "32"]);
    assert_eq!(&data[0..4], &[0x00, 0x00, 0x01, 0x00]);
}

#[test]
fn test_svg() {
    let text = generate_text(&["svg", "--width", "100", "--height", "100", "--shapes", "5"]);
    assert!(text.contains("<svg"));
    assert!(text.contains("</svg>"));
}

// ── Audio / documents / archives ────────────────────────────────────

#[test]
fn test_wav() {
    let data = generate(&["wav", "--duration", "0.3"]);
    assert_eq!(&data[0..4], b"RIFF");
    assert_eq!(&data[8..12], b"WAVE");
}

#[test]
fn test_pdf() {
    let data = generate(&["pdf", "--paragraphs", "5", "--headings", "2"]);
    assert_eq!(&data[0..5], b"%PDF-");
    assert!(String::from_utf8_lossy(&data).contains("%%EOF"));
}

#[test]
fn test_xlsx() {
    let data = generate(&["xlsx", "--rows", "5"]);
    assert_eq!(&data[0..2], b"PK");
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(data)).unwrap();
    assert!(archive.by_name("xl/workbook.xml").is_ok());
}

#[test]
fn test_tar() {
    let data = generate(&["tar", "--files", "3", "--contained-format", "txt"]);
    let mut archive = tar::Archive::new(std::io::Cursor::new(data));
    assert_eq!(archive.entries().unwrap().count(), 3);
}

#[test]
fn test_gzip() {
    use flate2::read::GzDecoder;
    use std::io::Read;
    let data = generate(&["gzip", "--paragraphs", "3"]);
    assert_eq!(&data[0..2], &[0x1f, 0x8b]);
    let mut text = String::new();
    GzDecoder::new(&data[..]).read_to_string(&mut text).unwrap();
    assert!(!text.is_empty());
}

// ── Schema engine features through the CLI ──────────────────────────

#[test]
fn test_schema_int_range_and_enum() {
    let text = generate_text(&[
        "jsonl",
        "--rows",
        "20",
        "--schema",
        "n:int(5..9),s:enum(a,b,c)",
    ]);
    for line in text.lines() {
        let v: serde_json::Value = serde_json::from_str(line).unwrap();
        let n = v["n"].as_i64().unwrap();
        assert!((5..=9).contains(&n));
        let s = v["s"].as_str().unwrap();
        assert!(["a", "b", "c"].contains(&s));
    }
}

#[test]
fn test_schema_sequence_increments() {
    let text = generate_text(&["jsonl", "--rows", "3", "--schema", "id:sequence(10)"]);
    let ids: Vec<i64> = text
        .lines()
        .map(|l| {
            serde_json::from_str::<serde_json::Value>(l).unwrap()["id"]
                .as_i64()
                .unwrap()
        })
        .collect();
    assert_eq!(ids, vec![10, 11, 12]);
}

#[test]
fn test_schema_nullable() {
    let text = generate_text(&["jsonl", "--rows", "5", "--schema", "x:int?1.0"]);
    for line in text.lines() {
        let v: serde_json::Value = serde_json::from_str(line).unwrap();
        assert!(v["x"].is_null());
    }
}

// ── Locale ──────────────────────────────────────────────────────────

#[test]
fn test_locale_de_produces_german_data() {
    let text = generate_text(&[
        "--locale",
        "de_de",
        "csv",
        "--rows",
        "30",
        "--schema",
        "city:city,country:country",
    ]);
    assert!(text.contains("Deutschland"));
}

// ── Meta subcommands & stdout ───────────────────────────────────────

#[test]
fn test_list_command() {
    cmd()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Structured data"))
        .stdout(predicate::str::contains("pdf"));
}

#[test]
fn test_completions_bash() {
    cmd()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_demodatagen"));
}

#[test]
fn test_stdout_flag() {
    cmd()
        .args([
            "-s",
            "7",
            "--quiet",
            "--stdout",
            "json",
            "--rows",
            "2",
            "--schema",
            "id:sequence",
        ])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("["));
}

#[test]
fn test_invalid_locale_fails() {
    cmd()
        .args(["--locale", "xx_yy", "--quiet", "txt"])
        .assert()
        .failure();
}

// ---- Formats added in 0.6.0 -------------------------------------------------

#[test]
fn test_vcf() {
    let bytes = generate(&["vcf", "--contacts", "3"]);
    let text = String::from_utf8(bytes).unwrap();
    assert_eq!(text.matches("BEGIN:VCARD").count(), 3);
    assert_eq!(text.matches("END:VCARD").count(), 3);
}

#[test]
fn test_vcf_locale_aware() {
    let bytes = generate(&["-l", "ja_jp", "vcf", "--contacts", "2"]);
    let text = String::from_utf8(bytes).unwrap();
    assert!(text.contains("Japan"));
}

#[test]
fn test_ics() {
    let bytes = generate(&["ics", "--events", "4"]);
    let text = String::from_utf8(bytes).unwrap();
    assert!(text.starts_with("BEGIN:VCALENDAR"));
    assert_eq!(text.matches("BEGIN:VEVENT").count(), 4);
}

#[test]
fn test_eml() {
    let bytes = generate(&["eml", "--paragraphs", "2"]);
    let text = String::from_utf8(bytes).unwrap();
    assert!(text.starts_with("From: "));
    assert!(text.contains("\r\n\r\n"));
}

#[test]
fn test_rtf() {
    let bytes = generate(&["rtf", "--paragraphs", "3", "--headings", "2"]);
    let text = String::from_utf8(bytes).unwrap();
    assert!(text.starts_with("{\\rtf1"));
    assert!(text.ends_with('}'));
}

#[test]
fn test_geojson() {
    let bytes = generate(&["geojson", "--rows", "3"]);
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed["type"], "FeatureCollection");
    assert_eq!(parsed["features"].as_array().unwrap().len(), 3);
}

#[test]
fn test_properties() {
    let bytes = generate(&["properties", "--sections", "2", "--keys", "3"]);
    let text = String::from_utf8(bytes).unwrap();
    let pairs = text
        .lines()
        .filter(|l| l.contains('=') && !l.starts_with('#'))
        .count();
    assert_eq!(pairs, 6);
}

#[test]
fn test_srt() {
    let bytes = generate(&["srt", "--cues", "5"]);
    let text = String::from_utf8(bytes).unwrap();
    assert!(text.starts_with("1\n"));
    assert_eq!(text.matches(" --> ").count(), 5);
}

#[test]
fn test_new_locales_generate_csv() {
    for locale in ["da_dk", "nb_no", "fi_fi", "cs_cz", "tr_tr", "ja_jp"] {
        let bytes = generate(&["-l", locale, "csv", "--rows", "5"]);
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.lines().count() > 1, "{locale} produced no rows");
    }
}
