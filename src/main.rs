use clap::Parser;
use eyre::{Context, Result};
use git::{Gitoxide, RepositoryExplorer};
use git_repository::discover;
use std::env;

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

    let output = args.output.unwrap_or(OutputMode::StdOut);

    let path_to_repo = env::current_dir().expect("current dir");
    let repository = discover(path_to_repo).expect("Repository not found or without commits");

    let git_explorer = Gitoxide::new(repository);

    let change_map = git_explorer
        .change_count_per_file()
        .wrap_err("Unable to obtain the change count per file")?;

    let results = metrics_per_file(change_map);

    print_output(output, results);

    Ok(())
}
