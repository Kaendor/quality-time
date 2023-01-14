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
            Cell::from(self.magnitude.to_string()),
        ]
    }
}

pub fn run_app(metrics: Vec<FileMetrics>) {
    enable_raw_mode().expect("raw mode");
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .expect("do something to the terminal");

    let backend = CrosstermBackend::new(stdout);
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

    let metric_data: Vec<(f64, f64)> = app
        .items
        .iter()
        .map(|metric| (metric.churn.as_f64(), metric.complexity))
        .collect();

    let maximum_churn = metric_data
        .iter()
        .max_by_key(|x| x.0.round() as i64)
        .map(|x| x.0)
        .unwrap_or_default()
        + 10.0;

    let churn_sum: i64 = metric_data.iter().map(|x| x.0 as i64).sum();
    let complexity_sum: i64 = metric_data.iter().map(|x| x.1 as i64).sum();

    let maximum_complexity = metric_data
        .iter()
        .max_by_key(|x| x.1.round() as i64)
        .map(|x| x.1)
        .unwrap_or_default()
        + 10.0;

    let complexity_threshold = complexity_sum as f64 / metric_data.len() as f64 / 2.0;
    let churn_threshold: f64 = churn_sum as f64 / metric_data.len() as f64 / 2.0;

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
        .map(|metric| metric.to_point())
        .into_iter()
        .collect();

    let metric_data: Vec<_> = metric_data
        .into_iter()
        .filter(|metric| Some(metric) != selected_metric.map(|f| f.to_point()).as_ref())
        .collect();

    let datasets = vec![
        Dataset::default()
            .marker(symbols::Marker::Dot)
            .style(Style::default().fg(Color::DarkGray))
            .graph_type(GraphType::Line)
            .data(&threshold_points),
        Dataset::default()
            .marker(symbols::Marker::Dot)
            .style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
            .data(&metric_data),
        Dataset::default()
            .marker(symbols::Marker::Block)
            .style(Style::default().fg(Color::Magenta))
            .graph_type(GraphType::Scatter)
            .data(&selected_point),
    ];

    let graph = create_chart(datasets, maximum_churn, maximum_complexity);
    f.render_widget(graph, rects[1]);
}
