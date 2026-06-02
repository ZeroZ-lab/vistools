//! Photography-oriented measurement commands for agents.
//!
//! These commands are read-only and return structured numeric metrics for a
//! source image or a selected rect region.
use std::path::Path;
use std::time::Instant;

use crate::error::ErrorInfo;
use crate::geom::{Point, Rect};
use crate::guard;
use crate::protocol::{
    ChannelHistogram, ClippingMetrics, ClippingOutput, ColorCastMetrics, ColorCastOutput,
    CommandResult, ContrastMetrics, ContrastOutput, ExposureOutput, FocusCell, FocusMapOutput,
    HistogramMetrics, HistogramOutput, RgbGains, RgbHistogram, RgbMeans, SharpnessMetrics,
    SharpnessOutput, SourceInfo, WhiteBalanceMetrics, WhiteBalanceOutput, ZoneInfo, ZoneMapOutput,
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

    let data = SharpnessOutput {
        source,
        region,
        sharpness: measure_sharpness(&img, region),
    };

    CommandResult::ok("sharpness", input_str, data)
        .with_elapsed_ms(start.elapsed().as_millis() as u64)
}

pub fn execute_focus_map(
    input: &Path,
    rect: Option<Rect>,
    rows: u32,
    cols: u32,
) -> CommandResult<FocusMapOutput> {
    let start = Instant::now();
    let input_str = input.display().to_string();

    let (img, source, region) = match load_region(input, rect) {
        Ok(loaded) => loaded,
        Err(error) => {
            return CommandResult::err("focus-map", input_str, error)
                .with_elapsed_ms(start.elapsed().as_millis() as u64);
        }
    };

    if let Err(error) = guard::validate_tile_count(rows, cols) {
        return CommandResult::err("focus-map", input_str, error)
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }
    if let Err(error) = guard::validate_tile_fits(rows, cols, region.width, region.height) {
        return CommandResult::err("focus-map", input_str, error)
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
    }

    // PD8: focus-map stays in photo.rs and reuses the sharpness kernel over a tile-style grid.
    let mut cells = Vec::with_capacity((rows * cols) as usize);
    let mut best_cell: Option<FocusCell> = None;

    let base_cell_w = region.width / cols;
    let base_cell_h = region.height / rows;
    let remainder_w = region.width % cols;
    let remainder_h = region.height % rows;

    let mut y = region.y;
    for row in 0..rows {
        let cell_h = base_cell_h + if row == rows - 1 { remainder_h } else { 0 };
        let mut x = region.x;

        for col in 0..cols {
            let cell_w = base_cell_w + if col == cols - 1 { remainder_w } else { 0 };
            let cell_region = Rect {
                x,
                y,
                width: cell_w,
                height: cell_h,
            };
            let cell = FocusCell {
                row,
                col,
                region: cell_region,
                sharpness: measure_sharpness(&img, cell_region),
            };

            if best_cell
                .as_ref()
                .is_none_or(|best| cell.sharpness.score > best.sharpness.score)
            {
                best_cell = Some(cell.clone());
            }
            cells.push(cell);
            x += cell_w;
        }

        y += cell_h;
    }

    let best_cell = best_cell.expect("validated rows/cols guarantee at least one cell");
    let focus_point = Point {
        x: best_cell.region.x + (best_cell.region.width / 2),
        y: best_cell.region.y + (best_cell.region.height / 2),
    };

    let data = FocusMapOutput {
        source,
        region,
        rows,
        cols,
        cells,
        best_cell,
        focus_point,
    };

    CommandResult::ok("focus-map", input_str, data)
        .with_elapsed_ms(start.elapsed().as_millis() as u64)
}

