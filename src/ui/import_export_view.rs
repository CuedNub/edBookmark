use crate::theme::Theme;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
use ratatui::Frame;

/// Render menu import/export (full-screen overlay)
pub fn render_menu(frame: &mut Frame, area: Rect, theme: &Theme) {
    let popup_width = 56u16.min(area.width.saturating_sub(4));
    let popup_height = 18u16.min(area.height.saturating_sub(4));
    let popup_x = (area.width.saturating_sub(popup_width)) / 2 + area.x;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2 + area.y;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent()))
        .title(" Import / Export ")
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

    let sections = Layout::vertical([
        Constraint::Length(1), // Import header
        Constraint::Length(4), // Import items
        Constraint::Length(1), // spacer
        Constraint::Length(1), // Export header
        Constraint::Length(3), // Export items
        Constraint::Length(1), // spacer
        Constraint::Length(1), // hint
    ])
    .split(inner);

    // Import header
    let import_header = Paragraph::new(Line::from(Span::styled(
        "IMPORT",
        Style::default()
            .fg(theme.header())
            .add_modifier(Modifier::BOLD),
    )));
    frame.render_widget(import_header, sections[0]);

    // Import items
    let import_items = vec![
        ListItem::new(Line::from(vec![
            Span::styled("  [1] ", Style::default().fg(theme.accent())),
            Span::styled("Import from JSON file", Style::default().fg(theme.fg())),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  [2] ", Style::default().fg(theme.accent())),
            Span::styled(
                "Import from HTML file (Chromium/Firefox)",
                Style::default().fg(theme.fg()),
            ),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  [3] ", Style::default().fg(theme.accent())),
            Span::styled("Import from XLSX file", Style::default().fg(theme.fg())),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  [4] ", Style::default().fg(theme.accent())),
            Span::styled(
                "Import from Chromium browser",
                Style::default().fg(theme.fg()),
            ),
        ])),
    ];
    let import_list = List::new(import_items);
    frame.render_widget(import_list, sections[1]);

    // Export header
    let export_header = Paragraph::new(Line::from(Span::styled(
        "EXPORT",
        Style::default()
            .fg(theme.header())
            .add_modifier(Modifier::BOLD),
    )));
    frame.render_widget(export_header, sections[3]);

    // Export items
    let export_items = vec![
        ListItem::new(Line::from(vec![
            Span::styled("  [5] ", Style::default().fg(theme.accent())),
            Span::styled("Export to JSON", Style::default().fg(theme.fg())),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  [6] ", Style::default().fg(theme.accent())),
            Span::styled("Export to HTML", Style::default().fg(theme.fg())),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  [7] ", Style::default().fg(theme.accent())),
            Span::styled("Export to XLSX", Style::default().fg(theme.fg())),
        ])),
    ];
    let export_list = List::new(export_items);
    frame.render_widget(export_list, sections[4]);

    // Hint
    let hint = Paragraph::new(Line::from(Span::styled(
        " [1-7] Select  [Esc] Cancel",
        Style::default().fg(theme.muted()),
    )));
    frame.render_widget(hint, sections[6]);
}

/// Render input path untuk import/export
pub fn render_path_input(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    path: &str,
    cursor_pos: usize,
    error_msg: &str,
    theme: &Theme,
) {
    let popup_width = 70u16.min(area.width.saturating_sub(4));
    let popup_height = 8u16.min(area.height.saturating_sub(4));
    let popup_x = (area.width.saturating_sub(popup_width)) / 2 + area.x;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2 + area.y;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent()))
        .title(format!(" {} ", title))
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
        Constraint::Length(1), // label
        Constraint::Length(3), // input field
        Constraint::Length(1), // error or hint
    ])
    .split(inner);

    // Label
    let label = Paragraph::new(Line::from(Span::styled(
        "File path:",
        Style::default().fg(theme.fg()),
    )));
    frame.render_widget(label, rows[0]);

    // Input field
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.field_active_border()))
        .style(Style::default().bg(theme.bg()));

    let input_text = Paragraph::new(Line::from(Span::styled(
        path,
        Style::default().fg(theme.field_text()),
    )))
    .block(input_block);

    frame.render_widget(input_text, rows[1]);

    // Cursor position
    let input_inner_x = rows[1].x + 1;
    let input_inner_y = rows[1].y + 1;
    let display_pos = if cursor_pos <= path.len() {
        path[..cursor_pos].chars().count() as u16
    } else {
        path.chars().count() as u16
    };
    let cursor_x = input_inner_x + display_pos;
    if cursor_x < rows[1].x + rows[1].width - 1
        && input_inner_y >= rows[1].y
        && input_inner_y < rows[1].y + rows[1].height
    {
        frame.set_cursor_position((cursor_x, input_inner_y));
    }

    // Error or hint
    if !error_msg.is_empty() {
        let error = Paragraph::new(Line::from(Span::styled(
            error_msg,
            Style::default().fg(theme.get("delete_text")),
        )));
        frame.render_widget(error, rows[2]);
    } else {
        let hint = Paragraph::new(Line::from(Span::styled(
            " [Enter] Confirm  [Esc] Cancel  [Ctrl+U] Clear",
            Style::default().fg(theme.muted()),
        )));
        frame.render_widget(hint, rows[2]);
    }
}
