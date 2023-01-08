use std::env;
use std::path::Path;

use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::Table;
use git_repository::discover;
use rust_code_analysis::{metrics, read_file_with_eol, ParserTrait, RustParser};

use crate::git::change_count_per_file;
use crate::metrics::FileMetrics;

mod git;
mod metrics;

// TODO: Use local error type instead of expects
fn main() {
    let path_to_repo = env::current_dir().expect("current dir");

    let repo = discover(path_to_repo).expect("repo");

    let change_map = change_count_per_file(repo);

    let results: Vec<FileMetrics> = change_map
        .into_iter() // TODO: parallelize
        .filter_map(|(filename, churn)| {
            let filename = filename;
            let path = Path::new(&filename);

            let complexity = read_file_with_eol(path)
                .ok()
                .and_then(|file| {
                    let parser = RustParser::new(file.expect("file content"), path, None);
                    metrics(&parser, path)
                })
                .map(|metrics| metrics.metrics.cyclomatic.cyclomatic_sum());

            complexity.map(|complexity| FileMetrics::new(filename.to_string(), churn, complexity))
        })
        .collect();

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
