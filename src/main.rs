use std::env;

use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::Table;
use git_repository::discover;

use crate::git::change_count_per_file;
use crate::metrics::metrics_per_file;

mod git;
mod metrics;

// TODO: Use local error type instead of expects
fn main() {
    let path_to_repo = env::current_dir().expect("current dir");

    let repo = discover(path_to_repo).expect("repo");

    let change_map = change_count_per_file(repo);

    let results = metrics_per_file(change_map);

    let mut table = Table::new();
    table
        .set_header(vec!["Filename", "Churn", "Complexity"])
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS);

    for metric in results.iter() {
        table.add_row(vec![
            &metric.filename,
            &metric.churn.to_string(),
            &metric.complexity.to_string(),
        ]);
    }

    println!("{table}");
}
