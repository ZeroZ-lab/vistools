//! Photography-oriented measurement commands for agents.
//!
//! These commands are read-only and return structured numeric metrics for a
//! source image or a selected rect region.
use std::path::Path;
use std::time::Instant;

use crate::error::ErrorInfo;
use crate::geom::Rect;
use crate::protocol::{
    ClippingMetrics, ClippingOutput, ColorCastMetrics, ColorCastOutput, CommandResult,
    ContrastMetrics, ContrastOutput, HistogramMetrics, HistogramOutput, SharpnessMetrics,
    SharpnessOutput, SourceInfo,
};
use crate::region;
use crate::source;

pub fn execute_sharpness(input: &Path, rect: Option<Rect>) -> CommandResult<SharpnessOutput> {
    let start = Instant::now();
    let input_str = input.display().to_string();

    let (img, source, region) = match load_region(input, rect) {
        Ok(loaded) => loaded,
        Err(error) => {
            return CommandResult::err("sharpness", input_str, error)
                .with_elapsed_ms(start.elapsed().as_millis() as u64);
        }
    };

    let mut sum = 0.0;
    let mut sum_sq = 0.0;
    let mut max_edge = 0.0_f64;
    let mut count = 0u64;

    if region.width >= 2 && region.height >= 2 {
        for y in region.y..(region.bottom() - 1) {
            for x in region.x..(region.right() - 1) {
                let l = luma(img.get_pixel(x, y).0);
                let right = luma(img.get_pixel(x + 1, y).0);
                let down = luma(img.get_pixel(x, y + 1).0);
                let edge = ((right - l).powi(2) + (down - l).powi(2)).sqrt();
                sum += edge;
                sum_sq += edge * edge;
                max_edge = max_edge.max(edge);
                count += 1;
            }
        }
    }

    let mean = if count == 0 { 0.0 } else { sum / count as f64 };
    let variance = if count == 0 {
        0.0
    } else {
        (sum_sq / count as f64) - mean * mean
    };

    let data = SharpnessOutput {
        source,
        region,
        sharpness: SharpnessMetrics {
            score: variance.max(0.0),
            mean_edge_strength: mean,
            max_edge_strength: max_edge,
        },
    };

    CommandResult::ok("sharpness", input_str, data)
        .with_elapsed_ms(start.elapsed().as_millis() as u64)
}

pub fn execute_histogram(input: &Path, rect: Option<Rect>) -> CommandResult<HistogramOutput> {
    let start = Instant::now();
    let input_str = input.display().to_string();

    let (img, source, region) = match load_region(input, rect) {
        Ok(loaded) => loaded,
        Err(error) => {
            return CommandResult::err("histogram", input_str, error)
                .with_elapsed_ms(start.elapsed().as_millis() as u64);
        }
    };

    let mut bins = vec![0u64; 256];
    let mut sum = 0u64;

    iterate_region(region, |x, y| {
        let value = luma_u8(img.get_pixel(x, y).0);
        bins[value as usize] += 1;
        sum += value as u64;
    });

    let pixel_count = region.area();
    let mean_luma = if pixel_count == 0 {
        0.0
    } else {
        sum as f64 / pixel_count as f64
    };

    let data = HistogramOutput {
        source,
        region,
        histogram: HistogramMetrics {
            median_luma: percentile_bin(&bins, pixel_count, 0.50),
            p05_luma: percentile_bin(&bins, pixel_count, 0.05),
            p95_luma: percentile_bin(&bins, pixel_count, 0.95),
            bins,
            pixel_count,
            mean_luma,
        },
    };

    CommandResult::ok("histogram", input_str, data)
        .with_elapsed_ms(start.elapsed().as_millis() as u64)
}

pub fn execute_highlight_clipping(
    input: &Path,
    rect: Option<Rect>,
    threshold: u8,
) -> CommandResult<ClippingOutput> {
    execute_clipping("highlight-clipping", input, rect, threshold, |value, t| {
        value >= t
    })
}

pub fn execute_shadow_clipping(
    input: &Path,
    rect: Option<Rect>,
    threshold: u8,
) -> CommandResult<ClippingOutput> {
    execute_clipping("shadow-clipping", input, rect, threshold, |value, t| {
        value <= t
    })
}

