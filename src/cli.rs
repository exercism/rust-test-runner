use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
pub struct CliArgs {
    pub slug: String,
    pub input_dir: PathBuf,
    pub output_dir: PathBuf,
}
