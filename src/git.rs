use std::collections::HashMap;
use std::path::PathBuf;

use eyre::{Result, WrapErr};
use git_repository::{discover, Repository};
use git_repository::{objs::tree::EntryMode, traverse::tree::Recorder, Commit};

use crate::metrics::Churn;

pub trait RepositoryExplorer {
    fn change_count_per_file(&self) -> Result<HashMap<String, Churn>>;
}

pub struct Gitoxide {
    repository: Repository,
}

impl Gitoxide {
    pub fn try_new(path_to_repo: PathBuf) -> Result<Self> {
        let repository =
            discover(path_to_repo).wrap_err("Repository not found or without commits")?;
        Ok(Self { repository })
    }
}

impl RepositoryExplorer for Gitoxide {
    fn change_count_per_file(&self) -> Result<HashMap<String, Churn>> {
        let head = self
            .repository
            .head_commit()
            .wrap_err("Unable to get the head of the repo")?;

        let commits = head
            .ancestors()
            .all()
            .wrap_err("Unable to obtain commit ancestors of the current HEAD")?
            .flatten()
            .flat_map(|reference| reference.object())
            .flat_map(Commit::try_from);

        let mut change_map = HashMap::new();

        for commit in commits {
            let change_tree = commit.tree().wrap_err(format!(
                "Unable to obtain the change tree of commit {}",
                commit.id
            ))?;

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

        let map = change_map
            .into_iter()
            .map(|(file, churn)| (file, Churn::from(churn)))
            .collect();

        Ok(map)
    }
}

#[cfg(test)]
mod integration {
    use std::env;

    use super::{Gitoxide, RepositoryExplorer};

    #[test]
    fn gitoxide_count_changes() {
        let path = env::current_dir().expect("current dir path");
        let explorer = Gitoxide::try_new(path).expect("gitoxide init");

        let churn_metrics = explorer
            .change_count_per_file()
            .expect("list of change count");
        assert!(churn_metrics.contains_key("src/main.rs"));
    }
}
