use crate::bookmark::BookmarkStore;
use crate::config::Config;
use crate::storage;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// ─────────────────────────────────────────────────────────────────────────────
// Structs
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub action: HistoryAction,
    pub description: String,
    pub snapshot_file: String,
    pub bookmark_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HistoryAction {
    Import,
    Export,
    Restore,
}

impl std::fmt::Display for HistoryAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HistoryAction::Import => write!(f, "Import"),
            HistoryAction::Export => write!(f, "Export"),
            HistoryAction::Restore => write!(f, "Restore"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryIndex {
    pub entries: Vec<HistoryEntry>,
}

impl Default for HistoryIndex {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Path helpers
// ─────────────────────────────────────────────────────────────────────────────

fn history_dir() -> PathBuf {
    let config = Config::load();
    let bookmarks_path = config.bookmarks_path();
    bookmarks_path
        .parent()
        .unwrap_or(&PathBuf::from("."))
        .join("history")
}

fn index_path() -> PathBuf {
    history_dir().join("index.json")
}

// ─────────────────────────────────────────────────────────────────────────────
// Load / Save index
// ─────────────────────────────────────────────────────────────────────────────

pub fn load_index() -> HistoryIndex {
    let path = index_path();
    if !path.exists() {
        return HistoryIndex::default();
    }
    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => HistoryIndex::default(),
    }
}

fn save_index(index: &HistoryIndex) -> Result<(), String> {
    let dir = history_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("Cannot create history dir: {}", e))?;

    let json = serde_json::to_string_pretty(index)
        .map_err(|e| format!("Cannot serialize history index: {}", e))?;
    fs::write(index_path(), json).map_err(|e| format!("Cannot write history index: {}", e))?;
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Create snapshot (called before import/export/restore)
// ─────────────────────────────────────────────────────────────────────────────

pub fn create_snapshot(
    action: HistoryAction,
    description: String,
    store: &BookmarkStore,
) -> Result<HistoryEntry, String> {
    let dir = history_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("Cannot create history dir: {}", e))?;

    let now = Utc::now();
    let id = now.format("%Y%m%d_%H%M%S").to_string();
    let action_str = format!("{}", action).to_lowercase();
    let snapshot_file = format!("{}_{}.snapshot.json", id, action_str);

    // Save snapshot of current store
    let snapshot_path = dir.join(&snapshot_file);
    let json = serde_json::to_string_pretty(store)
        .map_err(|e| format!("Cannot serialize snapshot: {}", e))?;
    fs::write(&snapshot_path, json).map_err(|e| format!("Cannot write snapshot: {}", e))?;

    let entry = HistoryEntry {
        id,
        timestamp: now,
        action,
        description,
        snapshot_file,
        bookmark_count: store.bookmarks.len(),
    };

    // Append to index
    let mut index = load_index();
    index.entries.insert(0, entry.clone());

    // Keep max 50 entries, remove oldest
    while index.entries.len() > 50 {
        if let Some(old) = index.entries.pop() {
            let old_path = dir.join(&old.snapshot_file);
            let _ = fs::remove_file(old_path);
        }
    }

    save_index(&index)?;
    Ok(entry)
}

// ─────────────────────────────────────────────────────────────────────────────
// Restore from snapshot
// ─────────────────────────────────────────────────────────────────────────────

pub fn restore_snapshot(entry_id: &str) -> Result<(usize, String), String> {
    let index = load_index();
    let entry = index
        .entries
        .iter()
        .find(|e| e.id == entry_id)
        .ok_or_else(|| format!("History entry '{}' not found", entry_id))?
        .clone();

    let snapshot_path = history_dir().join(&entry.snapshot_file);
    if !snapshot_path.exists() {
        return Err(format!("Snapshot file not found: {}", entry.snapshot_file));
    }

    // Load current store and save as restore snapshot first
    let config = Config::load();
    let store_path = config.bookmarks_path();
    let current_store = storage::load_bookmarks(&store_path).unwrap_or_default();
    let _current_count = current_store.bookmarks.len();

    create_snapshot(
        HistoryAction::Restore,
        format!("Before restore to: {}", entry.description),
        &current_store,
    )?;

    // Load snapshot and overwrite bookmarks
    let content = fs::read_to_string(&snapshot_path)
        .map_err(|e| format!("Cannot read snapshot: {}", e))?;
    let snapshot_store: BookmarkStore =
        serde_json::from_str(&content).map_err(|e| format!("Cannot parse snapshot: {}", e))?;

    let restored_count = snapshot_store.bookmarks.len();
    storage::save_bookmarks(&store_path, &snapshot_store)
        .map_err(|e| format!("Cannot save restored bookmarks: {}", e))?;

    Ok((restored_count, entry.description))
}

