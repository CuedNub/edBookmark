use crate::bookmark::{Bookmark, BookmarkStore};
use crate::config::Config;
use crate::storage;
use serde_json::Value;
use std::fs;

pub fn import_from_browser(browser: &str) -> Result<usize, String> {
    match browser.to_lowercase().as_str() {
        "chromium" | "chrome" => import_chromium(),
        "firefox" => Err("Firefox import: export bookmarks to HTML first, then use --import-file".to_string()),
        _ => Err(format!("Unknown browser: {}. Use: chromium, firefox", browser)),
    }
}

pub fn import_from_file(path: &str) -> Result<usize, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Cannot read {}: {}", path, e))?;

    if content.contains("<!DOCTYPE NETSCAPE-Bookmark") || content.contains("<DT><A HREF") {
        import_html_bookmarks(&content)
    } else if content.starts_with('{') {
        import_json_bookmarks(&content)
    } else {
        Err("Unknown file format. Supported: Netscape HTML, Chromium JSON".to_string())
    }
}

fn import_chromium() -> Result<usize, String> {
    let paths = vec![
        dirs::config_dir().unwrap().join("chromium/Default/Bookmarks"),
        dirs::config_dir().unwrap().join("google-chrome/Default/Bookmarks"),
        dirs::config_dir().unwrap().join("google-chrome-stable/Default/Bookmarks"),
    ];

    let bookmark_path = paths
        .iter()
        .find(|p| p.exists())
        .ok_or("Chromium bookmark file not found. Checked:\n  ~/.config/chromium/Default/Bookmarks\n  ~/.config/google-chrome/Default/Bookmarks")?;

    let content = fs::read_to_string(bookmark_path)
        .map_err(|e| format!("Cannot read {:?}: {}", bookmark_path, e))?;

    import_json_bookmarks(&content)
}

fn import_json_bookmarks(content: &str) -> Result<usize, String> {
    let data: Value = serde_json::from_str(content)
        .map_err(|e| format!("JSON parse error: {}", e))?;

    let config = Config::load();
    let store_path = config.bookmarks_path();
    let mut store = storage::load_bookmarks(&store_path).unwrap_or_default();

    let mut count = 0;

    if let Some(roots) = data.get("roots") {
        for (_name, node) in roots.as_object().unwrap_or(&serde_json::Map::new()) {
            count += extract_chrome_bookmarks(node, "Imported", &mut store);
        }
    }

    storage::save_bookmarks(&store_path, &store)
        .map_err(|e| format!("Cannot save: {}", e))?;

    Ok(count)
}

fn extract_chrome_bookmarks(node: &Value, folder: &str, store: &mut BookmarkStore) -> usize {
    let mut count = 0;

    match node.get("type").and_then(|t| t.as_str()) {
        Some("url") => {
            let url = node.get("url").and_then(|u| u.as_str()).unwrap_or("");
            let name = node.get("name").and_then(|n| n.as_str()).unwrap_or("");

            if !url.is_empty() && !store.url_exists(url) {
                store.add(Bookmark::new(
                    name.to_string(),
                    url.to_string(),
                    folder.to_string(),
                ));
                count += 1;
            }
        }
        Some("folder") => {
            let folder_name = node.get("name").and_then(|n| n.as_str()).unwrap_or(folder);
            if let Some(children) = node.get("children").and_then(|c| c.as_array()) {
                for child in children {
                    count += extract_chrome_bookmarks(child, folder_name, store);
                }
            }
        }
        _ => {}
    }

    count
}

fn import_html_bookmarks(content: &str) -> Result<usize, String> {
    let config = Config::load();
    let store_path = config.bookmarks_path();
    let mut store = storage::load_bookmarks(&store_path).unwrap_or_default();
    let mut count = 0;
    let mut current_folder = "Imported".to_string();

    for line in content.lines() {
        let trimmed = line.trim();

        // Detect folder: <DT><H3 ...>Folder Name</H3>
        if trimmed.starts_with("<DT><H3") {
            if let Some(start) = trimmed.find('>') {
                let rest = &trimmed[start + 1..];
                if let Some(end) = rest.find("</H3>") {
                    current_folder = rest[..end].to_string();
                }
            }
        }

        // Detect bookmark: <DT><A HREF="url">name</A>
        if trimmed.starts_with("<DT><A") {
            if let (Some(href_start), Some(href_end)) = (trimmed.find("HREF=\""), trimmed.find("\">")) {
                let url = &trimmed[href_start + 6..href_end];
                let name_start = href_end + 2;
                let name_end = trimmed.find("</A>").unwrap_or(trimmed.len());
                let name = &trimmed[name_start..name_end];

                if !url.is_empty() && !store.url_exists(url) {
                    store.add(Bookmark::new(
                        name.to_string(),
                        url.to_string(),
                        current_folder.clone(),
                    ));
                    count += 1;
                }
            }
        }
    }

    storage::save_bookmarks(&store_path, &store)
        .map_err(|e| format!("Cannot save: {}", e))?;

    Ok(count)
}

pub fn export_bookmarks(format: &str, output: &str) -> Result<usize, String> {
    let config = Config::load();
    let store_path = config.bookmarks_path();
    let store = storage::load_bookmarks(&store_path)?;
    let count = store.bookmarks.len();

    match format.to_lowercase().as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&store)
                .map_err(|e| format!("Serialize error: {}", e))?;
            fs::write(output, json)
                .map_err(|e| format!("Write error: {}", e))?;
        }
        "html" => {
            let mut html = String::from("<!DOCTYPE NETSCAPE-Bookmark-file-1>\n");
            html.push_str("<META HTTP-EQUIV=\"Content-Type\" CONTENT=\"text/html; charset=UTF-8\">\n");
            html.push_str("<TITLE>Bookmarks</TITLE>\n");
            html.push_str("<H1>Bookmarks</H1>\n");
            html.push_str("<DL><p>\n");

            let mut folders: std::collections::HashMap<&str, Vec<&Bookmark>> = std::collections::HashMap::new();
            for b in &store.bookmarks {
                folders.entry(&b.folder).or_default().push(b);
            }

            for (folder, bookmarks) in &folders {
                html.push_str(&format!("    <DT><H3>{}</H3>\n    <DL><p>\n", folder));
                for b in bookmarks {
                    html.push_str(&format!(
                        "        <DT><A HREF=\"{}\">{}</A>\n",
                        b.url, b.name
                    ));
                }
                html.push_str("    </DL><p>\n");
            }

            html.push_str("</DL><p>\n");
            fs::write(output, html)
                .map_err(|e| format!("Write error: {}", e))?;
        }
        _ => return Err(format!("Unknown format: {}. Use: json, html", format)),
    }

    Ok(count)
}
