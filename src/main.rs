use std::env;

use crate::git::change_count_in_path;
use crate::metrics::metrics_per_file;
use crate::output::{print_output, OutputMode};

mod git;
mod metrics;
mod output;

fn main() {
    let path_to_repo = env::current_dir().expect("current dir");
    let output = OutputMode::App;

    let change_map = change_count_in_path(path_to_repo);

    let results = metrics_per_file(change_map);

    print_output(output, results);
}
