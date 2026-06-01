//! Agent-safe input validation.
//!
//! All commands must call guard functions before processing.
//! Decisions: PD3 (Agent-safe), FD4 (centralized guard).
use std::path::Path;

use crate::types::{ErrorCode, ErrorInfo};

/// Rejects paths containing `..` components and checks the file exists.
/// PD3: path sandbox — no directory traversal.
pub fn validate_input_path(path: &Path) -> Result<(), ErrorInfo> {
    // PD3: reject `..` to prevent path escape
    if path.components().any(|c| c.as_os_str() == "..") {
        return Err(ErrorInfo::with_message(
            ErrorCode::PathEscape,
            format!("input path contains '..': {}", path.display()),
        ));
    }
    if !path.exists() {
        return Err(ErrorInfo::with_message(
            ErrorCode::FileNotFound,
            format!("input file not found: {}", path.display()),
        ));
    }
    if !path.is_file() {
        return Err(ErrorInfo::with_message(
            ErrorCode::FileNotFound,
            format!("input path is not a file: {}", path.display()),
        ));
    }
    Ok(())
}

/// Rejects paths containing `..` components.
/// PD3: output path sandbox.
pub fn validate_output_path(path: &Path) -> Result<(), ErrorInfo> {
    if path.components().any(|c| c.as_os_str() == "..") {
        return Err(ErrorInfo::with_message(
            ErrorCode::PathEscape,
            format!("output path contains '..': {}", path.display()),
        ));
    }
    Ok(())
}

/// Ensures output path differs from input path.
/// PD3: never overwrite source files.
pub fn validate_different_paths(input: &Path, output: &Path) -> Result<(), ErrorInfo> {
    // Resolve to canonical paths if both exist, otherwise compare directly
    let input_canon = input.canonicalize().ok();
    let output_canon = if output.exists() {
        output.canonicalize().ok()
    } else {
        None
    };

    let same = match (input_canon, output_canon) {
        (Some(i), Some(o)) => i == o,
        _ => input == output,
    };

    if same {
        return Err(ErrorInfo::with_message(
            ErrorCode::OutputSameAsInput,
            "output path must differ from input path",
        ));
    }
    Ok(())
}

/// Rejects images exceeding the pixel limit (100 MP).
/// PD3: pixel limit.
pub fn validate_dimensions(width: u32, height: u32) -> Result<(), ErrorInfo> {
    let pixels = (width as u64) * (height as u64);
    if pixels > crate::types::MAX_PIXELS {
        return Err(ErrorInfo::with_message(
            ErrorCode::PixelLimitExceeded,
            format!(
                "image is {}x{} ({}MP), exceeds {}MP limit",
                width,
                height,
                pixels / 1_000_000,
                crate::types::MAX_PIXELS / 1_000_000,
            ),
        ));
    }
    Ok(())
}

/// Rejects tile grids exceeding the tile count limit (64).
/// PD3: tile count limit.
pub fn validate_tile_count(rows: u32, cols: u32) -> Result<(), ErrorInfo> {
    let count = (rows as u64) * (cols as u64);
    if count > crate::types::MAX_TILE_COUNT as u64 {
        return Err(ErrorInfo::with_message(
            ErrorCode::InvalidParameters,
            format!(
                "tile grid {}x{} = {} tiles, exceeds {} limit",
                rows,
                cols,
                count,
                crate::types::MAX_TILE_COUNT,
            ),
        ));
    }
    if rows == 0 || cols == 0 {
        return Err(ErrorInfo::with_message(
            ErrorCode::InvalidParameters,
            "rows and cols must be >= 1",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn reject_dotdot_path() {
        let p = Path::new("../etc/passwd");
        let err = validate_input_path(p).unwrap_err();
        assert_eq!(err.code, "PATH_ESCAPE");
    }

    #[test]
    fn reject_nonexistent_file() {
        let p = Path::new("/tmp/vistools_nonexistent_test_file.png");
        let err = validate_input_path(p).unwrap_err();
        assert_eq!(err.code, "FILE_NOT_FOUND");
    }

    #[test]
    fn accept_valid_fixture() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let p = Path::new(&manifest_dir)
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("fixtures/64x64.png");
        assert!(validate_input_path(&p).is_ok());
    }

    #[test]
    fn reject_pixel_limit() {
        // 10,001 × 10,001 = ~100.02 MP > 100 MP
        let err = validate_dimensions(10_001, 10_001).unwrap_err();
        assert_eq!(err.code, "PIXEL_LIMIT_EXCEEDED");
    }

    #[test]
    fn accept_under_pixel_limit() {
        assert!(validate_dimensions(10_000, 10_000).is_ok());
    }

    #[test]
    fn reject_tile_count() {
        let err = validate_tile_count(10, 10).unwrap_err();
        assert_eq!(err.code, "INVALID_PARAMETERS");
    }

    #[test]
    fn accept_valid_tile_count() {
        assert!(validate_tile_count(8, 8).is_ok());
    }

    #[test]
    fn reject_same_input_output() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.png");
        fs::write(&file_path, b"fake").unwrap();
        let err = validate_different_paths(&file_path, &file_path).unwrap_err();
        assert_eq!(err.code, "OUTPUT_SAME_AS_INPUT");
    }

    #[test]
    fn reject_zero_tiles() {
        let err = validate_tile_count(0, 5).unwrap_err();
        assert_eq!(err.code, "INVALID_PARAMETERS");
    }
}
