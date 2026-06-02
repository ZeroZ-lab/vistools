use std::path::PathBuf;

use clap::Args;

#[derive(Args)]
pub struct TileArgs {
    /// Input image path.
    pub input: PathBuf,
    /// Number of rows.
    #[arg(long)]
    pub rows: u32,
    /// Number of columns.
    #[arg(long)]
    pub cols: u32,
    /// Output directory for tiles.
    #[arg(long = "out-dir")]
    pub out_dir: PathBuf,
}

pub fn run(args: TileArgs) -> (String, bool) {
    let result = vistools_core::tile::execute(&args.input, args.rows, args.cols, &args.out_dir);
    let ok = result.ok;
    (serde_json::to_string_pretty(&result).unwrap(), ok)
}
