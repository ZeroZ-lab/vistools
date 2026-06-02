//! JSON schema shape snapshots for the vistools CLI.
//!
//! These tests intentionally compare structure, not dynamic values such as
//! paths, elapsed time, file size, or exact image dimensions.

use assert_cmd::Command;
use serde_json::{Value, json};
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

fn run_json(args: &[String]) -> Value {
    let output = bin().args(args).output().unwrap();
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn shape(value: &Value) -> Value {
    match value {
        Value::Null => json!("null"),
        Value::Bool(_) => json!("bool"),
        Value::Number(_) => json!("number"),
        Value::String(_) => json!("string"),
        Value::Array(values) => {
            if let Some(first) = values.first() {
                json!([shape(first)])
            } else {
                json!([])
            }
        }
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, value)| (key.clone(), shape(value)))
                .collect(),
        ),
    }
}

#[test]
fn inspect_success_schema_snapshot() {
    let actual = run_json(&[
        "inspect".to_string(),
        fixture("256x256.png").display().to_string(),
    ]);

    assert_eq!(
        shape(&actual),
        json!({
            "ok": "bool",
            "operation": "string",
            "input": "string",
            "data": {
                "source": {
                    "width": "number",
                    "height": "number",
                    "format": "string",
                    "size_bytes": "number"
                },
                "suggestion": {
                    "needs_overview": "bool",
                    "max_tile_rows": "number",
                    "max_tile_cols": "number",
                    "recommended_next": "string",
                    "reason": "string",
                    "suggested_max_side": "number"
                }
            },
            "warnings": [],
            "elapsed_ms": "number"
        })
    );
}

#[test]
fn error_schema_snapshot() {
    let output = bin()
        .arg("inspect")
        .arg("/tmp/__vistools_schema_missing__.png")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let actual: Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(
        shape(&actual),
        json!({
            "ok": "bool",
            "operation": "string",
            "input": "string",
            "error": {
                "code": "string",
                "message": "string"
            },
            "warnings": [],
            "elapsed_ms": "number"
        })
    );
}

#[test]
fn overview_success_schema_snapshot() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("overview.png");
    let actual = run_json(&[
        "overview".to_string(),
        fixture("1000x1000.png").display().to_string(),
        output.display().to_string(),
        "--max-side".to_string(),
        "200".to_string(),
    ]);

    assert_eq!(
        shape(&actual),
        json!({
            "ok": "bool",
            "operation": "string",
            "input": "string",
            "data": {
                "output": "string",
                "source": {
                    "width": "number",
                    "height": "number",
                    "format": "string",
                    "size_bytes": "number"
                },
                "result": {
                    "width": "number",
                    "height": "number"
                },
                "scale_factor": "number",
                "coordinate_mapping": {
                    "source_origin": {
                        "x": "number",
                        "y": "number"
                    },
                    "scale_x": "number",
                    "scale_y": "number",
                    "formula": "string"
                }
            },
            "warnings": [],
            "elapsed_ms": "number"
        })
    );
}

#[test]
fn tile_success_schema_snapshot() {
    let dir = tempfile::tempdir().unwrap();
    let actual = run_json(&[
        "tile".to_string(),
        fixture("1000x1000.png").display().to_string(),
        "--rows".to_string(),
        "2".to_string(),
        "--cols".to_string(),
        "2".to_string(),
        "--out-dir".to_string(),
        dir.path().display().to_string(),
    ]);

    assert_eq!(
        shape(&actual),
        json!({
            "ok": "bool",
            "operation": "string",
            "input": "string",
            "data": {
                "source": {
                    "width": "number",
                    "height": "number",
                    "format": "string",
                    "size_bytes": "number"
                },
                "rows": "number",
                "cols": "number",
                "tiles": [{
                    "path": "string",
                    "row": "number",
                    "col": "number",
                    "width": "number",
                    "height": "number",
                    "source_region": {
                        "x": "number",
                        "y": "number",
                        "width": "number",
                        "height": "number"
                    }
                }]
            },
            "warnings": [],
            "elapsed_ms": "number"
        })
    );
}

