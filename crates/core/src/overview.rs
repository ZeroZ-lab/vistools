//! overview command — scale down to a preview image.
//!
//! Decisions: FD6 (output format from extension), FD2 (coordinate mapping).
use std::path::Path;
use std::time::Instant;

use crate::coord;
use crate::types::*;

/// Execute the overview command.
pub fn execute(input: &Path, output: &Path, max_width: u32) -> CommandResult<OverviewOutput> {
    let start = Instant::now();
    let input_str = input.display().to_string();

    // Guard
    if let Err(e) = crate::guard::validate_input_path(input) {
        return CommandResult::err("overview", input_str, e)
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }
    if let Err(e) = crate::guard::validate_output_path(output) {
        return CommandResult::err("overview", input_str, e)
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }
    if let Err(e) = crate::guard::validate_different_paths(input, output) {
        return CommandResult::err("overview", input_str, e)
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }

    // Load image
    let img = match image::open(input) {
        Ok(i) => i,
        Err(e) => {
            return CommandResult::err(
                "overview",
                input_str,
                ErrorInfo::with_message(ErrorCode::UnsupportedFormat, e.to_string()),
            )
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
        }
    };

    let (src_w, src_h) = (img.width(), img.height());

    // Guard: pixel limit
    if let Err(e) = crate::guard::validate_dimensions(src_w, src_h) {
        return CommandResult::err("overview", input_str, e)
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }

    // If max_width >= source width, just copy with a warning
    let (out_w, out_h, scale_factor, warning) = if max_width >= src_w {
        let w = Some(format!(
            "max_width ({max_width}) >= source width ({src_w}); copying without scaling",
        ));
        (src_w, src_h, 1.0, w)
    } else {
        let ratio = max_width as f64 / src_w as f64;
        let out_h = (src_h as f64 * ratio).round() as u32;
        (max_width, out_h, ratio, None)
    };

    // Resize using thumbnail for fast preview
    let resized = if scale_factor < 1.0 {
        img.thumbnail(out_w, out_h)
    } else {
        img.clone()
    };

    // Save output
    if let Err(e) = save_image(&resized, output) {
        return CommandResult::err(
            "overview",
            input_str,
            ErrorInfo::with_message(ErrorCode::OutputWriteError, e.to_string()),
        )
        .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }

    let file_meta = std::fs::metadata(input).unwrap_or_else(|_| std::fs::metadata(input).unwrap());

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

    let data = OverviewOutput {
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

    let mut r = CommandResult::ok("overview", input_str, data)
        .with_elapsed_ms(start.elapsed().as_millis() as u64);
    if let Some(w) = warning {
        r = r.with_warning(w);
    }
    r
}

/// Save an image to the specified path, inferring format from extension.
/// FD6: format from output extension.
fn save_image(img: &image::DynamicImage, path: &Path) -> Result<(), String> {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
    {
        Some(e) if e == "png" => img.save(path).map_err(|e| e.to_string()),
        Some(e) if e == "jpg" || e == "jpeg" => {
            let mut buf =
                std::io::BufWriter::new(std::fs::File::create(path).map_err(|e| e.to_string())?);
            img.write_to(&mut buf, image::ImageFormat::Jpeg)
                .map_err(|e| e.to_string())
        }
        Some(e) if e == "webp" => img.save(path).map_err(|e| e.to_string()),
        _ => img.save(path).map_err(|e| e.to_string()),
    }
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
    fn overview_scales_down() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("overview.png");
        let result = execute(&fixture("1000x1000.png"), &out, 200);
        assert!(result.ok, "error: {:?}", result.error);
        let data = result.data.unwrap();
        assert_eq!(data.result.width, 200);
        assert_eq!(data.result.height, 200);
        assert!((data.scale_factor - 0.2).abs() < 0.01);
        assert!(out.exists());
    }

    #[test]
    fn overview_warns_when_no_scale_needed() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("overview.png");
        let result = execute(&fixture("64x64.png"), &out, 200);
        assert!(result.ok);
        assert!(!result.warnings.is_empty());
        let data = result.data.unwrap();
        assert_eq!(data.result.width, 64);
    }

    #[test]
    fn overview_rejects_same_path() {
        let f = fixture("64x64.png");
        let result = execute(&f, &f, 32);
        assert!(!result.ok);
        assert_eq!(result.error.unwrap().code, "OUTPUT_SAME_AS_INPUT");
    }
}
