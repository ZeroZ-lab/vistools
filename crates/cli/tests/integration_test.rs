//! Integration tests for the vistools CLI.
//!
//! Uses assert_cmd to test CLI invocations end-to-end.

use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use std::path::PathBuf;

fn bin() -> Command {
    Command::cargo_bin("vistools").unwrap()
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
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

#[test]
fn inspect_recommends_overview_for_large_image() {
    bin()
        .arg("inspect")
        .arg(fixture("e2e/landscape_large.jpg"))
        .assert()
        .success()
        .stdout(predicates::str::contains("\"needs_overview\": true"))
        .stdout(predicates::str::contains(
            "\"recommended_next\": \"overview\"",
        ))
        .stdout(predicates::str::contains("\"suggested_max_side\": 1568"));
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
        .arg("--max-side")
        .arg("200")
        .assert()
        .success()
        .stdout(predicates::str::contains("\"width\": 200"))
        .stdout(predicates::str::contains("\"height\": 200"));

    assert!(out.exists());
}

#[test]
fn overview_scales_long_side() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("overview.jpg");

    bin()
        .arg("overview")
        .arg(fixture("e2e/portrait_tall.jpg"))
        .arg(&out)
        .arg("--max-side")
        .arg("600")
        .assert()
        .success()
        .stdout(predicates::str::contains("\"width\": 240"))
        .stdout(predicates::str::contains("\"height\": 600"));

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
fn viewport_percent_rejects_out_of_range() {
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
        .arg("1.5")
        .arg("--h")
        .arg("0.5")
        .assert()
        .code(1)
        .stdout(predicates::str::contains("INVALID_PARAMETERS"));
}

#[test]
fn viewport_percent_rejects_region_overflow() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("crop.png");

    bin()
        .arg("viewport")
        .arg("percent")
        .arg(fixture("1000x1000.png"))
        .arg(&out)
        .arg("--x")
        .arg("0.8")
        .arg("--y")
        .arg("0")
        .arg("--w")
        .arg("0.3")
        .arg("--h")
        .arg("0.5")
        .assert()
        .code(1)
        .stdout(predicates::str::contains("INVALID_COORDINATES"));
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
// sample
// ---------------------------------------------------------------------------

#[test]
fn sample_point_success() {
    bin()
        .arg("sample")
        .arg(fixture("64x64.png"))
        .arg("--x")
        .arg("10")
        .arg("--y")
        .arg("10")
        .assert()
        .success()
        .stdout(predicates::str::contains("\"mode\": \"point\""))
        .stdout(predicates::str::contains("\"hex\": \"#6496c8\""));
}

#[test]
fn sample_rect_success() {
    bin()
        .arg("sample")
        .arg(fixture("64x64.png"))
        .arg("--rect")
        .arg("0,0,2,2")
        .assert()
        .success()
        .stdout(predicates::str::contains("\"mode\": \"rect\""))
        .stdout(predicates::str::contains("\"pixel_count\": 4"));
}

#[test]
fn sample_malformed_rect_returns_invalid_parameters() {
    bin()
        .arg("sample")
        .arg(fixture("64x64.png"))
        .arg("--rect")
        .arg("0,0,2")
        .assert()
        .code(1)
        .stdout(predicates::str::contains("INVALID_PARAMETERS"));
}

#[test]
fn sample_missing_mode_returns_invalid_parameters() {
    bin()
        .arg("sample")
        .arg(fixture("64x64.png"))
        .assert()
        .code(1)
        .stdout(predicates::str::contains("INVALID_PARAMETERS"));
}

// ---------------------------------------------------------------------------
// photography metrics
// ---------------------------------------------------------------------------

#[test]
fn sharpness_success() {
    bin()
        .arg("sharpness")
        .arg(fixture("64x64.png"))
        .assert()
        .success()
        .stdout(predicates::str::contains("\"operation\": \"sharpness\""))
        .stdout(predicates::str::contains("\"score\""));
}

#[test]
fn histogram_success() {
    bin()
        .arg("histogram")
        .arg(fixture("64x64.png"))
        .assert()
        .success()
        .stdout(predicates::str::contains("\"operation\": \"histogram\""))
        .stdout(predicates::str::contains("\"mean_luma\""));
}

#[test]
fn highlight_clipping_success() {
    bin()
        .arg("highlight-clipping")
        .arg(fixture("64x64.png"))
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "\"operation\": \"highlight-clipping\"",
        ))
        .stdout(predicates::str::contains("\"threshold\": 250"));
}

#[test]
fn shadow_clipping_success() {
    bin()
        .arg("shadow-clipping")
        .arg(fixture("64x64.png"))
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "\"operation\": \"shadow-clipping\"",
        ))
        .stdout(predicates::str::contains("\"threshold\": 5"));
}

#[test]
fn contrast_success() {
    bin()
        .arg("contrast")
        .arg(fixture("64x64.png"))
        .assert()
        .success()
        .stdout(predicates::str::contains("\"operation\": \"contrast\""))
        .stdout(predicates::str::contains("\"rms_contrast\""));
}

#[test]
fn color_cast_success() {
    bin()
        .arg("color-cast")
        .arg(fixture("64x64.png"))
        .assert()
        .success()
        .stdout(predicates::str::contains("\"operation\": \"color-cast\""))
        .stdout(predicates::str::contains("\"dominant_channel\""));
}

