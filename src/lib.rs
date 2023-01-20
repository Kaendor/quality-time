use crate::metrics::metrics_per_file;
use eyre::{Context, Result};

pub mod git;
pub mod metrics;
pub mod output;

pub use crate::git::RepositoryExplorer;
pub use crate::metrics::{Churn, MetricReader, ProjectMetrics};

pub fn get_metrics(
    git_explorer: impl RepositoryExplorer,
    reader: impl MetricReader,
) -> Result<ProjectMetrics> {
    let change_map = git_explorer
        .change_count_per_file()
        .wrap_err("Unable to obtain the change count per file")?;
    let mut results = metrics_per_file(change_map, reader);

    results.sort_by(|a, b| a.magnitude().partial_cmp(&b.magnitude()).unwrap());
    results.reverse();

    Ok(ProjectMetrics::new(results))
}
