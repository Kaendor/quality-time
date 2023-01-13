use tui::{
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset},
};

pub fn complexity_churn_threshold(x: f64, complexity_threshold: f64, churn_threshold: f64) -> f64 {
    (100f64 / x - churn_threshold) + complexity_threshold
}

pub fn create_chart(datasets: Vec<Dataset>, maximum_churn: f64, maximum_complexity: f64) -> Chart {
    Chart::new(datasets)
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
        )
}