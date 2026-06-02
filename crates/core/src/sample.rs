//! sample command — point and rect color sampling for LLM inspection.
//!
//! Decisions: PD1 (JSON-first), PD2 (source-coordinate output).
use std::path::Path;
use std::time::Instant;

use crate::error::{ErrorCode, ErrorInfo};
use crate::geom::{Point, Rect, Size};
use crate::protocol::{AlphaStats, ColorInfo, CommandResult, SampleOutput, SampleResult};
use crate::region;
use crate::source;

/// Color sampling mode.
#[derive(Debug, Clone, Copy)]
pub enum SampleMode {
    Point { x: u32, y: u32 },
    Rect { rect: Rect },
}

/// Execute the sample command.
pub fn execute(input: &Path, mode: SampleMode) -> CommandResult<SampleOutput> {
    let start = Instant::now();
    let input_str = input.display().to_string();

    let source = match source::load_rgba_source(input) {
        Ok(source) => source,
        Err(error) => {
            return CommandResult::err("sample", input_str, error)
                .with_elapsed_ms(start.elapsed().as_millis() as u64);
        }
    };
    let img = source.image;
    let src_size = Size {
        width: source.info.width,
        height: source.info.height,
    };

    let sample = match mode {
        SampleMode::Point { x, y } => {
            if x >= src_size.width || y >= src_size.height {
                return CommandResult::err(
                    "sample",
                    input_str,
                    ErrorInfo::with_message(
                        ErrorCode::InvalidCoordinates,
                        format!(
                            "point ({x},{y}) exceeds source ({}x{})",
                            src_size.width, src_size.height
                        ),
                    ),
                )
                .with_elapsed_ms(start.elapsed().as_millis() as u64);
            }

            let rgba = img.get_pixel(x, y).0;
            SampleResult::Point {
                point: Point { x, y },
                color: color_info(rgba),
            }
        }
        SampleMode::Rect { rect } => {
            if rect.is_empty() {
                return CommandResult::err(
                    "sample",
                    input_str,
                    ErrorInfo::with_message(
                        ErrorCode::InvalidDimensions,
                        "rect width and height must be > 0",
                    ),
                )
                .with_elapsed_ms(start.elapsed().as_millis() as u64);
            }
            if let Err(error) = region::validate_rect(rect, src_size, "rect") {
                return CommandResult::err("sample", input_str, error)
                    .with_elapsed_ms(start.elapsed().as_millis() as u64);
            }

            let (average, alpha_stats) = sample_rect(&img, rect);
            SampleResult::Rect {
                region: rect,
                average,
                alpha_stats,
                pixel_count: rect.area(),
            }
        }
    };

    let data = SampleOutput {
        source: source.info,
        sample,
    };

    CommandResult::ok("sample", input_str, data).with_elapsed_ms(start.elapsed().as_millis() as u64)
}

fn sample_rect(img: &image::RgbaImage, rect: Rect) -> (ColorInfo, AlphaStats) {
    let mut sum = [0u64; 4];
    let mut min_alpha = u8::MAX;
    let mut max_alpha = u8::MIN;
    let mut transparent = 0u64;

    for y in rect.y..rect.bottom() {
        for x in rect.x..rect.right() {
            let rgba = img.get_pixel(x, y).0;
            for (sum_channel, channel) in sum.iter_mut().zip(rgba) {
                *sum_channel += channel as u64;
            }
            let alpha = rgba[3];
            min_alpha = min_alpha.min(alpha);
            max_alpha = max_alpha.max(alpha);
            if alpha == 0 {
                transparent += 1;
            }
        }
    }

    let pixel_count = rect.area();
    let averaged = sum.map(|channel| ((channel as f64) / (pixel_count as f64)).round() as u8);
    let alpha_average = sum[3] as f64 / pixel_count as f64;
    let alpha_stats = AlphaStats {
        min: min_alpha,
        max: max_alpha,
        average: alpha_average,
        transparent_ratio: transparent as f64 / pixel_count as f64,
    };

    (color_info(averaged), alpha_stats)
}

