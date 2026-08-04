use crate::keybinding::AppMode;
use crate::theme::Theme;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    mode: &AppMode,
    total: usize,
    filtered: usize,
    selected_idx: usize,
    multi_count: usize,
    message: &str,
    theme: &Theme,
) {
    let cols =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(area);

    // Left: mode + count
    let mode_str = match mode {
        AppMode::Normal => "NORMAL",
        AppMode::Search => "SEARCH",
        AppMode::Add => "ADD",
        AppMode::Edit => "EDIT",
        AppMode::DeleteConfirm => "DELETE",
        AppMode::Help => "HELP",
        AppMode::ImportExport => "IMPORT/EXPORT",
        AppMode::ImportExportInput => "PATH INPUT",
        AppMode::History => "HISTORY",
        AppMode::HistoryDeleteConfirm => "DELETE HISTORY",
        AppMode::HistoryExportSelect => "EXPORT HISTORY",
    };

    let mut left_spans = vec![
        Span::styled(
            format!(" {} ", mode_str),
            Style::default()
                .fg(theme.bg())
                .bg(theme.status_fg())
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
    ];

    if !message.is_empty() {
        left_spans.push(Span::styled(message, Style::default().fg(theme.accent())));
    } else {
        let pos = if filtered > 0 {
            format!("{}/{}", selected_idx + 1, filtered)
        } else {
            "0/0".to_string()
        };
        left_spans.push(Span::styled(pos, Style::default().fg(theme.fg())));
        if filtered != total {
            left_spans.push(Span::styled(
                format!(" (filtered from {})", total),
                Style::default().fg(theme.muted()),
            ));
        }
        if multi_count > 0 {
            left_spans.push(Span::styled(
                format!(" [{}sel]", multi_count),
                Style::default().fg(theme.accent()),
            ));
        }
    }

    let left = Paragraph::new(Line::from(left_spans)).style(Style::default().bg(theme.status_bg()));
    frame.render_widget(left, cols[0]);

    // Right: hints
    let hints = match mode {
        AppMode::Normal => "[/]Search [a]Add [I]Import/Export [X]History [?]Help [q]Quit",
        AppMode::Search => "[Enter]Confirm [Esc]Cancel [↓↑]Navigate",
        AppMode::Add | AppMode::Edit => "[Ctrl+S]Save [Esc]Cancel [Tab]Next",
        AppMode::DeleteConfirm => "[y]Delete [n]Cancel",
        AppMode::Help => "[Esc]Close",
        AppMode::ImportExport => "[1-7]Select [Esc]Cancel",
        AppMode::ImportExportInput => "[Enter]Confirm [Esc]Back [Ctrl+U]Clear",
        AppMode::History => "[r/Enter]Restore [d]Delete [E]Export [Esc]Back",
        AppMode::HistoryDeleteConfirm => "[y]Delete [n]Cancel",
        AppMode::HistoryExportSelect => "[1-3]Select [Esc]Cancel",
    };

    let right = Paragraph::new(Line::from(Span::styled(
        hints,
        Style::default().fg(theme.muted()),
    )))
    .style(Style::default().bg(theme.status_bg()))
    .alignment(ratatui::layout::Alignment::Right);
    frame.render_widget(right, cols[1]);
}
