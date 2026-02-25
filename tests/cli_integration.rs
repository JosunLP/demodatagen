/// Integration tests for the `demodatagen` CLI binary.
///
/// These tests exercise the binary end-to-end by spawning it as a child
/// process and asserting on exit codes, stdout/stderr and generated files.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper: build a [`Command`] pointing at our binary crate.
fn cmd() -> Command {
    Command::cargo_bin("demodatagen").expect("binary should be built")
}

// ── Happy-path tests ────────────────────────────────────────────────

#[test]
fn test_json_generates_file() {
    let tmp = TempDir::new().unwrap();
    cmd()
        .args([
            "-o",
            tmp.path().to_str().unwrap(),
            "-c",
            "1",
            "--quiet",
            "json",
            "--schema",
            "name:name,age:integer",
            "--rows",
            "5",
        ])
        .assert()
        .success();

    let files: Vec<_> = fs::read_dir(tmp.path())
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(files.len(), 1);

    let content = fs::read_to_string(files[0].path()).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(parsed.is_array());
    assert_eq!(parsed.as_array().unwrap().len(), 5);
}

#[test]
fn test_csv_generates_valid_csv() {
    let tmp = TempDir::new().unwrap();
    cmd()
        .args([
            "-o",
            tmp.path().to_str().unwrap(),
            "-c",
            "1",
            "--quiet",
            "csv",
            "--schema",
            "first:name,email:email",
            "--rows",
            "3",
        ])
        .assert()
        .success();

    let files: Vec<_> = fs::read_dir(tmp.path())
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(files.len(), 1);

    let content = fs::read_to_string(files[0].path()).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    // header + 3 data rows
    assert_eq!(lines.len(), 4);
    assert!(lines[0].contains("first"));
    assert!(lines[0].contains("email"));
}

#[test]
fn test_xml_generates_valid_xml() {
    let tmp = TempDir::new().unwrap();
    cmd()
        .args([
            "-o",
            tmp.path().to_str().unwrap(),
            "-c",
            "1",
            "--quiet",
            "xml",
            "--schema",
            "user:name,score:integer",
            "--rows",
            "4",
        ])
        .assert()
        .success();

    let files: Vec<_> = fs::read_dir(tmp.path())
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(files.len(), 1);

    let content = fs::read_to_string(files[0].path()).unwrap();
    assert!(content.starts_with("<?xml"));
    assert!(content.contains("<records>"));
    assert!(content.contains("</records>"));
}

#[test]
fn test_txt_generates_file() {
    let tmp = TempDir::new().unwrap();
    cmd()
        .args([
            "-o",
            tmp.path().to_str().unwrap(),
            "-c",
            "2",
            "--quiet",
            "txt",
            "--paragraphs",
            "3",
        ])
        .assert()
        .success();

    let files: Vec<_> = fs::read_dir(tmp.path())
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(files.len(), 2);
}

#[test]
fn test_markdown_generates_file() {
    let tmp = TempDir::new().unwrap();
    cmd()
        .args([
            "-o",
            tmp.path().to_str().unwrap(),
            "-c",
            "1",
            "--quiet",
            "markdown",
        ])
        .assert()
        .success();

    let files: Vec<_> = fs::read_dir(tmp.path())
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(files.len(), 1);

    let content = fs::read_to_string(files[0].path()).unwrap();
    assert!(content.starts_with("# "));
}

#[test]
fn test_png_generates_valid_png() {
    let tmp = TempDir::new().unwrap();
    cmd()
        .args([
            "-o",
            tmp.path().to_str().unwrap(),
            "-c",
            "1",
            "--quiet",
            "png",
            "--width",
            "16",
            "--height",
            "16",
        ])
        .assert()
        .success();

    let files: Vec<_> = fs::read_dir(tmp.path())
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(files.len(), 1);

    let data = fs::read(files[0].path()).unwrap();
    // PNG magic bytes
    assert_eq!(&data[..4], &[0x89, 0x50, 0x4E, 0x47]);
}

#[test]
fn test_jpg_generates_valid_jpg() {
    let tmp = TempDir::new().unwrap();
    cmd()
        .args([
            "-o",
            tmp.path().to_str().unwrap(),
            "-c",
            "1",
            "--quiet",
            "jpg",
            "--width",
            "16",
            "--height",
            "16",
        ])
        .assert()
        .success();

    let files: Vec<_> = fs::read_dir(tmp.path())
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(files.len(), 1);

    let data = fs::read(files[0].path()).unwrap();
    // JPEG starts with FF D8
    assert_eq!(&data[..2], &[0xFF, 0xD8]);
}

