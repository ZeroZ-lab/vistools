//! vistools CLI entry point.
//!
//! Decisions: PD1 (JSON-first), PD5 (binary name vistools),
//! FD1 (CLI is thin layer calling core).
use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};
use vistools_core as core;
use vistools_core::types::*;

#[derive(Parser)]
#[command(
    name = "vistools",
    version,
    about = "Visual tools for AI agents — inspect, navigate, and crop images"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inspect image metadata and get strategy suggestion.
    Inspect {
        /// Input image path.
        input: PathBuf,
    },

    /// Generate a scaled-down overview image.
    Overview {
        /// Input image path.
        input: PathBuf,
        /// Output image path.
        output: PathBuf,
        /// Maximum output width in pixels.
        #[arg(long)]
        max_width: u32,
    },

    /// Split image into a grid of tiles.
    Tile {
        /// Input image path.
        input: PathBuf,
        /// Number of rows.
        #[arg(long)]
        rows: u32,
        /// Number of columns.
        #[arg(long)]
        cols: u32,
        /// Output directory for tiles.
        #[arg(long = "out-dir")]
        out_dir: PathBuf,
    },

    /// Crop a viewport region from the image.
    #[command(subcommand)]
    Viewport(ViewportCommands),

    /// Resize the image.
    Resize {
        /// Input image path.
        input: PathBuf,
        /// Output image path.
        output: PathBuf,
        /// Target width.
        #[arg(long)]
        width: u32,
        /// Target height (omit for proportional).
        #[arg(long)]
        height: Option<u32>,
    },

    /// Rotate the image (90/180/270 degrees).
    Rotate {
        /// Input image path.
        input: PathBuf,
        /// Output image path.
        output: PathBuf,
        /// Rotation degrees (0, 90, 180, 270).
        #[arg(long)]
        degrees: u32,
    },
}

#[derive(Subcommand)]
enum ViewportCommands {
    /// Crop using a nine-position anchor.
    Anchor {
        /// Input image path.
        input: PathBuf,
        /// Output image path.
        output: PathBuf,
        /// Anchor position (top-left, top, top-right, left, center, right, bottom-left, bottom, bottom-right).
        #[arg(long)]
        anchor: AnchorArg,
        /// Viewport width in pixels.
        #[arg(long)]
        width: u32,
        /// Viewport height in pixels.
        #[arg(long)]
        height: u32,
    },

    /// Crop using percentage coordinates (0.0–1.0).
    Percent {
        /// Input image path.
        input: PathBuf,
        /// Output image path.
        output: PathBuf,
        /// X origin as fraction (0.0–1.0).
        #[arg(long)]
        x: f64,
        /// Y origin as fraction (0.0–1.0).
        #[arg(long)]
        y: f64,
        /// Width as fraction (0.0–1.0).
        #[arg(long)]
        w: f64,
        /// Height as fraction (0.0–1.0).
        #[arg(long)]
        h: f64,
    },

    /// Crop using pixel coordinates.
    Rect {
        /// Input image path.
        input: PathBuf,
        /// Output image path.
        output: PathBuf,
        /// X origin in pixels.
        #[arg(long)]
        x: u32,
        /// Y origin in pixels.
        #[arg(long)]
        y: u32,
        /// Width in pixels.
        #[arg(long)]
        width: u32,
        /// Height in pixels.
        #[arg(long)]
        height: u32,
    },
}

/// Newtype wrapper for parsing Anchor from CLI string.
#[derive(Clone)]
struct AnchorArg(Anchor);

impl std::str::FromStr for AnchorArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "top-left" => Ok(AnchorArg(Anchor::TopLeft)),
            "top" => Ok(AnchorArg(Anchor::Top)),
            "top-right" => Ok(AnchorArg(Anchor::TopRight)),
            "left" => Ok(AnchorArg(Anchor::Left)),
            "center" => Ok(AnchorArg(Anchor::Center)),
            "right" => Ok(AnchorArg(Anchor::Right)),
            "bottom-left" => Ok(AnchorArg(Anchor::BottomLeft)),
            "bottom" => Ok(AnchorArg(Anchor::Bottom)),
            "bottom-right" => Ok(AnchorArg(Anchor::BottomRight)),
            _ => Err(format!(
                "unknown anchor '{s}'. Valid: top-left, top, top-right, left, center, right, bottom-left, bottom, bottom-right",
            )),
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let (json, ok) = match cli.command {
        Commands::Inspect { input } => {
            let result = core::inspect::execute(&input);
            let ok = result.ok;
            let json = serde_json::to_string_pretty(&result).unwrap();
            (json, ok)
        }
        Commands::Overview {
            input,
            output,
            max_width,
        } => {
            let result = core::overview::execute(&input, &output, max_width);
            let ok = result.ok;
            let json = serde_json::to_string_pretty(&result).unwrap();
            (json, ok)
        }
        Commands::Tile {
            input,
            rows,
            cols,
            out_dir,
        } => {
            let result = core::tile::execute(&input, rows, cols, &out_dir);
            let ok = result.ok;
            let json = serde_json::to_string_pretty(&result).unwrap();
            (json, ok)
        }
        Commands::Viewport(sub) => match sub {
            ViewportCommands::Anchor {
                input,
                output,
                anchor,
                width,
                height,
            } => {
                let mode = core::viewport::ViewportMode::Anchor {
                    anchor: anchor.0,
                    width,
                    height,
                };
                let result = core::viewport::execute(&input, &output, mode);
                let ok = result.ok;
                let json = serde_json::to_string_pretty(&result).unwrap();
                (json, ok)
            }
            ViewportCommands::Percent {
                input,
                output,
                x,
                y,
                w,
                h,
            } => {
                let mode = core::viewport::ViewportMode::Percent {
                    pct: Percent { x, y, w, h },
                };
                let result = core::viewport::execute(&input, &output, mode);
                let ok = result.ok;
                let json = serde_json::to_string_pretty(&result).unwrap();
                (json, ok)
            }
            ViewportCommands::Rect {
                input,
                output,
                x,
                y,
                width,
                height,
            } => {
                let mode = core::viewport::ViewportMode::Rect {
                    rect: Rect {
                        x,
                        y,
                        width,
                        height,
                    },
                };
                let result = core::viewport::execute(&input, &output, mode);
                let ok = result.ok;
                let json = serde_json::to_string_pretty(&result).unwrap();
                (json, ok)
            }
        },
        Commands::Resize {
            input,
            output,
            width,
            height,
        } => {
            let result = core::resize::execute(&input, &output, width, height);
            let ok = result.ok;
            let json = serde_json::to_string_pretty(&result).unwrap();
            (json, ok)
        }
        Commands::Rotate {
            input,
            output,
            degrees,
        } => {
            let result = core::rotate::execute(&input, &output, degrees);
            let ok = result.ok;
            let json = serde_json::to_string_pretty(&result).unwrap();
            (json, ok)
        }
    };

    println!("{json}");

    if !ok {
        process::exit(1);
    }
}
