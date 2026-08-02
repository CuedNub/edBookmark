use crate::bookmark::{Bookmark, BookmarkStore};
use crate::config::Config;
use crate::storage;
use serde_json::Value;
use std::fs;

// ─────────────────────────────────────────────────────────────────────────────
// IMPORT
// ─────────────────────────────────────────────────────────────────────────────

pub fn import_from_browser(browser: &str) -> Result<usize, String> {
    match browser.to_lowercase().as_str() {
        "chromium" | "chrome" => import_chromium(),
        "firefox" => Err(
            "Firefox import: export bookmarks to HTML first, then use --import-file".to_string(),
        ),
        _ => Err(format!(
            "Unknown browser: {}. Use: chromium, firefox",
            browser
        )),
    }
}

pub fn import_from_file(path: &str) -> Result<usize, String> {
    // Deteksi berdasarkan ekstensi terlebih dahulu
    let lower = path.to_lowercase();
    if lower.ends_with(".xlsx") {
        return import_from_xlsx(path);
    }

    let content =
        fs::read_to_string(path).map_err(|e| format!("Cannot read {}: {}", path, e))?;

    if content.contains("<!DOCTYPE NETSCAPE-Bookmark") || content.contains("<DT><A HREF") {
        import_html_bookmarks(&content)
    } else if content.starts_with('{') {
        import_json_bookmarks(&content)
    } else {
        Err("Unknown file format. Supported: Netscape HTML, Chromium JSON, XLSX".to_string())
    }
}

