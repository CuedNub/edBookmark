use crate::history::HistoryEntry;
use crate::theme::Theme;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table, TableState};
use ratatui::Frame;

/// Render history list (full-screen view)
pub fn render(
    frame: &mut Frame,
    area: Rect,
    entries: &[HistoryEntry],
    selected: usize,
    multi_selected: &[String],
    theme: &Theme,
) {
    // Outer block
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border_top()))
        .title(" History ")
        .title_style(
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(theme.bg()));

    frame.render_widget(outer, area);

    let inner = Rect::new(
        area.x + 1,
        area.y + 1,
        area.width.saturating_sub(2),
        area.height.saturating_sub(2),
    );

    let layout = Layout::vertical([
        Constraint::Min(5),    // table
        Constraint::Length(1), // status/hints
    ])
    .split(inner);

    // Table
    if entries.is_empty() {
        let empty = Paragraph::new(Line::from(Span::styled(
            "  No history entries",
            Style::default().fg(theme.muted()),
        )));
        frame.render_widget(empty, layout[0]);
    } else {
        render_table(frame, layout[0], entries, selected, multi_selected, theme);
    }

    // Hints
    let sel_count = multi_selected.len();
    let hint_text = if sel_count > 0 {
        format!(
            " [{}sel] [r/Enter] Restore  [d] Delete  [D] Bulk delete  [E] Export  [Esc] Back",
            sel_count
        )
    } else {
        " [r/Enter] Restore  [d] Delete  [Space] Select  [E] Export  [Esc] Back".to_string()
    };

    let hints = Paragraph::new(Line::from(Span::styled(
        hint_text,
        Style::default().fg(theme.muted()),
    )));
    frame.render_widget(hints, layout[1]);
}

fn render_table(
    frame: &mut Frame,
    area: Rect,
    entries: &[HistoryEntry],
    selected: usize,
    multi_selected: &[String],
    theme: &Theme,
) {
    let header_cells = ["#", "Date", "Type", "Description", "Count"].iter().map(|h| {
        Cell::from(*h).style(
            Style::default()
                .fg(theme.header())
                .add_modifier(Modifier::BOLD),
        )
    });
    let header = Row::new(header_cells)
        .style(Style::default().bg(theme.header_bg()))
        .height(1);

    let rows: Vec<Row> = entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_multi = multi_selected.contains(&entry.id);
            let marker = if is_multi { "●" } else { " " };

            let cells = vec![
                Cell::from(format!("{}{}", marker, i + 1)),
                Cell::from(entry.timestamp.format("%Y-%m-%d %H:%M").to_string()),
                Cell::from(format!("{}", entry.action)),
                Cell::from(entry.description.as_str()),
                Cell::from(format!("{}", entry.bookmark_count)),
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
        Constraint::Length(17),
        Constraint::Length(8),
        Constraint::Percentage(60),
        Constraint::Length(6),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border_top()))
        .title(format!(" Entries ({}) ", entries.len()))
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
    if !entries.is_empty() {
        state.select(Some(selected));
    }

    frame.render_stateful_widget(table, area, &mut state);
}

/// Render delete confirmation dialog for history
pub fn render_delete_confirm(
    frame: &mut Frame,
    area: Rect,
    delete_count: usize,
    theme: &Theme,
) {
    let popup_width = 50u16.min(area.width.saturating_sub(4));
    let popup_height = 7u16.min(area.height.saturating_sub(4));
    let popup_x = (area.width.saturating_sub(popup_width)) / 2 + area.x;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2 + area.y;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.get("delete_border")))
        .title(" Delete History ")
        .title_style(
            Style::default()
                .fg(theme.get("delete_text"))
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(theme.bg()));

    frame.render_widget(block, popup_area);

    let inner = Rect::new(
        popup_area.x + 2,
        popup_area.y + 2,
        popup_area.width.saturating_sub(4),
        popup_area.height.saturating_sub(3),
    );

    let rows = Layout::vertical([
        Constraint::Length(1), // message
        Constraint::Length(1), // spacer
        Constraint::Length(1), // hint
    ])
    .split(inner);

    let msg = format!(
        "Delete {} history entr{}?",
        delete_count,
        if delete_count == 1 { "y" } else { "ies" }
    );
    let message = Paragraph::new(Line::from(Span::styled(
        msg,
        Style::default().fg(theme.get("delete_text")),
    )));
    frame.render_widget(message, rows[0]);

    let hint = Paragraph::new(Line::from(Span::styled(
        " [y/Enter] Delete  [n/Esc] Cancel",
        Style::default().fg(theme.muted()),
    )));
    frame.render_widget(hint, rows[2]);
}

/// Render export format selection for history
pub fn render_export_select(frame: &mut Frame, area: Rect, theme: &Theme) {
    let popup_width = 44u16.min(area.width.saturating_sub(4));
    let popup_height = 10u16.min(area.height.saturating_sub(4));
    let popup_x = (area.width.saturating_sub(popup_width)) / 2 + area.x;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2 + area.y;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent()))
        .title(" Export History ")
        .title_style(
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(theme.bg()));

    frame.render_widget(block, popup_area);

    let inner = Rect::new(
        popup_area.x + 2,
        popup_area.y + 1,
        popup_area.width.saturating_sub(4),
        popup_area.height.saturating_sub(2),
    );

    let rows = Layout::vertical([
        Constraint::Length(1), // header
        Constraint::Length(3), // items
        Constraint::Length(1), // spacer
        Constraint::Length(1), // hint
    ])
    .split(inner);

    let header = Paragraph::new(Line::from(Span::styled(
        "Select format:",
        Style::default()
            .fg(theme.header())
            .add_modifier(Modifier::BOLD),
    )));
    frame.render_widget(header, rows[0]);

    let items = vec![
        ListItem::new(Line::from(vec![
            Span::styled("  [1] ", Style::default().fg(theme.accent())),
            Span::styled("JSON", Style::default().fg(theme.fg())),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  [2] ", Style::default().fg(theme.accent())),
            Span::styled("HTML", Style::default().fg(theme.fg())),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  [3] ", Style::default().fg(theme.accent())),
            Span::styled("XLSX", Style::default().fg(theme.fg())),
        ])),
    ];
    let list = List::new(items);
    frame.render_widget(list, rows[1]);

    let hint = Paragraph::new(Line::from(Span::styled(
        " [1-3] Select  [Esc] Cancel",
        Style::default().fg(theme.muted()),
    )));
    frame.render_widget(hint, rows[3]);
}
