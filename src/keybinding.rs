use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    Search,
    Add,
    Edit,
    DeleteConfirm,
    Help,
    ImportExport,
    ImportExportInput,
    History,
    HistoryDeleteConfirm,
    HistoryExportSelect,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FormField {
    Name,
    Url,
    Folder,
}

impl FormField {
    pub fn next(&self) -> Self {
        match self {
            Self::Name => Self::Url,
            Self::Url => Self::Folder,
            Self::Folder => Self::Name,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Self::Name => Self::Folder,
            Self::Url => Self::Name,
            Self::Folder => Self::Url,
        }
    }
}

#[derive(Debug)]
pub enum Action {
    // Navigation
    MoveDown,
    MoveUp,
    GoTop,
    GoBottom,

    // Mode switches
    EnterSearch,
    EnterAdd,
    EnterEdit,
    EnterDeleteConfirm,
    EnterImportExport,
    EnterHistory,
    ShowHelp,
    Cancel,

    // Actions
    Open,
    ToggleSelect,
    BulkDelete,
    YankUrl,
    Quit,

    // Search
    SearchInput(char),
    SearchBackspace,
    SearchClear,
    SearchDeleteWord,
    SearchConfirm,
    SearchNavigateDown,
    SearchNavigateUp,
    SearchCursorLeft,
    SearchCursorRight,
    SearchCursorHome,
    SearchCursorEnd,
    SearchDelete,

    // Form
    FormInput(char),
    FormBackspace,
    FormDelete,
    FormDeleteWord,
    FormClearField,
    FormNextField,
    FormPrevField,
    FormSave,
    FormCancel,
    FormCursorLeft,
    FormCursorRight,
    FormCursorHome,
    FormCursorEnd,
    FormEnter,

    // Delete confirm
    ConfirmDelete,
    CancelDelete,

    // Import/Export
    ImportExportSelect(char),
    ImportExportCancel,

    // Import/Export path input
    PathInput(char),
    PathBackspace,
    PathDelete,
    PathCursorLeft,
    PathCursorRight,
    PathCursorHome,
    PathCursorEnd,
    PathDeleteWord,
    PathClear,
    PathConfirm,
    PathCancel,

    // History
    HistoryRestore,
    HistoryDelete,
    HistoryBulkDelete,
    HistoryExport,
    HistoryToggleSelect,
    HistoryCancel,

    // History delete confirm
    HistoryConfirmDelete,
    HistoryCancelDelete,

    // History export select
    HistoryExportSelect(char),
    HistoryExportCancel,

    // No action
    None,
}

pub fn handle_key(mode: &AppMode, key: KeyEvent) -> Action {
    match mode {
        AppMode::Normal => handle_normal(key),
        AppMode::Search => handle_search(key),
        AppMode::Add | AppMode::Edit => handle_form(key),
        AppMode::DeleteConfirm => handle_delete_confirm(key),
        AppMode::Help => handle_help(key),
        AppMode::ImportExport => handle_import_export(key),
        AppMode::ImportExportInput => handle_import_export_input(key),
        AppMode::History => handle_history(key),
        AppMode::HistoryDeleteConfirm => handle_history_delete_confirm(key),
        AppMode::HistoryExportSelect => handle_history_export_select(key),
    }
}

fn handle_normal(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('j') | KeyCode::Down => Action::MoveDown,
        KeyCode::Char('k') | KeyCode::Up => Action::MoveUp,
        KeyCode::Char('g') => Action::GoTop,
        KeyCode::Char('G') => Action::GoBottom,
        KeyCode::Char('/') => Action::EnterSearch,
        KeyCode::Char('a') => Action::EnterAdd,
        KeyCode::Char('e') => Action::EnterEdit,
        KeyCode::Char('d') => Action::EnterDeleteConfirm,
        KeyCode::Char(' ') => Action::ToggleSelect,
        KeyCode::Char('D') => Action::BulkDelete,
        KeyCode::Char('y') => Action::YankUrl,
        KeyCode::Char('?') => Action::ShowHelp,
        KeyCode::Char('I') => Action::EnterImportExport,
        KeyCode::Char('X') => Action::EnterHistory,
        KeyCode::Enter => Action::Open,
        KeyCode::Esc => Action::Quit,
        _ => Action::None,
    }
}

