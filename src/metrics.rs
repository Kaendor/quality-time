use std::{collections::HashMap, fmt::Display, path::Path};

use rust_code_analysis::{metrics, read_file_with_eol, CodeMetrics, ParserTrait, RustParser};

pub trait MetricReader {
    fn get_cyclomatic_from_path_and_content(&self, path: &Path) -> Option<f64>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Churn(i32);

impl Churn {
    pub fn as_f64(self) -> f64 {
        self.0 as f64
    }
}

impl From<i32> for Churn {
    fn from(src: i32) -> Self {
        Self(src)
    }
}

impl Display for Churn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub struct CodeAnalysisReader {}

impl Default for CodeAnalysisReader {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeAnalysisReader {
    pub fn new() -> Self {
        Self {}
    }

    fn metric_from_path_and_content(
        &self,
        content: Option<Vec<u8>>,
        path: &Path,
    ) -> Option<CodeMetrics> {
        let parser = RustParser::new(content.expect("file content"), path, None);
        metrics(&parser, path).map(|v| v.metrics)
    }
}

impl MetricReader for CodeAnalysisReader {
    fn get_cyclomatic_from_path_and_content(&self, path: &Path) -> Option<f64> {
        read_file_with_eol(path)
            .ok()
            .and_then(|file| self.metric_from_path_and_content(file, path))
            .map(|metrics| metrics.cyclomatic.cyclomatic_sum())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileMetrics {
    pub filename: String,
    pub churn: Churn,
    pub complexity: f64,
}

#[derive(Debug, Clone)]
pub struct ProjectMetrics {
    file_metrics: Vec<FileMetrics>,
}

impl ProjectMetrics {
    pub fn new(metrics: Vec<FileMetrics>) -> Self {
        Self {
            file_metrics: metrics,
        }
    }

    pub fn file_metrics(&self) -> &Vec<FileMetrics> {
        &self.file_metrics
    }

    pub fn maximum_churn(&self) -> f64 {
        self.file_metrics
            .iter()
            .max_by_key(|x| x.churn.as_f64() as i64)
            .map(|x| x.churn.as_f64() + 10.0)
            .unwrap_or(10.0)
    }

    pub fn churn_sum(&self) -> f64 {
        self.file_metrics.iter().map(|x| x.churn.as_f64()).sum()
    }

    pub fn maximum_complexity(&self) -> f64 {
        self.file_metrics
            .iter()
            .max_by_key(|x| x.complexity.round() as i64)
            .map(|x| x.complexity)
            .unwrap_or_default()
    }

    pub fn complexity_sum(&self) -> f64 {
        self.file_metrics.iter().map(|x| x.complexity).sum()
    }
}

impl FileMetrics {
    pub fn new(filename: String, churn: Churn, complexity: f64) -> Self {
        Self {
            filename,
            churn,
            complexity,
        }
    }

    pub fn magnitude(&self) -> f64 {
        let origin = (0.0, 0.0);
        let distance_to_origin = ((origin.0 - self.churn.as_f64()).powi(2)
            + (origin.1 - self.complexity as f64).powi(2))
        .sqrt();
        distance_to_origin
    }

    pub fn to_point(&self) -> (f64, f64) {
        (self.churn.as_f64(), self.complexity)
    }
}

pub fn metrics_per_file(
    file_map: HashMap<String, Churn>,
    reader: impl MetricReader,
) -> Vec<FileMetrics> {
    file_map
        .into_iter() // TODO: parallelize
        .filter_map(|(filename, churn)| {
            let path = Path::new(&filename);

            let complexity = reader.get_cyclomatic_from_path_and_content(path);

            complexity.map(|complexity| FileMetrics::new(filename.to_string(), churn, complexity))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::Path};

    use crate::metrics::Churn;

    use super::{metrics_per_file, FileMetrics, MetricReader, ProjectMetrics};

    struct TestReader {}

    impl MetricReader for TestReader {
        fn get_cyclomatic_from_path_and_content(&self, _path: &Path) -> Option<f64> {
            Some(1.0)
        }
    }

    #[test]
    fn project_metric_compute() {
        let metric_data = vec![
            FileMetrics {
                churn: Churn::from(15),
                complexity: 20.0,
                filename: "foo.rs".to_string(),
            },
            FileMetrics {
                churn: Churn::from(10),
                complexity: 30.0,
                filename: "foo.rs".to_string(),
            },
            FileMetrics {
                churn: Churn::from(20),
                complexity: 10.0,
                filename: "foo.rs".to_string(),
            },
        ];

        let metrics = ProjectMetrics::new(metric_data);
        let max_churn = metrics.maximum_churn();
        let sum_churn = metrics.churn_sum();

        let max_complexity = metrics.maximum_complexity();
        let sum_complexity = metrics.complexity_sum();

        assert_eq!(sum_churn, 45.0);
        assert_eq!(max_churn, 30.0);

        assert_eq!(max_complexity, 30.0);
        assert_eq!(sum_complexity, 60.0);
    }

    #[test]
    fn magnitude() {
        let metric = FileMetrics::new("foo.rs".to_string(), Churn::from(2), 2.0);

        assert_eq!(metric.magnitude(), 2.8284271247461903)
    }

    #[test]
    fn build_array_of_metric() {
        let file_map = HashMap::from([("file".to_string(), Churn::from(1))]);

        let results = metrics_per_file(file_map, TestReader {});

        assert!(results.len() == 1);
        assert!(results[0].churn == Churn::from(1));
        assert!(results[0].complexity == 1.0);
    }

    mod integration {
        use std::fs::File;
        use std::io::Write;

        use tempfile::tempdir;

        use crate::metrics::{CodeAnalysisReader, MetricReader};

        #[test]
        fn read_cyclomatic_from_file() {
            let reader = CodeAnalysisReader::default();

            let dir = tempdir().expect("temp dir obtained");

            let file_path = dir.path().join("foo.rs");
            let mut file = File::create(file_path.clone()).expect("file created");
            writeln!(
                file,
                r#"fn f() {{ // +2 (+1 unit space)
                if true {{ // +1
                    match true {{
                        true => println!(\"test\"), // +1
                        false => println!(\"test\"), // +1
                    }}
                }}
            }}"#
            )
            .expect("file written");

            let metric = reader.get_cyclomatic_from_path_and_content(file_path.as_path());

            metric.expect("cyclomatic complexity");
        }
    }
}