// ─────────────────────────────────────────────────────────────────────────────
// Delete history entries
// ─────────────────────────────────────────────────────────────────────────────

pub fn delete_entries(entry_ids: &[String]) -> Result<usize, String> {
    let mut index = load_index();
    let dir = history_dir();
    let mut deleted = 0;

    index.entries.retain(|e| {
        if entry_ids.contains(&e.id) {
            let snapshot_path = dir.join(&e.snapshot_file);
            let _ = fs::remove_file(snapshot_path);
            deleted += 1;
            false
        } else {
            true
        }
    });

    save_index(&index)?;
    Ok(deleted)
}

// ─────────────────────────────────────────────────────────────────────────────
// Export history list
// ─────────────────────────────────────────────────────────────────────────────

pub fn export_history(format: &str, output: &str) -> Result<usize, String> {
    let index = load_index();
    let count = index.entries.len();

    if count == 0 {
        return Err("No history entries to export".to_string());
    }

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&index.entries)
                .map_err(|e| format!("Serialize error: {}", e))?;
            fs::write(output, json).map_err(|e| format!("Write error: {}", e))?;
        }
        "html" => {
            let mut html = String::from("<!DOCTYPE html>\n<html><head>\n");
            html.push_str("<meta charset=\"UTF-8\">\n");
            html.push_str("<title>edbookmark History</title>\n");
            html.push_str("<style>table{border-collapse:collapse;width:100%}");
            html.push_str("th,td{border:1px solid #ddd;padding:8px;text-align:left}");
            html.push_str("th{background:#2D4F67;color:white}</style>\n");
            html.push_str("</head><body>\n<h1>edbookmark History</h1>\n");
            html.push_str("<table><tr><th>#</th><th>Date</th><th>Type</th>");
            html.push_str("<th>Description</th><th>Bookmarks</th></tr>\n");

            for (i, e) in index.entries.iter().enumerate() {
                html.push_str(&format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                    i + 1,
                    e.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    e.action,
                    e.description,
                    e.bookmark_count,
                ));
            }

            html.push_str("</table>\n</body></html>");
            fs::write(output, html).map_err(|e| format!("Write error: {}", e))?;
        }
        "xlsx" => {
            export_history_xlsx(&index.entries, output)?;
        }
        _ => return Err(format!("Unknown format: {}. Use: json, html, xlsx", format)),
    }

    Ok(count)
}

fn export_history_xlsx(entries: &[HistoryEntry], output: &str) -> Result<(), String> {
    use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook, XlsxError};

    let mut workbook = Workbook::new();
    let sheet = workbook
        .add_worksheet()
        .set_name("History")
        .map_err(|e: XlsxError| format!("Cannot create sheet: {}", e))?;

    let header_fmt = Format::new()
        .set_bold()
        .set_font_color(Color::White)
        .set_background_color(Color::RGB(0x2D_4F_67))
        .set_border_bottom(FormatBorder::Thin)
        .set_align(FormatAlign::Center);

    let data_fmt = Format::new().set_align(FormatAlign::Left);

    let headers = ["#", "Date", "Type", "Description", "Bookmarks"];
    for (col, &h) in headers.iter().enumerate() {
        sheet
            .write_with_format(0, col as u16, h, &header_fmt)
            .map_err(|e: XlsxError| format!("Write header error: {}", e))?;
    }

    for (i, entry) in entries.iter().enumerate() {
        let row = (i + 1) as u32;
        sheet
            .write_with_format(row, 0, (i + 1) as u32, &data_fmt)
            .map_err(|e: XlsxError| format!("Write error: {}", e))?;
        sheet
            .write_with_format(
                row,
                1,
                entry.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                &data_fmt,
            )
            .map_err(|e: XlsxError| format!("Write error: {}", e))?;
        sheet
            .write_with_format(row, 2, format!("{}", entry.action), &data_fmt)
            .map_err(|e: XlsxError| format!("Write error: {}", e))?;
        sheet
            .write_with_format(row, 3, &entry.description, &data_fmt)
            .map_err(|e: XlsxError| format!("Write error: {}", e))?;
        sheet
            .write_with_format(row, 4, entry.bookmark_count as u32, &data_fmt)
            .map_err(|e: XlsxError| format!("Write error: {}", e))?;
    }

    let col_widths: [(u16, f64); 5] = [(0, 5.0), (1, 20.0), (2, 10.0), (3, 45.0), (4, 12.0)];
    for (col, width) in col_widths {
        sheet
            .set_column_width(col, width)
            .map_err(|e: XlsxError| format!("Column width error: {}", e))?;
    }

    sheet
        .set_freeze_panes(1, 0)
        .map_err(|e: XlsxError| format!("Freeze panes error: {}", e))?;

    workbook
        .save(output)
        .map_err(|e: XlsxError| format!("Cannot save XLSX: {}", e))?;

    Ok(())
}
