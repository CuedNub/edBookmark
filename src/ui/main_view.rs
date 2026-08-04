use crate::app::AppState;
use crate::keybinding::AppMode;
use crate::theme::Theme;
use crate::ui::{
    bookmark_list, delete_dialog, form_view, help_popup, history_view, import_export_view,
    search_bar, status_bar,
};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders};
use ratatui::Frame;

pub fn render(frame: &mut Frame, state: &AppState, theme: &Theme) {
    let size = frame.area();

    // History view is full-screen, render separately
    if matches!(
        state.mode,
        AppMode::History | AppMode::HistoryDeleteConfirm | AppMode::HistoryExportSelect
    ) {
        history_view::render(
            frame,
            size,
            &state.history_entries,
            state.history_selected,
            &state.history_multi_selected,
            theme,
        );

        // Overlay dialogs on top of history
        match &state.mode {
            AppMode::HistoryDeleteConfirm => {
                history_view::render_delete_confirm(
                    frame,
                    size,
                    state.history_delete_count,
                    theme,
                );
            }
            AppMode::HistoryExportSelect => {
                history_view::render_export_select(frame, size, theme);
            }
            _ => {}
        }
        return;
    }

    // Normal main view
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

    let layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(5),
        Constraint::Length(1),
    ])
    .split(inner);

    // Search bar
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
            help_popup::render(frame, size, state.help_scroll, theme);
        }
        AppMode::ImportExport => {
            import_export_view::render_menu(frame, size, theme);
        }
        AppMode::ImportExportInput => {
            let title = match state.import_export_operation {
                Some('1') => "Import from JSON",
                Some('2') => "Import from HTML",
                Some('3') => "Import from XLSX",
                Some('5') => "Export to JSON",
                Some('6') => "Export to HTML",
                Some('7') => "Export to XLSX",
                _ => "File Path",
            };
            import_export_view::render_path_input(
                frame,
                size,
                title,
                &state.import_export_path,
                state.import_export_cursor,
                &state.import_export_error,
                theme,
            );
        }
        _ => {}
    }
}
