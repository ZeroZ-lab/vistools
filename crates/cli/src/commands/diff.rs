use std::path::PathBuf;

use clap::Args;

use crate::parse::{invalid_region_parameters, parse_optional_rect_arg};

#[derive(Args)]
pub struct DiffArgs {
    /// Expected/reference image path.
    pub expected: PathBuf,
    /// Actual image path.
    pub actual: PathBuf,
    /// Optional source-coordinate rect compared in both images: x,y,width,height.
    #[arg(long)]
    pub rect: Option<String>,
}

pub fn run(args: DiffArgs) -> (String, bool) {
    let rect = match parse_optional_rect_arg(args.rect) {
        Ok(rect) => rect,
        Err(e) => {
            return invalid_region_parameters::<vistools_core::DiffOutput>(
                "diff",
                &args.expected,
                e,
            );
        }
    };

    let result = vistools_core::diff::execute_diff(&args.expected, &args.actual, rect);
    let ok = result.ok;
    (serde_json::to_string_pretty(&result).unwrap(), ok)
}
