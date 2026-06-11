//! Integration tests for the features added in 0.5.0: schema presets, the
//! `presets`/`info` subcommands, `--dry-run`, `--jobs`, the expanded interface
//! languages, and the schema "did you mean" hint.
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper: build a [`Command`] pointing at our binary crate.
fn cmd() -> Command {
    #[allow(deprecated)]
    {
        Command::cargo_bin("demodatagen").expect("binary should be built")
    }
}

#[test]
fn test_presets_subcommand_lists_presets() {
    cmd()
        .args(["--color", "never", "--skip-update", "presets"])
        .assert()
        .success()
        .stdout(predicate::str::contains("users"))
        .stdout(predicate::str::contains("sensors"))
        .stdout(predicate::str::contains("Schema:"));
}

#[test]
fn test_info_subcommand_reports_environment() {
    cmd()
        .args(["--color", "never", "--skip-update", "info"])
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")))
        // 33 formats / 9 interface languages are reported in the panel.
        .stdout(predicate::str::contains("33"))
        .stdout(predicate::str::contains("9"));
}

#[test]
fn test_dry_run_writes_no_files_but_plans() {
    let tmp = TempDir::new().unwrap();
    cmd()
        .args([
            "-o",
            tmp.path().to_str().unwrap(),
            "-c",
            "4",
            "--color",
            "never",
            "--skip-update",
            "--dry-run",
            "json",
        ])
        .assert()
        .success()
        // Planned file paths are printed to stdout.
        .stdout(predicate::str::contains("demo_0.json"))
        .stdout(predicate::str::contains("demo_3.json"));

    // Nothing should have been written to disk.
    let count = fs::read_dir(tmp.path()).unwrap().count();
    assert_eq!(count, 0, "dry run must not write files");
}

#[test]
fn test_preset_flag_generates_expected_schema() {
    let tmp = TempDir::new().unwrap();
    cmd()
        .args([
            "-o",
            tmp.path().to_str().unwrap(),
            "-c",
            "1",
            "-s",
            "1",
            "--quiet",
            "--skip-update",
            "json",
            "--preset",
            "users",
            "--rows",
            "3",
        ])
        .assert()
        .success();

    let file = fs::read_dir(tmp.path())
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let parsed: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(file).unwrap()).unwrap();
    let first = &parsed.as_array().unwrap()[0];
    assert!(first.get("username").is_some(), "users preset has username");
    assert!(first.get("email").is_some(), "users preset has email");
}

#[test]
fn test_preset_and_schema_conflict() {
    cmd()
        .args([
            "--skip-update",
            "json",
            "--preset",
            "users",
            "--schema",
            "id:int",
        ])
        .assert()
        .failure();
}

#[test]
fn test_unknown_preset_errors() {
    cmd()
        .args(["--skip-update", "json", "--preset", "does-not-exist"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("does-not-exist"));
}

#[test]
fn test_schema_typo_emits_suggestion() {
    let tmp = TempDir::new().unwrap();
    cmd()
        .args([
            "-o",
            tmp.path().to_str().unwrap(),
            "-c",
            "1",
            "--color",
            "never",
            "--skip-update",
            "json",
            "--schema",
            "x:emial",
        ])
        .assert()
        .success()
        // Generation still succeeds, but a hint points at the likely intent.
        .stderr(predicate::str::contains("email"));
}

#[test]
fn test_jobs_flag_is_accepted() {
    let tmp = TempDir::new().unwrap();
    cmd()
        .args([
            "-o",
            tmp.path().to_str().unwrap(),
            "-c",
            "4",
            "-j",
            "2",
            "--quiet",
            "--skip-update",
            "txt",
        ])
        .assert()
        .success();
    assert_eq!(fs::read_dir(tmp.path()).unwrap().count(), 4);
}

#[test]
fn test_polish_interface_language() {
    let tmp = TempDir::new().unwrap();
    cmd()
        .args([
            "-o",
            tmp.path().to_str().unwrap(),
            "-c",
            "1",
            "--lang",
            "pl",
            "--color",
            "never",
            "--skip-update",
            "json",
        ])
        .assert()
        .success()
        // Polish summary verb for "generated".
        .stderr(predicate::str::contains("Wygenerowano"));
}

#[test]
fn test_new_faker_types_generate() {
    cmd()
        .args([
            "-s",
            "7",
            "--skip-update",
            "--stdout",
            "json",
            "--rows",
            "1",
            "--schema",
            "bic:bic,ean:ean,imei:imei,status:http_status,coords:coordinates",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"bic\""))
        .stdout(predicate::str::contains("\"imei\""));
}
