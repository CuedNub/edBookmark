use crate::keybinding::{AppMode, FormField};
use crate::theme::Theme;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

#[derive(Clone)]
pub struct FormData {
    pub name: String,
    pub url: String,
    pub folder: String,
    pub active_field: FormField,
    pub cursor_pos: usize,
}

impl FormData {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            url: String::new(),
            folder: String::new(),
            active_field: FormField::Name,
            cursor_pos: 0,
        }
    }

    pub fn from_bookmark(name: &str, url: &str, folder: &str) -> Self {
        let name = name.to_string();
        let cursor = name.len();
        Self {
            name,
            url: url.to_string(),
            folder: folder.to_string(),
            active_field: FormField::Name,
            cursor_pos: cursor,
        }
    }

    fn get_active_field(&self) -> &str {
        match self.active_field {
            FormField::Name => &self.name,
            FormField::Url => &self.url,
            FormField::Folder => &self.folder,
        }
    }

    fn set_active_field(&mut self, val: String) {
        match self.active_field {
            FormField::Name => self.name = val,
            FormField::Url => self.url = val,
            FormField::Folder => self.folder = val,
        }
    }

    /// Clamp cursor to valid range
    pub fn clamp_cursor(&mut self) {
        let len = self.get_active_field().len();
        if self.cursor_pos > len {
            self.cursor_pos = len;
        }
    }

    /// Insert char at cursor position
    pub fn insert_char(&mut self, c: char) {
        self.clamp_cursor();
        let mut val = self.get_active_field().to_string();
        let pos = self.cursor_pos;
        if pos >= val.len() {
            val.push(c);
        } else {
            val.insert(pos, c);
        }
        self.cursor_pos = pos + c.len_utf8();
        self.set_active_field(val);
    }

    /// Delete char before cursor (Backspace)
    pub fn backspace(&mut self) {
        self.clamp_cursor();
        if self.cursor_pos == 0 {
            return;
        }
        let val = self.get_active_field().to_string();
        let before = &val[..self.cursor_pos];
        if let Some(prev_char) = before.chars().last() {
            let char_len = prev_char.len_utf8();
            let new_pos = self.cursor_pos - char_len;
            let mut new_val = String::with_capacity(val.len() - char_len);
            new_val.push_str(&val[..new_pos]);
            new_val.push_str(&val[self.cursor_pos..]);
            self.cursor_pos = new_pos;
            self.set_active_field(new_val);
        }
    }

    /// Delete char at cursor position (Delete key)
    pub fn delete_at_cursor(&mut self) {
        self.clamp_cursor();
        let val = self.get_active_field().to_string();
        if self.cursor_pos >= val.len() {
            return;
        }
        let after = &val[self.cursor_pos..];
        if let Some(next_char) = after.chars().next() {
            let char_len = next_char.len_utf8();
            let mut new_val = String::with_capacity(val.len() - char_len);
            new_val.push_str(&val[..self.cursor_pos]);
            new_val.push_str(&val[self.cursor_pos + char_len..]);
            self.set_active_field(new_val);
        }
    }

    /// Move cursor left
    pub fn cursor_left(&mut self) {
        self.clamp_cursor();
        if self.cursor_pos == 0 {
            return;
        }
        let val = self.get_active_field();
        let before = &val[..self.cursor_pos];
        if let Some(prev_char) = before.chars().last() {
            self.cursor_pos -= prev_char.len_utf8();
        }
    }

    /// Move cursor right
    pub fn cursor_right(&mut self) {
        self.clamp_cursor();
        let val = self.get_active_field();
        if self.cursor_pos >= val.len() {
            return;
        }
        let after = &val[self.cursor_pos..];
        if let Some(next_char) = after.chars().next() {
            self.cursor_pos += next_char.len_utf8();
        }
    }

    /// Move cursor to start
    pub fn cursor_home(&mut self) {
        self.cursor_pos = 0;
    }

    /// Move cursor to end
    pub fn cursor_end(&mut self) {
        self.cursor_pos = self.get_active_field().len();
    }

    /// Delete word before cursor (Ctrl+W)
    pub fn delete_word_before_cursor(&mut self) {
        self.clamp_cursor();
        if self.cursor_pos == 0 {
            return;
        }

        let val = self.get_active_field().to_string();
        let before = &val[..self.cursor_pos];

        // Skip trailing spaces
        let trimmed_len = before.trim_end().len();
        if trimmed_len == 0 {
            let after = &val[self.cursor_pos..];
            let new_val = after.to_string();
            self.cursor_pos = 0;
            self.set_active_field(new_val);
            return;
        }

        let trimmed = &before[..trimmed_len];
        let new_pos = match trimmed.rfind(' ') {
            Some(pos) => pos + 1,
            None => 0,
        };

        let mut new_val = String::new();
        new_val.push_str(&val[..new_pos]);
        new_val.push_str(&val[self.cursor_pos..]);
        self.cursor_pos = new_pos;
        self.set_active_field(new_val);
    }

    /// Clear entire field (Ctrl+U)
    pub fn clear_field(&mut self) {
        self.set_active_field(String::new());
        self.cursor_pos = 0;
    }

    /// Switch to next field with cursor at end
    pub fn next_field(&mut self) {
        self.active_field = self.active_field.next();
        self.cursor_pos = self.get_active_field().len();
    }

    /// Switch to prev field with cursor at end
    pub fn prev_field(&mut self) {
        self.active_field = self.active_field.prev();
        self.cursor_pos = self.get_active_field().len();
    }
}

