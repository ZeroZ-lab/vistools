use std::path::{Path, PathBuf};

use clap::Args;
use vistools_core::photo::MeteringMode;
use vistools_core::{CommandResult, DEFAULT_HIGHLIGHT_THRESHOLD, DEFAULT_SHADOW_THRESHOLD, Rect};

use crate::parse::{invalid_region_parameters, parse_optional_rect_arg};

#[derive(Args)]
pub struct RegionArgs {
    /// Input image path.
    pub input: PathBuf,
    /// Optional rect region: x,y,width,height.
    #[arg(long)]
    pub rect: Option<String>,
}

#[derive(Args)]
pub struct HistogramArgs {
    /// Input image path.
    pub input: PathBuf,
    /// Optional rect region: x,y,width,height.
    #[arg(long)]
    pub rect: Option<String>,
    /// Include per-channel R/G/B histograms.
    #[arg(long, default_value_t = false)]
    pub rgb: bool,
}

#[derive(Args)]
pub struct ExposureArgs {
    /// Input image path.
    pub input: PathBuf,
    /// Optional rect region: x,y,width,height.
    #[arg(long)]
    pub rect: Option<String>,
    /// Metering mode: evaluative, spot, center-weighted, or highlight-weighted.
    #[arg(long, default_value = "evaluative")]
    pub mode: String,
    /// Spot metering X coordinate (required when mode=spot).
    #[arg(long)]
    pub x: Option<u32>,
    /// Spot metering Y coordinate (required when mode=spot).
    #[arg(long)]
    pub y: Option<u32>,
}

#[derive(Args)]
pub struct FocusMapArgs {
    /// Input image path.
    pub input: PathBuf,
    /// Optional rect region: x,y,width,height.
    #[arg(long)]
    pub rect: Option<String>,
    /// Grid rows.
    #[arg(long)]
    pub rows: u32,
    /// Grid columns.
    #[arg(long)]
    pub cols: u32,
}

#[derive(Args)]
pub struct ThresholdRegionArgs {
    /// Input image path.
    pub input: PathBuf,
    /// Optional rect region: x,y,width,height.
    #[arg(long)]
    pub rect: Option<String>,
    /// Luma threshold counted as clipped.
    #[arg(long, default_value_t = DEFAULT_HIGHLIGHT_THRESHOLD)]
    pub threshold: u8,
}

#[derive(Args)]
pub struct ShadowThresholdRegionArgs {
    /// Input image path.
    pub input: PathBuf,
    /// Optional rect region: x,y,width,height.
    #[arg(long)]
    pub rect: Option<String>,
    /// Luma threshold counted as clipped.
    #[arg(long, default_value_t = DEFAULT_SHADOW_THRESHOLD)]
    pub threshold: u8,
}

pub fn run_sharpness(args: RegionArgs) -> (String, bool) {
    run_region_metric("sharpness", &args.input, args.rect, |input, rect| {
        vistools_core::photo::execute_sharpness(input, rect)
    })
}

pub fn run_histogram(args: HistogramArgs) -> (String, bool) {
    let rect = match parse_optional_rect_arg(args.rect) {
        Ok(rect) => rect,
        Err(e) => {
            return invalid_region_parameters::<vistools_core::HistogramOutput>(
                "histogram",
                &args.input,
                e,
            );
        }
    };

    let result = vistools_core::photo::execute_histogram(&args.input, rect, args.rgb);
    let ok = result.ok;
    (serde_json::to_string_pretty(&result).unwrap(), ok)
}

pub fn run_zone_map(args: RegionArgs) -> (String, bool) {
    run_region_metric("zone-map", &args.input, args.rect, |input, rect| {
        vistools_core::photo::execute_zone_map(input, rect)
    })
}

