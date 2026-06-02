//! Pixel diff measurement command for agent-readable visual comparison.
use std::path::Path;
use std::time::Instant;

use crate::error::{ErrorCode, ErrorInfo};
use crate::geom::{Rect, Size};
use crate::protocol::{CommandResult, DiffMetrics, DiffOutput};
use crate::region;
use crate::source;

pub fn execute_diff(
    expected: &Path,
    actual: &Path,
    rect: Option<Rect>,
) -> CommandResult<DiffOutput> {
    let start = Instant::now();
    let input_str = format!("{} {}", expected.display(), actual.display());

    let expected_loaded = match source::load_rgba_source(expected) {
        Ok(loaded) => loaded,
        Err(error) => {
            return CommandResult::err("diff", input_str, error)
                .with_elapsed_ms(start.elapsed().as_millis() as u64);
        }
    };
    let actual_loaded = match source::load_rgba_source(actual) {
        Ok(loaded) => loaded,
        Err(error) => {
            return CommandResult::err("diff", input_str, error)
                .with_elapsed_ms(start.elapsed().as_millis() as u64);
        }
    };

    if expected_loaded.info.width != actual_loaded.info.width
        || expected_loaded.info.height != actual_loaded.info.height
    {
        return CommandResult::err(
            "diff",
            input_str,
            ErrorInfo::with_message(
                ErrorCode::InvalidDimensions,
                "expected and actual image dimensions must match",
            ),
        )
        .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }

    let region = rect.unwrap_or(Rect {
        x: 0,
        y: 0,
        width: expected_loaded.info.width,
        height: expected_loaded.info.height,
    });
    if let Err(error) = region::validate_rect(
        region,
        Size {
            width: expected_loaded.info.width,
            height: expected_loaded.info.height,
        },
        "rect",
    ) {
        return CommandResult::err("diff", input_str, error)
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }

    // From: v1-agent-command-surface contract. `diff` is a measurement primitive with source coordinates.
    let diff = measure_diff(&expected_loaded.image, &actual_loaded.image, region);
    let data = DiffOutput {
        expected_source: expected_loaded.info,
        actual_source: actual_loaded.info,
        region,
        diff,
    };

    CommandResult::ok("diff", input_str, data).with_elapsed_ms(start.elapsed().as_millis() as u64)
}

fn measure_diff(
    expected: &image::RgbaImage,
    actual: &image::RgbaImage,
    region: Rect,
) -> DiffMetrics {
    let mut changed_pixels = 0u64;
    let mut total_delta = 0.0;
    let mut max_delta = 0.0_f64;
    let mut min_x = u32::MAX;
    let mut min_y = u32::MAX;
    let mut max_x = 0u32;
    let mut max_y = 0u32;

    for y in region.y..region.bottom() {
        for x in region.x..region.right() {
            let expected_pixel = expected.get_pixel(x, y).0;
            let actual_pixel = actual.get_pixel(x, y).0;
            let delta = pixel_delta(expected_pixel, actual_pixel);
            total_delta += delta;
            max_delta = max_delta.max(delta);

            if expected_pixel != actual_pixel {
                changed_pixels += 1;
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }

    let pixel_count = region.area();
    let bounding_rect = if changed_pixels == 0 {
        None
    } else {
        Some(Rect {
            x: min_x,
            y: min_y,
            width: max_x - min_x + 1,
            height: max_y - min_y + 1,
        })
    };

    DiffMetrics {
        pixel_count,
        changed_pixels,
        changed_ratio: if pixel_count == 0 {
            0.0
        } else {
            changed_pixels as f64 / pixel_count as f64
        },
        mean_delta: if pixel_count == 0 {
            0.0
        } else {
            total_delta / pixel_count as f64
        },
        max_delta,
        bounding_rect,
    }
}

fn pixel_delta(expected: [u8; 4], actual: [u8; 4]) -> f64 {
    expected
        .iter()
        .zip(actual)
        .map(|(a, b)| (*a as f64 - b as f64).abs())
        .sum::<f64>()
        / 4.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    fn write_temp_image(img: ImageBuffer<Rgba<u8>, Vec<u8>>) -> tempfile::TempPath {
        let file = tempfile::Builder::new().suffix(".png").tempfile().unwrap();
        img.save(file.path()).unwrap();
        file.into_temp_path()
    }

    #[test]
    fn diff_reports_no_changes_for_identical_images() {
        let img = ImageBuffer::from_pixel(4, 4, Rgba([128, 128, 128, 255]));
        let expected = write_temp_image(img.clone());
        let actual = write_temp_image(img);

        let result = execute_diff(expected.as_ref(), actual.as_ref(), None);
        assert!(result.ok);
        let diff = result.data.unwrap().diff;
        assert_eq!(diff.pixel_count, 16);
        assert_eq!(diff.changed_pixels, 0);
        assert_eq!(diff.changed_ratio, 0.0);
        assert!(diff.bounding_rect.is_none());
    }

    #[test]
    fn diff_reports_changed_pixels_and_bounds() {
        let expected = ImageBuffer::from_pixel(4, 4, Rgba([0, 0, 0, 255]));
        let actual = ImageBuffer::from_fn(4, 4, |x, y| {
            if (x == 1 && y == 1) || (x == 2 && y == 3) {
                Rgba([255, 0, 0, 255])
            } else {
                Rgba([0, 0, 0, 255])
            }
        });

        let result = execute_diff(
            write_temp_image(expected).as_ref(),
            write_temp_image(actual).as_ref(),
            None,
        );
        assert!(result.ok);
        let diff = result.data.unwrap().diff;
        assert_eq!(diff.changed_pixels, 2);
        assert_eq!(
            diff.bounding_rect,
            Some(Rect {
                x: 1,
                y: 1,
                width: 2,
                height: 3
            })
        );
        assert!(diff.max_delta > 0.0);
    }

    #[test]
    fn diff_respects_rect_region() {
        let expected = ImageBuffer::from_pixel(4, 4, Rgba([0, 0, 0, 255]));
        let actual = ImageBuffer::from_fn(4, 4, |x, _| {
            if x >= 2 {
                Rgba([255, 255, 255, 255])
            } else {
                Rgba([0, 0, 0, 255])
            }
        });

        let result = execute_diff(
            write_temp_image(expected).as_ref(),
            write_temp_image(actual).as_ref(),
            Some(Rect {
                x: 0,
                y: 0,
                width: 2,
                height: 4,
            }),
        );
        assert!(result.ok);
        let diff = result.data.unwrap().diff;
        assert_eq!(diff.pixel_count, 8);
        assert_eq!(diff.changed_pixels, 0);
    }

    #[test]
    fn diff_rejects_mismatched_dimensions() {
        let expected = ImageBuffer::from_pixel(4, 4, Rgba([0, 0, 0, 255]));
        let actual = ImageBuffer::from_pixel(5, 4, Rgba([0, 0, 0, 255]));

        let result = execute_diff(
            write_temp_image(expected).as_ref(),
            write_temp_image(actual).as_ref(),
            None,
        );
        assert!(!result.ok);
        assert_eq!(result.error.unwrap().code, "INVALID_DIMENSIONS");
    }
}
