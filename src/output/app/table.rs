use tui::{
    layout::Constraint,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::metrics::FileMetrics;

pub fn file_table(items: &[FileMetrics]) -> Table {
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().add_modifier(Modifier::BOLD);
    let header_cells = ["Filename", "Churn", "Complexity", "Magnitude"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);

    let rows = items.iter().map(|item| {
        let cells = item.to_cells();
        Row::new(cells).height(1_u16).bottom_margin(1)
    });
    Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Files"))
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Percentage(50),
            Constraint::Min(5),
            Constraint::Min(5),
            Constraint::Min(5),
        ])
}

#[cfg(test)]
mod tests {
    use crate::Churn;

    use super::*;

    // This tests nothing but Table is not a testable struct since all fields are private and no method allow access.
    #[test]
    fn create_file_table() {
        let items = vec![
            FileMetrics {
                filename: "file1.txt".to_string(),
                churn: Churn::from(15),
                complexity: 20.0,
            },
            FileMetrics {
                filename: "file2.txt".to_string(),
                churn: Churn::from(10),
                complexity: 30.0,
            },
        ];

        let _table = file_table(&items);
    }
}
