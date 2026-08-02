use crate::keybinding::AppMode;
use crate::theme::Theme;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    query: &str,
    cursor_pos: usize,
    mode: &AppMode,
    theme: &Theme,
) {
    let is_active = matches!(mode, AppMode::Search);

    let border_color = if is_active {
        theme.search_border()
    } else {
        theme.muted()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(" 🔍 Search ")
        .title_style(Style::default().fg(theme.accent()));

    let display_text = if query.is_empty() && !is_active {
        "Type / to search..."
    } else {
        query
    };

    let text_color = if query.is_empty() && !is_active {
        theme.field_placeholder()
    } else {
        theme.fg()
    };

    let paragraph = Paragraph::new(display_text)
        .style(Style::default().fg(text_color).bg(theme.bg()))
        .block(block);

    frame.render_widget(paragraph, area);

    // Bug fix #5: guard cursor batas horizontal DAN vertikal
    if is_active {
        let display_pos = if cursor_pos <= query.len() {
            query[..cursor_pos].chars().count() as u16
        } else {
            query.chars().count() as u16
        };
        let cursor_x = area.x + 1 + display_pos;
        let cursor_y = area.y + 1;
        if cursor_x < area.x + area.width - 1
            && cursor_y >= area.y
            && cursor_y < area.y + area.height
        {
            frame.set_cursor_position((cursor_x, cursor_y));
        }
    }
}
