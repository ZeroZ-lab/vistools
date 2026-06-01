//! tile command — grid-based image splitting.
//!
//! Decisions: FD5 (last tile includes remainder pixels), FD6 (output format).
use std::path::Path;
use std::time::Instant;

use crate::guard;
use crate::types::*;

/// Execute the tile command.
pub fn execute(input: &Path, rows: u32, cols: u32, out_dir: &Path) -> CommandResult<TileOutput> {
    let start = Instant::now();
    let input_str = input.display().to_string();

    // Guard
    if let Err(e) = guard::validate_input_path(input) {
        return CommandResult::err("tile", input_str, e)
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }
    if let Err(e) = guard::validate_output_path(out_dir) {
        return CommandResult::err("tile", input_str, e)
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }
    if let Err(e) = guard::validate_tile_count(rows, cols) {
        return CommandResult::err("tile", input_str, e)
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }

    // Load image
    let mut img = match image::open(input) {
        Ok(i) => i,
        Err(e) => {
            return CommandResult::err(
                "tile",
                input_str,
                ErrorInfo::with_message(ErrorCode::UnsupportedFormat, e.to_string()),
            )
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
        }
    };

    let (src_w, src_h) = (img.width(), img.height());

    // Guard: pixel limit
    if let Err(e) = guard::validate_dimensions(src_w, src_h) {
        return CommandResult::err("tile", input_str, e)
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }

    // Create output directory if needed
    if let Err(e) = std::fs::create_dir_all(out_dir) {
        return CommandResult::err(
            "tile",
            input_str,
            ErrorInfo::with_message(ErrorCode::OutputWriteError, e.to_string()),
        )
        .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }

    // FD5: remainder strategy — last tile per row/col includes remainder pixels
    let base_tile_w = src_w / cols;
    let base_tile_h = src_h / rows;
    let remainder_w = src_w % cols;
    let remainder_h = src_h % rows;

    let ext = input
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_string();

    let mut tiles = Vec::new();
    let mut y = 0u32;

    for row in 0..rows {
        // FD5: last row gets extra height from remainder
        let tile_h = base_tile_h + if row == rows - 1 { remainder_h } else { 0 };
        let mut x = 0u32;

        for col in 0..cols {
            // FD5: last col gets extra width from remainder
            let tile_w = base_tile_w + if col == cols - 1 { remainder_w } else { 0 };

            let cropped = img.crop(x, y, tile_w, tile_h);

            let filename = format!("row-{row}-col-{col}.{ext}");
            let tile_path = out_dir.join(&filename);

            if let Err(e) = cropped.save(&tile_path) {
                return CommandResult::err(
                    "tile",
                    input_str,
                    ErrorInfo::with_message(ErrorCode::OutputWriteError, e.to_string()),
                )
                .with_elapsed_ms(start.elapsed().as_millis() as u64);
            }

            tiles.push(TileInfo {
                path: tile_path.display().to_string(),
                row,
                col,
                width: tile_w,
                height: tile_h,
                source_region: Rect {
                    x,
                    y,
                    width: tile_w,
                    height: tile_h,
                },
            });

            x += tile_w;
        }
        y += tile_h;
    }

    let file_meta = std::fs::metadata(input).unwrap();

    let data = TileOutput {
        source: SourceInfo {
            width: src_w,
            height: src_h,
            format: crate::inspect::infer_format(input),
            size_bytes: file_meta.len(),
        },
        rows,
        cols,
        tiles,
    };

    CommandResult::ok("tile", input_str, data).with_elapsed_ms(start.elapsed().as_millis() as u64)
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
    fn tile_2x2_on_1000x1000() {
        let dir = tempfile::tempdir().unwrap();
        let result = execute(&fixture("1000x1000.png"), 2, 2, dir.path());
        assert!(result.ok, "error: {:?}", result.error);
        let data = result.data.unwrap();
        assert_eq!(data.tiles.len(), 4);
        // Each tile should be 500x500
        for t in &data.tiles {
            assert_eq!(t.width, 500);
            assert_eq!(t.height, 500);
        }
    }

    #[test]
    fn tile_3x2_with_remainder() {
        // 1000px / 3 cols → base=333, remainder=1 → last col=334
        let dir = tempfile::tempdir().unwrap();
        let result = execute(&fixture("1000x1000.png"), 2, 3, dir.path());
        assert!(result.ok, "error: {:?}", result.error);
        let data = result.data.unwrap();
        assert_eq!(data.tiles.len(), 6);

        // Verify remainder: last col should be wider
        let last_col_tile = data.tiles.iter().find(|t| t.col == 2).unwrap();
        assert_eq!(last_col_tile.width, 334); // 333 + 1 remainder
    }

    #[test]
    fn tile_rejects_excessive_count() {
        let dir = tempfile::tempdir().unwrap();
        let result = execute(&fixture("1000x1000.png"), 10, 10, dir.path());
        assert!(!result.ok);
        assert_eq!(result.error.unwrap().code, "INVALID_PARAMETERS");
    }

    #[test]
    fn tile_full_coverage() {
        // FD5: all tiles must cover the full source image seamlessly
        let dir = tempfile::tempdir().unwrap();
        let result = execute(&fixture("256x256.png"), 3, 4, dir.path());
        assert!(result.ok);
        let data = result.data.unwrap();

        // Sum of all tile areas must equal source area
        let total_area: u64 = data.tiles.iter().map(|t| t.source_region.area()).sum();
        assert_eq!(total_area, (256 * 256) as u64);
    }
}
