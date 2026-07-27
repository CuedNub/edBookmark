#![allow(dead_code)]
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub window: WindowConfig,
    pub launcher: LauncherConfig,
    pub paths: PathsConfig,
    pub theme: ThemeConfig,
    pub keybindings: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WindowConfig {
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LauncherConfig {
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PathsConfig {
    pub bookmarks: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ThemeConfig {
    pub preset: String,
    pub transparent_bg: bool,
    pub colors: HashMap<String, String>,
}

impl Config {
    pub fn load() -> Self {
        // Cari config di beberapa lokasi
        let paths = vec![
            config_dir().join("config.toml"),
            config_dir().join("default.toml"),
            PathBuf::from("config/default.toml"),
        ];

        for path in &paths {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(path) {
                    match toml::from_str::<Config>(&content) {
                        Ok(config) => return config,
                        Err(e) => {
                            eprintln!("Warning: Config parse error in {:?}: {}", path, e);
                        }
                    }
                }
            }
        }

        // Fallback default
        Self::default()
    }

    pub fn bookmarks_path(&self) -> PathBuf {
        let path = self.paths.bookmarks.replace("~", &dirs::home_dir().unwrap().to_string_lossy());
        PathBuf::from(path)
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut colors = HashMap::new();
        colors.insert("bg".to_string(), "reset".to_string());
        colors.insert("fg".to_string(), "#BFBDB6".to_string());
        colors.insert("name".to_string(), "#E6E1CF".to_string());
        colors.insert("url".to_string(), "#95E6CB".to_string());
        colors.insert("folder".to_string(), "#D2A6FF".to_string());
        colors.insert("header".to_string(), "#FF8F40".to_string());
        colors.insert("header_bg".to_string(), "reset".to_string());
        colors.insert("selected_fg".to_string(), "#E6E1CF".to_string());
        colors.insert("selected_bg".to_string(), "#2D4F67".to_string());
        colors.insert("multiselect_fg".to_string(), "#E6E1CF".to_string());
        colors.insert("multiselect_bg".to_string(), "#3E4B59".to_string());
        colors.insert("search_fg".to_string(), "#E6E1CF".to_string());
        colors.insert("search_border".to_string(), "#39BAE6".to_string());
        colors.insert("match_highlight".to_string(), "#F07178".to_string());
        colors.insert("status_fg".to_string(), "#AAD94C".to_string());
        colors.insert("status_bg".to_string(), "reset".to_string());
        colors.insert("accent".to_string(), "#E6B450".to_string());
        colors.insert("muted".to_string(), "#565B66".to_string());
        colors.insert("border_top".to_string(), "#39BAE6".to_string());
        colors.insert("border_right".to_string(), "#AAD94C".to_string());
        colors.insert("border_bottom".to_string(), "#FF8F40".to_string());
        colors.insert("border_left".to_string(), "#D2A6FF".to_string());
        colors.insert("field_active_border".to_string(), "#39BAE6".to_string());
        colors.insert("field_inactive_border".to_string(), "#565B66".to_string());
        colors.insert("field_text".to_string(), "#E6E1CF".to_string());
        colors.insert("field_placeholder".to_string(), "#565B66".to_string());
        colors.insert("delete_border".to_string(), "#F07178".to_string());
        colors.insert("delete_text".to_string(), "#F07178".to_string());
        colors.insert("button_save_fg".to_string(), "#0D1017".to_string());
        colors.insert("button_save_bg".to_string(), "#AAD94C".to_string());
        colors.insert("button_cancel_fg".to_string(), "#BFBDB6".to_string());
        colors.insert("button_cancel_bg".to_string(), "#3E4B59".to_string());
        colors.insert("button_delete_fg".to_string(), "#0D1017".to_string());
        colors.insert("button_delete_bg".to_string(), "#F07178".to_string());

        Self {
            window: WindowConfig {
                width: 100,
                height: 30,
            },
            launcher: LauncherConfig {
                command: "omarchy-launch-webapp".to_string(),
                args: vec!["--isolate".to_string()],
            },
            paths: PathsConfig {
                bookmarks: "~/.local/share/edbookmark/bookmarks.json".to_string(),
            },
            theme: ThemeConfig {
                preset: "ayu-dark".to_string(),
                transparent_bg: true,
                colors,
            },
            keybindings: HashMap::new(),
        }
    }
}

fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("edbookmark")
}
