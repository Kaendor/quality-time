use std::collections::HashMap;

use git_repository::Repository;
use git_repository::{objs::tree::EntryMode, traverse::tree::Recorder, Commit};

pub fn change_count_per_file(repository: Repository) -> HashMap<String, i32> {
    let head = repository.head_commit().expect("head");
    let mut change_map = HashMap::new();

    // TODO: optimize git lookup
    for reference in head.ancestors().all().expect("all refs").flatten() {
        let object = reference.object().expect("object");

        let commit: Commit = Commit::try_from(object).expect("commit");

        let change_tree = commit.tree().expect("tree");

        let changes = change_tree.traverse();

        let mut recorder = Recorder::default();
        let _ = changes.breadthfirst(&mut recorder);

        for entry in recorder.records {
            if let &EntryMode::Blob = &entry.mode {
                match change_map.entry(entry.filepath.to_string()) {
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

    change_map
}