#[test]
fn viewport_success_schema_snapshot() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("crop.png");
    let actual = run_json(&[
        "viewport".to_string(),
        "rect".to_string(),
        fixture("1000x1000.png").display().to_string(),
        output.display().to_string(),
        "--x".to_string(),
        "100".to_string(),
        "--y".to_string(),
        "200".to_string(),
        "--width".to_string(),
        "300".to_string(),
        "--height".to_string(),
        "400".to_string(),
    ]);

    assert_eq!(
        shape(&actual),
        json!({
            "ok": "bool",
            "operation": "string",
            "input": "string",
            "data": {
                "output": "string",
                "source": {
                    "width": "number",
                    "height": "number",
                    "format": "string",
                    "size_bytes": "number"
                },
                "crop": {
                    "spec": {
                        "mode": "string",
                        "rect": {
                            "x": "number",
                            "y": "number",
                            "width": "number",
                            "height": "number"
                        }
                    },
                    "region": {
                        "x": "number",
                        "y": "number",
                        "width": "number",
                        "height": "number"
                    }
                },
                "result": {
                    "width": "number",
                    "height": "number"
                },
                "coordinate_mapping": {
                    "source_origin": {
                        "x": "number",
                        "y": "number"
                    },
                    "scale_x": "number",
                    "scale_y": "number",
                    "formula": "string"
                }
            },
            "warnings": [],
            "elapsed_ms": "number"
        })
    );
}

#[test]
fn sample_point_success_schema_snapshot() {
    let actual = run_json(&[
        "sample".to_string(),
        fixture("64x64.png").display().to_string(),
        "--x".to_string(),
        "10".to_string(),
        "--y".to_string(),
        "10".to_string(),
    ]);

    assert_eq!(
        shape(&actual),
        json!({
            "ok": "bool",
            "operation": "string",
            "input": "string",
            "data": {
                "source": {
                    "width": "number",
                    "height": "number",
                    "format": "string",
                    "size_bytes": "number"
                },
                "sample": {
                    "mode": "string",
                    "point": {
                        "x": "number",
                        "y": "number"
                    },
                    "color": {
                        "rgba": ["number"],
                        "rgb": ["number"],
                        "hex": "string",
                        "alpha": "number"
                    }
                }
            },
            "warnings": [],
            "elapsed_ms": "number"
        })
    );
}

#[test]
fn sample_rect_success_schema_snapshot() {
    let actual = run_json(&[
        "sample".to_string(),
        fixture("64x64.png").display().to_string(),
        "--rect".to_string(),
        "0,0,2,2".to_string(),
    ]);

    assert_eq!(
        shape(&actual),
        json!({
            "ok": "bool",
            "operation": "string",
            "input": "string",
            "data": {
                "source": {
                    "width": "number",
                    "height": "number",
                    "format": "string",
                    "size_bytes": "number"
                },
                "sample": {
                    "mode": "string",
                    "region": {
                        "x": "number",
                        "y": "number",
                        "width": "number",
                        "height": "number"
                    },
                    "average": {
                        "rgba": ["number"],
                        "rgb": ["number"],
                        "hex": "string",
                        "alpha": "number"
                    },
                    "alpha_stats": {
                        "min": "number",
                        "max": "number",
                        "average": "number",
                        "transparent_ratio": "number"
                    },
                    "pixel_count": "number"
                }
            },
            "warnings": [],
            "elapsed_ms": "number"
        })
    );
}

#[test]
fn sharpness_success_schema_snapshot() {
    let actual = run_json(&[
        "sharpness".to_string(),
        fixture("64x64.png").display().to_string(),
    ]);

    assert_eq!(
        shape(&actual),
        json!({
            "ok": "bool",
            "operation": "string",
            "input": "string",
            "data": {
                "source": {
                    "width": "number",
                    "height": "number",
                    "format": "string",
                    "size_bytes": "number"
                },
                "region": {
                    "x": "number",
                    "y": "number",
                    "width": "number",
                    "height": "number"
                },
                "sharpness": {
                    "score": "number",
                    "mean_edge_strength": "number",
                    "max_edge_strength": "number"
                }
            },
            "warnings": [],
            "elapsed_ms": "number"
        })
    );
}

#[test]
fn histogram_success_schema_snapshot() {
    let actual = run_json(&[
        "histogram".to_string(),
        fixture("64x64.png").display().to_string(),
    ]);

    assert_eq!(
        shape(&actual),
        json!({
            "ok": "bool",
            "operation": "string",
            "input": "string",
            "data": {
                "source": {
                    "width": "number",
                    "height": "number",
                    "format": "string",
                    "size_bytes": "number"
                },
                "region": {
                    "x": "number",
                    "y": "number",
                    "width": "number",
                    "height": "number"
                },
                "histogram": {
                    "bins": ["number"],
                    "pixel_count": "number",
                    "mean_luma": "number",
                    "median_luma": "number",
                    "p05_luma": "number",
                    "p95_luma": "number"
                }
            },
            "warnings": [],
            "elapsed_ms": "number"
        })
    );
}

