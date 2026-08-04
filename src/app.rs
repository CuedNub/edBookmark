use crate::bookmark::{Bookmark, BookmarkStore};
use crate::config::Config;
use crate::keybinding::{handle_key, Action, AppMode};
use crate::launcher;
use crate::search::SearchEngine;
use crate::storage;
use crate::theme::Theme;
use crate::ui::form_view::FormData;
use crate::ui::main_view;

use crossterm::event::{self, Event, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::prelude::*;
use std::io::stdout;
use std::path::PathBuf;
use std::time::Duration;

pub struct AppState<'a> {
    pub mode: AppMode,
    pub search_query: String,
    pub search_cursor: usize,
    pub selected_index: usize,
    pub multi_selected: Vec<String>,
    pub filtered_bookmarks: Vec<(usize, i64, &'a Bookmark)>,
    pub total_count: usize,
    pub form_data: Option<FormData>,
    pub delete_names: Vec<String>,
    pub status_message: String,
    // Import/Export state
    pub import_export_path: String,
    pub import_export_cursor: usize,
    pub import_export_error: String,
    pub import_export_operation: Option<char>,
    // History state
    pub history_entries: Vec<crate::history::HistoryEntry>,
    pub history_selected: usize,
    pub history_multi_selected: Vec<String>,
    pub history_delete_count: usize,
    pub help_scroll: u16,
}

pub struct App;

impl App {
    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let config = Config::load();
        let theme = Theme::from_config(&config.theme);
        let search_engine = SearchEngine::new();
        let store_path = config.bookmarks_path();

        let mut store = storage::load_bookmarks(&store_path).unwrap_or_default();

        // Setup terminal
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout());
        let mut terminal = Terminal::new(backend)?;

        let result = Self::main_loop(
            &mut terminal,
            &mut store,
            &config,
            &theme,
            &search_engine,
            &store_path,
        );

        // Restore terminal
        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;

        result?;
        Ok(())
    }

    fn main_loop(
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
        store: &mut BookmarkStore,
        config: &Config,
        theme: &Theme,
        search: &SearchEngine,
        store_path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut mode = AppMode::Normal;
        let mut search_query = String::new();
        let mut search_cursor: usize = 0;
        let mut selected_index: usize = 0;
        let mut multi_selected: Vec<String> = Vec::new();
        let mut form_data: Option<FormData> = None;
        let mut editing_id: Option<String> = None;
        let mut status_message = String::new();
        let mut message_timer: u8 = 0;

        let mut import_export_path = String::new();
        let mut import_export_cursor: usize = 0;
        let mut import_export_error = String::new();
        let mut import_export_operation: Option<char> = None;

        let mut history_selected: usize = 0;
        let mut help_scroll: u16 = 0;
        let mut history_multi_selected: Vec<String> = Vec::new();

        loop {
            // Filter bookmarks
            let filtered = search.filter(&store.bookmarks, &search_query);

            // Clamp selected index
            if !filtered.is_empty() && selected_index >= filtered.len() {
                selected_index = filtered.len() - 1;
            }

            // Prepare delete names
            let delete_names: Vec<String> =
                if !multi_selected.is_empty() && mode == AppMode::DeleteConfirm {
                    store
                        .bookmarks
                        .iter()
                        .filter(|b| multi_selected.contains(&b.id))
                        .map(|b| b.name.clone())
                        .collect()
                } else if mode == AppMode::DeleteConfirm && !filtered.is_empty() {
                    vec![filtered[selected_index].2.name.clone()]
                } else {
                    Vec::new()
                };

            let history_entries = crate::history::load_index().entries;

            if history_entries.is_empty() {
                history_selected = 0;
            } else if history_selected >= history_entries.len() {
                history_selected = history_entries.len() - 1;
            }

            let history_delete_count = if !history_multi_selected.is_empty() {
                history_multi_selected.len()
            } else if !history_entries.is_empty() {
                1
            } else {
                0
            };

            // Build state for rendering
            let state = AppState {
                mode: mode.clone(),
                search_query: search_query.clone(),
                search_cursor,
                selected_index,
                multi_selected: multi_selected.clone(),
                filtered_bookmarks: filtered.iter().map(|(i, s, b)| (*i, *s, *b)).collect(),
                total_count: store.bookmarks.len(),
                form_data: form_data.clone(),
                delete_names: delete_names.clone(),
                status_message: status_message.clone(),
                import_export_path: import_export_path.clone(),
                import_export_cursor,
                import_export_error: import_export_error.clone(),
                import_export_operation,
                history_entries: history_entries.clone(),
                history_selected,
                history_multi_selected: history_multi_selected.clone(),
                history_delete_count,
                help_scroll,
            };

            // Render
            terminal.draw(|frame| {
                main_view::render(frame, &state, theme);
            })?;

            // Decrease message timer
            if message_timer > 0 {
                message_timer -= 1;
                if message_timer == 0 {
                    status_message.clear();
                }
            }

            // Event handling
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }

                    let action = handle_key(&mode, key);

                    match action {
                        Action::Quit => break,

                        // ── Normal navigation ──
                        Action::MoveDown => {
                            if mode == AppMode::Help {
                                let term_h = terminal.size().map(|s| s.height).unwrap_or(30);
                                let popup_h = 30u16.min(term_h.saturating_sub(2));
                                let max_scroll = crate::ui::help_popup::content_height(popup_h);
                                if help_scroll < max_scroll {
                                    help_scroll += 1;
                                }
                            } else if mode == AppMode::History {
                                let entries = crate::history::load_index().entries;
                                if !entries.is_empty() && history_selected < entries.len() - 1 {
                                    history_selected += 1;
                                }
                            } else {
                                if !filtered.is_empty() && selected_index < filtered.len() - 1 {
                                    selected_index += 1;
                                }
                            }
                        }
                        Action::MoveUp => {
                            if mode == AppMode::Help {
                                if help_scroll > 0 {
                                    help_scroll -= 1;
                                }
                            } else if mode == AppMode::History {
                                if history_selected > 0 {
                                    history_selected -= 1;
                                }
                            } else {
                                if selected_index > 0 {
                                    selected_index -= 1;
                                }
                            }
                        }
                        Action::GoTop => {
                            if mode == AppMode::Help {
                                help_scroll = 0;
                            } else if mode == AppMode::History {
                                history_selected = 0;
                            } else {
                                selected_index = 0;
                            }
                        }
                        Action::GoBottom => {
                            if mode == AppMode::Help {
                                let term_h = terminal.size().map(|s| s.height).unwrap_or(30);
                                let popup_h = 30u16.min(term_h.saturating_sub(2));
                                help_scroll = crate::ui::help_popup::content_height(popup_h);
                            } else if mode == AppMode::History {
                                let entries = crate::history::load_index().entries;
                                if !entries.is_empty() {
                                    history_selected = entries.len() - 1;
                                }
                            } else {
                                if !filtered.is_empty() {
                                    selected_index = filtered.len() - 1;
                                }
                            }
                        }

                        // ── Mode switches ──
                        Action::EnterSearch => {
                            mode = AppMode::Search;
                            search_cursor = search_query.len();
                        }
                        Action::Cancel => {
                            mode = AppMode::Normal;
                            search_query.clear();
                            search_cursor = 0;
                            selected_index = 0;
                            help_scroll = 0;
                        }

                        // ── Search input with cursor ──
                        Action::SearchInput(c) => {
                            if search_cursor >= search_query.len() {
                                search_query.push(c);
                            } else {
                                search_query.insert(search_cursor, c);
                            }
                            search_cursor += c.len_utf8();
                            selected_index = 0;
                        }
                        Action::SearchBackspace => {
                            if search_cursor > 0 {
                                let before = &search_query[..search_cursor];
                                if let Some(prev) = before.chars().last() {
                                    search_cursor -= prev.len_utf8();
                                    search_query.remove(search_cursor);
                                }
                            }
                            selected_index = 0;
                        }
                        Action::SearchDelete => {
                            if search_cursor < search_query.len() {
                                search_query.remove(search_cursor);
                            }
                            selected_index = 0;
                        }
                        Action::SearchClear => {
                            search_query.clear();
                            search_cursor = 0;
                            selected_index = 0;
                        }
                        Action::SearchDeleteWord => {
                            if search_cursor > 0 {
                                let before = search_query[..search_cursor].to_string();
                                let trimmed_len = before.trim_end().len();
                                let new_pos = if trimmed_len == 0 {
                                    0
                                } else {
                                    match before[..trimmed_len].rfind(' ') {
                                        Some(pos) => pos + 1,
                                        None => 0,
                                    }
                                };
                                let after = search_query[search_cursor..].to_string();
                                search_query = format!("{}{}", &before[..new_pos], after);
                                search_cursor = new_pos;
                            }
                            selected_index = 0;
                        }
                        Action::SearchCursorLeft => {
                            if search_cursor > 0 {
                                let before = &search_query[..search_cursor];
                                if let Some(prev) = before.chars().last() {
                                    search_cursor -= prev.len_utf8();
                                }
                            }
                        }
                        Action::SearchCursorRight => {
                            if search_cursor < search_query.len() {
                                let after = &search_query[search_cursor..];
                                if let Some(next) = after.chars().next() {
                                    search_cursor += next.len_utf8();
                                }
                            }
                        }
                        Action::SearchCursorHome => search_cursor = 0,
                        Action::SearchCursorEnd => search_cursor = search_query.len(),
                        Action::SearchConfirm => mode = AppMode::Normal,
                        Action::SearchNavigateDown => {
                            if !filtered.is_empty() && selected_index < filtered.len() - 1 {
                                selected_index += 1;
                            }
                        }
                        Action::SearchNavigateUp => {
                            if selected_index > 0 {
                                selected_index -= 1;
                            }
                        }

                        // ── Open ──
                        Action::Open => {
                            if !filtered.is_empty() {
                                let url = filtered[selected_index].2.url.clone();
                                disable_raw_mode()?;
                                stdout().execute(LeaveAlternateScreen)?;
                                let _ = launcher::open_bookmark(&url, config);
                                return Ok(());
                            }
                        }

                        // ── Yank ──
                        Action::YankUrl => {
                            if !filtered.is_empty() {
                                let url = &filtered[selected_index].2.url;
                                match launcher::yank_to_clipboard(url) {
                                    Ok(()) => {
                                        status_message = format!("Copied: {}", url);
                                        message_timer = 20;
                                    }
                                    Err(e) => {
                                        status_message = format!("Yank error: {}", e);
                                        message_timer = 30;
                                    }
                                }
                            }
                        }

                        // ── Select ──
                        Action::ToggleSelect => {
                            if !filtered.is_empty() {
                                let id = filtered[selected_index].2.id.clone();
                                if multi_selected.contains(&id) {
                                    multi_selected.retain(|s| s != &id);
                                } else {
                                    multi_selected.push(id);
                                }
                            }
                        }

                        // ── Add ──
                        Action::EnterAdd => {
                            mode = AppMode::Add;
                            form_data = Some(FormData::new());
                            editing_id = None;
                        }

                        // ── Edit ──
                        Action::EnterEdit => {
                            if !filtered.is_empty() {
                                let bm = filtered[selected_index].2;
                                mode = AppMode::Edit;
                                form_data =
                                    Some(FormData::from_bookmark(&bm.name, &bm.url, &bm.folder));
                                editing_id = Some(bm.id.clone());
                            }
                        }

                        // ── Delete ──
                        Action::EnterDeleteConfirm => {
                            if !filtered.is_empty() || !multi_selected.is_empty() {
                                mode = AppMode::DeleteConfirm;
                            }
                        }
                        Action::BulkDelete => {
                            if !multi_selected.is_empty() {
                                mode = AppMode::DeleteConfirm;
                            }
                        }
                        Action::ConfirmDelete => {
                            let ids = if !multi_selected.is_empty() {
                                multi_selected.clone()
                            } else if !filtered.is_empty() {
                                vec![filtered[selected_index].2.id.clone()]
                            } else {
                                vec![]
                            };
                            let count = ids.len();
                            store.remove_many(&ids);
                            multi_selected.clear();
                            let _ = storage::save_bookmarks(store_path, store);
                            status_message = format!("Deleted {} bookmark(s)", count);
                            message_timer = 20;
                            mode = AppMode::Normal;
                            if selected_index > 0 {
                                selected_index -= 1;
                            }
                        }
                        Action::CancelDelete => mode = AppMode::Normal,

                        // ── Form cursor & input ──
                        Action::FormInput(c) => {
                            if let Some(ref mut form) = form_data {
                                form.insert_char(c);
                            }
                        }
                        Action::FormBackspace => {
                            if let Some(ref mut form) = form_data {
                                form.backspace();
                            }
                        }
                        Action::FormDelete => {
                            if let Some(ref mut form) = form_data {
                                form.delete_at_cursor();
                            }
                        }
                        Action::FormDeleteWord => {
                            if let Some(ref mut form) = form_data {
                                form.delete_word_before_cursor();
                            }
                        }
                        Action::FormClearField => {
                            if let Some(ref mut form) = form_data {
                                form.clear_field();
                            }
                        }
                        Action::FormCursorLeft => {
                            if let Some(ref mut form) = form_data {
                                form.cursor_left();
                            }
                        }
                        Action::FormCursorRight => {
                            if let Some(ref mut form) = form_data {
                                form.cursor_right();
                            }
                        }
                        Action::FormCursorHome => {
                            if let Some(ref mut form) = form_data {
                                form.cursor_home();
                            }
                        }
                        Action::FormCursorEnd => {
                            if let Some(ref mut form) = form_data {
                                form.cursor_end();
                            }
                        }
                        Action::FormNextField => {
                            if let Some(ref mut form) = form_data {
                                form.next_field();
                            }
                        }
                        Action::FormPrevField => {
                            if let Some(ref mut form) = form_data {
                                form.prev_field();
                            }
                        }

                        // Bug fix UX#3: Enter di field terakhir = save,
                        // di field lain = next field
                        Action::FormEnter => {
                            let is_last = form_data
                                .as_ref()
                                .map(|f| f.is_last_field())
                                .unwrap_or(false);

                            if is_last {
                                Self::do_save(
                                    &mut form_data,
                                    &mut editing_id,
                                    store,
                                    store_path,
                                    &mut status_message,
                                    &mut message_timer,
                                    &mut mode,
                                );
                            } else if let Some(ref mut form) = form_data {
                                form.next_field();
                            }
                        }

                        Action::FormSave => {
                            Self::do_save(
                                &mut form_data,
                                &mut editing_id,
                                store,
                                store_path,
                                &mut status_message,
                                &mut message_timer,
                                &mut mode,
                            );
                        }

                        Action::FormCancel => {
                            mode = AppMode::Normal;
                            form_data = None;
                            editing_id = None;
                        }

                        // ── Help ──
                        Action::ShowHelp => mode = AppMode::Help,


                        // ── Import/Export ──
                        Action::EnterImportExport => {
                            mode = AppMode::ImportExport;
                            import_export_path.clear();
                            import_export_cursor = 0;
                            import_export_error.clear();
                            import_export_operation = None;
                        }
                        Action::ImportExportSelect(c) => {
                            import_export_operation = Some(c);
                            if c == '4' {
                                // Import from Chromium browser (no path needed)
                                match crate::import_export::import_from_browser("chromium") {
                                    Ok(count) => {
                                        crate::history::create_snapshot(
                                            crate::history::HistoryAction::Import,
                                            format!("Import {} from Chromium", count),
                                            store,
                                        ).ok();
                                        *store = storage::load_bookmarks(store_path).unwrap_or_default();
                                        status_message = format!("✓ Imported {} bookmarks from Chromium", count);
                                        message_timer = 30;
                                        mode = AppMode::Normal;
                                    }
                                    Err(e) => {
                                        import_export_error = e;
                                        mode = AppMode::ImportExportInput;
                                    }
                                }
                            } else {
                                match c {
                                    '1' | '2' | '3' => {
                                        let default_dir = dirs::data_dir()
                                            .unwrap_or_else(|| std::path::PathBuf::from("."))
                                            .join("edbookmark/imports/");
                                        import_export_path = default_dir.to_string_lossy().to_string();
                                        import_export_cursor = import_export_path.len();
                                        import_export_error.clear();
                                        mode = AppMode::ImportExportInput;
                                    }
                                    '5' | '6' | '7' => {
                                        // Export langsung tanpa input path
                                        let export_dir = dirs::data_dir()
                                            .unwrap_or_else(|| std::path::PathBuf::from("."))
                                            .join("edbookmark/exports");
                                        let _ = std::fs::create_dir_all(&export_dir);
                                        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                                        let (format, ext) = match c {
                                            '5' => ("json", "json"),
                                            '6' => ("html", "html"),
                                            '7' => ("xlsx", "xlsx"),
                                            _ => ("json", "json"),
                                        };
                                        let filename = format!("bookmarks_{}.{}", timestamp, ext);
                                        let output_path = export_dir.join(&filename);
                                        let output_str = output_path.to_string_lossy().to_string();
                                        crate::history::create_snapshot(
                                            crate::history::HistoryAction::Export,
                                            format!("Export to {}", filename),
                                            store,
                                        ).ok();
                                        match crate::import_export::export_bookmarks(format, &output_str) {
                                            Ok(count) => {
                                                status_message = format!("✓ Exported {} bookmarks to {}", count, filename);
                                                message_timer = 30;
                                                mode = AppMode::Normal;
                                            }
                                            Err(e) => {
                                                status_message = format!("✗ Export failed: {}", e);
                                                message_timer = 30;
                                                mode = AppMode::Normal;
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Action::ImportExportCancel => {
                            mode = AppMode::Normal;
                            import_export_operation = None;
                        }
                        Action::PathInput(c) => {
                            import_export_path.insert(import_export_cursor, c);
                            import_export_cursor += c.len_utf8();
                            import_export_error.clear();
                        }
                        Action::PathBackspace => {
                            if import_export_cursor > 0 {
                                let before = &import_export_path[..import_export_cursor];
                                if let Some(prev) = before.chars().last() {
                                    import_export_cursor -= prev.len_utf8();
                                    import_export_path.remove(import_export_cursor);
                                }
                            }
                        }
                        Action::PathDelete => {
                            if import_export_cursor < import_export_path.len() {
                                import_export_path.remove(import_export_cursor);
                            }
                        }
                        Action::PathCursorLeft => {
                            if import_export_cursor > 0 {
                                let before = &import_export_path[..import_export_cursor];
                                if let Some(prev) = before.chars().last() {
                                    import_export_cursor -= prev.len_utf8();
                                }
                            }
                        }
                        Action::PathCursorRight => {
                            if import_export_cursor < import_export_path.len() {
                                let after = &import_export_path[import_export_cursor..];
                                if let Some(next) = after.chars().next() {
                                    import_export_cursor += next.len_utf8();
                                }
                            }
                        }
                        Action::PathCursorHome => import_export_cursor = 0,
                        Action::PathCursorEnd => import_export_cursor = import_export_path.len(),
                        Action::PathClear => {
                            import_export_path.clear();
                            import_export_cursor = 0;
                        }
                        Action::PathDeleteWord => {
                            if import_export_cursor > 0 {
                                let before = import_export_path[..import_export_cursor].to_string();
                                let trimmed_len = before.trim_end().len();
                                let new_pos = if trimmed_len == 0 {
                                    0
                                } else {
                                    match before[..trimmed_len].rfind('/') {
                                        Some(pos) => pos + 1,
                                        None => 0,
                                    }
                                };
                                let after = import_export_path[import_export_cursor..].to_string();
                                import_export_path = format!("{}{}", &before[..new_pos], after);
                                import_export_cursor = new_pos;
                            }
                        }
                        Action::PathCancel => {
                            mode = AppMode::ImportExport;
                            import_export_error.clear();
                        }
                        Action::PathConfirm => {
                            let path = import_export_path.trim().replace("~", &std::env::var("HOME").unwrap_or_default());
                            if path.is_empty() {
                                import_export_error = "Path cannot be empty".to_string();
                            } else {
                                match import_export_operation {
                                    Some('1') | Some('2') | Some('3') => {
                                        // Import
                                        crate::history::create_snapshot(
                                            crate::history::HistoryAction::Import,
                                            format!("Before import from {}", path),
                                            store,
                                        ).ok();
                                        match crate::import_export::import_from_file(&path) {
                                            Ok(count) => {
                                                *store = storage::load_bookmarks(store_path).unwrap_or_default();
                                                status_message = format!("✓ Imported {} bookmarks", count);
                                                message_timer = 30;
                                                mode = AppMode::Normal;
                                                import_export_path.clear();
                                            }
                                            Err(e) => import_export_error = e,
                                        }
                                    }
                                    Some('5') => {
                                        crate::history::create_snapshot(
                                            crate::history::HistoryAction::Export,
                                            format!("Export to {}", path),
                                            store,
                                        ).ok();
                                        match crate::import_export::export_bookmarks("json", &path) {
                                            Ok(count) => {
                                                status_message = format!("✓ Exported {} bookmarks to JSON", count);
                                                message_timer = 30;
                                                mode = AppMode::Normal;
                                                import_export_path.clear();
                                            }
                                            Err(e) => import_export_error = e,
                                        }
                                    }
                                    Some('6') => {
                                        crate::history::create_snapshot(
                                            crate::history::HistoryAction::Export,
                                            format!("Export to {}", path),
                                            store,
                                        ).ok();
                                        match crate::import_export::export_bookmarks("html", &path) {
                                            Ok(count) => {
                                                status_message = format!("✓ Exported {} bookmarks to HTML", count);
                                                message_timer = 30;
                                                mode = AppMode::Normal;
                                                import_export_path.clear();
                                            }
                                            Err(e) => import_export_error = e,
                                        }
                                    }
                                    Some('7') => {
                                        crate::history::create_snapshot(
                                            crate::history::HistoryAction::Export,
                                            format!("Export to {}", path),
                                            store,
                                        ).ok();
                                        match crate::import_export::export_bookmarks("xlsx", &path) {
                                            Ok(count) => {
                                                status_message = format!("✓ Exported {} bookmarks to XLSX", count);
                                                message_timer = 30;
                                                mode = AppMode::Normal;
                                                import_export_path.clear();
                                            }
                                            Err(e) => import_export_error = e,
                                        }
                                    }
                                    _ => import_export_error = "Invalid operation".to_string(),
                                }
                            }
                        }

                        // ── History ──
                        Action::EnterHistory => {
                            mode = AppMode::History;
                            history_selected = 0;
                            history_multi_selected.clear();
                        }
                        Action::HistoryRestore => {
                            let entries = crate::history::load_index().entries;
                            if !entries.is_empty() && history_selected < entries.len() {
                                let entry_id = entries[history_selected].id.clone();
                                match crate::history::restore_snapshot(&entry_id) {
                                    Ok((count, desc)) => {
                                        *store = storage::load_bookmarks(store_path).unwrap_or_default();
                                        status_message = format!("✓ Restored: {} ({} bookmarks)", desc, count);
                                        message_timer = 30;
                                        mode = AppMode::Normal;
                                    }
                                    Err(e) => {
                                        status_message = format!("✗ Restore failed: {}", e);
                                        message_timer = 30;
                                    }
                                }
                            }
                        }
                        Action::HistoryToggleSelect => {
                            let entries = crate::history::load_index().entries;
                            if !entries.is_empty() && history_selected < entries.len() {
                                let id = entries[history_selected].id.clone();
                                if history_multi_selected.contains(&id) {
                                    history_multi_selected.retain(|s| s != &id);
                                } else {
                                    history_multi_selected.push(id);
                                }
                            }
                        }
                        Action::HistoryDelete => {
                            let entries = crate::history::load_index().entries;
                            if !entries.is_empty() {
                                mode = AppMode::HistoryDeleteConfirm;
                            }
                        }
                        Action::HistoryBulkDelete => {
                            if !history_multi_selected.is_empty() {
                                mode = AppMode::HistoryDeleteConfirm;
                            }
                        }
                        Action::HistoryConfirmDelete => {
                            let ids: Vec<String> = if !history_multi_selected.is_empty() {
                                history_multi_selected.clone()
                            } else {
                                let entries = crate::history::load_index().entries;
                                if !entries.is_empty() && history_selected < entries.len() {
                                    vec![entries[history_selected].id.clone()]
                                } else {
                                    vec![]
                                }
                            };
                            if !ids.is_empty() {
                                match crate::history::delete_entries(&ids) {
                                    Ok(count) => {
                                        status_message = format!("✓ Deleted {} history entries", count);
                                        message_timer = 20;
                                        history_multi_selected.clear();
                                        if history_selected > 0 {
                                            history_selected -= 1;
                                        }
                                    }
                                    Err(e) => {
                                        status_message = format!("✗ Delete failed: {}", e);
                                        message_timer = 30;
                                    }
                                }
                            }
                            mode = AppMode::History;
                        }
                        Action::HistoryCancelDelete => {
                            mode = AppMode::History;
                        }
                        Action::HistoryExport => {
                            let entries = crate::history::load_index().entries;
                            if !entries.is_empty() {
                                mode = AppMode::HistoryExportSelect;
                            } else {
                                status_message = "No history to export".to_string();
                                message_timer = 20;
                            }
                        }
                        Action::HistoryExportSelect(c) => {
                            let (format, ext) = match c {
                                '1' => ("json", "json"),
                                '2' => ("html", "html"),
                                '3' => ("xlsx", "xlsx"),
                                _ => ("json", "json"),
                            };
                            let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                            let filename = format!("history_export_{}.{}", timestamp, ext);
                            let output_dir = dirs::download_dir()
                                .or_else(dirs::home_dir)
                                .unwrap_or_else(|| std::path::PathBuf::from("."));
                            let output_path = output_dir.join(&filename);
                            let output_str = output_path.to_string_lossy().to_string();
                            match crate::history::export_history(format, &output_str) {
                                Ok(count) => {
                                    status_message = format!("✓ Exported {} entries to {}", count, filename);
                                    message_timer = 30;
                                }
                                Err(e) => {
                                    status_message = format!("✗ Export failed: {}", e);
                                    message_timer = 30;
                                }
                            }
                            mode = AppMode::History;
                        }
                        Action::HistoryExportCancel => {
                            mode = AppMode::History;
                        }
                        Action::HistoryCancel => {
                            mode = AppMode::Normal;
                            history_multi_selected.clear();
                        }
                        Action::None => {}
                    }
                }
            }
        }

        let _ = storage::save_bookmarks(store_path, store);
        Ok(())
    }

    /// Helper: save form data (dipakai oleh FormSave dan FormEnter)
    /// Semua parameter adalah &mut sehingga tidak ada konflik borrow
    fn do_save(
        form_data: &mut Option<FormData>,
        editing_id: &mut Option<String>,
        store: &mut BookmarkStore,
        store_path: &PathBuf,
        status_message: &mut String,
        message_timer: &mut u8,
        mode: &mut AppMode,
    ) {
        // Clone snapshot dari form_data agar bisa mutasi form_data setelahnya
        let snapshot = match form_data.clone() {
            Some(form) => form,
            None => return,
        };

        if snapshot.name.is_empty() || snapshot.url.is_empty() {
            *status_message = "Name and URL are required!".to_string();
            *message_timer = 20;
            return;
        }

        // Bug fix #3: folder kosong = "Uncategorized" untuk KEDUA mode
        let folder = if snapshot.folder.trim().is_empty() {
            "Uncategorized".to_string()
        } else {
            snapshot.folder.clone()
        };

        match editing_id {
            Some(id) => {
                store.update(id, snapshot.name.clone(), snapshot.url.clone(), folder);
                *status_message = format!("Updated: {}", snapshot.name);
            }
            None => {
                store.add(Bookmark::new(
                    snapshot.name.clone(),
                    snapshot.url.clone(),
                    folder,
                ));
                *status_message = format!("Added: {}", snapshot.name);
            }
        }

        let _ = storage::save_bookmarks(store_path, store);
        *message_timer = 20;
        *mode = AppMode::Normal;
        *form_data = None;
        *editing_id = None;
    }
}