pub fn render(frame: &mut Frame, area: Rect, mode: &AppMode, form: &FormData, theme: &Theme) {
    let title = match mode {
        AppMode::Add => " Add Bookmark ",
        AppMode::Edit => " Edit Bookmark ",
        _ => return,
    };

    let popup_width = 60u16.min(area.width.saturating_sub(4));
    let popup_height = 14u16.min(area.height.saturating_sub(4));
    let popup_x = (area.width.saturating_sub(popup_width)) / 2 + area.x;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2 + area.y;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent()))
        .title(title)
        .title_style(
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(theme.bg()));

    frame.render_widget(block, popup_area);

    let inner = Rect::new(
        popup_area.x + 2,
        popup_area.y + 1,
        popup_area.width.saturating_sub(4),
        popup_area.height.saturating_sub(2),
    );

    let field_areas = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(2),
    ])
    .split(inner);

    let fields: [(&str, &str, FormField); 3] = [
        ("Name", &form.name, FormField::Name),
        ("URL", &form.url, FormField::Url),
        ("Folder", &form.folder, FormField::Folder),
    ];

    for (i, (label, value, field)) in fields.iter().enumerate() {
        let cursor = if form.active_field == *field {
            Some(form.cursor_pos)
        } else {
            None
        };
        render_field(frame, field_areas[i], label, value, &form.active_field, field, cursor, theme);
    }

    let hint = Paragraph::new(" Ctrl+S: Save │ Esc: Cancel │ Tab: Next │ ←→: Cursor")
        .style(Style::default().fg(theme.muted()));
    frame.render_widget(hint, field_areas[3]);
}

fn render_field(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    value: &str,
    active_field: &FormField,
    this_field: &FormField,
    cursor_pos: Option<usize>,
    theme: &Theme,
) {
    let is_active = active_field == this_field;
    let border_color = if is_active {
        theme.field_active_border()
    } else {
        theme.field_inactive_border()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(format!(" {} ", label))
        .title_style(Style::default().fg(if is_active {
            theme.field_active_border()
        } else {
            theme.muted()
        }));

    let display = if value.is_empty() && !is_active {
        format!("Enter {}...", label.to_lowercase())
    } else {
        value.to_string()
    };

    let text_color = if value.is_empty() && !is_active {
        theme.field_placeholder()
    } else {
        theme.field_text()
    };

    let paragraph = Paragraph::new(display)
        .style(Style::default().fg(text_color).bg(theme.bg()))
        .block(block);

    frame.render_widget(paragraph, area);

    if let Some(pos) = cursor_pos {
        let inner_x = area.x + 1;
        let inner_y = area.y + 1;
        let display_pos = if pos <= value.len() {
            value[..pos].chars().count() as u16
        } else {
            value.chars().count() as u16
        };
        let cursor_x = inner_x + display_pos;
        let cursor_y = inner_y;
        if cursor_x < area.x + area.width - 1 {
            frame.set_cursor_position((cursor_x, cursor_y));
        }
    }
}
