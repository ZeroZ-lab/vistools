//! viewport command — three crop modes (anchor / percent / rect).
//!
//! Decisions: FD2 (coordinate mapping), PD2 (unified coordinate system).
use std::path::Path;
use std::time::Instant;

use crate::coord;
use crate::guard;
use crate::types::*;

/// Crop mode parameters.
pub enum ViewportMode {
    Anchor {
        anchor: Anchor,
        width: u32,
        height: u32,
    },
    Percent {
        pct: Percent,
    },
    Rect {
        rect: Rect,
    },
}

/// Execute the viewport command.
pub fn execute(input: &Path, output: &Path, mode: ViewportMode) -> CommandResult<ViewportOutput> {
    let start = Instant::now();
    let input_str = input.display().to_string();

    // Guard
    if let Err(e) = guard::validate_input_path(input) {
        return CommandResult::err("viewport", input_str, e)
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }
    if let Err(e) = guard::validate_output_path(output) {
        return CommandResult::err("viewport", input_str, e)
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }
    if let Err(e) = guard::validate_different_paths(input, output) {
        return CommandResult::err("viewport", input_str, e)
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }

    // Load image
    let mut img = match image::open(input) {
        Ok(i) => i,
        Err(e) => {
            return CommandResult::err(
                "viewport",
                input_str,
                ErrorInfo::with_message(ErrorCode::UnsupportedFormat, e.to_string()),
            )
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
        }
    };

    let src_size = Size {
        width: img.width(),
        height: img.height(),
    };

    // Guard: pixel limit
    if let Err(e) = guard::validate_dimensions(src_size.width, src_size.height) {
        return CommandResult::err("viewport", input_str, e)
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }

    // Resolve crop region based on mode
    let (crop_rect, mode_name, params) = match mode {
        ViewportMode::Anchor {
            anchor,
            width,
            height,
        } => {
            // Validate dimensions
            if width == 0 || height == 0 {
                return CommandResult::err(
                    "viewport",
                    input_str,
                    ErrorInfo::with_message(
                        ErrorCode::InvalidDimensions,
                        "viewport width and height must be > 0",
                    ),
                )
                .with_elapsed_ms(start.elapsed().as_millis() as u64);
            }
            let rect = coord::anchor_to_rect(anchor, width, height, src_size);
            let params = serde_json::json!({
                "anchor": format!("{:?}", anchor),
                "width": width,
                "height": height,
            });
            (rect, "anchor", params)
        }
        ViewportMode::Percent { pct } => {
            // Validate percent bounds
            if pct.x < 0.0 || pct.y < 0.0 || pct.w <= 0.0 || pct.h <= 0.0 {
                return CommandResult::err(
                    "viewport",
                    input_str,
                    ErrorInfo::with_message(
                        ErrorCode::InvalidCoordinates,
                        "percent x,y must be >= 0 and w,h must be > 0",
                    ),
                )
                .with_elapsed_ms(start.elapsed().as_millis() as u64);
            }
            let rect = coord::percent_to_rect(pct, src_size);
            let params = serde_json::json!({
                "x": pct.x, "y": pct.y, "w": pct.w, "h": pct.h
            });
            (rect, "percent", params)
        }
        ViewportMode::Rect { rect } => {
            // Validate rect bounds
            if rect.width == 0 || rect.height == 0 {
                return CommandResult::err(
                    "viewport",
                    input_str,
                    ErrorInfo::with_message(
                        ErrorCode::InvalidDimensions,
                        "rect width and height must be > 0",
                    ),
                )
                .with_elapsed_ms(start.elapsed().as_millis() as u64);
            }
            if rect.right() > src_size.width || rect.bottom() > src_size.height {
                return CommandResult::err(
                    "viewport",
                    input_str,
                    ErrorInfo::with_message(
                        ErrorCode::InvalidCoordinates,
                        format!(
                            "rect ({},{},{},{}) exceeds source ({},{})",
                            rect.x,
                            rect.y,
                            rect.width,
                            rect.height,
                            src_size.width,
                            src_size.height
                        ),
                    ),
                )
                .with_elapsed_ms(start.elapsed().as_millis() as u64);
            }
            let params = serde_json::json!({
                "x": rect.x, "y": rect.y, "width": rect.width, "height": rect.height
            });
            (rect, "rect", params)
        }
    };

    // Crop
    let cropped = img.crop(crop_rect.x, crop_rect.y, crop_rect.width, crop_rect.height);

    // Save
    if let Err(e) = cropped.save(output) {
        return CommandResult::err(
            "viewport",
            input_str,
            ErrorInfo::with_message(ErrorCode::OutputWriteError, e.to_string()),
        )
        .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }

    let file_meta = std::fs::metadata(input).unwrap();
    let result_size = Size {
        width: cropped.width(),
        height: cropped.height(),
    };
    let mapping = coord::make_mapping(crop_rect, src_size, result_size);

    let data = ViewportOutput {
        output: output.display().to_string(),
        source: SourceInfo {
            width: src_size.width,
            height: src_size.height,
            format: crate::inspect::infer_format(input),
            size_bytes: file_meta.len(),
        },
        crop: CropInfo {
            mode: mode_name.to_string(),
            region: crop_rect,
            params,
        },
        result: result_size,
        coordinate_mapping: mapping,
    };

    CommandResult::ok("viewport", input_str, data)
        .with_elapsed_ms(start.elapsed().as_millis() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("fixtures")
            .join(name)
    }

    #[test]
    fn viewport_anchor_right() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("crop.png");
        let result = execute(
            &fixture("1000x1000.png"),
            &out,
            ViewportMode::Anchor {
                anchor: Anchor::Right,
                width: 500,
                height: 1000,
            },
        );
        assert!(result.ok, "error: {:?}", result.error);
        let data = result.data.unwrap();
        assert_eq!(data.crop.region.x, 500);
        assert_eq!(data.crop.region.width, 500);
        assert_eq!(data.result.width, 500);
        assert_eq!(data.result.height, 1000);
    }

    #[test]
    fn viewport_percent() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("crop.png");
        let result = execute(
            &fixture("1000x1000.png"),
            &out,
            ViewportMode::Percent {
                pct: Percent {
                    x: 0.0,
                    y: 0.0,
                    w: 0.5,
                    h: 0.5,
                },
            },
        );
        assert!(result.ok, "error: {:?}", result.error);
        let data = result.data.unwrap();
        assert_eq!(data.result.width, 500);
        assert_eq!(data.result.height, 500);
        assert_eq!(data.crop.mode, "percent");
    }

    #[test]
    fn viewport_rect() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("crop.png");
        let result = execute(
            &fixture("1000x1000.png"),
            &out,
            ViewportMode::Rect {
                rect: Rect {
                    x: 100,
                    y: 200,
                    width: 300,
                    height: 400,
                },
            },
        );
        assert!(result.ok, "error: {:?}", result.error);
        let data = result.data.unwrap();
        assert_eq!(data.result.width, 300);
        assert_eq!(data.result.height, 400);
        assert_eq!(data.crop.region.x, 100);
    }

    #[test]
    fn viewport_rect_out_of_bounds() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("crop.png");
        let result = execute(
            &fixture("256x256.png"),
            &out,
            ViewportMode::Rect {
                rect: Rect {
                    x: 200,
                    y: 200,
                    width: 100,
                    height: 100,
                },
            },
        );
        assert!(!result.ok);
        assert_eq!(result.error.unwrap().code, "INVALID_COORDINATES");
    }

    #[test]
    fn viewport_zero_dimensions() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("crop.png");
        let result = execute(
            &fixture("256x256.png"),
            &out,
            ViewportMode::Anchor {
                anchor: Anchor::Center,
                width: 0,
                height: 100,
            },
        );
        assert!(!result.ok);
        assert_eq!(result.error.unwrap().code, "INVALID_DIMENSIONS");
    }

    #[test]
    fn viewport_rejects_same_path() {
        let f = fixture("64x64.png");
        let result = execute(
            &f,
            &f,
            ViewportMode::Rect {
                rect: Rect {
                    x: 0,
                    y: 0,
                    width: 32,
                    height: 32,
                },
            },
        );
        assert!(!result.ok);
        assert_eq!(result.error.unwrap().code, "OUTPUT_SAME_AS_INPUT");
    }
}
