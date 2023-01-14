use clap::Parser;
use eyre::{Context, Result};
use std::env;

use quality_time::{
    get_metrics,
    git::Gitoxide,
    metrics::CodeAnalysisReader,
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
    let git_explorer =
        Gitoxide::try_new(path_to_repo).wrap_err("Unable to initialise repository")?;

    let reader = CodeAnalysisReader {};

    let results = get_metrics(git_explorer, reader)?;

    print_output(output, results);

    Ok(())
}
