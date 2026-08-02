#![allow(dead_code)]
use crate::config::ThemeConfig;
use ratatui::style::Color;
use std::collections::HashMap;

pub struct Theme {
    colors: HashMap<String, Color>,
}

impl Theme {
    pub fn from_config(config: &ThemeConfig) -> Self {
        let mut colors = HashMap::new();
        for (key, value) in &config.colors {
            colors.insert(key.clone(), parse_color(value));
        }
        Self { colors }
    }

    pub fn get(&self, key: &str) -> Color {
        self.colors.get(key).copied().unwrap_or(Color::Reset)
    }

    pub fn bg(&self) -> Color {
        self.get("bg")
    }
    pub fn fg(&self) -> Color {
        self.get("fg")
    }
    pub fn name(&self) -> Color {
        self.get("name")
    }
    pub fn url(&self) -> Color {
        self.get("url")
    }
    pub fn folder(&self) -> Color {
        self.get("folder")
    }
    pub fn header(&self) -> Color {
        self.get("header")
    }
    pub fn header_bg(&self) -> Color {
        self.get("header_bg")
    }
    pub fn selected_fg(&self) -> Color {
        self.get("selected_fg")
    }
    pub fn selected_bg(&self) -> Color {
        self.get("selected_bg")
    }
    pub fn multiselect_fg(&self) -> Color {
        self.get("multiselect_fg")
    }
    pub fn multiselect_bg(&self) -> Color {
        self.get("multiselect_bg")
    }
    pub fn search_border(&self) -> Color {
        self.get("search_border")
    }
    pub fn match_highlight(&self) -> Color {
        self.get("match_highlight")
    }
    pub fn status_fg(&self) -> Color {
        self.get("status_fg")
    }
    pub fn status_bg(&self) -> Color {
        self.get("status_bg")
    }
    pub fn accent(&self) -> Color {
        self.get("accent")
    }
    pub fn muted(&self) -> Color {
        self.get("muted")
    }
    pub fn border_top(&self) -> Color {
        self.get("border_top")
    }
    pub fn border_right(&self) -> Color {
        self.get("border_right")
    }
    pub fn border_bottom(&self) -> Color {
        self.get("border_bottom")
    }
    pub fn border_left(&self) -> Color {
        self.get("border_left")
    }
    pub fn field_active_border(&self) -> Color {
        self.get("field_active_border")
    }
    pub fn field_inactive_border(&self) -> Color {
        self.get("field_inactive_border")
    }
    pub fn field_text(&self) -> Color {
        self.get("field_text")
    }
    pub fn field_placeholder(&self) -> Color {
        self.get("field_placeholder")
    }
    pub fn delete_border(&self) -> Color {
        self.get("delete_border")
    }
    pub fn delete_text(&self) -> Color {
        self.get("delete_text")
    }
    pub fn button_save_fg(&self) -> Color {
        self.get("button_save_fg")
    }
    pub fn button_save_bg(&self) -> Color {
        self.get("button_save_bg")
    }
    pub fn button_cancel_fg(&self) -> Color {
        self.get("button_cancel_fg")
    }
    pub fn button_cancel_bg(&self) -> Color {
        self.get("button_cancel_bg")
    }
    pub fn button_delete_fg(&self) -> Color {
        self.get("button_delete_fg")
    }
    pub fn button_delete_bg(&self) -> Color {
        self.get("button_delete_bg")
    }
}

fn parse_color(s: &str) -> Color {
    match s.to_lowercase().as_str() {
        "reset" | "transparent" | "none" => Color::Reset,
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        s if s.starts_with('#') && s.len() == 7 => {
            let r = u8::from_str_radix(&s[1..3], 16).unwrap_or(0);
            let g = u8::from_str_radix(&s[3..5], 16).unwrap_or(0);
            let b = u8::from_str_radix(&s[5..7], 16).unwrap_or(0);
            Color::Rgb(r, g, b)
        }
        _ => Color::Reset,
    }
}