pub fn run_exposure(args: ExposureArgs) -> (String, bool) {
    let rect = match parse_optional_rect_arg(args.rect) {
        Ok(rect) => rect,
        Err(e) => {
            return invalid_region_parameters::<vistools_core::ExposureOutput>(
                "exposure",
                &args.input,
                e,
            );
        }
    };

    let mode = match parse_metering_mode(&args.mode) {
        Ok(m) => m,
        Err(e) => {
            return invalid_region_parameters::<vistools_core::ExposureOutput>(
                "exposure",
                &args.input,
                e,
            );
        }
    };

    let spot_point = if mode == MeteringMode::Spot {
        match (args.x, args.y) {
            (Some(x), Some(y)) => Some(vistools_core::Point { x, y }),
            _ => {
                return invalid_region_parameters::<vistools_core::ExposureOutput>(
                    "exposure",
                    &args.input,
                    "spot mode requires --x and --y",
                );
            }
        }
    } else {
        None
    };

    let result = vistools_core::photo::execute_exposure(&args.input, rect, mode, spot_point);
    let ok = result.ok;
    (serde_json::to_string_pretty(&result).unwrap(), ok)
}

pub fn run_focus_map(args: FocusMapArgs) -> (String, bool) {
    let rect = match parse_optional_rect_arg(args.rect) {
        Ok(rect) => rect,
        Err(e) => {
            return invalid_region_parameters::<vistools_core::FocusMapOutput>(
                "focus-map",
                &args.input,
                e,
            );
        }
    };

    let result = vistools_core::photo::execute_focus_map(&args.input, rect, args.rows, args.cols);
    let ok = result.ok;
    (serde_json::to_string_pretty(&result).unwrap(), ok)
}

pub fn run_white_balance(args: RegionArgs) -> (String, bool) {
    run_region_metric("white-balance", &args.input, args.rect, |input, rect| {
        vistools_core::photo::execute_white_balance(input, rect)
    })
}

fn parse_metering_mode(s: &str) -> Result<MeteringMode, String> {
    match s {
        "evaluative" => Ok(MeteringMode::Evaluative),
        "spot" => Ok(MeteringMode::Spot),
        "center-weighted" => Ok(MeteringMode::CenterWeighted),
        "highlight-weighted" => Ok(MeteringMode::HighlightWeighted),
        _ => Err(format!(
            "invalid metering mode '{s}', expected one of: evaluative, spot, center-weighted, highlight-weighted"
        )),
    }
}

pub fn run_highlight_clipping(args: ThresholdRegionArgs) -> (String, bool) {
    run_region_metric(
        "highlight-clipping",
        &args.input,
        args.rect,
        |input, rect| vistools_core::photo::execute_highlight_clipping(input, rect, args.threshold),
    )
}

pub fn run_shadow_clipping(args: ShadowThresholdRegionArgs) -> (String, bool) {
    run_region_metric("shadow-clipping", &args.input, args.rect, |input, rect| {
        vistools_core::photo::execute_shadow_clipping(input, rect, args.threshold)
    })
}

pub fn run_contrast(args: RegionArgs) -> (String, bool) {
    run_region_metric("contrast", &args.input, args.rect, |input, rect| {
        vistools_core::photo::execute_contrast(input, rect)
    })
}

pub fn run_color_cast(args: RegionArgs) -> (String, bool) {
    run_region_metric("color-cast", &args.input, args.rect, |input, rect| {
        vistools_core::photo::execute_color_cast(input, rect)
    })
}

fn run_region_metric<T: serde::Serialize>(
    operation: &str,
    input: &Path,
    rect: Option<String>,
    run: impl FnOnce(&Path, Option<Rect>) -> CommandResult<T>,
) -> (String, bool) {
    let rect = match parse_optional_rect_arg(rect) {
        Ok(rect) => rect,
        Err(e) => return invalid_region_parameters::<T>(operation, input, e),
    };

    let result = run(input, rect);
    let ok = result.ok;
    (serde_json::to_string_pretty(&result).unwrap(), ok)
}
