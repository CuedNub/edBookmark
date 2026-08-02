#![allow(dead_code)]
use crate::bookmark::Bookmark;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

pub struct SearchEngine {
    matcher: SkimMatcherV2,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
        }
    }

    /// Filter bookmarks berdasarkan query.
    /// Query "pano rama" → split menjadi ["pano", "rama"]
    /// Semua kata harus match (AND logic)
    pub fn filter<'a>(
        &self,
        bookmarks: &'a [Bookmark],
        query: &str,
    ) -> Vec<(usize, i64, &'a Bookmark)> {
        let query = query.trim();
        if query.is_empty() {
            return bookmarks
                .iter()
                .enumerate()
                .map(|(i, b)| (i, 0, b))
                .collect();
        }

        let words: Vec<&str> = query.split_whitespace().collect();

        let mut results: Vec<(usize, i64, &Bookmark)> = Vec::new();

        for (idx, bookmark) in bookmarks.iter().enumerate() {
            let text = bookmark.searchable_text();
            let mut all_match = true;
            let mut total_score: i64 = 0;

            for word in &words {
                if let Some(score) = self.matcher.fuzzy_match(&text, word) {
                    total_score += score;
                } else {
                    all_match = false;
                    break;
                }
            }

            if all_match {
                results.push((idx, total_score, bookmark));
            }
        }

        // Sort by score descending
        results.sort_by(|a, b| b.1.cmp(&a.1));
        results
    }

    /// Fuzzy match untuk folder autocomplete
    pub fn filter_folders<'a>(&self, folders: &'a [String], query: &str) -> Vec<&'a String> {
        if query.is_empty() {
            return folders.iter().collect();
        }
        let mut scored: Vec<(i64, &String)> = folders
            .iter()
            .filter_map(|f| self.matcher.fuzzy_match(f, query).map(|score| (score, f)))
            .collect();
        scored.sort_by(|a, b| b.0.cmp(&a.0));
        scored.into_iter().map(|(_, f)| f).collect()
    }
}
