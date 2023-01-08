#[derive(Debug, Clone)]
pub struct FileMetrics {
    pub filename: String,
    pub churn: i32,
    pub complexity: f64,
}

impl FileMetrics {
    pub fn new(filename: String, churn: i32, complexity: f64) -> Self {
        Self {
            filename,
            churn,
            complexity,
        }
    }
}
