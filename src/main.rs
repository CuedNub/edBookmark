mod app;
mod bookmark;
mod config;
mod import_export;
mod keybinding;
mod launcher;
mod search;
mod storage;
mod theme;
mod ui;

use app::App;
use clap::Parser;
use std::process;

#[derive(Parser, Debug)]
#[command(name = "edbookmark", version, about = "TUI Bookmark Manager")]
struct Cli {
    /// Import bookmarks from browser: chromium, firefox
    #[arg(long, value_name = "BROWSER")]
    import: Option<String>,

    /// Import bookmarks from file path
    #[arg(long, value_name = "FILE")]
    import_file: Option<String>,

    /// Export bookmarks to format: json, html
    #[arg(long, value_name = "FORMAT")]
    export: Option<String>,

    /// Output file for export
    #[arg(short, long, value_name = "FILE")]
    output: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    // Handle import
    if let Some(browser) = &cli.import {
        match import_export::import_from_browser(browser) {
            Ok(count) => {
                println!("✓ Imported {} bookmarks from {}", count, browser);
                process::exit(0);
            }
            Err(e) => {
                eprintln!("✗ Import error: {}", e);
                process::exit(1);
            }
        }
    }

    // Handle import from file
    if let Some(file) = &cli.import_file {
        match import_export::import_from_file(file) {
            Ok(count) => {
                println!("✓ Imported {} bookmarks from {}", count, file);
                process::exit(0);
            }
            Err(e) => {
                eprintln!("✗ Import error: {}", e);
                process::exit(1);
            }
        }
    }

    // Handle export
    if let Some(format) = &cli.export {
        let output = cli.output.unwrap_or_else(|| {
            match format.as_str() {
                "json" => "bookmarks_export.json".to_string(),
                "html" => "bookmarks_export.html".to_string(),
                _ => "bookmarks_export.txt".to_string(),
            }
        });
        match import_export::export_bookmarks(format, &output) {
            Ok(count) => {
                println!("✓ Exported {} bookmarks to {}", count, output);
                process::exit(0);
            }
            Err(e) => {
                eprintln!("✗ Export error: {}", e);
                process::exit(1);
            }
        }
    }

    // Launch TUI
    if let Err(e) = App::run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
