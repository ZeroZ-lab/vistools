//! Integration tests for the vistools CLI.
//!
//! Uses assert_cmd to test CLI invocations end-to-end.

use assert_cmd::Command;
use std::path::PathBuf;

fn bin() -> Command {
    Command::cargo_bin("vistools").unwrap()
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("fixtures")
        .join(name)
}

// ---------------------------------------------------------------------------
// inspect
// ---------------------------------------------------------------------------

#[test]
fn inspect_success() {
    bin()
        .arg("inspect")
        .arg(fixture("256x256.png"))
        .assert()
        .success()
        .stdout(predicates::str::contains("\"ok\": true"))
        .stdout(predicates::str::contains("\"width\": 256"))
        .stdout(predicates::str::contains("\"height\": 256"));
}

#[test]
fn inspect_nonexistent() {
    bin()
        .arg("inspect")
        .arg("/tmp/__nonexistent_test__.png")
        .assert()
        .code(1)
        .stdout(predicates::str::contains("FILE_NOT_FOUND"));
}

#[test]
fn inspect_1000x1000() {
    bin()
        .arg("inspect")
        .arg(fixture("1000x1000.png"))
        .assert()
        .success()
        .stdout(predicates::str::contains("\"needs_overview\": false"));
}

// ---------------------------------------------------------------------------
// overview
// ---------------------------------------------------------------------------

#[test]
fn overview_scales_down() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("overview.png");

    bin()
        .arg("overview")
        .arg(fixture("1000x1000.png"))
        .arg(&out)
        .arg("--max-width")
        .arg("200")
        .assert()
        .success()
        .stdout(predicates::str::contains("\"width\": 200"))
        .stdout(predicates::str::contains("\"height\": 200"));

    assert!(out.exists());
}

// ---------------------------------------------------------------------------
// tile
// ---------------------------------------------------------------------------

#[test]
fn tile_2x2() {
    let dir = tempfile::tempdir().unwrap();

    bin()
        .arg("tile")
        .arg(fixture("1000x1000.png"))
        .arg("--rows")
        .arg("2")
        .arg("--cols")
        .arg("2")
        .arg("--out-dir")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("\"ok\": true"));

    // Should produce 4 tile files
    let tile_count = std::fs::read_dir(dir.path()).unwrap().count();
    assert_eq!(tile_count, 4);
}

#[test]
fn tile_excessive_count() {
    let dir = tempfile::tempdir().unwrap();

    bin()
        .arg("tile")
        .arg(fixture("1000x1000.png"))
        .arg("--rows")
        .arg("10")
        .arg("--cols")
        .arg("10")
        .arg("--out-dir")
        .arg(dir.path())
        .assert()
        .code(1)
        .stdout(predicates::str::contains("INVALID_PARAMETERS"));
}

// ---------------------------------------------------------------------------
// viewport
// ---------------------------------------------------------------------------

#[test]
fn viewport_anchor_right() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("crop.png");

    bin()
        .arg("viewport")
        .arg("anchor")
        .arg(fixture("1000x1000.png"))
        .arg(&out)
        .arg("--anchor")
        .arg("right")
        .arg("--width")
        .arg("500")
        .arg("--height")
        .arg("1000")
        .assert()
        .success()
        .stdout(predicates::str::contains("\"x\": 500"))
        .stdout(predicates::str::contains("\"width\": 500"));
}

#[test]
fn viewport_percent() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("crop.png");

    bin()
        .arg("viewport")
        .arg("percent")
        .arg(fixture("1000x1000.png"))
        .arg(&out)
        .arg("--x")
        .arg("0")
        .arg("--y")
        .arg("0")
        .arg("--w")
        .arg("0.5")
        .arg("--h")
        .arg("0.5")
        .assert()
        .success()
        .stdout(predicates::str::contains("\"mode\": \"percent\""));
}

#[test]
fn viewport_rect() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("crop.png");

    bin()
        .arg("viewport")
        .arg("rect")
        .arg(fixture("1000x1000.png"))
        .arg(&out)
        .arg("--x")
        .arg("100")
        .arg("--y")
        .arg("200")
        .arg("--width")
        .arg("300")
        .arg("--height")
        .arg("400")
        .assert()
        .success()
        .stdout(predicates::str::contains("\"width\": 300"));
}

#[test]
fn viewport_out_of_bounds() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("crop.png");

    bin()
        .arg("viewport")
        .arg("rect")
        .arg(fixture("256x256.png"))
        .arg(&out)
        .arg("--x")
        .arg("200")
        .arg("--y")
        .arg("200")
        .arg("--width")
        .arg("100")
        .arg("--height")
        .arg("100")
        .assert()
        .code(1)
        .stdout(predicates::str::contains("INVALID_COORDINATES"));
}

// ---------------------------------------------------------------------------
// resize
// ---------------------------------------------------------------------------

#[test]
fn resize_proportional() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("resized.png");

    bin()
        .arg("resize")
        .arg(fixture("1000x1000.png"))
        .arg(&out)
        .arg("--width")
        .arg("200")
        .assert()
        .success()
        .stdout(predicates::str::contains("\"width\": 200"))
        .stdout(predicates::str::contains("\"height\": 200"));
}

#[test]
fn resize_forced() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("resized.png");

    bin()
        .arg("resize")
        .arg(fixture("1000x1000.png"))
        .arg(&out)
        .arg("--width")
        .arg("800")
        .arg("--height")
        .arg("600")
        .assert()
        .success()
        .stdout(predicates::str::contains("\"width\": 800"))
        .stdout(predicates::str::contains("\"height\": 600"));
}

// ---------------------------------------------------------------------------
// rotate
// ---------------------------------------------------------------------------

#[test]
fn rotate_90() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("rotated.png");

    bin()
        .arg("rotate")
        .arg(fixture("256x256.png"))
        .arg(&out)
        .arg("--degrees")
        .arg("90")
        .assert()
        .success()
        .stdout(predicates::str::contains("\"degrees\": 90"));
}

#[test]
fn rotate_invalid_degrees() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("rotated.png");

    bin()
        .arg("rotate")
        .arg(fixture("64x64.png"))
        .arg(&out)
        .arg("--degrees")
        .arg("45")
        .assert()
        .code(1)
        .stdout(predicates::str::contains("INVALID_PARAMETERS"));
}
