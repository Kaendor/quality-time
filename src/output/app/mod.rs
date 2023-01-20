use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, vec};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    widgets::{Cell, Dataset, GraphType, TableState},
    Frame, Terminal,
};

use crate::metrics::{FileMetrics, ProjectMetrics};

use self::{
    chart::{complexity_churn_threshold, create_chart},
    table::file_table,
};

mod chart;
mod table;

pub struct App {
    state: TableState,
    metrics: ProjectMetrics,
}

impl App {
    fn new(metrics: ProjectMetrics) -> Self {
        App {
            state: TableState::default(),
            metrics,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.metrics.file_metrics().len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.metrics.file_metrics().len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

impl FileMetrics {
    fn to_cells(&self) -> Vec<Cell<'static>> {
        vec![
            Cell::from(self.filename.clone()),
            Cell::from(self.churn.to_string()),
            Cell::from(self.complexity.to_string()),
            Cell::from(self.magnitude().to_string()),
        ]
    }
}

pub fn run_app(metrics: ProjectMetrics, mut writer: impl std::io::Write) {
    enable_raw_mode().expect("raw mode");

    execute!(writer, EnterAlternateScreen, EnableMouseCapture)
        .expect("do something to the terminal");

    let backend = CrosstermBackend::new(writer);

    let mut terminal = Terminal::new(backend).expect("terminal backend");

    let app = App::new(metrics);
    let _ = run_terminal_app(&mut terminal, app);

    disable_raw_mode().expect("Disable raw mode");
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .expect("Leave alternate screen");

    terminal.show_cursor().expect("show cursor");
}

fn run_terminal_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Down => app.next(),
                KeyCode::Up => app.previous(),
                _ => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let rects = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .margin(1)
        .split(f.size());

    let t = file_table(&app.metrics.file_metrics());
    f.render_stateful_widget(t, rects[0], &mut app.state);

    let selected_metric = app
        .state
        .selected()
        .and_then(|selected_index| app.metrics.file_metrics().get(selected_index));

    let churn_sum = app.metrics.churn_sum();
    let maximum_churn = app.metrics.maximum_churn();
    let maximum_complexity = app.metrics.maximum_complexity();
    let complexity_sum = app.metrics.complexity_sum();

    let complexity_threshold =
        complexity_sum as f64 / app.metrics.file_metrics().len() as f64 / 2.0;
    let churn_threshold = churn_sum as f64 / app.metrics.file_metrics().len() as f64 / 2.0;

    let threshold_points: Vec<(f64, f64)> = (1..(maximum_churn as i64 + 10))
        .into_iter()
        .map(|x| {
            (
                x as f64,
                complexity_churn_threshold(x as f64, complexity_threshold, churn_threshold),
            )
        })
        .collect();

    let selected_point: Vec<(f64, f64)> = selected_metric
        .cloned()
        .into_iter()
        .map(|m| m.to_point())
        .collect();
    let points: Vec<_> = app
        .metrics
        .file_metrics()
        .iter()
        .map(|m| m.to_point())
        .collect();
    let metric_data = filter_out_selected_metric(&points, &selected_point);
    let datasets = create_datasets(&threshold_points, &metric_data, &selected_point);
    let graph = create_chart(datasets, maximum_churn, maximum_complexity + 10.0);
    f.render_widget(graph, rects[1]);
}

fn filter_out_selected_metric(
    metric_data: &[(f64, f64)],
    selected_metric: &[(f64, f64)],
) -> Vec<(f64, f64)> {
    metric_data
        .into_iter()
        .filter(|metric| !selected_metric.contains(metric))
        .cloned()
        .collect()
}

fn create_datasets<'a>(
    threshold_points: &'a [(f64, f64)],
    metric_data: &'a [(f64, f64)],
    selected_point: &'a [(f64, f64)],
) -> Vec<Dataset<'a>> {
    let threshold_points = Dataset::default()
        .marker(symbols::Marker::Dot)
        .style(Style::default().fg(Color::DarkGray))
        .graph_type(GraphType::Line)
        .data(threshold_points);
    let metric_data = Dataset::default()
        .marker(symbols::Marker::Dot)
        .style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .data(metric_data);
    let selected_point = Dataset::default()
        .marker(symbols::Marker::Block)
        .style(Style::default().fg(Color::Magenta))
        .graph_type(GraphType::Scatter)
        .data(selected_point);
    vec![threshold_points, metric_data, selected_point]
}

#[cfg(test)]
mod tests {
    use crate::Churn;

    use super::*;

    #[test]
    fn test_filter_out_selected_metric() {
        let metric_data = vec![(15.0, 20.0), (10.0, 30.0), (20.0, 10.0)];
        let selected_metric = vec![(20.0, 10.0)];

        let result = filter_out_selected_metric(&metric_data, &selected_metric);
        assert_eq!(result, vec![(15.0, 20.0), (10.0, 30.0)]);
    }

    #[test]
    fn test_create_datasets() {
        let threshold_points = vec![(1.0, 2.0), (3.0, 4.0)];
        let metric_data = vec![(15.0, 20.0), (10.0, 30.0)];

        let selected_point = vec![(20.0, 10.0)];
        let result = create_datasets(&threshold_points, &metric_data, &selected_point);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn app_selection_next() {
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

        let mut app = App::new(metrics);

        assert!(app.state.selected().is_none());

        app.next();

        assert!(app.state.selected().is_some());

        app.next();
        app.next();
        app.next();

        assert!(app.state.selected().is_some());
    }

    #[test]
    fn app_selection_previous() {
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

        let mut app = App::new(metrics);

        assert!(app.state.selected().is_none());

        app.previous();

        assert!(app.state.selected().is_some());

        app.previous();
        app.previous();
        app.previous();

        assert!(app.state.selected().is_some());
    }
}
