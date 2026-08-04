mod app;
mod bookmark;
mod config;
mod import_export;
mod history;
mod keybinding;
mod launcher;
mod search;
mod storage;
mod theme;
mod ui;

use app::App;
use std::process;

fn main() {
    if let Err(e) = App::run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
