use std::env;

use crate::git::change_count_in_path;
use crate::metrics::metrics_per_file;
use crate::output::{print_output, OutputMode};

mod git;
mod metrics;
mod output;

// TODO: Add all the tests
// TODO: Use local error type instead of expects
// TODO: Add ignoring file possible
// TODO: Add repo path configurable
fn main() {
    let path_to_repo = env::current_dir().expect("current dir");
    let output = OutputMode::StdOut;

    let change_map = change_count_in_path(path_to_repo);

    let results = metrics_per_file(change_map);

    print_output(output, results);
}
