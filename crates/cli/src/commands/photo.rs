use std::path::{Path, PathBuf};

use clap::Args;
use vistools_core::{CommandResult, Rect, DEFAULT_HIGHLIGHT_THRESHOLD, DEFAULT_SHADOW_THRESHOLD};

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

pub fn run_histogram(args: RegionArgs) -> (String, bool) {
    run_region_metric("histogram", &args.input, args.rect, |input, rect| {
        vistools_core::photo::execute_histogram(input, rect)
    })
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
