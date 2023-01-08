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
    text::Span,
    widgets::{Axis, Block, Borders, Cell, Chart, Dataset, Row, Table, TableState},
    Frame, Terminal,
};

use crate::metrics::FileMetrics;

struct App {
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
}

impl FileMetrics {
    fn to_cells(&self) -> Vec<Cell<'static>> {
        vec![
            Cell::from(self.filename.clone()),
            Cell::from(self.churn.to_string()),
            Cell::from(self.complexity.to_string()),
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
    .expect("Leavint alternate screen");
    terminal.show_cursor().expect("show cursor");
}

fn run_terminal_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                // KeyCode::Down => app.next(),
                // KeyCode::Up => app.previous(),
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

    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().add_modifier(Modifier::BOLD);
    let header_cells = ["Filename", "Churn", "Complexity"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
    let rows = app.items.iter().map(|item| {
        let cells = item.to_cells();
        Row::new(cells).height(1_u16).bottom_margin(1)
    });
    let t = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Files"))
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Percentage(50),
            Constraint::Min(5),
            Constraint::Min(5),
        ]);
    f.render_stateful_widget(t, rects[0], &mut app.state);

    // let dataset: Vec<Dataset> = app.items.iter().map(|metric| {

    // }).collect();

    let metric_data: Vec<(f64, f64)> = app
        .items
        .iter()
        .map(|metric| (metric.churn.into(), metric.complexity))
        .collect();

    let datasets = vec![Dataset::default()
        // .name("Churn vs Complexity")
        .marker(symbols::Marker::Dot)
        .data(&metric_data)];

    let maximum_churn = 50f64;
    let maximum_complexity = 59f64;

    let graph = Chart::new(datasets)
        .block(
            Block::default()
                .title(Span::styled(
                    "Churn vs Complexity",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .title("Churn")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, maximum_churn])
                .labels(vec![
                    Span::styled("0", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw((maximum_churn.div_euclid(2.0)).to_string()),
                    Span::styled(
                        maximum_churn.to_string(),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                ]),
        )
        .y_axis(
            Axis::default()
                .title("Complexity")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, maximum_complexity])
                .labels(vec![
                    Span::styled("0", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw((maximum_complexity.div_euclid(2.0)).to_string()),
                    Span::styled(
                        maximum_complexity.to_string(),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                ]),
        );
    f.render_widget(graph, rects[1]);
}