pub fn execute_histogram(
    input: &Path,
    rect: Option<Rect>,
    rgb: bool,
) -> CommandResult<HistogramOutput> {
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

    // Per-channel bins (always computed, only included in output when rgb=true)
    let mut r_bins = vec![0u64; 256];
    let mut g_bins = vec![0u64; 256];
    let mut b_bins = vec![0u64; 256];
    let mut r_sum = 0u64;
    let mut g_sum = 0u64;
    let mut b_sum = 0u64;

    iterate_region(region, |x, y| {
        let [r, g, b, _] = img.get_pixel(x, y).0;
        let value = luma_u8([r, g, b, 255]);
        bins[value as usize] += 1;
        sum += value as u64;
        r_bins[r as usize] += 1;
        g_bins[g as usize] += 1;
        b_bins[b as usize] += 1;
        r_sum += r as u64;
        g_sum += g as u64;
        b_sum += b as u64;
    });

    let pixel_count = region.area();
    let mean_luma = if pixel_count == 0 {
        0.0
    } else {
        sum as f64 / pixel_count as f64
    };

    let rgb_histogram = if rgb && pixel_count > 0 {
        Some(RgbHistogram {
            r: build_channel_histogram(&r_bins, r_sum, pixel_count),
            g: build_channel_histogram(&g_bins, g_sum, pixel_count),
            b: build_channel_histogram(&b_bins, b_sum, pixel_count),
        })
    } else {
        None
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
            rgb: rgb_histogram,
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

pub fn execute_white_balance(
    input: &Path,
    rect: Option<Rect>,
) -> CommandResult<WhiteBalanceOutput> {
    let start = Instant::now();
    let input_str = input.display().to_string();

    let (img, source, region) = match load_region(input, rect) {
        Ok(loaded) => loaded,
        Err(error) => {
            return CommandResult::err("white-balance", input_str, error)
                .with_elapsed_ms(start.elapsed().as_millis() as u64);
        }
    };

    let rgb_mean = measure_rgb_mean(&img, region);
    let white_balance = build_white_balance_metrics(rgb_mean);

    let data = WhiteBalanceOutput {
        source,
        region,
        white_balance,
    };

    CommandResult::ok("white-balance", input_str, data)
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

// ── zone-map (FD2) ─────────────────────────────────────────────────────────

const ZONE_LABELS: [&str; 11] = [
    "0", "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X",
];

/// Map a luma value (0–255) to a Zone System index (0–10).
fn zone_index(luma: u8) -> u8 {
    let idx = (luma as u16 * 11) / 256;
    idx.min(10) as u8
}

pub fn execute_zone_map(input: &Path, rect: Option<Rect>) -> CommandResult<ZoneMapOutput> {
    let start = Instant::now();
    let input_str = input.display().to_string();

    let (img, source, region) = match load_region(input, rect) {
        Ok(loaded) => loaded,
        Err(error) => {
            return CommandResult::err("zone-map", input_str, error)
                .with_elapsed_ms(start.elapsed().as_millis() as u64);
        }
    };

    let mut zone_counts = [0u64; 11];
    let mut first_pixels: [Option<Point>; 11] = Default::default();

    iterate_region(region, |x, y| {
        let l = luma_u8(img.get_pixel(x, y).0);
        let idx = zone_index(l) as usize;
        if first_pixels[idx].is_none() {
            first_pixels[idx] = Some(Point { x, y });
        }
        zone_counts[idx] += 1;
    });

    let pixel_count = region.area();
    let zones: Vec<ZoneInfo> = (0..11)
        .map(|i| {
            let low = (i as u16 * 256 / 11) as u8;
            let high = if i == 10 {
                255
            } else {
                (((i + 1) as u16 * 256 / 11) - 1) as u8
            };
            ZoneInfo {
                zone: i as u8,
                label: ZONE_LABELS[i].to_string(),
                luma_range: (low, high),
                pixel_count: zone_counts[i],
                ratio: if pixel_count == 0 {
                    0.0
                } else {
                    zone_counts[i] as f64 / pixel_count as f64
                },
                representative_rect: first_pixels[i]
                    .map(|p| Rect {
                        x: p.x,
                        y: p.y,
                        width: 1,
                        height: 1,
                    })
                    .unwrap_or(Rect {
                        x: 0,
                        y: 0,
                        width: 0,
                        height: 0,
                    }),
            }
        })
        .collect();

    let data = ZoneMapOutput {
        source,
        region,
        zones,
    };

    CommandResult::ok("zone-map", input_str, data)
        .with_elapsed_ms(start.elapsed().as_millis() as u64)
}

// ── exposure (FD3, FD4, FD5) ───────────────────────────────────────────────

/// Camera metering mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeteringMode {
    Evaluative,
    Spot,
    CenterWeighted,
    HighlightWeighted,
}

impl MeteringMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            MeteringMode::Evaluative => "evaluative",
            MeteringMode::Spot => "spot",
            MeteringMode::CenterWeighted => "center_weighted",
            MeteringMode::HighlightWeighted => "highlight_weighted",
        }
    }
}

