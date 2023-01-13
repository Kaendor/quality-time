use eyre::{Context, Result};
use git::{Gitoxide, RepositoryExplorer};
use metrics::FileMetrics;
use std::path::PathBuf;

use crate::metrics::metrics_per_file;

pub mod git;
pub mod metrics;
pub mod output;

pub fn get_metrics(path_to_repo: PathBuf) -> Result<Vec<FileMetrics>> {
    let git_explorer =
        Gitoxide::try_new(path_to_repo).wrap_err("Unable to initialise repository")?;
    let change_map = git_explorer
        .change_count_per_file()
        .wrap_err("Unable to obtain the change count per file")?;
    let results = metrics_per_file(change_map);
    Ok(results)
}