fn handle_search(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::Cancel,
        KeyCode::Enter => Action::SearchConfirm,
        KeyCode::Backspace => Action::SearchBackspace,
        KeyCode::Delete => Action::SearchDelete,
        KeyCode::Left => Action::SearchCursorLeft,
        KeyCode::Right => Action::SearchCursorRight,
        KeyCode::Home => Action::SearchCursorHome,
        KeyCode::End => Action::SearchCursorEnd,
        KeyCode::Down => Action::SearchNavigateDown,
        KeyCode::Up => Action::SearchNavigateUp,
        KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::SearchCursorLeft
        }
        KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::SearchCursorRight
        }
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::SearchCursorHome
        }
        KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::SearchCursorEnd
        }
        KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::SearchNavigateDown
        }
        KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::SearchNavigateUp
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::SearchClear,
        KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::SearchDeleteWord
        }
        KeyCode::Char(c) => Action::SearchInput(c),
        _ => Action::None,
    }
}

fn handle_form(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::FormCancel,
        KeyCode::Tab => Action::FormNextField,
        KeyCode::BackTab => Action::FormPrevField,
        KeyCode::Enter => Action::FormEnter,
        KeyCode::Backspace => Action::FormBackspace,
        KeyCode::Delete => Action::FormDelete,
        KeyCode::Left => Action::FormCursorLeft,
        KeyCode::Right => Action::FormCursorRight,
        KeyCode::Home => Action::FormCursorHome,
        KeyCode::End => Action::FormCursorEnd,
        KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::FormCursorLeft
        }
        KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::FormCursorRight
        }
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::FormCursorHome
        }
        KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::FormCursorEnd
        }
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::FormSave,
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::FormClearField
        }
        KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::FormDeleteWord
        }
        KeyCode::Char(c) => Action::FormInput(c),
        _ => Action::None,
    }
}

fn handle_delete_confirm(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('y') | KeyCode::Enter => Action::ConfirmDelete,
        KeyCode::Char('n') | KeyCode::Esc => Action::CancelDelete,
        _ => Action::None,
    }
}

fn handle_help(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => Action::Cancel,
        KeyCode::Char('j') | KeyCode::Down => Action::MoveDown,
        KeyCode::Char('k') | KeyCode::Up => Action::MoveUp,
        KeyCode::Char('g') => Action::GoTop,
        KeyCode::Char('G') => Action::GoBottom,
        _ => Action::None,
    }
}

fn handle_import_export(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char(c @ '1'..='7') => Action::ImportExportSelect(c),
        KeyCode::Esc => Action::ImportExportCancel,
        _ => Action::None,
    }
}

fn handle_import_export_input(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::PathCancel,
        KeyCode::Enter => Action::PathConfirm,
        KeyCode::Backspace => Action::PathBackspace,
        KeyCode::Delete => Action::PathDelete,
        KeyCode::Left => Action::PathCursorLeft,
        KeyCode::Right => Action::PathCursorRight,
        KeyCode::Home => Action::PathCursorHome,
        KeyCode::End => Action::PathCursorEnd,
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::PathClear,
        KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::PathDeleteWord
        }
        KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::PathCursorLeft
        }
        KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::PathCursorRight
        }
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::PathCursorHome
        }
        KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::PathCursorEnd
        }
        KeyCode::Char(c) => Action::PathInput(c),
        _ => Action::None,
    }
}

fn handle_history(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Action::MoveDown,
        KeyCode::Char('k') | KeyCode::Up => Action::MoveUp,
        KeyCode::Char('g') => Action::GoTop,
        KeyCode::Char('G') => Action::GoBottom,
        KeyCode::Char('r') | KeyCode::Enter => Action::HistoryRestore,
        KeyCode::Char('d') => Action::HistoryDelete,
        KeyCode::Char('D') => Action::HistoryBulkDelete,
        KeyCode::Char('E') => Action::HistoryExport,
        KeyCode::Char(' ') => Action::HistoryToggleSelect,
        KeyCode::Esc => Action::HistoryCancel,
        _ => Action::None,
    }
}

fn handle_history_delete_confirm(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('y') | KeyCode::Enter => Action::HistoryConfirmDelete,
        KeyCode::Char('n') | KeyCode::Esc => Action::HistoryCancelDelete,
        _ => Action::None,
    }
}

fn handle_history_export_select(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char(c @ '1'..='3') => Action::HistoryExportSelect(c),
        KeyCode::Esc => Action::HistoryExportCancel,
        _ => Action::None,
    }
}