#[test]
fn sharpness_malformed_rect_returns_invalid_parameters() {
    bin()
        .arg("sharpness")
        .arg(fixture("64x64.png"))
        .arg("--rect")
        .arg("1,2,3")
        .assert()
        .code(1)
        .stdout(predicates::str::contains("INVALID_PARAMETERS"));
}

#[test]
fn histogram_rgb_success() {
    bin()
        .arg("histogram")
        .arg(fixture("64x64.png"))
        .arg("--rgb")
        .assert()
        .success()
        .stdout(predicates::str::contains("\"operation\": \"histogram\""))
        .stdout(predicates::str::contains("\"rgb\""))
        .stdout(predicates::str::contains("\"r\""))
        .stdout(predicates::str::contains("\"g\""))
        .stdout(predicates::str::contains("\"b\""));
}

#[test]
fn histogram_without_rgb_has_no_rgb_field() {
    bin()
        .arg("histogram")
        .arg(fixture("64x64.png"))
        .assert()
        .success()
        .stdout(predicates::str::contains("\"operation\": \"histogram\""))
        .stdout(predicates::str::contains("\"mean_luma\""))
        .stdout(predicates::str::contains("\"rgb\"").not());
}

#[test]
fn zone_map_success() {
    bin()
        .arg("zone-map")
        .arg(fixture("64x64.png"))
        .assert()
        .success()
        .stdout(predicates::str::contains("\"operation\": \"zone-map\""))
        .stdout(predicates::str::contains("\"zones\""))
        .stdout(predicates::str::contains("\"zone\""))
        .stdout(predicates::str::contains("\"label\""))
        .stdout(predicates::str::contains("\"ratio\""));
}

#[test]
fn zone_map_malformed_rect_returns_invalid_parameters() {
    bin()
        .arg("zone-map")
        .arg(fixture("64x64.png"))
        .arg("--rect")
        .arg("1,2")
        .assert()
        .code(1)
        .stdout(predicates::str::contains("INVALID_PARAMETERS"));
}

#[test]
fn exposure_evaluative_success() {
    bin()
        .arg("exposure")
        .arg(fixture("64x64.png"))
        .assert()
        .success()
        .stdout(predicates::str::contains("\"operation\": \"exposure\""))
        .stdout(predicates::str::contains("\"metering\": \"evaluative\""))
        .stdout(predicates::str::contains("\"ev\""))
        .stdout(predicates::str::contains("\"assessment\""));
}

#[test]
fn exposure_spot_success() {
    bin()
        .arg("exposure")
        .arg(fixture("64x64.png"))
        .arg("--mode")
        .arg("spot")
        .arg("--x")
        .arg("32")
        .arg("--y")
        .arg("32")
        .assert()
        .success()
        .stdout(predicates::str::contains("\"metering\": \"spot\""))
        .stdout(predicates::str::contains("\"spot_point\""));
}

#[test]
fn exposure_center_weighted_success() {
    bin()
        .arg("exposure")
        .arg(fixture("64x64.png"))
        .arg("--mode")
        .arg("center-weighted")
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "\"metering\": \"center_weighted\"",
        ));
}

#[test]
fn exposure_highlight_weighted_success() {
    bin()
        .arg("exposure")
        .arg(fixture("64x64.png"))
        .arg("--mode")
        .arg("highlight-weighted")
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "\"metering\": \"highlight_weighted\"",
        ));
}

#[test]
fn exposure_spot_missing_coordinates_returns_invalid_parameters() {
    bin()
        .arg("exposure")
        .arg(fixture("64x64.png"))
        .arg("--mode")
        .arg("spot")
        .assert()
        .code(1)
        .stdout(predicates::str::contains("INVALID_PARAMETERS"));
}

#[test]
fn exposure_invalid_mode_returns_invalid_parameters() {
    bin()
        .arg("exposure")
        .arg(fixture("64x64.png"))
        .arg("--mode")
        .arg("invalid-mode")
        .assert()
        .code(1)
        .stdout(predicates::str::contains("INVALID_PARAMETERS"));
}

#[test]
fn exposure_malformed_rect_returns_invalid_parameters() {
    bin()
        .arg("exposure")
        .arg(fixture("64x64.png"))
        .arg("--rect")
        .arg("1,2,3")
        .assert()
        .code(1)
        .stdout(predicates::str::contains("INVALID_PARAMETERS"));
}

#[test]
fn focus_map_success() {
    bin()
        .arg("focus-map")
        .arg(fixture("64x64.png"))
        .arg("--rows")
        .arg("2")
        .arg("--cols")
        .arg("2")
        .assert()
        .success()
        .stdout(predicates::str::contains("\"operation\": \"focus-map\""))
        .stdout(predicates::str::contains("\"best_cell\""))
        .stdout(predicates::str::contains("\"focus_point\""))
        .stdout(predicates::str::contains("\"cells\""));
}

#[test]
fn focus_map_invalid_grid_returns_invalid_parameters() {
    bin()
        .arg("focus-map")
        .arg(fixture("64x64.png"))
        .arg("--rows")
        .arg("0")
        .arg("--cols")
        .arg("2")
        .assert()
        .code(1)
        .stdout(predicates::str::contains("INVALID_PARAMETERS"));
}