#[test]
fn highlight_clipping_success_schema_snapshot() {
    let actual = run_json(&[
        "highlight-clipping".to_string(),
        fixture("64x64.png").display().to_string(),
    ]);

    assert_eq!(
        shape(&actual),
        json!({
            "ok": "bool",
            "operation": "string",
            "input": "string",
            "data": {
                "source": {
                    "width": "number",
                    "height": "number",
                    "format": "string",
                    "size_bytes": "number"
                },
                "region": {
                    "x": "number",
                    "y": "number",
                    "width": "number",
                    "height": "number"
                },
                "clipping": {
                    "threshold": "number",
                    "clipped_pixels": "number",
                    "clipped_ratio": "number",
                    "pixel_count": "number"
                }
            },
            "warnings": [],
            "elapsed_ms": "number"
        })
    );
}

#[test]
fn contrast_success_schema_snapshot() {
    let actual = run_json(&[
        "contrast".to_string(),
        fixture("64x64.png").display().to_string(),
    ]);

    assert_eq!(
        shape(&actual),
        json!({
            "ok": "bool",
            "operation": "string",
            "input": "string",
            "data": {
                "source": {
                    "width": "number",
                    "height": "number",
                    "format": "string",
                    "size_bytes": "number"
                },
                "region": {
                    "x": "number",
                    "y": "number",
                    "width": "number",
                    "height": "number"
                },
                "contrast": {
                    "rms_contrast": "number",
                    "luma_stddev": "number",
                    "min_luma": "number",
                    "max_luma": "number",
                    "dynamic_range": "number"
                }
            },
            "warnings": [],
            "elapsed_ms": "number"
        })
    );
}

#[test]
fn color_cast_success_schema_snapshot() {
    let actual = run_json(&[
        "color-cast".to_string(),
        fixture("64x64.png").display().to_string(),
    ]);

    assert_eq!(
        shape(&actual),
        json!({
            "ok": "bool",
            "operation": "string",
            "input": "string",
            "data": {
                "source": {
                    "width": "number",
                    "height": "number",
                    "format": "string",
                    "size_bytes": "number"
                },
                "region": {
                    "x": "number",
                    "y": "number",
                    "width": "number",
                    "height": "number"
                },
                "color_cast": {
                    "channel_means": ["number"],
                    "neutral_mean": "number",
                    "cast_vector": ["number"],
                    "cast_strength": "number",
                    "dominant_channel": "string"
                }
            },
            "warnings": [],
            "elapsed_ms": "number"
        })
    );
}

#[test]
fn histogram_rgb_schema_snapshot() {
    let actual = run_json(&[
        "histogram".to_string(),
        fixture("64x64.png").display().to_string(),
        "--rgb".to_string(),
    ]);

    assert_eq!(
        shape(&actual),
        json!({
            "ok": "bool",
            "operation": "string",
            "input": "string",
            "data": {
                "source": {
                    "width": "number",
                    "height": "number",
                    "format": "string",
                    "size_bytes": "number"
                },
                "region": {
                    "x": "number",
                    "y": "number",
                    "width": "number",
                    "height": "number"
                },
                "histogram": {
                    "bins": ["number"],
                    "pixel_count": "number",
                    "mean_luma": "number",
                    "median_luma": "number",
                    "p05_luma": "number",
                    "p95_luma": "number",
                    "rgb": {
                        "r": {
                            "bins": ["number"],
                            "mean": "number",
                            "p05": "number",
                            "p50": "number",
                            "p95": "number",
                            "clipping_low": "number",
                            "clipping_high": "number"
                        },
                        "g": {
                            "bins": ["number"],
                            "mean": "number",
                            "p05": "number",
                            "p50": "number",
                            "p95": "number",
                            "clipping_low": "number",
                            "clipping_high": "number"
                        },
                        "b": {
                            "bins": ["number"],
                            "mean": "number",
                            "p05": "number",
                            "p50": "number",
                            "p95": "number",
                            "clipping_low": "number",
                            "clipping_high": "number"
                        }
                    }
                }
            },
            "warnings": [],
            "elapsed_ms": "number"
        })
    );
}

#[test]
fn zone_map_schema_snapshot() {
    let actual = run_json(&[
        "zone-map".to_string(),
        fixture("64x64.png").display().to_string(),
    ]);

    assert_eq!(
        shape(&actual),
        json!({
            "ok": "bool",
            "operation": "string",
            "input": "string",
            "data": {
                "source": {
                    "width": "number",
                    "height": "number",
                    "format": "string",
                    "size_bytes": "number"
                },
                "region": {
                    "x": "number",
                    "y": "number",
                    "width": "number",
                    "height": "number"
                },
                "zones": [{
                    "zone": "number",
                    "label": "string",
                    "luma_range": ["number"],
                    "pixel_count": "number",
                    "ratio": "number",
                    "representative_rect": {
                        "x": "number",
                        "y": "number",
                        "width": "number",
                        "height": "number"
                    }
                }]
            },
            "warnings": [],
            "elapsed_ms": "number"
        })
    );
}