fn import_chromium() -> Result<usize, String> {
    let paths = vec![
        dirs::config_dir()
            .unwrap()
            .join("chromium/Default/Bookmarks"),
        dirs::config_dir()
            .unwrap()
            .join("google-chrome/Default/Bookmarks"),
        dirs::config_dir()
            .unwrap()
            .join("google-chrome-stable/Default/Bookmarks"),
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
    let data: Value =
        serde_json::from_str(content).map_err(|e| format!("JSON parse error: {}", e))?;

    let config = Config::load();
    let store_path = config.bookmarks_path();
    let mut store = storage::load_bookmarks(&store_path).unwrap_or_default();

    let mut count = 0;

    if let Some(roots) = data.get("roots") {
        for (_name, node) in roots.as_object().unwrap_or(&serde_json::Map::new()) {
            count += extract_chrome_bookmarks(node, "Imported", &mut store);
        }
    }

    storage::save_bookmarks(&store_path, &store).map_err(|e| format!("Cannot save: {}", e))?;

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

        if trimmed.starts_with("<DT><H3") {
            if let Some(start) = trimmed.find('>') {
                let rest = &trimmed[start + 1..];
                if let Some(end) = rest.find("</H3>") {
                    current_folder = rest[..end].to_string();
                }
            }
        }

        if trimmed.starts_with("<DT><A") {
            if let (Some(href_start), Some(href_end)) =
                (trimmed.find("HREF=\""), trimmed.find("\">"))
            {
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

    storage::save_bookmarks(&store_path, &store).map_err(|e| format!("Cannot save: {}", e))?;

    Ok(count)
}

// ─────────────────────────────────────────────────────────────────────────────
// IMPORT DARI XLSX
// ─────────────────────────────────────────────────────────────────────────────

/// Import bookmark dari file .xlsx.
///
/// Format kolom yang diharapkan (baris pertama = header, diabaikan):
///   A: Name  |  B: URL  |  C: Folder
///
/// Baris yang tidak memiliki URL akan dilewati.
/// URL yang sudah ada di store akan dilewati (deduplikasi).
pub fn import_from_xlsx(path: &str) -> Result<usize, String> {
    use calamine::{open_workbook, Reader, Xlsx};

    let mut workbook: Xlsx<_> =
        open_workbook(path).map_err(|e| format!("Cannot open XLSX '{}': {}", path, e))?;

    // Ambil sheet pertama
    let sheet_names = workbook.sheet_names().to_vec();
    let sheet_name = sheet_names
        .first()
        .ok_or_else(|| "XLSX file has no sheets".to_string())?
        .clone();

    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|e| format!("Cannot read sheet '{}': {}", sheet_name, e))?;

    let config = Config::load();
    let store_path = config.bookmarks_path();
    let mut store = storage::load_bookmarks(&store_path).unwrap_or_default();

    let mut count = 0;
    let mut is_first_row = true;

    for row in range.rows() {
        // Lewati baris header
        if is_first_row {
            is_first_row = false;
            // Deteksi apakah ini baris header (kolom pertama berisi "name" / "title")
            let first_cell = cell_to_string(row.first());
            let lower = first_cell.to_lowercase();
            if lower == "name" || lower == "title" || lower == "nama" {
                continue;
            }
        }

        let name = cell_to_string(row.get(0));
        let url = cell_to_string(row.get(1));
        let folder = {
            let f = cell_to_string(row.get(2));
            if f.trim().is_empty() {
                "Imported".to_string()
            } else {
                f
            }
        };

        // URL wajib ada
        if url.trim().is_empty() {
            continue;
        }

        // Deduplikasi
        if store.url_exists(&url) {
            continue;
        }

        let display_name = if name.trim().is_empty() {
            url.clone()
        } else {
            name
        };

        store.add(Bookmark::new(display_name, url, folder));
        count += 1;
    }

    storage::save_bookmarks(&store_path, &store).map_err(|e| format!("Cannot save: {}", e))?;

    Ok(count)
}

/// Konversi sel calamine ke String
fn cell_to_string(cell: Option<&calamine::Data>) -> String {
    match cell {
        Some(calamine::Data::String(s)) => s.trim().to_string(),
        Some(calamine::Data::Float(f)) => {
            // Angka: tampilkan tanpa desimal jika bulat
            if f.fract() == 0.0 {
                format!("{}", *f as i64)
            } else {
                format!("{}", f)
            }
        }
        Some(calamine::Data::Int(i)) => i.to_string(),
        Some(calamine::Data::Bool(b)) => b.to_string(),
        Some(calamine::Data::DateTimeIso(dt)) => dt.to_string(),
        Some(calamine::Data::DurationIso(dt)) => dt.to_string(),
        _ => String::new(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// EXPORT
// ─────────────────────────────────────────────────────────────────────────────

pub fn export_bookmarks(format: &str, output: &str) -> Result<usize, String> {
    let config = Config::load();
    let store_path = config.bookmarks_path();
    let store = storage::load_bookmarks(&store_path)?;
    let count = store.bookmarks.len();

    match format.to_lowercase().as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&store)
                .map_err(|e| format!("Serialize error: {}", e))?;
            fs::write(output, json).map_err(|e| format!("Write error: {}", e))?;
        }
        "html" => {
            let mut html = String::from("<!DOCTYPE NETSCAPE-Bookmark-file-1>\n");
            html.push_str(
                "<META HTTP-EQUIV=\"Content-Type\" CONTENT=\"text/html; charset=UTF-8\">\n",
            );
            html.push_str("<TITLE>Bookmarks</TITLE>\n");
            html.push_str("<H1>Bookmarks</H1>\n");
            html.push_str("<DL><p>\n");

            let mut folders: std::collections::HashMap<&str, Vec<&Bookmark>> =
                std::collections::HashMap::new();
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
            fs::write(output, html).map_err(|e| format!("Write error: {}", e))?;
        }
        "xlsx" => {
            export_to_xlsx(&store, output)?;
        }
        _ => {
            return Err(format!(
                "Unknown format: {}. Use: json, html, xlsx",
                format
            ))
        }
    }

    Ok(count)
}

// ─────────────────────────────────────────────────────────────────────────────
// EXPORT KE XLSX
// ─────────────────────────────────────────────────────────────────────────────

/// Export semua bookmark ke file .xlsx.
///
/// Struktur sheet:
///   Baris 1 : Header (Name, URL, Folder, Created At, Updated At)
///   Baris 2+ : Data bookmark
///
/// Fitur:
///   - Header di-bold dan diberi background warna
///   - Kolom URL menggunakan format hyperlink
///   - Lebar kolom otomatis (auto-fit estimasi)
///   - Sheet bernama "Bookmarks"
fn export_to_xlsx(store: &BookmarkStore, output: &str) -> Result<(), String> {
    use rust_xlsxwriter::{
        Color, Format, FormatAlign, FormatBorder, Url, Workbook, XlsxError,
    };

    let mut workbook = Workbook::new();
    let sheet = workbook
        .add_worksheet()
        .set_name("Bookmarks")
        .map_err(|e: XlsxError| format!("Cannot create sheet: {}", e))?;

    // ── Format header ──
    let header_fmt = Format::new()
        .set_bold()
        .set_font_color(Color::White)
        .set_background_color(Color::RGB(0x2D_4F_67))  // biru gelap (warna selected_bg theme)
        .set_border_bottom(FormatBorder::Thin)
        .set_align(FormatAlign::Center);

    // ── Format URL (biru, underline) ──
    let url_fmt = Format::new()
        .set_font_color(Color::RGB(0x39_BA_E6))  // warna url theme
        .set_underline(rust_xlsxwriter::FormatUnderline::Single);

    // ── Format data biasa ──
    let data_fmt = Format::new().set_align(FormatAlign::Left);

    // ── Format alternating row (sedikit abu) ──
    let alt_fmt = Format::new()
        .set_background_color(Color::RGB(0xF5_F5_F5))
        .set_align(FormatAlign::Left);

    // ── Header row ──
    let headers = ["Name", "URL", "Folder", "Created At", "Updated At"];
    for (col, &h) in headers.iter().enumerate() {
        sheet
            .write_with_format(0, col as u16, h, &header_fmt)
            .map_err(|e: XlsxError| format!("Write header error: {}", e))?;
    }

    // ── Data rows ──
    for (i, bm) in store.bookmarks.iter().enumerate() {
        let row = (i + 1) as u32;
        let fmt = if i % 2 == 0 { &data_fmt } else { &alt_fmt };

        // Kolom A: Name
        sheet
            .write_with_format(row, 0, &bm.name, fmt)
            .map_err(|e: XlsxError| format!("Write name error: {}", e))?;

        // Kolom B: URL sebagai teks biasa
        sheet
            .write_with_format(row, 1, &bm.url, fmt)
            .map_err(|e: XlsxError| format!("Write url error: {}", e))?;

        // Kolom C: Folder
        sheet
            .write_with_format(row, 2, &bm.folder, fmt)
            .map_err(|e: XlsxError| format!("Write folder error: {}", e))?;

        // Kolom D: Created At
        let created = bm.created_at.format("%Y-%m-%d %H:%M:%S").to_string();
        sheet
            .write_with_format(row, 3, created, fmt)
            .map_err(|e: XlsxError| format!("Write created_at error: {}", e))?;

        // Kolom E: Updated At
        let updated = bm.updated_at.format("%Y-%m-%d %H:%M:%S").to_string();
        sheet
            .write_with_format(row, 4, updated, fmt)
            .map_err(|e: XlsxError| format!("Write updated_at error: {}", e))?;
    }

    // ── Auto-fit lebar kolom (estimasi) ──
    // Name: max 40, URL: max 60, Folder: max 25, Dates: 20
    let col_widths: [(u16, f64); 5] = [(0, 40.0), (1, 60.0), (2, 25.0), (3, 20.0), (4, 20.0)];
    for (col, width) in col_widths {
        sheet
            .set_column_width(col, width)
            .map_err(|e: XlsxError| format!("Set column width error: {}", e))?;
    }

    // ── Freeze baris header ──
    sheet
        .set_freeze_panes(1, 0)
        .map_err(|e: XlsxError| format!("Freeze panes error: {}", e))?;

    // ── Simpan file ──
    workbook
        .save(output)
        .map_err(|e: XlsxError| format!("Cannot save XLSX '{}': {}", output, e))?;

    Ok(())
}
