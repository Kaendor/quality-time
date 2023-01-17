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
    let mut results = metrics_per_file(change_map, reader);

    results.sort_by(|a, b| a.magnitude().partial_cmp(&b.magnitude()).unwrap());
    results.reverse();

    Ok(results)
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::Path};

    use crate::{
        git::RepositoryExplorer,
        metrics::{Churn, MetricReader},
    };

    use super::get_metrics;

    struct TestReader {}

    impl MetricReader for TestReader {
        fn get_cyclomatic_from_path_and_content(&self, _path: &Path) -> Option<f64> {
            Some(1.0)
        }
    }

    struct TestExplorer {}

    impl RepositoryExplorer for TestExplorer {
        fn change_count_per_file(
            &self,
        ) -> eyre::Result<std::collections::HashMap<String, crate::metrics::Churn>> {
            Ok(HashMap::from([("file".to_string(), Churn::from(1))]))
        }
    }

    #[test]
    fn list_metrics() {
        let metrics = get_metrics(TestExplorer {}, TestReader {}).expect("metrics");

        assert!(!metrics.is_empty())
    }
}
