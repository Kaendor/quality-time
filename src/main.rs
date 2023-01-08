use std::path::Path;
use std::{collections::HashMap, env};

use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::Table;
use git_repository::{discover, objs::tree::EntryMode, traverse::tree::Recorder, Commit};
use rust_code_analysis::{metrics, read_file_with_eol, ParserTrait, RustParser};

use crate::metrics::FileMetrics;

mod metrics;

fn main() {
    let current_dir = env::current_dir().expect("current dir");

    let repo = discover(current_dir).expect("repo");

    let head = repo.head_commit().expect("head");
    let mut change_map = HashMap::new();

    for reference in head.ancestors().all().expect("all refs").flatten() {
        let object = reference.object().expect("object");

        let commit: Commit = Commit::try_from(object).expect("commit");

        let change_tree = commit.tree().expect("tree");

        let changes = change_tree.traverse();

        let mut recorder = Recorder::default();
        let _ = changes.breadthfirst(&mut recorder);

        for entry in recorder.records {
            if let &EntryMode::Blob = &entry.mode {
                match change_map.entry(entry.filepath) {
                    std::collections::hash_map::Entry::Occupied(mut e) => {
                        *e.get_mut() += 1;
                    }
                    std::collections::hash_map::Entry::Vacant(e) => {
                        e.insert(1);
                    }
                }
            }
        }
    }

    let results: Vec<FileMetrics> = change_map
        .into_iter() // TODO: parallelize
        .filter_map(|(filename, churn)| {
            let filename = filename.to_string();
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
