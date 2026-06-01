//! inspect command — read image metadata + strategy suggestion.
//!
//! Decisions: FD7 (1568px overview threshold), PD1 (JSON-first).
use std::fs;
use std::path::Path;
use std::time::Instant;

use crate::types::*;

/// Execute the inspect command.
pub fn execute(input: &Path) -> CommandResult<InspectOutput> {
    let start = Instant::now();

    // Guard: validate input path
    if let Err(e) = crate::guard::validate_input_path(input) {
        return CommandResult::err("inspect", input.display().to_string(), e)
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }

    // Read file metadata
    let file_meta = match fs::metadata(input) {
        Ok(m) => m,
        Err(e) => {
            return CommandResult::err(
                "inspect",
                input.display().to_string(),
                ErrorInfo::with_message(ErrorCode::FileNotFound, e.to_string()),
            )
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
        }
    };

    // Read dimensions without loading full image (performance: <1ms)
    let (width, height) = match image::image_dimensions(input) {
        Ok(d) => d,
        Err(e) => {
            return CommandResult::err(
                "inspect",
                input.display().to_string(),
                ErrorInfo::with_message(ErrorCode::UnsupportedFormat, e.to_string()),
            )
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
        }
    };

    // Guard: pixel limit
    if let Err(e) = crate::guard::validate_dimensions(width, height) {
        return CommandResult::err("inspect", input.display().to_string(), e)
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }

    // Infer format from extension
    let format = infer_format(input);

    // FD7: strategy suggestion based on 1568px threshold
    let long_side = width.max(height);
    let needs_overview = long_side > OVERVIEW_THRESHOLD;

    // Calculate max tile grid that keeps each tile ≤ OVERVIEW_THRESHOLD
    let max_tile_cols = if width > OVERVIEW_THRESHOLD {
        (width as f64 / OVERVIEW_THRESHOLD as f64).ceil() as u32
    } else {
        1
    };
    let max_tile_rows = if height > OVERVIEW_THRESHOLD {
        (height as f64 / OVERVIEW_THRESHOLD as f64).ceil() as u32
    } else {
        1
    };

    let data = InspectOutput {
        source: SourceInfo {
            width,
            height,
            format,
            size_bytes: file_meta.len(),
        },
        suggestion: Suggestion {
            needs_overview,
            max_tile_rows,
            max_tile_cols,
        },
    };

    CommandResult::ok("inspect", input.display().to_string(), data)
        .with_elapsed_ms(start.elapsed().as_millis() as u64)
}

/// Infer image format from file extension.
pub fn infer_format(path: &Path) -> String {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
    {
        Some(e) if e == "png" => "png".to_string(),
        Some(e) if e == "jpg" || e == "jpeg" => "jpeg".to_string(),
        Some(e) if e == "webp" => "webp".to_string(),
        Some(e) if e == "tiff" || e == "tif" => "tiff".to_string(),
        Some(e) if e == "bmp" => "bmp".to_string(),
        Some(e) if e == "gif" => "gif".to_string(),
        _ => "unknown".to_string(),
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
    fn inspect_256x256() {
        let result = execute(&fixture("256x256.png"));
        assert!(result.ok);
        let data = result.data.unwrap();
        assert_eq!(data.source.width, 256);
        assert_eq!(data.source.height, 256);
        assert_eq!(data.source.format, "png");
        assert!(!data.suggestion.needs_overview);
    }

    #[test]
    fn inspect_1000x1000() {
        let result = execute(&fixture("1000x1000.png"));
        assert!(result.ok);
        let data = result.data.unwrap();
        assert!(!data.suggestion.needs_overview);
    }

    #[test]
    fn inspect_nonexistent() {
        let result = execute(Path::new("/tmp/__nonexistent_image_test__.png"));
        assert!(!result.ok);
        assert_eq!(result.error.unwrap().code, "FILE_NOT_FOUND");
    }

    #[test]
    fn infer_format_variants() {
        assert_eq!(infer_format(Path::new("test.png")), "png");
        assert_eq!(infer_format(Path::new("test.jpg")), "jpeg");
        assert_eq!(infer_format(Path::new("test.JPEG")), "jpeg");
        assert_eq!(infer_format(Path::new("test.webp")), "webp");
        assert_eq!(infer_format(Path::new("test.unknown")), "unknown");
    }
}
