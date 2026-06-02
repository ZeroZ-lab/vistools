use std::path::PathBuf;

use clap::Args;

#[derive(Args)]
pub struct OverviewArgs {
    /// Input image path.
    pub input: PathBuf,
    /// Output image path.
    pub output: PathBuf,
    /// Maximum output long side in pixels.
    #[arg(long)]
    pub max_side: u32,
}

pub fn run(args: OverviewArgs) -> (String, bool) {
    let result = vistools_core::overview::execute(&args.input, &args.output, args.max_side);
    let ok = result.ok;
    (serde_json::to_string_pretty(&result).unwrap(), ok)
}
