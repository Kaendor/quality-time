use clap::Parser;
use eyre::{Context, Result};
use std::env;

use crate::git::change_count_in_path;
use crate::metrics::metrics_per_file;
use crate::output::{print_output, OutputMode};

mod git;
mod metrics;
mod output;

/// Simple program to get complexity and churn metrics
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Output style of the CLI
    #[arg(short, long, value_enum)]
    output: Option<OutputMode>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let path_to_repo = env::current_dir().expect("current dir");
    let output = args.output.unwrap_or(OutputMode::StdOut);

    let change_map = change_count_in_path(path_to_repo)
        .wrap_err("Unable to obtain the change count per file")?;

    let results = metrics_per_file(change_map);

    print_output(output, results);

    Ok(())
}