#[test]
fn exposure_schema_snapshot() {
    let actual = run_json(&[
        "exposure".to_string(),
        fixture("64x64.png").display().to_string(),
    ]);

    assert_eq!(
        shape(&actual),
        json!({
            "ok": "bool",
            "operation": "string",
            "input": "string",
            "data": {
                "source": {
                    "width": "number",
                    "height": "number",
                    "format": "string",
                    "size_bytes": "number"
                },
                "region": {
                    "x": "number",
                    "y": "number",
                    "width": "number",
                    "height": "number"
                },
                "metering": "string",
                "ev": "number",
                "assessment": "string",
                "mean_luma": "number"
            },
            "warnings": [],
            "elapsed_ms": "number"
        })
    );
}

#[test]
fn exposure_spot_schema_snapshot() {
    let actual = run_json(&[
        "exposure".to_string(),
        fixture("64x64.png").display().to_string(),
        "--mode".to_string(),
        "spot".to_string(),
        "--x".to_string(),
        "32".to_string(),
        "--y".to_string(),
        "32".to_string(),
    ]);

    assert_eq!(
        shape(&actual),
        json!({
            "ok": "bool",
            "operation": "string",
            "input": "string",
            "data": {
                "source": {
                    "width": "number",
                    "height": "number",
                    "format": "string",
                    "size_bytes": "number"
                },
                "region": {
                    "x": "number",
                    "y": "number",
                    "width": "number",
                    "height": "number"
                },
                "metering": "string",
                "ev": "number",
                "assessment": "string",
                "mean_luma": "number",
                "spot_point": {
                    "x": "number",
                    "y": "number"
                }
            },
            "warnings": [],
            "elapsed_ms": "number"
        })
    );
}

#[test]
fn focus_map_schema_snapshot() {
    let actual = run_json(&[
        "focus-map".to_string(),
        fixture("64x64.png").display().to_string(),
        "--rows".to_string(),
        "2".to_string(),
        "--cols".to_string(),
        "2".to_string(),
    ]);

    assert_eq!(
        shape(&actual),
        json!({
            "ok": "bool",
            "operation": "string",
            "input": "string",
            "data": {
                "source": {
                    "width": "number",
                    "height": "number",
                    "format": "string",
                    "size_bytes": "number"
                },
                "region": {
                    "x": "number",
                    "y": "number",
                    "width": "number",
                    "height": "number"
                },
                "rows": "number",
                "cols": "number",
                "cells": [{
                    "row": "number",
                    "col": "number",
                    "region": {
                        "x": "number",
                        "y": "number",
                        "width": "number",
                        "height": "number"
                    },
                    "sharpness": {
                        "score": "number",
                        "mean_edge_strength": "number",
                        "max_edge_strength": "number"
                    }
                }],
                "best_cell": {
                    "row": "number",
                    "col": "number",
                    "region": {
                        "x": "number",
                        "y": "number",
                        "width": "number",
                        "height": "number"
                    },
                    "sharpness": {
                        "score": "number",
                        "mean_edge_strength": "number",
                        "max_edge_strength": "number"
                    }
                },
                "focus_point": {
                    "x": "number",
                    "y": "number"
                }
            },
            "warnings": [],
            "elapsed_ms": "number"
        })
    );
}

#[test]
fn white_balance_schema_snapshot() {
    let actual = run_json(&[
        "white-balance".to_string(),
        fixture("64x64.png").display().to_string(),
    ]);

    assert_eq!(
        shape(&actual),
        json!({
            "ok": "bool",
            "operation": "string",
            "input": "string",
            "data": {
                "source": {
                    "width": "number",
                    "height": "number",
                    "format": "string",
                    "size_bytes": "number"
                },
                "region": {
                    "x": "number",
                    "y": "number",
                    "width": "number",
                    "height": "number"
                },
                "white_balance": {
                    "rgb_mean": {
                        "r": "number",
                        "g": "number",
                        "b": "number"
                    },
                    "gray_world_gains": {
                        "r": "number",
                        "g": "number",
                        "b": "number"
                    },
                    "temperature_bias": "string",
                    "tint_bias": "string",
                    "assessment": "string"
                }
            },
            "warnings": [],
            "elapsed_ms": "number"
        })
    );
}
