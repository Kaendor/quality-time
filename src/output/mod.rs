use crate::metrics::ProjectMetrics;
use clap::ValueEnum;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::Table;
use eyre::{Context, Result};

use self::app::run_app;

mod app;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputMode {
    /// Print the results in the terminal as a human readable table
    StdOut,
    /// DIsplay the results with a graph in a terminal application
    Tui,
}

pub fn print_output(
    output_mode: OutputMode,
    metrics: ProjectMetrics,
    mut writer: impl std::io::Write,
) -> Result<()> {
    match output_mode {
        OutputMode::StdOut => {
            let mut table = Table::new();
            table
                .set_header(vec!["Filename", "Churn", "Complexity"])
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS);

            for metric in metrics.file_metrics().iter() {
                table.add_row(vec![
                    &metric.filename,
                    &metric.churn.to_string(),
                    &metric.complexity.to_string(),
                ]);
            }

            writeln!(writer, "{table}").wrap_err("unable to write on writer")?;
        }
        OutputMode::Tui => {
            run_app(metrics, writer);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::metrics::{Churn, FileMetrics, ProjectMetrics};

    use super::print_output;

    #[test]
    fn display_to_stdout() {
        let metrics = vec![FileMetrics::new("foo.rs".to_string(), Churn::from(1), 1.0)];
        let mut writer = vec![];
        print_output(
            super::OutputMode::StdOut,
            ProjectMetrics::new(metrics),
            &mut writer,
        )
        .expect("print in writer");

        assert!(!writer.is_empty());

        let content = String::from_utf8(writer).expect("bytes to utf8");

        assert!(content.contains("foo.rs"));
    }
}
