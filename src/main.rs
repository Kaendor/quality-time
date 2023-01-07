mod cli;

use std::error::Error;
use std::fmt;
use std::{collections::HashMap, env};

use git_repository::{discover, objs::tree::EntryMode, traverse::tree::Recorder, Commit};

#[derive(Debug)]
struct MyError {}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", "WTF")
    }
}

impl Error for MyError {
    fn description(&self) -> &str {
        "WTF"
    }
}

fn main() {
    // let opts = Opts::parse();

    // start_cli(opts);
    let repo = discover(env::current_dir().expect("current dir")).expect("repo");

    let head = repo.head_commit().expect("head");
    let mut change_map = HashMap::new();

    for reference in head.ancestors().all().expect("all refs").flatten() {
        let object = reference.object().expect("object");

        let commit: Commit = Commit::try_from(object).expect("commit");

        dbg!(commit.message());

        let change_tree = commit.tree().expect("tree");

        let changes = change_tree.traverse();

        let mut recorder = Recorder::default();
        let _ = changes.breadthfirst(&mut recorder);

        for entry in recorder.records {
            if let &EntryMode::Blob = &entry.mode {
                dbg!(&entry.filepath);
                if change_map.contains_key(&entry.filepath) {
                    change_map
                        .entry(entry.filepath)
                        .and_modify(|count| *count += 1);
                } else {
                    change_map.insert(entry.filepath, 1);
                }
            }
        }

        dbg!(&change_map);
    }
}
