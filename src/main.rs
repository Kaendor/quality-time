use clap::Parser;
use eyre::{Context, Result};
use std::path::PathBuf;

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

    /// The path of the repository to analyse
    #[arg(short, long, value_name = "PROJECT")]
    project_path: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let output = args.output.unwrap_or(OutputMode::StdOut);

    let git_explorer =
        Gitoxide::try_new(args.project_path).wrap_err("Unable to initialise repository")?;

    let reader = CodeAnalysisReader::default();

    let results = get_metrics(git_explorer, reader)?;

    print_output(output, results);

    Ok(())
}