pub fn execute_exposure(
    input: &Path,
    rect: Option<Rect>,
    mode: MeteringMode,
    spot_point: Option<Point>,
) -> CommandResult<ExposureOutput> {
    let start = Instant::now();
    let input_str = input.display().to_string();

    let (img, source, region) = match load_region(input, rect) {
        Ok(loaded) => loaded,
        Err(error) => {
            return CommandResult::err("exposure", input_str, error)
                .with_elapsed_ms(start.elapsed().as_millis() as u64);
        }
    };

    // Validate spot point is within region
    if let Some(sp) = spot_point {
        if sp.x < region.x || sp.x >= region.right() || sp.y < region.y || sp.y >= region.bottom() {
            return CommandResult::err(
                "exposure",
                input_str,
                ErrorInfo::with_message(
                    crate::error::ErrorCode::InvalidCoordinates,
                    "spot point is outside the region",
                ),
            )
            .with_elapsed_ms(start.elapsed().as_millis() as u64);
        }
    }

    let weighted_mean_luma = match mode {
        MeteringMode::Evaluative => {
            let mut sum = 0.0f64;
            let mut count = 0u64;
            iterate_region(region, |x, y| {
                sum += luma(img.get_pixel(x, y).0);
                count += 1;
            });
            if count == 0 { 0.0 } else { sum / count as f64 }
        }
        MeteringMode::Spot => {
            if let Some(sp) = spot_point {
                luma(img.get_pixel(sp.x, sp.y).0)
            } else {
                0.0
            }
        }
        MeteringMode::CenterWeighted => {
            let cx = region.x as f64 + region.width as f64 / 2.0;
            let cy = region.y as f64 + region.height as f64 / 2.0;
            let sigma = (region.width.min(region.height)) as f64 / 3.0;
            let two_sigma_sq = 2.0 * sigma * sigma;

            let mut weighted_sum = 0.0f64;
            let mut weight_total = 0.0f64;
            iterate_region(region, |x, y| {
                let dx = x as f64 - cx;
                let dy = y as f64 - cy;
                let w = (-(dx * dx + dy * dy) / two_sigma_sq).exp();
                weighted_sum += w * luma(img.get_pixel(x, y).0);
                weight_total += w;
            });
            if weight_total == 0.0 {
                0.0
            } else {
                weighted_sum / weight_total
            }
        }
        MeteringMode::HighlightWeighted => {
            let mut lumas = Vec::new();
            iterate_region(region, |x, y| {
                lumas.push(luma(img.get_pixel(x, y).0));
            });
            if lumas.is_empty() {
                0.0
            } else {
                lumas.sort_unstable_by(|a, b| b.partial_cmp(a).unwrap());
                let top_n = ((lumas.len() as f64) * 0.1).ceil().max(1.0) as usize;
                let sum: f64 = lumas[..top_n].iter().sum();
                sum / top_n as f64
            }
        }
    };

    let ev = if weighted_mean_luma > 0.0 {
        (weighted_mean_luma / 118.0).log2()
    } else {
        f64::NEG_INFINITY
    };

    let data = ExposureOutput {
        source,
        region,
        metering: mode.as_str().to_string(),
        ev,
        assessment: assess_ev(ev).to_string(),
        mean_luma: weighted_mean_luma,
        spot_point,
    };

    CommandResult::ok("exposure", input_str, data)
        .with_elapsed_ms(start.elapsed().as_millis() as u64)
}

