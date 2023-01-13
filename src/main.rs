use clap::Parser;
use eyre::Result;
use std::env;

use quality_time::{
    get_metrics,
    output::{print_output, OutputMode},
};

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
    let output = args.output.unwrap_or(OutputMode::StdOut);
    let path_to_repo = env::current_dir().expect("current dir");

    let results = get_metrics(path_to_repo)?;

    print_output(output, results);

    Ok(())
}
