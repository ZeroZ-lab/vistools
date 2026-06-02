//! viewport command — three crop modes (anchor / percent / rect).
//!
//! Decisions: FD2 (coordinate mapping), PD2 (unified coordinate system).
use std::path::Path;
use std::time::Instant;

use crate::error::{ErrorCode, ErrorInfo};
use crate::geom::{Anchor, Percent, Rect, Size};
use crate::guard;
use crate::protocol::{CommandResult, CropInfo, ViewportOutput};
use crate::region;
use crate::source;

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

    let source = match source::load_image_source(input) {
        Ok(source) => source,
        Err(error) => {
            return CommandResult::err("viewport", input_str, error)
                .with_elapsed_ms(start.elapsed().as_millis() as u64);
        }
    };
    let mut img = source.image;
    let src_size = Size {
        width: source.info.width,
        height: source.info.height,
    };

    // Resolve crop region based on mode
    let (crop_rect, spec, vp_warning) = match mode {
        ViewportMode::Anchor {
            anchor,
            width,
            height,
        } => match region::resolve_anchor(anchor, Size { width, height }, src_size) {
            Ok((rect, spec, warning)) => (rect, spec, warning),
            Err(error) => {
                return CommandResult::err("viewport", input_str, error)
                    .with_elapsed_ms(start.elapsed().as_millis() as u64);
            }
        },
        ViewportMode::Percent { pct } => match region::resolve_percent(pct, src_size) {
            Ok((rect, spec)) => (rect, spec, None),
            Err(error) => {
                return CommandResult::err("viewport", input_str, error)
                    .with_elapsed_ms(start.elapsed().as_millis() as u64);
            }
        },
        ViewportMode::Rect { rect } => match region::resolve_rect(rect, src_size) {
            Ok((rect, spec)) => (rect, spec, None),
            Err(error) => {
                return CommandResult::err("viewport", input_str, error)
                    .with_elapsed_ms(start.elapsed().as_millis() as u64);
            }
        },
    };

    // Crop
    let cropped = img.crop(crop_rect.x, crop_rect.y, crop_rect.width, crop_rect.height);

    // Save
    if let Err(e) = crate::util::save_image(&cropped, output) {
        return CommandResult::err(
            "viewport",
            input_str,
            ErrorInfo::with_message(ErrorCode::OutputWriteError, e.to_string()),
        )
        .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }

    let result_size = Size {
        width: cropped.width(),
        height: cropped.height(),
    };
    let mapping = region::coordinate_mapping(crop_rect, result_size);

    let data = ViewportOutput {
        output: output.display().to_string(),
        source: source.info,
        crop: CropInfo {
            spec,
            region: crop_rect,
        },
        result: result_size,
        coordinate_mapping: mapping,
    };

    let mut r = CommandResult::ok("viewport", input_str, data)
        .with_elapsed_ms(start.elapsed().as_millis() as u64);
    if let Some(w) = vp_warning {
        r = r.with_warning(w);
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::fixture;

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
        match data.crop.spec {
            crate::protocol::CropSpec::Percent { .. } => {}
            _ => panic!("expected percent crop spec"),
        }
    }

    #[test]
    fn viewport_percent_rejects_out_of_range_values() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("crop.png");
        let result = execute(
            &fixture("1000x1000.png"),
            &out,
            ViewportMode::Percent {
                pct: Percent {
                    x: 0.0,
                    y: 0.0,
                    w: 1.5,
                    h: 0.5,
                },
            },
        );
        assert!(!result.ok);
        assert_eq!(result.error.unwrap().code, "INVALID_PARAMETERS");
    }

    #[test]
    fn viewport_percent_rejects_region_overflow() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("crop.png");
        let result = execute(
            &fixture("1000x1000.png"),
            &out,
            ViewportMode::Percent {
                pct: Percent {
                    x: 0.8,
                    y: 0.0,
                    w: 0.3,
                    h: 0.5,
                },
            },
        );
        assert!(!result.ok);
        assert_eq!(result.error.unwrap().code, "INVALID_COORDINATES");
    }

    #[test]
    fn viewport_percent_rejects_nan() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("crop.png");
        let result = execute(
            &fixture("1000x1000.png"),
            &out,
            ViewportMode::Percent {
                pct: Percent {
                    x: f64::NAN,
                    y: 0.0,
                    w: 0.5,
                    h: 0.5,
                },
            },
        );
        assert!(!result.ok);
        assert_eq!(result.error.unwrap().code, "INVALID_PARAMETERS");
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

    #[test]
    fn viewport_warns_when_larger_than_source() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("crop.png");
        let result = execute(
            &fixture("64x64.png"),
            &out,
            ViewportMode::Anchor {
                anchor: Anchor::Center,
                width: 200,
                height: 200,
            },
        );
        assert!(result.ok, "error: {:?}", result.error);
        assert!(!result.warnings.is_empty());
        assert!(result.warnings[0].contains("exceeds source"));
        let data = result.data.unwrap();
        assert_eq!(data.crop.region.width, 64);
        assert_eq!(data.crop.region.height, 64);
        assert_eq!(data.result.width, 64);
        assert_eq!(data.result.height, 64);
        assert_eq!(data.coordinate_mapping.source_origin.x, 0);
        assert_eq!(data.coordinate_mapping.source_origin.y, 0);
    }
}
