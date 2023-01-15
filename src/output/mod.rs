use crate::metrics::FileMetrics;
use clap::ValueEnum;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::Table;

use self::app::run_app;

mod app;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputMode {
    /// Print the results in the terminal as a human readable table
    StdOut,
    /// DIsplay the results with a graph in a terminal application
    Tui,
}

pub fn print_output(output_mode: OutputMode, metrics: Vec<FileMetrics>) {
    match output_mode {
        OutputMode::StdOut => {
            let mut table = Table::new();
            table
                .set_header(vec!["Filename", "Churn", "Complexity"])
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS);

            for metric in metrics.iter() {
                table.add_row(vec![
                    &metric.filename,
                    &metric.churn.to_string(),
                    &metric.complexity.to_string(),
                ]);
            }

            println!("{table}");
        }
        OutputMode::Tui => {
            run_app(metrics);
        }
    }
}
