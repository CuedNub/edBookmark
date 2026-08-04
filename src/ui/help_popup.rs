use crate::theme::Theme;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

pub fn render(frame: &mut Frame, area: Rect, scroll: u16, theme: &Theme) {
    let popup_width = 62u16.min(area.width.saturating_sub(4));
    let popup_height = 30u16.min(area.height.saturating_sub(2));
    let popup_x = (area.width.saturating_sub(popup_width)) / 2 + area.x;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2 + area.y;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent()))
        .title(" Help (j/k to scroll) ")
        .title_style(
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(theme.bg()));

    let help_items = vec![
        (
            "Navigation",
            vec![
                ("j / ↓", "Move down"),
                ("k / ↑", "Move up"),
                ("g", "Go to top"),
                ("G", "Go to bottom"),
            ],
        ),
        (
            "Actions",
            vec![
                ("Enter", "Open bookmark"),
                ("/", "Search"),
                ("a", "Add bookmark"),
                ("e", "Edit bookmark"),
                ("d", "Delete bookmark"),
                ("Space", "Toggle select"),
                ("D", "Bulk delete selected"),
                ("y", "Yank (copy) URL"),
                ("I", "Import / Export"),
                ("X", "History"),
            ],
        ),
        (
            "History View (X)",
            vec![
                ("j / k", "Navigate entries"),
                ("r / Enter", "Restore selected snapshot"),
                ("Space", "Toggle select"),
                ("d", "Delete selected entry"),
                ("D", "Bulk delete selected"),
                ("E", "Export history to file"),
                ("Esc", "Back to Normal"),
            ],
        ),
        (
            "Form / Search Editing",
            vec![
                ("← / Ctrl+B", "Cursor left"),
                ("→ / Ctrl+F", "Cursor right"),
                ("Home / Ctrl+A", "Cursor to start"),
                ("End / Ctrl+E", "Cursor to end"),
                ("Ctrl+W", "Delete word"),
                ("Ctrl+U", "Clear field"),
                ("Tab / Shift+Tab", "Next / Prev field"),
                ("Ctrl+S / Enter", "Save"),
                ("Esc", "Cancel"),
            ],
        ),
    ];

    let path_items = vec![
        ("Config", "~/.config/edbookmark/config.toml"),
        ("Data", "~/.local/share/edbookmark/bookmarks.json"),
        ("Imports", "~/.local/share/edbookmark/imports/"),
        ("Exports", "~/.local/share/edbookmark/exports/"),
        ("History", "~/.local/share/edbookmark/history/"),
        ("Log", "~/.local/state/edbookmark/launcher.log"),
        ("Desktop", "~/.local/share/applications/edbookmark.desktop"),
        ("Binary", "~/.local/bin/edbookmark"),
    ];

    let mut lines: Vec<Line> = Vec::new();

    // Keybinding sections
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
                    format!("    {:18}", key),
                    Style::default().fg(theme.get("url")),
                ),
                Span::styled(*desc, Style::default().fg(theme.fg())),
            ]));
        }
        lines.push(Line::from(""));
    }

    // Path info section
    lines.push(Line::from(Span::styled(
        "  Paths",
        Style::default()
            .fg(theme.accent())
            .add_modifier(Modifier::BOLD),
    )));
    for (label, path) in &path_items {
        lines.push(Line::from(vec![
            Span::styled(
                format!("    {:10}", label),
                Style::default().fg(theme.get("folder")),
            ),
            Span::styled(*path, Style::default().fg(theme.muted())),
        ]));
    }
    lines.push(Line::from(""));

    // Scroll indicator
    let total_lines = lines.len() as u16;
    let visible_lines = popup_height.saturating_sub(2);
    if total_lines > visible_lines {
        lines.push(Line::from(Span::styled(
            format!("  ── scroll {}/{} ──", scroll + 1, total_lines.saturating_sub(visible_lines) + 1),
            Style::default().fg(theme.muted()),
        )));
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .scroll((scroll, 0))
        .style(Style::default().fg(theme.fg()));

    frame.render_widget(paragraph, popup_area);
}

/// Hitung total baris konten help (untuk clamp scroll)
pub fn content_height(popup_height: u16) -> u16 {
    // Total content lines:
    // Navigation: 1+4+1 = 6
    // Actions: 1+10+1 = 12
    // History: 1+7+1 = 9
    // Form: 1+9+1 = 11
    // Paths: 1+6+1 = 8
    // Scroll indicator: 1
    // Total = 47
    let total: u16 = 49;
    let visible = popup_height.saturating_sub(2); // minus border
    total.saturating_sub(visible)
}