/// Classify EV into under / correct / over (FD5: ±0.5 threshold).
fn assess_ev(ev: f64) -> &'static str {
    if ev < -0.5 {
        "under"
    } else if ev > 0.5 {
        "over"
    } else {
        "correct"
    }
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

fn measure_sharpness(img: &image::RgbaImage, region: Rect) -> SharpnessMetrics {
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

    SharpnessMetrics {
        score: variance.max(0.0),
        mean_edge_strength: mean,
        max_edge_strength: max_edge,
    }
}

fn measure_rgb_mean(img: &image::RgbaImage, region: Rect) -> RgbMeans {
    let mut sums = [0u64; 3];
    iterate_region(region, |x, y| {
        let [r, g, b, _] = img.get_pixel(x, y).0;
        sums[0] += r as u64;
        sums[1] += g as u64;
        sums[2] += b as u64;
    });

    let pixel_count = region.area() as f64;
    if pixel_count == 0.0 {
        return RgbMeans {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
    }

    RgbMeans {
        r: sums[0] as f64 / pixel_count,
        g: sums[1] as f64 / pixel_count,
        b: sums[2] as f64 / pixel_count,
    }
}

fn build_white_balance_metrics(rgb_mean: RgbMeans) -> WhiteBalanceMetrics {
    let neutral_mean = (rgb_mean.r + rgb_mean.g + rgb_mean.b) / 3.0;
    // FD9: gray-world gains describe channel correction direction without writing an image.
    let gray_world_gains = RgbGains {
        r: gray_world_gain(neutral_mean, rgb_mean.r),
        g: gray_world_gain(neutral_mean, rgb_mean.g),
        b: gray_world_gain(neutral_mean, rgb_mean.b),
    };

    let threshold = neutral_mean * 0.05;
    // FD10: report directional bias only; no Kelvin estimate from RGB pixels.
    let temperature_bias = if rgb_mean.r - rgb_mean.b > threshold {
        "warm"
    } else if rgb_mean.b - rgb_mean.r > threshold {
        "cool"
    } else {
        "neutral"
    };

    let rb_mean = (rgb_mean.r + rgb_mean.b) / 2.0;
    let tint_bias = if rgb_mean.g - rb_mean > threshold {
        "green"
    } else if rb_mean - rgb_mean.g > threshold {
        "magenta"
    } else {
        "neutral"
    };

    let assessment = if temperature_bias == "neutral" && tint_bias == "neutral" {
        "neutral"
    } else {
        "biased"
    };

    WhiteBalanceMetrics {
        rgb_mean,
        gray_world_gains,
        temperature_bias: temperature_bias.to_string(),
        tint_bias: tint_bias.to_string(),
        assessment: assessment.to_string(),
    }
}

fn gray_world_gain(neutral_mean: f64, channel_mean: f64) -> f64 {
    if channel_mean == 0.0 {
        1.0
    } else {
        neutral_mean / channel_mean
    }
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

fn build_channel_histogram(bins: &[u64], sum: u64, pixel_count: u64) -> ChannelHistogram {
    let mean = sum as f64 / pixel_count as f64;
    let mut clipping_low = 0u64;
    let mut clipping_high = 0u64;
    for (i, &count) in bins.iter().enumerate() {
        if i <= 5 {
            clipping_low += count;
        }
        if i >= 250 {
            clipping_high += count;
        }
    }
    ChannelHistogram {
        bins: bins.to_vec(),
        mean,
        p05: percentile_bin(bins, pixel_count, 0.05),
        p50: percentile_bin(bins, pixel_count, 0.50),
        p95: percentile_bin(bins, pixel_count, 0.95),
        clipping_low,
        clipping_high,
    }
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
    fn focus_map_splits_grid_with_remainder() {
        let img = ImageBuffer::from_pixel(5, 3, Rgba([128, 128, 128, 255]));
        let result = execute_focus_map(write_temp_image(img).as_ref(), None, 2, 2);
        assert!(result.ok);
        let data = result.data.unwrap();
        assert_eq!(data.cells.len(), 4);
        assert_eq!(data.cells[0].region.width, 2);
        assert_eq!(data.cells[0].region.height, 1);
        assert_eq!(data.cells[3].region.width, 3);
        assert_eq!(data.cells[3].region.height, 2);
        assert_eq!(data.best_cell.region.width, 2);
    }

    #[test]
    fn focus_map_best_cell_tracks_sharpest_region() {
        let img = ImageBuffer::from_fn(8, 4, |x, y| {
            if x < 4 {
                Rgba([120, 120, 120, 255])
            } else if y < 2 && x % 2 == 0 {
                Rgba([0, 0, 0, 255])
            } else {
                Rgba([255, 255, 255, 255])
            }
        });
        let result = execute_focus_map(write_temp_image(img).as_ref(), None, 1, 2);
        assert!(result.ok);
        let data = result.data.unwrap();
        assert_eq!(data.best_cell.col, 1);
        assert!(data.best_cell.sharpness.score > data.cells[0].sharpness.score);
        assert_eq!(data.focus_point.x, 6);
        assert_eq!(data.focus_point.y, 2);
    }

    #[test]
    fn focus_map_rejects_zero_rows() {
        let img = ImageBuffer::from_pixel(4, 4, Rgba([128, 128, 128, 255]));
        let result = execute_focus_map(write_temp_image(img).as_ref(), None, 0, 2);
        assert!(!result.ok);
        assert_eq!(result.error.unwrap().code, "INVALID_PARAMETERS");
    }

    #[test]
    fn focus_map_rejects_grid_larger_than_region() {
        let img = ImageBuffer::from_pixel(4, 4, Rgba([128, 128, 128, 255]));
        let result = execute_focus_map(write_temp_image(img).as_ref(), None, 5, 1);
        assert!(!result.ok);
        assert_eq!(result.error.unwrap().code, "INVALID_PARAMETERS");
    }

    #[test]
    fn histogram_reports_expected_median() {
        let img = ImageBuffer::from_fn(2, 2, |x, y| match (x, y) {
            (0, 0) | (0, 1) => Rgba([0, 0, 0, 255]),
            _ => Rgba([255, 255, 255, 255]),
        });
        let result = execute_histogram(write_temp_image(img).as_ref(), None, false);
        assert!(result.ok);
        let histogram = result.data.unwrap().histogram;
        assert_eq!(histogram.pixel_count, 4);
        assert_eq!(histogram.p05_luma, 0);
        assert_eq!(histogram.p95_luma, 255);
        assert!(histogram.rgb.is_none());
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
    fn white_balance_reports_neutral_gray() {
        // Test for: AC-05-1
        let img = ImageBuffer::from_pixel(4, 4, Rgba([128, 128, 128, 255]));
        let result = execute_white_balance(write_temp_image(img).as_ref(), None);
        assert!(result.ok);
        let white_balance = result.data.unwrap().white_balance;
        assert_eq!(white_balance.assessment, "neutral");
        assert_eq!(white_balance.temperature_bias, "neutral");
        assert_eq!(white_balance.tint_bias, "neutral");
        assert!((white_balance.gray_world_gains.r - 1.0).abs() < f64::EPSILON);
        assert!((white_balance.gray_world_gains.g - 1.0).abs() < f64::EPSILON);
        assert!((white_balance.gray_world_gains.b - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn white_balance_reports_warm_bias() {
        // Test for: AC-05-2
        let img = ImageBuffer::from_pixel(4, 4, Rgba([210, 140, 80, 255]));
        let result = execute_white_balance(write_temp_image(img).as_ref(), None);
        assert!(result.ok);
        let white_balance = result.data.unwrap().white_balance;
        assert_eq!(white_balance.temperature_bias, "warm");
        assert_eq!(white_balance.assessment, "biased");
        assert!(white_balance.gray_world_gains.r < 1.0);
        assert!(white_balance.gray_world_gains.b > 1.0);
    }

    #[test]
    fn white_balance_reports_cool_bias() {
        // Test for: AC-05-3
        let img = ImageBuffer::from_pixel(4, 4, Rgba([80, 140, 210, 255]));
        let result = execute_white_balance(write_temp_image(img).as_ref(), None);
        assert!(result.ok);
        let white_balance = result.data.unwrap().white_balance;
        assert_eq!(white_balance.temperature_bias, "cool");
        assert_eq!(white_balance.assessment, "biased");
        assert!(white_balance.gray_world_gains.b < 1.0);
        assert!(white_balance.gray_world_gains.r > 1.0);
    }

    #[test]
    fn white_balance_reports_green_and_magenta_tint() {
        // Test for: AC-05-4
        let green = ImageBuffer::from_pixel(4, 4, Rgba([120, 180, 120, 255]));
        let magenta = ImageBuffer::from_pixel(4, 4, Rgba([180, 120, 180, 255]));

        let green_result = execute_white_balance(write_temp_image(green).as_ref(), None);
        let magenta_result = execute_white_balance(write_temp_image(magenta).as_ref(), None);

        assert_eq!(green_result.data.unwrap().white_balance.tint_bias, "green");
        assert_eq!(
            magenta_result.data.unwrap().white_balance.tint_bias,
            "magenta"
        );
    }

    #[test]
    fn white_balance_uses_rect_region() {
        // Test for: AC-05-5
        let img = ImageBuffer::from_fn(4, 2, |x, _| {
            if x < 2 {
                Rgba([128, 128, 128, 255])
            } else {
                Rgba([220, 120, 80, 255])
            }
        });
        let result = execute_white_balance(
            write_temp_image(img).as_ref(),
            Some(Rect {
                x: 2,
                y: 0,
                width: 2,
                height: 2,
            }),
        );
        assert!(result.ok);
        let data = result.data.unwrap();
        assert_eq!(data.region.x, 2);
        assert_eq!(data.region.width, 2);
        assert_eq!(data.white_balance.temperature_bias, "warm");
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

    // ── histogram --rgb tests ──────────────────────────────────────────

    #[test]
    fn histogram_rgb_reports_three_channels() {
        let img = ImageBuffer::from_fn(2, 2, |_, _| Rgba([200, 100, 50, 255]));
        let result = execute_histogram(write_temp_image(img).as_ref(), None, true);
        assert!(result.ok);
        let histogram = result.data.unwrap().histogram;
        let rgb = histogram.rgb.as_ref().unwrap();
        assert_eq!(rgb.r.p50, 200);
        assert_eq!(rgb.g.p50, 100);
        assert_eq!(rgb.b.p50, 50);
        assert_eq!(rgb.r.bins.len(), 256);
    }

    #[test]
    fn histogram_rgb_off_returns_none() {
        let img = ImageBuffer::from_pixel(4, 4, Rgba([128, 128, 128, 255]));
        let result = execute_histogram(write_temp_image(img).as_ref(), None, false);
        assert!(result.ok);
        assert!(result.data.unwrap().histogram.rgb.is_none());
    }

    #[test]
    fn histogram_rgb_channel_clipping() {
        // Red channel fully overexposed, G/B normal
        let img = ImageBuffer::from_pixel(4, 4, Rgba([253, 100, 100, 255]));
        let result = execute_histogram(write_temp_image(img).as_ref(), None, true);
        assert!(result.ok);
        let rgb = result.data.unwrap().histogram.rgb.unwrap();
        assert!(rgb.r.clipping_high > 0);
        assert_eq!(rgb.g.clipping_high, 0);
        assert_eq!(rgb.b.clipping_high, 0);
    }

    // ── zone-map tests ─────────────────────────────────────────────────

    #[test]
    fn zone_map_black_image() {
        let img = ImageBuffer::from_pixel(8, 8, Rgba([0, 0, 0, 255]));
        let result = execute_zone_map(write_temp_image(img).as_ref(), None);
        assert!(result.ok);
        let data = result.data.unwrap();
        assert_eq!(data.zones.len(), 11);
        assert_eq!(data.zones[0].pixel_count, 64);
        assert!((data.zones[0].ratio - 1.0).abs() < f64::EPSILON);
        for z in &data.zones[1..] {
            assert_eq!(z.pixel_count, 0);
        }
    }

    #[test]
    fn zone_map_white_image() {
        let img = ImageBuffer::from_pixel(8, 8, Rgba([255, 255, 255, 255]));
        let result = execute_zone_map(write_temp_image(img).as_ref(), None);
        assert!(result.ok);
        let data = result.data.unwrap();
        assert_eq!(data.zones[10].pixel_count, 64);
        assert_eq!(data.zones[10].label, "X");
        for z in &data.zones[0..10] {
            assert_eq!(z.pixel_count, 0);
        }
    }

    #[test]
    fn zone_map_gradient_has_all_zones() {
        let img = ImageBuffer::from_fn(256, 1, |x, _| {
            let v = x as u8;
            Rgba([v, v, v, 255])
        });
        let result = execute_zone_map(write_temp_image(img).as_ref(), None);
        assert!(result.ok);
        let data = result.data.unwrap();
        // All 11 zones should have pixels
        for z in &data.zones {
            assert!(z.pixel_count > 0, "Zone {} should have pixels", z.label);
        }
        // Ratio sum should be ~1.0
        let ratio_sum: f64 = data.zones.iter().map(|z| z.ratio).sum();
        assert!((ratio_sum - 1.0).abs() < 0.01);
    }

    #[test]
    fn zone_map_representative_rect_in_bounds() {
        let img = ImageBuffer::from_fn(256, 1, |x, _| {
            let v = x as u8;
            Rgba([v, v, v, 255])
        });
        let result = execute_zone_map(write_temp_image(img).as_ref(), None);
        assert!(result.ok);
        let data = result.data.unwrap();
        for z in &data.zones {
            if z.pixel_count > 0 {
                let r = &z.representative_rect;
                assert!(r.x < 256);
                assert!(r.y < 1);
            }
        }
    }

    // ── exposure tests ─────────────────────────────────────────────────

    #[test]
    fn exposure_correct_image_ev_near_zero() {
        // luma ≈ 118 (sRGB mid-gray)
        let img = ImageBuffer::from_pixel(8, 8, Rgba([118, 118, 118, 255]));
        let result = execute_exposure(
            write_temp_image(img).as_ref(),
            None,
            MeteringMode::Evaluative,
            None,
        );
        assert!(result.ok);
        let data = result.data.unwrap();
        assert!(data.ev.abs() < 0.3, "ev should be near 0, got {}", data.ev);
        assert_eq!(data.assessment, "correct");
        assert_eq!(data.metering, "evaluative");
    }

    #[test]
    fn exposure_over_image() {
        // luma=240: EV = log2(240/118) ≈ 1.02
        let img = ImageBuffer::from_pixel(8, 8, Rgba([240, 240, 240, 255]));
        let result = execute_exposure(
            write_temp_image(img).as_ref(),
            None,
            MeteringMode::Evaluative,
            None,
        );
        assert!(result.ok);
        let data = result.data.unwrap();
        assert!(data.ev > 0.5, "ev should be > 0.5, got {}", data.ev);
        assert_eq!(data.assessment, "over");
    }

    #[test]
    fn exposure_under_image() {
        let img = ImageBuffer::from_pixel(8, 8, Rgba([30, 30, 30, 255]));
        let result = execute_exposure(
            write_temp_image(img).as_ref(),
            None,
            MeteringMode::Evaluative,
            None,
        );
        assert!(result.ok);
        let data = result.data.unwrap();
        assert!(data.ev < -1.0);
        assert_eq!(data.assessment, "under");
    }

    #[test]
    fn exposure_spot_mode() {
        let img = ImageBuffer::from_fn(8, 8, |x, _| {
            if x < 4 {
                Rgba([0, 0, 0, 255])
            } else {
                Rgba([255, 255, 255, 255])
            }
        });
        // Spot at bright area
        let result = execute_exposure(
            write_temp_image(img).as_ref(),
            None,
            MeteringMode::Spot,
            Some(Point { x: 6, y: 0 }),
        );
        assert!(result.ok);
        let data = result.data.unwrap();
        assert_eq!(data.metering, "spot");
        assert!(data.ev > 0.5);
        assert_eq!(data.spot_point, Some(Point { x: 6, y: 0 }));
    }

    #[test]
    fn exposure_spot_out_of_region_returns_error() {
        let img = ImageBuffer::from_pixel(4, 4, Rgba([128, 128, 128, 255]));
        let result = execute_exposure(
            write_temp_image(img).as_ref(),
            None,
            MeteringMode::Spot,
            Some(Point { x: 10, y: 10 }),
        );
        assert!(!result.ok);
        assert_eq!(result.error.unwrap().code, "INVALID_COORDINATES");
    }

    #[test]
    fn exposure_center_weighted_favors_center() {
        // Center bright, edges dark
        let make_img = || {
            ImageBuffer::from_fn(10, 10, |x, y| {
                let cx = 5.0_f64;
                let cy = 5.0_f64;
                let d = ((x as f64 - cx).powi(2) + (y as f64 - cy).powi(2)).sqrt();
                if d < 3.0 {
                    Rgba([255, 255, 255, 255])
                } else {
                    Rgba([20, 20, 20, 255])
                }
            })
        };
        let eval = execute_exposure(
            write_temp_image(make_img()).as_ref(),
            None,
            MeteringMode::Evaluative,
            None,
        );
        let cw = execute_exposure(
            write_temp_image(make_img()).as_ref(),
            None,
            MeteringMode::CenterWeighted,
            None,
        );
        assert!(eval.ok);
        assert!(cw.ok);
        assert!(
            cw.data.as_ref().unwrap().mean_luma > eval.data.as_ref().unwrap().mean_luma,
            "center-weighted should be higher than evaluative when center is brighter"
        );
    }

    #[test]
    fn exposure_highlight_weighted_uses_top10() {
        let img = ImageBuffer::from_fn(10, 10, |x, _| {
            if x == 0 {
                Rgba([255, 255, 255, 255]) // 10 pixels at 255
            } else {
                Rgba([20, 20, 20, 255]) // 90 pixels at 20
            }
        });
        let result = execute_exposure(
            write_temp_image(img).as_ref(),
            None,
            MeteringMode::HighlightWeighted,
            None,
        );
        assert!(result.ok);
        let data = result.data.unwrap();
        // Top 10% are the 10 bright pixels (luma=255), so mean_luma ≈ 255
        assert!(
            data.mean_luma > 240.0,
            "highlight-weighted should favor top 10%, got {}",
            data.mean_luma
        );
    }

    #[test]
    fn exposure_assessment_boundaries() {
        // EV boundary: -0.5 is the edge between under and correct
        // luma where ev = -0.5: luma = 118 * 2^(-0.5) ≈ 83.4
        let img_under = ImageBuffer::from_pixel(4, 4, Rgba([80, 80, 80, 255]));
        let r = execute_exposure(
            write_temp_image(img_under).as_ref(),
            None,
            MeteringMode::Evaluative,
            None,
        );
        assert_eq!(r.data.unwrap().assessment, "under");

        // luma where ev = 0.5: luma = 118 * 2^(0.5) ≈ 166.9
        let img_over = ImageBuffer::from_pixel(4, 4, Rgba([170, 170, 170, 255]));
        let r = execute_exposure(
            write_temp_image(img_over).as_ref(),
            None,
            MeteringMode::Evaluative,
            None,
        );
        assert_eq!(r.data.unwrap().assessment, "over");

        // luma ≈ 118 → correct
        let img_ok = ImageBuffer::from_pixel(4, 4, Rgba([118, 118, 118, 255]));
        let r = execute_exposure(
            write_temp_image(img_ok).as_ref(),
            None,
            MeteringMode::Evaluative,
            None,
        );
        assert_eq!(r.data.unwrap().assessment, "correct");
    }
}
