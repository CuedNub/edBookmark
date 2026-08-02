use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    Search,
    Add,
    Edit,
    DeleteConfirm,
    Help,
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
    FormEnter, // Bug fix: Enter di field terakhir = save

    // Delete confirm
    ConfirmDelete,
    CancelDelete,

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
        _ => Action::None,
    }
}