#[test]
fn test_gif_generates_valid_gif() {
    let tmp = TempDir::new().unwrap();
    cmd()
        .args([
            "-o",
            tmp.path().to_str().unwrap(),
            "-c",
            "1",
            "--quiet",
            "gif",
            "--width",
            "16",
            "--height",
            "16",
        ])
        .assert()
        .success();

    let files: Vec<_> = fs::read_dir(tmp.path())
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(files.len(), 1);

    let data = fs::read(files[0].path()).unwrap();
    // GIF starts with GIF89a or GIF87a
    let header = String::from_utf8_lossy(&data[..3]);
    assert_eq!(header, "GIF");
}

#[test]
fn test_webp_generates_valid_webp() {
    let tmp = TempDir::new().unwrap();
    cmd()
        .args([
            "-o",
            tmp.path().to_str().unwrap(),
            "-c",
            "1",
            "--quiet",
            "webp",
            "--width",
            "16",
            "--height",
            "16",
        ])
        .assert()
        .success();

    let files: Vec<_> = fs::read_dir(tmp.path())
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(files.len(), 1);

    let data = fs::read(files[0].path()).unwrap();
    assert_eq!(&data[..4], b"RIFF");
    assert_eq!(&data[8..12], b"WEBP");
}

#[test]
fn test_mp3_generates_file() {
    let tmp = TempDir::new().unwrap();
    cmd()
        .args([
            "-o",
            tmp.path().to_str().unwrap(),
            "-c",
            "1",
            "--quiet",
            "mp3",
            "--duration",
            "1",
        ])
        .assert()
        .success();

    let files: Vec<_> = fs::read_dir(tmp.path())
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(files.len(), 1);

    let data = fs::read(files[0].path()).unwrap();
    assert!(data.len() > 100, "MP3 file should have substantial content");
}

#[test]
fn test_mp4_generates_file() {
    let tmp = TempDir::new().unwrap();
    cmd()
        .args([
            "-o",
            tmp.path().to_str().unwrap(),
            "-c",
            "1",
            "--quiet",
            "mp4",
            "--width",
            "16",
            "--height",
            "16",
            "--duration",
            "1",
        ])
        .assert()
        .success();

    let files: Vec<_> = fs::read_dir(tmp.path())
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(files.len(), 1);

    let data = fs::read(files[0].path()).unwrap();
    // ISO Base Media File ftyp box
    assert_eq!(&data[4..8], b"ftyp");
}

#[test]
fn test_webm_generates_file() {
    let tmp = TempDir::new().unwrap();
    cmd()
        .args([
            "-o",
            tmp.path().to_str().unwrap(),
            "-c",
            "1",
            "--quiet",
            "webm",
            "--width",
            "16",
            "--height",
            "16",
            "--duration",
            "1",
        ])
        .assert()
        .success();

    let files: Vec<_> = fs::read_dir(tmp.path())
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(files.len(), 1);

    let data = fs::read(files[0].path()).unwrap();
    // EBML header starts with 0x1A45DFA3
    assert_eq!(data[0], 0x1A);
}

#[test]
fn test_exe_generates_pe() {
    let tmp = TempDir::new().unwrap();
    cmd()
        .args([
            "-o",
            tmp.path().to_str().unwrap(),
            "-c",
            "1",
            "--quiet",
            "exe",
            "--size",
            "4096",
        ])
        .assert()
        .success();

    let files: Vec<_> = fs::read_dir(tmp.path())
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(files.len(), 1);

    let data = fs::read(files[0].path()).unwrap();
    assert_eq!(&data[..2], b"MZ");
}

#[test]
fn test_dll_generates_pe_with_dll_flag() {
    let tmp = TempDir::new().unwrap();
    cmd()
        .args([
            "-o",
            tmp.path().to_str().unwrap(),
            "-c",
            "1",
            "--quiet",
            "dll",
            "--size",
            "4096",
        ])
        .assert()
        .success();

    let files: Vec<_> = fs::read_dir(tmp.path())
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(files.len(), 1);

    let data = fs::read(files[0].path()).unwrap();
    assert_eq!(&data[..2], b"MZ");
}

