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

use crate::metrics::FileMetrics;

use self::{
    chart::{complexity_churn_threshold, create_chart},
    table::file_table,
};

mod chart;
mod table;

pub struct App {
    state: TableState,
    items: Vec<FileMetrics>,
}

impl App {
    fn new(metrics: Vec<FileMetrics>) -> Self {
        App {
            state: TableState::default(),
            items: metrics,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
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
                    self.items.len() - 1
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

pub fn run_app(metrics: Vec<FileMetrics>, mut writer: impl std::io::Write) {
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

    let t = file_table(&app.items);
    f.render_stateful_widget(t, rects[0], &mut app.state);

    let selected_metric = app
        .state
        .selected()
        .and_then(|selected_index| app.items.get(selected_index));

    let (maximum_churn, churn_sum) = get_maximum_churn_and_sum(&app.items);
    let (maximum_complexity, complexity_sum) = get_maximum_complexity_and_sum(&app.items);

    let complexity_threshold = complexity_sum as f64 / app.items.len() as f64 / 2.0;
    let churn_threshold = churn_sum as f64 / app.items.len() as f64 / 2.0;

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
    let points: Vec<_> = app.items.iter().map(|m| m.to_point()).collect();
    let metric_data = filter_out_selected_metric(&points, &selected_point);
    let datasets = create_datasets(&threshold_points, &metric_data, &selected_point);
    let graph = create_chart(datasets, maximum_churn, maximum_complexity);
    f.render_widget(graph, rects[1]);
}

fn get_maximum_churn_and_sum(metric_data: &[FileMetrics]) -> (f64, f64) {
    let maximum_churn = metric_data
        .iter()
        .max_by_key(|x| x.churn.as_f64() as i64)
        .map(|x| x.churn.as_f64() + 10.0)
        .unwrap_or(10.0);
    let churn_sum: i64 = metric_data.iter().map(|x| x.churn.as_f64() as i64).sum();
    (maximum_churn, churn_sum as f64)
}

fn get_maximum_complexity_and_sum(metric_data: &[FileMetrics]) -> (f64, f64) {
    let maximum_complexity = metric_data
        .iter()
        .max_by_key(|x| x.complexity.round() as i64)
        .map(|x| x.complexity)
        .unwrap_or_default()
        + 10.0;
    let complexity_sum: i64 = metric_data.iter().map(|x| x.complexity as i64).sum();
    (maximum_complexity, complexity_sum as f64)
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
    use super::*;
    use crate::metrics::{Churn, FileMetrics};

    #[test]
    fn test_get_maximum_churn_and_sum() {
        let metric_data = vec![
            FileMetrics {
                churn: Churn::from(15),
                complexity: 20.0,
                filename: "todo!()".to_string(),
            },
            FileMetrics {
                churn: Churn::from(10),
                complexity: 30.0,
                filename: "todo!()".to_string(),
            },
            FileMetrics {
                churn: Churn::from(20),
                complexity: 10.0,
                filename: "todo!()".to_string(),
            },
        ];
        let (maximum_churn, sum) = get_maximum_churn_and_sum(&metric_data);
        assert_eq!(maximum_churn, 30.0);
        assert_eq!(sum, 45.0);
    }

    #[test]
    fn test_get_maximum_complexity_and_sum() {
        let metric_data = vec![
            FileMetrics {
                churn: Churn::from(15),
                complexity: 20.0,
                filename: "todo!()".to_string(),
            },
            FileMetrics {
                churn: Churn::from(10),
                complexity: 30.0,
                filename: "todo!()".to_string(),
            },
            FileMetrics {
                churn: Churn::from(20),
                complexity: 10.0,
                filename: "todo!()".to_string(),
            },
        ];
        let (maximum_complexity, sum) = get_maximum_complexity_and_sum(&metric_data);
        assert_eq!(maximum_complexity, 40.0);
        assert_eq!(sum, 60.0);
    }

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
}
