use std::{collections::HashMap, path::Path};

use rust_code_analysis::{metrics, read_file_with_eol, ParserTrait, RustParser};

#[derive(Debug, Clone, PartialEq)]
pub struct FileMetrics {
    pub filename: String,
    pub churn: i32,
    pub complexity: f64,
    pub magnitude: f64,
}

impl FileMetrics {
    pub fn new(filename: String, churn: i32, complexity: f64) -> Self {
        let origin = (0.0, 0.0);
        let distance_to_origin =
            ((origin.0 - churn as f64).powi(2) + (origin.1 - complexity as f64).powi(2)).sqrt();

        Self {
            filename,
            churn,
            complexity,
            magnitude: distance_to_origin,
        }
    }

    pub fn to_point(&self) -> (f64, f64) {
        (self.churn as f64, self.complexity)
    }
}

pub fn metrics_per_file(file_map: HashMap<String, i32>) -> Vec<FileMetrics> {
    file_map
        .into_iter() // TODO: parallelize
        .filter_map(|(filename, churn)| {
            let path = Path::new(&filename);

            let complexity = read_file_with_eol(path)
                .ok()
                .and_then(|file| {
                    let parser = RustParser::new(file.expect("file content"), path, None);
                    metrics(&parser, path)
                })
                .map(|metrics| metrics.metrics.cyclomatic.cyclomatic_sum());

            complexity.map(|complexity| FileMetrics::new(filename.to_string(), churn, complexity))
        })
        .collect()
}
