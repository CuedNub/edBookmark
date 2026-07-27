#![allow(dead_code)]
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: String,
    pub name: String,
    pub url: String,
    pub folder: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Bookmark {
    pub fn new(name: String, url: String, folder: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            url,
            folder,
            created_at: now,
            updated_at: now,
        }
    }

    /// Gabungkan semua field untuk pencarian
    pub fn searchable_text(&self) -> String {
        format!("{} {} {}", self.name, self.url, self.folder)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkStore {
    pub version: String,
    pub bookmarks: Vec<Bookmark>,
}

impl Default for BookmarkStore {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            bookmarks: Vec::new(),
        }
    }
}

impl BookmarkStore {
    pub fn add(&mut self, bookmark: Bookmark) {
        self.bookmarks.push(bookmark);
    }

    pub fn remove(&mut self, id: &str) {
        self.bookmarks.retain(|b| b.id != id);
    }

    pub fn remove_many(&mut self, ids: &[String]) {
        self.bookmarks.retain(|b| !ids.contains(&b.id));
    }

    pub fn update(&mut self, id: &str, name: String, url: String, folder: String) {
        if let Some(b) = self.bookmarks.iter_mut().find(|b| b.id == id) {
            b.name = name;
            b.url = url;
            b.folder = folder;
            b.updated_at = Utc::now();
        }
    }

    pub fn find_by_id(&self, id: &str) -> Option<&Bookmark> {
        self.bookmarks.iter().find(|b| b.id == id)
    }

    /// Daftar folder unik yang pernah dipakai
    pub fn folders(&self) -> Vec<String> {
        let mut folders: Vec<String> = self
            .bookmarks
            .iter()
            .map(|b| b.folder.clone())
            .collect();
        folders.sort();
        folders.dedup();
        folders
    }

    /// Cek apakah URL sudah ada (untuk deduplikasi saat import)
    pub fn url_exists(&self, url: &str) -> bool {
        self.bookmarks.iter().any(|b| b.url == url)
    }
}
