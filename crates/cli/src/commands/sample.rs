use std::path::PathBuf;

use crate::parse::{invalid_sample_parameters, parse_rect_arg, parse_u32_arg};
use clap::Args;

#[derive(Args)]
pub struct SampleArgs {
    /// Input image path.
    pub input: PathBuf,
    /// X coordinate for point sampling.
    #[arg(long)]
    pub x: Option<String>,
    /// Y coordinate for point sampling.
    #[arg(long)]
    pub y: Option<String>,
    /// Rect region for average sampling: x,y,width,height.
    #[arg(long)]
    pub rect: Option<String>,
}

pub fn run(args: SampleArgs) -> (String, bool) {
    let mode = match (args.x.as_deref(), args.y.as_deref(), args.rect.as_deref()) {
        (Some(x), Some(y), None) => match (parse_u32_arg("x", x), parse_u32_arg("y", y)) {
            (Ok(x), Ok(y)) => vistools_core::sample::SampleMode::Point { x, y },
            (Err(e), _) | (_, Err(e)) => return invalid_sample_parameters(&args.input, e),
        },
        (None, None, Some(rect)) => match parse_rect_arg(rect) {
            Ok(rect) => vistools_core::sample::SampleMode::Rect { rect },
            Err(e) => return invalid_sample_parameters(&args.input, e),
        },
        (None, None, None) => {
            return invalid_sample_parameters(
                &args.input,
                "sample requires either --x and --y, or --rect x,y,width,height",
            );
        }
        (Some(_), None, None) | (None, Some(_), None) => {
            return invalid_sample_parameters(&args.input, "--x and --y must be passed together");
        }
        _ => {
            return invalid_sample_parameters(
                &args.input,
                "sample accepts point mode (--x and --y) or rect mode (--rect), not both",
            );
        }
    };

    let result = vistools_core::sample::execute(&args.input, mode);
    let ok = result.ok;
    (serde_json::to_string_pretty(&result).unwrap(), ok)
}
