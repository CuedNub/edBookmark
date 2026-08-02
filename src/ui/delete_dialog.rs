use crate::theme::Theme;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

pub fn render(frame: &mut Frame, area: Rect, names: &[String], theme: &Theme) {
    let popup_width = 45u16.min(area.width.saturating_sub(4));
    let popup_height = 9u16.min(area.height.saturating_sub(4));
    let popup_x = (area.width.saturating_sub(popup_width)) / 2 + area.x;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2 + area.y;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.delete_border()))
        .title(" Delete Bookmark ")
        .title_style(
            Style::default()
                .fg(theme.delete_text())
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

    let display_name = if names.len() == 1 {
        format!("\"{}\"", names[0])
    } else {
        format!("{} bookmarks", names.len())
    };

    let lines = vec![
        Line::from(vec![
            Span::raw("Delete "),
            Span::styled(
                display_name,
                Style::default()
                    .fg(theme.delete_text())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("?"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " [y] Delete ",
                Style::default()
                    .fg(theme.button_delete_fg())
                    .bg(theme.button_delete_bg()),
            ),
            Span::raw("  "),
            Span::styled(
                " [n] Cancel ",
                Style::default()
                    .fg(theme.button_cancel_fg())
                    .bg(theme.button_cancel_bg()),
            ),
        ]),
    ];

    let paragraph = Paragraph::new(lines).style(Style::default().fg(theme.fg()));
    frame.render_widget(paragraph, inner);
}
