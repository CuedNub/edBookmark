use crate::bookmark::BookmarkStore;
use std::fs;
use std::path::PathBuf;

pub fn load_bookmarks(path: &PathBuf) -> Result<BookmarkStore, String> {
    if !path.exists() {
        return Ok(BookmarkStore::default());
    }
    let content = fs::read_to_string(path).map_err(|e| format!("Cannot read {:?}: {}", path, e))?;
    let store: BookmarkStore =
        serde_json::from_str(&content).map_err(|e| format!("Cannot parse {:?}: {}", path, e))?;
    Ok(store)
}

pub fn save_bookmarks(path: &PathBuf, store: &BookmarkStore) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Cannot create dir {:?}: {}", parent, e))?;
    }
    let json =
        serde_json::to_string_pretty(store).map_err(|e| format!("Cannot serialize: {}", e))?;
    fs::write(path, json).map_err(|e| format!("Cannot write {:?}: {}", path, e))?;
    Ok(())
}
