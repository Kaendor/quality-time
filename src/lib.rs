use eyre::{Context, Result};
use git::RepositoryExplorer;
use metrics::{FileMetrics, MetricReader};

use crate::metrics::metrics_per_file;

pub mod git;
pub mod metrics;
pub mod output;

pub fn get_metrics(
    git_explorer: impl RepositoryExplorer,
    reader: impl MetricReader,
) -> Result<Vec<FileMetrics>> {
    let change_map = git_explorer
        .change_count_per_file()
        .wrap_err("Unable to obtain the change count per file")?;
    let results = metrics_per_file(change_map, reader);
    Ok(results)
}
