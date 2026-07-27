use crate::theme::Theme;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

pub fn render(frame: &mut Frame, area: Rect, theme: &Theme) {
    let popup_width = 58u16.min(area.width.saturating_sub(4));
    let popup_height = 26u16.min(area.height.saturating_sub(2));
    let popup_x = (area.width.saturating_sub(popup_width)) / 2 + area.x;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2 + area.y;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent()))
        .title(" Keybindings ")
        .title_style(
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(theme.bg()));

    let help_items = vec![
        ("Navigation", vec![
            ("j / ↓", "Move down"),
            ("k / ↑", "Move up"),
            ("g", "Go to top"),
            ("G", "Go to bottom"),
        ]),
        ("Actions", vec![
            ("Enter", "Open bookmark"),
            ("/", "Search"),
            ("a", "Add bookmark"),
            ("e", "Edit bookmark"),
            ("d", "Delete bookmark"),
            ("Space", "Toggle select"),
            ("D", "Bulk delete selected"),
            ("y", "Yank (copy) URL"),
        ]),
        ("Form / Search Editing", vec![
            ("← / Ctrl+B", "Cursor left"),
            ("→ / Ctrl+F", "Cursor right"),
            ("Home / Ctrl+A", "Cursor to start"),
            ("End / Ctrl+E", "Cursor to end"),
            ("Delete", "Delete at cursor"),
            ("Backspace", "Delete before cursor"),
            ("Ctrl+W", "Delete word"),
            ("Ctrl+U", "Clear field"),
            ("Tab", "Next field"),
            ("Shift+Tab", "Previous field"),
            ("Ctrl+S", "Save"),
            ("Esc", "Cancel"),
        ]),
    ];

    let mut lines: Vec<Line> = Vec::new();
    for (section, items) in &help_items {
        lines.push(Line::from(Span::styled(
            format!("  {}", section),
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        )));
        for (key, desc) in items {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("    {:16}", key),
                    Style::default().fg(theme.get("url")),
                ),
                Span::styled(*desc, Style::default().fg(theme.fg())),
            ]));
        }
        lines.push(Line::from(""));
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .style(Style::default().fg(theme.fg()));

    frame.render_widget(paragraph, popup_area);
}
