use std::{collections::HashMap, path::Path};

use quality_time::{get_metrics, Churn, MetricReader, RepositoryExplorer};

struct TestReader {}

impl MetricReader for TestReader {
    fn get_cyclomatic_from_path_and_content(&self, _path: &Path) -> Option<f64> {
        Some(1.0)
    }
}

struct TestExplorer {}

impl RepositoryExplorer for TestExplorer {
    fn change_count_per_file(&self) -> eyre::Result<std::collections::HashMap<String, Churn>> {
        Ok(HashMap::from([("file".to_string(), Churn::from(1))]))
    }
}

#[test]
fn list_metrics() {
    let metrics = get_metrics(TestExplorer {}, TestReader {}).expect("metrics");

    assert!(!metrics.file_metrics().is_empty())
}
