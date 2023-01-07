use std::path::Path;
use std::{collections::HashMap, env};

use git_repository::{discover, objs::tree::EntryMode, traverse::tree::Recorder, Commit};
use rust_code_analysis::{metrics, read_file_with_eol, ParserTrait, RustParser};

#[derive(Debug, Clone)]
struct FileMetrics {
    filename: String,
    churn: i32,
    complexity: f64,
}

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
        .into_iter()
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

            complexity.map(|complexity| FileMetrics {
                churn,
                filename: filename.to_string(),
                complexity,
            })
        })
        .collect();

    dbg!(results);
}
