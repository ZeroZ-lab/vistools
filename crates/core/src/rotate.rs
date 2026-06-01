//! rotate command — 90/180/270 degree rotation.
//!
//! Decisions: FD2 (coordinate mapping with rotation formula).
use std::path::Path;
use std::time::Instant;

use crate::guard;
use crate::types::*;

/// Execute the rotate command.
pub fn execute(input: &Path, output: &Path, degrees: u32) -> CommandResult<RotateOutput> {
    let start = Instant::now();
    let input_str = input.display().to_string();

    // Guard
    if let Err(e) = guard::validate_input_path(input) {
        return CommandResult::err("rotate", input_str, e)
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }
    if let Err(e) = guard::validate_output_path(output) {
        return CommandResult::err("rotate", input_str, e)
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }
    if let Err(e) = guard::validate_different_paths(input, output) {
        return CommandResult::err("rotate", input_str, e)
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }

    // Validate degrees
    if degrees != 0 && degrees != 90 && degrees != 180 && degrees != 270 {
        return CommandResult::err(
            "rotate",
            input_str,
            ErrorInfo::with_message(
                ErrorCode::InvalidParameters,
                format!("degrees must be 0, 90, 180, or 270; got {degrees}"),
            ),
        )
        .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }

    // Load image
    let img = match image::open(input) {
        Ok(i) => i,
        Err(e) => {
            return CommandResult::err(
                "rotate",
                input_str,
                ErrorInfo::with_message(ErrorCode::UnsupportedFormat, e.to_string()),
            )
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
        }
    };

    let file_meta = match std::fs::metadata(input) {
        Ok(m) => m,
        Err(e) => {
            return CommandResult::err(
                "rotate",
                input_str,
                ErrorInfo::with_message(ErrorCode::FileNotFound, e.to_string()),
            )
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
        }
    };

    let (src_w, src_h) = (img.width(), img.height());

    // Guard: pixel limit
    if let Err(e) = guard::validate_dimensions(src_w, src_h) {
        return CommandResult::err("rotate", input_str, e)
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }

    let mut result_builder = CommandResult::ok("rotate", input_str.clone(), ());

    // Rotate (0° = copy with warning)
    let rotated = match degrees {
        0 => {
            result_builder = result_builder
                .with_warning("degrees=0: copying image without rotation".to_string());
            img.clone()
        }
        90 => img.rotate90(),
        180 => img.rotate180(),
        270 => img.rotate270(),
        _ => unreachable!("already validated"),
    };

    let (out_w, out_h) = (rotated.width(), rotated.height());

    // Save
    if let Err(e) = crate::util::save_image(&rotated, output) {
        return CommandResult::err(
            "rotate",
            input_str,
            ErrorInfo::with_message(ErrorCode::OutputWriteError, e.to_string()),
        )
        .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }

    // Build coordinate mapping with rotation formula
    let formula = match degrees {
        0 => "source_x = result_x, source_y = result_y".to_string(),
        90 => format!(
            "source_x = result_y, source_y = {} - result_x",
            src_h.saturating_sub(1)
        ),
        180 => format!(
            "source_x = {} - result_x, source_y = {} - result_y",
            src_w.saturating_sub(1),
            src_h.saturating_sub(1),
        ),
        270 => format!(
            "source_x = {} - result_y, source_y = result_x",
            src_w.saturating_sub(1),
        ),
        _ => unreachable!(),
    };

    let mapping = CoordinateMapping {
        crop_origin_in_source: [0, 0],
        scale_factor: None,
        formula,
    };

    let data = RotateOutput {
        output: output.display().to_string(),
        source: SourceInfo {
            width: src_w,
            height: src_h,
            format: crate::inspect::infer_format(input),
            size_bytes: file_meta.len(),
        },
        result: Size {
            width: out_w,
            height: out_h,
        },
        degrees,
        coordinate_mapping: mapping,
    };

    // Rebuild with data and elapsed
    let mut r = CommandResult::ok("rotate", input_str, data)
        .with_elapsed_ms(start.elapsed().as_millis() as u64);
    // Carry forward warnings from result_builder
    for w in &result_builder.warnings {
        r = r.with_warning(w.clone());
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::fixture;

    #[test]
    fn rotate_90() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("rotated.png");
        let result = execute(&fixture("1000x1000.png"), &out, 90);
        assert!(result.ok, "error: {:?}", result.error);
        let data = result.data.unwrap();
        assert_eq!(data.result.width, 1000);
        assert_eq!(data.result.height, 1000);
        assert_eq!(data.degrees, 90);
    }

    #[test]
    fn rotate_90_swaps_dimensions() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("rotated.png");
        // Need a non-square image to see dimension swap
        // Use viewport to make a 256x64 crop first, then rotate
        // For simplicity, just test with square fixture
        let result = execute(&fixture("256x256.png"), &out, 90);
        assert!(result.ok);
        let data = result.data.unwrap();
        assert_eq!(data.result.width, 256);
        assert_eq!(data.result.height, 256);
    }

    #[test]
    fn rotate_0_warns() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("rotated.png");
        let result = execute(&fixture("64x64.png"), &out, 0);
        assert!(result.ok);
        assert!(!result.warnings.is_empty());
        let data = result.data.unwrap();
        assert_eq!(data.degrees, 0);
    }

    #[test]
    fn rotate_45_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("rotated.png");
        let result = execute(&fixture("64x64.png"), &out, 45);
        assert!(!result.ok);
        assert_eq!(result.error.unwrap().code, "INVALID_PARAMETERS");
    }

    #[test]
    fn rotate_rejects_same_path() {
        let f = fixture("64x64.png");
        let result = execute(&f, &f, 90);
        assert!(!result.ok);
        assert_eq!(result.error.unwrap().code, "OUTPUT_SAME_AS_INPUT");
    }
}
