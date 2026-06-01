//! resize command — proportional or forced resize.
//!
//! Decisions: FD2 (coordinate mapping), FD6 (output format from extension).
use std::path::Path;
use std::time::Instant;

use crate::coord;
use crate::guard;
use crate::types::*;

/// Execute the resize command.
///
/// If only `width` is given, height is calculated proportionally.
/// If both are given, the image is forced to that size.
pub fn execute(
    input: &Path,
    output: &Path,
    width: u32,
    height: Option<u32>,
) -> CommandResult<ResizeOutput> {
    let start = Instant::now();
    let input_str = input.display().to_string();

    // Guard
    if let Err(e) = guard::validate_input_path(input) {
        return CommandResult::err("resize", input_str, e)
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }
    if let Err(e) = guard::validate_output_path(output) {
        return CommandResult::err("resize", input_str, e)
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }
    if let Err(e) = guard::validate_different_paths(input, output) {
        return CommandResult::err("resize", input_str, e)
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }
    if width == 0 {
        return CommandResult::err(
            "resize",
            input_str,
            ErrorInfo::with_message(ErrorCode::InvalidParameters, "width must be > 0"),
        )
        .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }
    if let Some(h) = height {
        if h == 0 {
            return CommandResult::err(
                "resize",
                input_str,
                ErrorInfo::with_message(ErrorCode::InvalidParameters, "height must be > 0"),
            )
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
        }
    }

    // Load image
    let img = match image::open(input) {
        Ok(i) => i,
        Err(e) => {
            return CommandResult::err(
                "resize",
                input_str,
                ErrorInfo::with_message(ErrorCode::UnsupportedFormat, e.to_string()),
            )
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
        }
    };

    let (src_w, src_h) = (img.width(), img.height());

    // Guard: pixel limit
    if let Err(e) = guard::validate_dimensions(src_w, src_h) {
        return CommandResult::err("resize", input_str, e)
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }

    // Calculate output dimensions
    let (out_w, out_h) = match height {
        Some(h) => (width, h),
        None => {
            let ratio = width as f64 / src_w as f64;
            let h = (src_h as f64 * ratio).round() as u32;
            (width, h.max(1))
        }
    };

    let resized = match height {
        // Forced resize: use resize_exact to hit exact dimensions
        Some(_) => img.resize_exact(
            out_w,
            out_h,
            image::imageops::FilterType::Lanczos3,
        ),
        // Proportional: thumbnail preserves aspect ratio
        None => img.thumbnail(out_w, out_h),
    };

    // Save
    if let Err(e) = resized.save(output) {
        return CommandResult::err(
            "resize",
            input_str,
            ErrorInfo::with_message(ErrorCode::OutputWriteError, e.to_string()),
        )
        .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }

    let file_meta = std::fs::metadata(input).unwrap();
    let scale_factor = out_w as f64 / src_w as f64;
    let source_rect = Rect {
        x: 0,
        y: 0,
        width: src_w,
        height: src_h,
    };
    let mapping = coord::make_mapping(
        source_rect,
        Size {
            width: src_w,
            height: src_h,
        },
        Size {
            width: out_w,
            height: out_h,
        },
    );

    let data = ResizeOutput {
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
        scale_factor,
        coordinate_mapping: mapping,
    };

    CommandResult::ok("resize", input_str, data).with_elapsed_ms(start.elapsed().as_millis() as u64)
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
    fn resize_proportional() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("resized.png");
        // 1000x1000 → width 200 → 200x200
        let result = execute(&fixture("1000x1000.png"), &out, 200, None);
        assert!(result.ok, "error: {:?}", result.error);
        let data = result.data.unwrap();
        assert_eq!(data.result.width, 200);
        assert_eq!(data.result.height, 200);
        assert!((data.scale_factor - 0.2).abs() < 0.01);
    }

    #[test]
    fn resize_forced() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("resized.png");
        let result = execute(&fixture("1000x1000.png"), &out, 800, Some(600));
        assert!(result.ok, "error: {:?}", result.error);
        let data = result.data.unwrap();
        assert_eq!(data.result.width, 800);
        assert_eq!(data.result.height, 600);
    }

    #[test]
    fn resize_rejects_zero_width() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("resized.png");
        let result = execute(&fixture("1000x1000.png"), &out, 0, None);
        assert!(!result.ok);
        assert_eq!(result.error.unwrap().code, "INVALID_PARAMETERS");
    }

    #[test]
    fn resize_rejects_same_path() {
        let f = fixture("64x64.png");
        let result = execute(&f, &f, 32, None);
        assert!(!result.ok);
        assert_eq!(result.error.unwrap().code, "OUTPUT_SAME_AS_INPUT");
    }
}