#[test]
fn test_zip_generates_valid_zip() {
    let tmp = TempDir::new().unwrap();
    cmd()
        .args([
            "-o",
            tmp.path().to_str().unwrap(),
            "-c",
            "1",
            "--quiet",
            "zip",
            "--files",
            "3",
        ])
        .assert()
        .success();

    let files: Vec<_> = fs::read_dir(tmp.path())
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(files.len(), 1);

    let data = fs::read(files[0].path()).unwrap();
    // ZIP magic bytes: PK\x03\x04
    assert_eq!(&data[..2], b"PK");
}

// ── Batch & naming tests ────────────────────────────────────────────

#[test]
fn test_batch_count_creates_correct_number_of_files() {
    let tmp = TempDir::new().unwrap();
    cmd()
        .args([
            "-o",
            tmp.path().to_str().unwrap(),
            "-c",
            "5",
            "--quiet",
            "txt",
            "--paragraphs",
            "1",
        ])
        .assert()
        .success();

    let files: Vec<_> = fs::read_dir(tmp.path())
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(files.len(), 5);
}

#[test]
fn test_custom_name_pattern() {
    let tmp = TempDir::new().unwrap();
    cmd()
        .args([
            "-o",
            tmp.path().to_str().unwrap(),
            "-c",
            "1",
            "-n",
            "myfile_{n}",
            "--quiet",
            "txt",
        ])
        .assert()
        .success();

    let files: Vec<_> = fs::read_dir(tmp.path())
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(files.len(), 1);
    let name = files[0].file_name();
    assert!(
        name.to_str().unwrap().starts_with("myfile_"),
        "File should start with 'myfile_'"
    );
}

#[test]
fn test_seed_produces_deterministic_output() {
    let tmp1 = TempDir::new().unwrap();
    let tmp2 = TempDir::new().unwrap();

    for tmp in [&tmp1, &tmp2] {
        cmd()
            .args([
                "-o",
                tmp.path().to_str().unwrap(),
                "-c",
                "1",
                "-s",
                "12345",
                "--quiet",
                "json",
                "--schema",
                "name:name",
                "--rows",
                "3",
            ])
            .assert()
            .success();
    }

    let content1 = {
        let files: Vec<_> = fs::read_dir(tmp1.path())
            .unwrap()
            .filter_map(Result::ok)
            .collect();
        fs::read_to_string(files[0].path()).unwrap()
    };
    let content2 = {
        let files: Vec<_> = fs::read_dir(tmp2.path())
            .unwrap()
            .filter_map(Result::ok)
            .collect();
        fs::read_to_string(files[0].path()).unwrap()
    };

    assert_eq!(content1, content2, "Same seed should produce identical output");
}

// ── Error handling tests ────────────────────────────────────────────

#[test]
fn test_no_overwrite_by_default() {
    let tmp = TempDir::new().unwrap();

    // First run – creates files
    cmd()
        .args([
            "-o",
            tmp.path().to_str().unwrap(),
            "-c",
            "1",
            "--quiet",
            "txt",
        ])
        .assert()
        .success();

    // Second run – should fail because file already exists
    cmd()
        .args([
            "-o",
            tmp.path().to_str().unwrap(),
            "-c",
            "1",
            "--quiet",
            "txt",
        ])
        .assert()
        .failure();
}

#[test]
fn test_overwrite_flag_allows_replacement() {
    let tmp = TempDir::new().unwrap();

    // First run
    cmd()
        .args([
            "-o",
            tmp.path().to_str().unwrap(),
            "-c",
            "1",
            "--quiet",
            "txt",
        ])
        .assert()
        .success();

    // Second run with --overwrite
    cmd()
        .args([
            "-o",
            tmp.path().to_str().unwrap(),
            "-c",
            "1",
            "--quiet",
            "--overwrite",
            "txt",
        ])
        .assert()
        .success();
}

#[test]
fn test_unknown_format_schema_type() {
    let tmp = TempDir::new().unwrap();
    // "nonexistent" isn't a known type but should still generate something
    // since value_for_type falls back to a default string.
    cmd()
        .args([
            "-o",
            tmp.path().to_str().unwrap(),
            "-c",
            "1",
            "--quiet",
            "json",
            "--schema",
            "field:nonexistent",
            "--rows",
            "1",
        ])
        .assert()
        .success();
}

// ── Help & version tests ────────────────────────────────────────────

#[test]
fn test_help_output() {
    cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("demodatagen"));
}

#[test]
fn test_version_output() {
    cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_format_help() {
    cmd()
        .args(["json", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("schema"));
}
