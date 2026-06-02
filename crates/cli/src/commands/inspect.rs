use std::path::PathBuf;

use clap::Args;

#[derive(Args)]
pub struct InspectArgs {
    /// Input image path.
    pub input: PathBuf,
}

pub fn run(args: InspectArgs) -> (String, bool) {
    let result = vistools_core::inspect::execute(&args.input);
    let ok = result.ok;
    (serde_json::to_string_pretty(&result).unwrap(), ok)
}
