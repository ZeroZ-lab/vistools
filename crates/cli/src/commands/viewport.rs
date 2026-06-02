use std::path::PathBuf;
use std::str::FromStr;

use clap::{Args, Subcommand};
use vistools_core::{Anchor, Percent, Rect};

#[derive(Args)]
pub struct ViewportArgs {
    #[command(subcommand)]
    pub command: ViewportCommands,
}

#[derive(Subcommand)]
pub enum ViewportCommands {
    /// Crop using a nine-position anchor.
    Anchor(ViewportAnchorArgs),
    /// Crop using percentage coordinates (0.0–1.0).
    Percent(ViewportPercentArgs),
    /// Crop using pixel coordinates.
    Rect(ViewportRectArgs),
}

#[derive(Args)]
pub struct ViewportAnchorArgs {
    /// Input image path.
    pub input: PathBuf,
    /// Output image path.
    pub output: PathBuf,
    /// Anchor position (top-left, top, top-right, left, center, right, bottom-left, bottom, bottom-right).
    #[arg(long)]
    pub anchor: AnchorArg,
    /// Viewport width in pixels.
    #[arg(long)]
    pub width: u32,
    /// Viewport height in pixels.
    #[arg(long)]
    pub height: u32,
}

#[derive(Args)]
pub struct ViewportPercentArgs {
    /// Input image path.
    pub input: PathBuf,
    /// Output image path.
    pub output: PathBuf,
    /// X origin as fraction (0.0–1.0).
    #[arg(long)]
    pub x: f64,
    /// Y origin as fraction (0.0–1.0).
    #[arg(long)]
    pub y: f64,
    /// Width as fraction (0.0–1.0).
    #[arg(long)]
    pub w: f64,
    /// Height as fraction (0.0–1.0).
    #[arg(long)]
    pub h: f64,
}

#[derive(Args)]
pub struct ViewportRectArgs {
    /// Input image path.
    pub input: PathBuf,
    /// Output image path.
    pub output: PathBuf,
    /// X origin in pixels.
    #[arg(long)]
    pub x: u32,
    /// Y origin in pixels.
    #[arg(long)]
    pub y: u32,
    /// Width in pixels.
    #[arg(long)]
    pub width: u32,
    /// Height in pixels.
    #[arg(long)]
    pub height: u32,
}

/// Newtype wrapper for parsing Anchor from CLI string.
#[derive(Clone)]
pub struct AnchorArg(pub Anchor);

impl FromStr for AnchorArg {
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

pub fn run(args: ViewportArgs) -> (String, bool) {
    let result = match args.command {
        ViewportCommands::Anchor(args) => vistools_core::viewport::execute(
            &args.input,
            &args.output,
            vistools_core::viewport::ViewportMode::Anchor {
                anchor: args.anchor.0,
                width: args.width,
                height: args.height,
            },
        ),
        ViewportCommands::Percent(args) => vistools_core::viewport::execute(
            &args.input,
            &args.output,
            vistools_core::viewport::ViewportMode::Percent {
                pct: Percent {
                    x: args.x,
                    y: args.y,
                    w: args.w,
                    h: args.h,
                },
            },
        ),
        ViewportCommands::Rect(args) => vistools_core::viewport::execute(
            &args.input,
            &args.output,
            vistools_core::viewport::ViewportMode::Rect {
                rect: Rect {
                    x: args.x,
                    y: args.y,
                    width: args.width,
                    height: args.height,
                },
            },
        ),
    };
    let ok = result.ok;
    (serde_json::to_string_pretty(&result).unwrap(), ok)
}