pub fn execute_contrast(input: &Path, rect: Option<Rect>) -> CommandResult<ContrastOutput> {
    let start = Instant::now();
    let input_str = input.display().to_string();

    let (img, source, region) = match load_region(input, rect) {
        Ok(loaded) => loaded,
        Err(error) => {
            return CommandResult::err("contrast", input_str, error)
                .with_elapsed_ms(start.elapsed().as_millis() as u64);
        }
    };

    let mut sum = 0.0;
    let mut sum_sq = 0.0;
    let mut min_luma = u8::MAX;
    let mut max_luma = u8::MIN;

    iterate_region(region, |x, y| {
        let value = luma_u8(img.get_pixel(x, y).0);
        min_luma = min_luma.min(value);
        max_luma = max_luma.max(value);
        let value_f = value as f64;
        sum += value_f;
        sum_sq += value_f * value_f;
    });

    let pixel_count = region.area() as f64;
    let mean = if pixel_count == 0.0 {
        0.0
    } else {
        sum / pixel_count
    };
    let variance = if pixel_count == 0.0 {
        0.0
    } else {
        (sum_sq / pixel_count) - mean * mean
    };
    let stddev = variance.max(0.0).sqrt();

    let data = ContrastOutput {
        source,
        region,
        contrast: ContrastMetrics {
            rms_contrast: stddev / 255.0,
            luma_stddev: stddev,
            min_luma,
            max_luma,
            dynamic_range: max_luma.saturating_sub(min_luma),
        },
    };

    CommandResult::ok("contrast", input_str, data)
        .with_elapsed_ms(start.elapsed().as_millis() as u64)
}

pub fn execute_color_cast(input: &Path, rect: Option<Rect>) -> CommandResult<ColorCastOutput> {
    let start = Instant::now();
    let input_str = input.display().to_string();

    let (img, source, region) = match load_region(input, rect) {
        Ok(loaded) => loaded,
        Err(error) => {
            return CommandResult::err("color-cast", input_str, error)
                .with_elapsed_ms(start.elapsed().as_millis() as u64);
        }
    };

    let mut sums = [0u64; 3];
    iterate_region(region, |x, y| {
        let [r, g, b, _] = img.get_pixel(x, y).0;
        sums[0] += r as u64;
        sums[1] += g as u64;
        sums[2] += b as u64;
    });

    let pixel_count = region.area() as f64;
    let channel_means = if pixel_count == 0.0 {
        [0.0; 3]
    } else {
        [
            sums[0] as f64 / pixel_count,
            sums[1] as f64 / pixel_count,
            sums[2] as f64 / pixel_count,
        ]
    };
    let neutral_mean = (channel_means[0] + channel_means[1] + channel_means[2]) / 3.0;
    let cast_vector = [
        channel_means[0] - neutral_mean,
        channel_means[1] - neutral_mean,
        channel_means[2] - neutral_mean,
    ];
    let cast_strength =
        (cast_vector[0].powi(2) + cast_vector[1].powi(2) + cast_vector[2].powi(2)).sqrt();
    let dominant_channel = ["red", "green", "blue"][channel_means
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(idx, _)| idx)
        .unwrap_or(0)]
    .to_string();

    let data = ColorCastOutput {
        source,
        region,
        color_cast: ColorCastMetrics {
            channel_means,
            neutral_mean,
            cast_vector,
            cast_strength,
            dominant_channel,
        },
    };

    CommandResult::ok("color-cast", input_str, data)
        .with_elapsed_ms(start.elapsed().as_millis() as u64)
}

fn execute_clipping(
    operation: &str,
    input: &Path,
    rect: Option<Rect>,
    threshold: u8,
    predicate: impl Fn(u8, u8) -> bool,
) -> CommandResult<ClippingOutput> {
    let start = Instant::now();
    let input_str = input.display().to_string();

    let (img, source, region) = match load_region(input, rect) {
        Ok(loaded) => loaded,
        Err(error) => {
            return CommandResult::err(operation, input_str, error)
                .with_elapsed_ms(start.elapsed().as_millis() as u64);
        }
    };

    let mut clipped_pixels = 0u64;
    iterate_region(region, |x, y| {
        if predicate(luma_u8(img.get_pixel(x, y).0), threshold) {
            clipped_pixels += 1;
        }
    });

    let pixel_count = region.area();
    let data = ClippingOutput {
        source,
        region,
        clipping: ClippingMetrics {
            threshold,
            clipped_pixels,
            clipped_ratio: if pixel_count == 0 {
                0.0
            } else {
                clipped_pixels as f64 / pixel_count as f64
            },
            pixel_count,
        },
    };

    CommandResult::ok(operation, input_str, data)
        .with_elapsed_ms(start.elapsed().as_millis() as u64)
}

fn load_region(
    input: &Path,
    rect: Option<Rect>,
) -> Result<(image::RgbaImage, SourceInfo, Rect), ErrorInfo> {
    let loaded = source::load_rgba_source(input)?;
    let img = loaded.image;

    let region = rect.unwrap_or(Rect {
        x: 0,
        y: 0,
        width: loaded.info.width,
        height: loaded.info.height,
    });

    region::validate_rect(
        region,
        crate::geom::Size {
            width: loaded.info.width,
            height: loaded.info.height,
        },
        "rect",
    )?;

    Ok((img, loaded.info, region))
}

