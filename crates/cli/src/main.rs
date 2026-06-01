//! vistools CLI entry point.
//!
//! Decisions: PD1 (JSON-first), PD5 (binary name vistools),
//! FD1 (CLI is thin layer calling core).
use std::path::{Path, PathBuf};
use std::process;

use clap::{Parser, Subcommand};
use vistools_core as core;
use vistools_core::types::*;

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
        /// Maximum output long side in pixels.
        #[arg(long)]
        max_side: u32,
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

    /// Sample point or region color without writing an output image.
    Sample {
        /// Input image path.
        input: PathBuf,
        /// X coordinate for point sampling.
        #[arg(long)]
        x: Option<String>,
        /// Y coordinate for point sampling.
        #[arg(long)]
        y: Option<String>,
        /// Rect region for average sampling: x,y,width,height.
        #[arg(long)]
        rect: Option<String>,
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
            max_side,
        } => {
            let result = core::overview::execute(&input, &output, max_side);
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
        Commands::Sample { input, x, y, rect } => run_sample(input, x, y, rect),
    };

    println!("{json}");

    if !ok {
        process::exit(1);
    }
}

fn run_sample(
    input: PathBuf,
    x: Option<String>,
    y: Option<String>,
    rect: Option<String>,
) -> (String, bool) {
    let mode = match (x.as_deref(), y.as_deref(), rect.as_deref()) {
        (Some(x), Some(y), None) => match (parse_u32_arg("x", x), parse_u32_arg("y", y)) {
            (Ok(x), Ok(y)) => core::sample::SampleMode::Point { x, y },
            (Err(e), _) | (_, Err(e)) => return invalid_sample_parameters(&input, e),
        },
        (None, None, Some(rect)) => match parse_rect_arg(rect) {
            Ok(rect) => core::sample::SampleMode::Rect { rect },
            Err(e) => return invalid_sample_parameters(&input, e),
        },
        (None, None, None) => {
            return invalid_sample_parameters(
                &input,
                "sample requires either --x and --y, or --rect x,y,width,height",
            );
        }
        (Some(_), None, None) | (None, Some(_), None) => {
            return invalid_sample_parameters(&input, "--x and --y must be passed together");
        }
        _ => {
            return invalid_sample_parameters(
                &input,
                "sample accepts point mode (--x and --y) or rect mode (--rect), not both",
            );
        }
    };

    let result = core::sample::execute(&input, mode);
    let ok = result.ok;
    let json = serde_json::to_string_pretty(&result).unwrap();
    (json, ok)
}

fn invalid_sample_parameters(input: &Path, message: impl Into<String>) -> (String, bool) {
    let result = CommandResult::<SampleOutput>::err(
        "sample",
        input.display().to_string(),
        ErrorInfo::with_message(ErrorCode::InvalidParameters, message),
    );
    let json = serde_json::to_string_pretty(&result).unwrap();
    (json, false)
}

fn parse_u32_arg(name: &str, value: &str) -> Result<u32, String> {
    value
        .trim()
        .parse::<u32>()
        .map_err(|_| format!("{name} must be an unsigned integer"))
}

fn parse_rect_arg(value: &str) -> Result<Rect, String> {
    let parts: Vec<_> = value.split(',').map(str::trim).collect();
    if parts.len() != 4 || parts.iter().any(|part| part.is_empty()) {
        return Err("rect must use x,y,width,height syntax".to_string());
    }

    let x = parse_u32_arg("rect.x", parts[0])?;
    let y = parse_u32_arg("rect.y", parts[1])?;
    let width = parse_u32_arg("rect.width", parts[2])?;
    let height = parse_u32_arg("rect.height", parts[3])?;

    Ok(Rect {
        x,
        y,
        width,
        height,
    })
}
