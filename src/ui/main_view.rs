use crate::app::AppState;
use crate::keybinding::AppMode;
use crate::theme::Theme;
use crate::ui::{bookmark_list, delete_dialog, form_view, help_popup, search_bar, status_bar};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders};
use ratatui::Frame;

pub fn render(frame: &mut Frame, state: &AppState, theme: &Theme) {
    let size = frame.area();

    // Outer border with rainbow colors
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border_top()))
        .title(" edbookmark ")
        .title_style(Style::default().fg(theme.accent()));

    frame.render_widget(outer_block, size);

    let inner = Rect::new(
        size.x + 1,
        size.y + 1,
        size.width.saturating_sub(2),
        size.height.saturating_sub(2),
    );

    // Layout: search bar (3) + bookmark list (fill) + status bar (1)
    let layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(5),
        Constraint::Length(1),
    ])
    .split(inner);

    // Search bar with cursor
    search_bar::render(
        frame,
        layout[0],
        &state.search_query,
        state.search_cursor,
        &state.mode,
        theme,
    );

    // Bookmark list
    bookmark_list::render(
        frame,
        layout[1],
        &state.filtered_bookmarks,
        state.selected_index,
        &state.multi_selected,
        theme,
    );

    // Status bar
    status_bar::render(
        frame,
        layout[2],
        &state.mode,
        state.total_count,
        state.filtered_bookmarks.len(),
        state.selected_index,
        state.multi_selected.len(),
        &state.status_message,
        theme,
    );

    // Overlay popups
    match &state.mode {
        AppMode::Add | AppMode::Edit => {
            if let Some(form) = &state.form_data {
                form_view::render(frame, size, &state.mode, form, theme);
            }
        }
        AppMode::DeleteConfirm => {
            delete_dialog::render(frame, size, &state.delete_names, theme);
        }
        AppMode::Help => {
            help_popup::render(frame, size, theme);
        }
        _ => {}
    }
}
