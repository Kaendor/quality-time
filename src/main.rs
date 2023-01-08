use std::env;

use git_repository::discover;

use crate::git::change_count_per_file;
use crate::metrics::metrics_per_file;
use crate::output::{print_output, OutputMode};

mod git;
mod metrics;
mod output;

// TODO: Use local error type instead of expects
fn main() {
    let path_to_repo = env::current_dir().expect("current dir");
    let output = OutputMode::StdOut;

    let repo = discover(path_to_repo).expect("repo");

    let change_map = change_count_per_file(repo);

    let results = metrics_per_file(change_map);

    print_output(output, results);
}
