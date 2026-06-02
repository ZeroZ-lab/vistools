//! vistools CLI entry point.
//!
//! Decisions: PD1 (JSON-first), PD5 (binary name vistools),
//! FD1 (CLI is thin layer calling core).
use std::process;

use clap::{Parser, Subcommand};

mod commands;
mod parse;

#[derive(Parser)]
#[command(
    name = "vistools",
    version,
    about = "Visual instruments for AI agents — inspect, navigate, crop, and sample images with coordinate mapping"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Inspect(commands::inspect::InspectArgs),
    Overview(commands::overview::OverviewArgs),
    Tile(commands::tile::TileArgs),
    Viewport(commands::viewport::ViewportArgs),
    Sample(commands::sample::SampleArgs),
    Diff(commands::diff::DiffArgs),
    Sharpness(commands::photo::RegionArgs),
    Histogram(commands::photo::HistogramArgs),
    HighlightClipping(commands::photo::ThresholdRegionArgs),
    ShadowClipping(commands::photo::ShadowThresholdRegionArgs),
    Contrast(commands::photo::RegionArgs),
    ColorCast(commands::photo::RegionArgs),
    #[command(name = "zone-map")]
    ZoneMap(commands::photo::RegionArgs),
    Exposure(commands::photo::ExposureArgs),
    #[command(name = "focus-map")]
    FocusMap(commands::photo::FocusMapArgs),
    #[command(name = "white-balance")]
    WhiteBalance(commands::photo::RegionArgs),
}

fn main() {
    let cli = Cli::parse();
    let (json, ok) = match cli.command {
        Commands::Inspect(args) => commands::inspect::run(args),
        Commands::Overview(args) => commands::overview::run(args),
        Commands::Tile(args) => commands::tile::run(args),
        Commands::Viewport(args) => commands::viewport::run(args),
        Commands::Sample(args) => commands::sample::run(args),
        Commands::Diff(args) => commands::diff::run(args),
        Commands::Sharpness(args) => commands::photo::run_sharpness(args),
        Commands::Histogram(args) => commands::photo::run_histogram(args),
        Commands::HighlightClipping(args) => commands::photo::run_highlight_clipping(args),
        Commands::ShadowClipping(args) => commands::photo::run_shadow_clipping(args),
        Commands::Contrast(args) => commands::photo::run_contrast(args),
        Commands::ColorCast(args) => commands::photo::run_color_cast(args),
        Commands::ZoneMap(args) => commands::photo::run_zone_map(args),
        Commands::Exposure(args) => commands::photo::run_exposure(args),
        Commands::FocusMap(args) => commands::photo::run_focus_map(args),
        Commands::WhiteBalance(args) => commands::photo::run_white_balance(args),
    };

    println!("{json}");

    if !ok {
        process::exit(1);
    }
}
