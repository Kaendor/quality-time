use std::collections::HashMap;

use eyre::{Result, WrapErr};
use git_repository::Repository;
use git_repository::{objs::tree::EntryMode, traverse::tree::Recorder, Commit};

pub trait RepositoryExplorer {
    fn commits(&self) -> Result<Vec<Commit>>;

    fn change_count_per_file(&self) -> Result<HashMap<String, i32>>;
}

pub struct Gitoxide {
    repository: Repository,
}
impl Gitoxide {
    pub fn new(repository: Repository) -> Self {
        Self { repository }
    }
}

impl RepositoryExplorer for Gitoxide {
    fn commits(&self) -> Result<Vec<Commit>> {
        let head = self
            .repository
            .head_commit()
            .wrap_err("Unable to get the head of the repo")?;
        // TODO: optimize git lookup
        let commits = head
            .ancestors()
            .all()
            .expect("all refs")
            .flatten()
            .flat_map(|reference| reference.object())
            .flat_map(Commit::try_from)
            .collect();

        Ok(commits)
    }

    fn change_count_per_file(&self) -> Result<HashMap<String, i32>> {
        let mut change_map = HashMap::new();

        let commits = self
            .commits()
            .wrap_err("Unable to get all the commit for the repo")?;

        for commit in commits {
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

        Ok(change_map)
    }
}