fn color_info(rgba: [u8; 4]) -> ColorInfo {
    ColorInfo {
        rgba,
        rgb: [rgba[0], rgba[1], rgba[2]],
        hex: format!("#{:02x}{:02x}{:02x}", rgba[0], rgba[1], rgba[2]),
        alpha: rgba[3],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::fixture;
    use image::{ImageBuffer, Rgba};

    #[test]
    fn point_sample_returns_expected_rgba_and_hex() {
        let result = execute(&fixture("64x64.png"), SampleMode::Point { x: 10, y: 10 });
        assert!(result.ok, "error: {:?}", result.error);

        let data = result.data.unwrap();
        let SampleResult::Point { point, color } = data.sample else {
            panic!("expected point sample");
        };

        assert_eq!(point, Point { x: 10, y: 10 });
        assert_eq!(color.rgba, [100, 150, 200, 255]);
        assert_eq!(color.rgb, [100, 150, 200]);
        assert_eq!(color.hex, "#6496c8");
        assert_eq!(color.alpha, 255);
    }

    #[test]
    fn rect_average_returns_deterministic_average() {
        let result = execute(
            &fixture("64x64.png"),
            SampleMode::Rect {
                rect: Rect {
                    x: 0,
                    y: 0,
                    width: 2,
                    height: 2,
                },
            },
        );
        assert!(result.ok, "error: {:?}", result.error);

        let data = result.data.unwrap();
        let SampleResult::Rect {
            average,
            alpha_stats,
            pixel_count,
            ..
        } = data.sample
        else {
            panic!("expected rect sample");
        };

        assert_eq!(average.rgba, [100, 150, 200, 255]);
        assert_eq!(average.hex, "#6496c8");
        assert_eq!(alpha_stats.min, 255);
        assert_eq!(alpha_stats.max, 255);
        assert_eq!(alpha_stats.average, 255.0);
        assert_eq!(alpha_stats.transparent_ratio, 0.0);
        assert_eq!(pixel_count, 4);
    }

    #[test]
    fn rect_alpha_stats_cover_transparent_pixels() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("alpha.png");
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_fn(2, 2, |x, y| match (x, y) {
            (0, 0) => Rgba([10, 20, 30, 0]),
            (1, 0) => Rgba([20, 40, 60, 128]),
            (0, 1) => Rgba([30, 60, 90, 255]),
            _ => Rgba([40, 80, 120, 0]),
        });
        img.save(&input).unwrap();

        let result = execute(
            &input,
            SampleMode::Rect {
                rect: Rect {
                    x: 0,
                    y: 0,
                    width: 2,
                    height: 2,
                },
            },
        );
        assert!(result.ok, "error: {:?}", result.error);

        let data = result.data.unwrap();
        let SampleResult::Rect {
            average,
            alpha_stats,
            ..
        } = data.sample
        else {
            panic!("expected rect sample");
        };

        assert_eq!(average.rgba, [25, 50, 75, 96]);
        assert_eq!(alpha_stats.min, 0);
        assert_eq!(alpha_stats.max, 255);
        assert!((alpha_stats.average - 95.75).abs() < f64::EPSILON);
        assert_eq!(alpha_stats.transparent_ratio, 0.5);
    }

    #[test]
    fn point_out_of_bounds_returns_invalid_coordinates() {
        let result = execute(&fixture("64x64.png"), SampleMode::Point { x: 64, y: 0 });
        assert!(!result.ok);
        assert_eq!(result.error.unwrap().code, "INVALID_COORDINATES");
    }

    #[test]
    fn rect_zero_size_returns_invalid_dimensions() {
        let result = execute(
            &fixture("64x64.png"),
            SampleMode::Rect {
                rect: Rect {
                    x: 0,
                    y: 0,
                    width: 0,
                    height: 1,
                },
            },
        );
        assert!(!result.ok);
        assert_eq!(result.error.unwrap().code, "INVALID_DIMENSIONS");
    }

    #[test]
    fn rect_out_of_bounds_returns_invalid_coordinates() {
        let result = execute(
            &fixture("64x64.png"),
            SampleMode::Rect {
                rect: Rect {
                    x: 63,
                    y: 63,
                    width: 2,
                    height: 1,
                },
            },
        );
        assert!(!result.ok);
        assert_eq!(result.error.unwrap().code, "INVALID_COORDINATES");
    }
}