fn iterate_region(region: Rect, mut visit: impl FnMut(u32, u32)) {
    for y in region.y..region.bottom() {
        for x in region.x..region.right() {
            visit(x, y);
        }
    }
}

fn luma(rgba: [u8; 4]) -> f64 {
    (0.299 * rgba[0] as f64) + (0.587 * rgba[1] as f64) + (0.114 * rgba[2] as f64)
}

fn luma_u8(rgba: [u8; 4]) -> u8 {
    luma(rgba).round().clamp(0.0, 255.0) as u8
}

fn percentile_bin(bins: &[u64], pixel_count: u64, pct: f64) -> u8 {
    if pixel_count == 0 {
        return 0;
    }
    let target = ((pixel_count as f64) * pct).ceil() as u64;
    let mut seen = 0u64;
    for (idx, count) in bins.iter().enumerate() {
        seen += count;
        if seen >= target.max(1) {
            return idx as u8;
        }
    }
    255
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
    fn sharpness_detects_edge_detail() {
        let flat = ImageBuffer::from_pixel(4, 4, Rgba([100, 100, 100, 255]));
        let edge = ImageBuffer::from_fn(4, 4, |x, _| {
            if x < 2 {
                Rgba([0, 0, 0, 255])
            } else {
                Rgba([255, 255, 255, 255])
            }
        });

        let flat_result = execute_sharpness(write_temp_image(flat).as_ref(), None);
        let edge_result = execute_sharpness(write_temp_image(edge).as_ref(), None);

        assert!(flat_result.ok);
        assert!(edge_result.ok);
        assert!(
            edge_result.data.unwrap().sharpness.score > flat_result.data.unwrap().sharpness.score
        );
    }

    #[test]
    fn histogram_reports_expected_median() {
        let img = ImageBuffer::from_fn(2, 2, |x, y| match (x, y) {
            (0, 0) | (0, 1) => Rgba([0, 0, 0, 255]),
            _ => Rgba([255, 255, 255, 255]),
        });
        let result = execute_histogram(write_temp_image(img).as_ref(), None);
        assert!(result.ok);
        let histogram = result.data.unwrap().histogram;
        assert_eq!(histogram.pixel_count, 4);
        assert_eq!(histogram.p05_luma, 0);
        assert_eq!(histogram.p95_luma, 255);
    }

    #[test]
    fn highlight_clipping_counts_bright_pixels() {
        let img = ImageBuffer::from_fn(2, 2, |x, y| match (x, y) {
            (0, 0) => Rgba([255, 255, 255, 255]),
            _ => Rgba([10, 10, 10, 255]),
        });
        let result = execute_highlight_clipping(write_temp_image(img).as_ref(), None, 250);
        assert!(result.ok);
        let clipping = result.data.unwrap().clipping;
        assert_eq!(clipping.clipped_pixels, 1);
        assert!((clipping.clipped_ratio - 0.25).abs() < f64::EPSILON);
    }

    #[test]
    fn shadow_clipping_counts_dark_pixels() {
        let img = ImageBuffer::from_fn(2, 2, |x, y| match (x, y) {
            (0, 0) => Rgba([0, 0, 0, 255]),
            _ => Rgba([240, 240, 240, 255]),
        });
        let result = execute_shadow_clipping(write_temp_image(img).as_ref(), None, 5);
        assert!(result.ok);
        let clipping = result.data.unwrap().clipping;
        assert_eq!(clipping.clipped_pixels, 1);
    }

    #[test]
    fn contrast_reports_zero_for_flat_image() {
        let img = ImageBuffer::from_pixel(4, 4, Rgba([128, 128, 128, 255]));
        let result = execute_contrast(write_temp_image(img).as_ref(), None);
        assert!(result.ok);
        let contrast = result.data.unwrap().contrast;
        assert_eq!(contrast.dynamic_range, 0);
        assert_eq!(contrast.luma_stddev, 0.0);
    }

    #[test]
    fn color_cast_detects_dominant_red_channel() {
        let img = ImageBuffer::from_pixel(4, 4, Rgba([200, 100, 100, 255]));
        let result = execute_color_cast(write_temp_image(img).as_ref(), None);
        assert!(result.ok);
        let color_cast = result.data.unwrap().color_cast;
        assert_eq!(color_cast.dominant_channel, "red");
        assert!(color_cast.cast_strength > 0.0);
    }

    #[test]
    fn rect_out_of_bounds_returns_invalid_coordinates() {
        let img = ImageBuffer::from_pixel(4, 4, Rgba([128, 128, 128, 255]));
        let result = execute_contrast(
            write_temp_image(img).as_ref(),
            Some(Rect {
                x: 3,
                y: 3,
                width: 2,
                height: 2,
            }),
        );
        assert!(!result.ok);
        assert_eq!(result.error.unwrap().code, "INVALID_COORDINATES");
    }
}
