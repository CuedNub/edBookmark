use crate::bookmark::Bookmark;
use crate::theme::Theme;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};
use ratatui::Frame;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    bookmarks: &[(usize, i64, &Bookmark)],
    selected: usize,
    multi_selected: &[String],
    theme: &Theme,
) {
    let header_cells = ["#", "Name", "URL", "Folder"].iter().map(|h| {
        Cell::from(*h).style(
            Style::default()
                .fg(theme.header())
                .add_modifier(Modifier::BOLD),
        )
    });
    let header = Row::new(header_cells)
        .style(Style::default().bg(theme.header_bg()))
        .height(1);

    let rows: Vec<Row> = bookmarks
        .iter()
        .enumerate()
        .map(|(i, (_orig_idx, _score, bookmark))| {
            let is_multi = multi_selected.contains(&bookmark.id);
            let marker = if is_multi { "●" } else { " " };

            let cells = vec![
                Cell::from(format!("{}{}", marker, i + 1)),
                Cell::from(bookmark.name.as_str()),
                Cell::from(bookmark.url.as_str()),
                Cell::from(bookmark.folder.as_str()),
            ];

            let style = if is_multi {
                Style::default()
                    .fg(theme.multiselect_fg())
                    .bg(theme.multiselect_bg())
            } else {
                Style::default().fg(theme.fg()).bg(theme.bg())
            };

            Row::new(cells).style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(4),
        Constraint::Percentage(30),
        Constraint::Percentage(45),
        Constraint::Percentage(25),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border_top()))
        .title(format!(" Bookmarks ({}) ", bookmarks.len()))
        .title_style(Style::default().fg(theme.accent()));

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .row_highlight_style(
            Style::default()
                .fg(theme.selected_fg())
                .bg(theme.selected_bg())
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut state = TableState::default();
    if !bookmarks.is_empty() {
        state.select(Some(selected));
    }

    frame.render_stateful_widget(table, area, &mut state);
}
