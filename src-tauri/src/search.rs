use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use crate::indexer::{get_index};
use meval::eval_str;
use tauri::Manager;

#[derive(serde::Serialize)]
pub struct SearchResult {
    pub name: String,
    pub path: String,
    pub kind: String,
    pub score: i64,
    pub icon_base64: Option<String>,
}

#[tauri::command]
pub fn search_items(app: tauri::AppHandle, query: String) -> Vec<SearchResult> {
    let start = std::time::Instant::now();
    if query.trim().is_empty() {
        return vec![];
    }

    let matcher = SkimMatcherV2::default();
    let q = query.to_lowercase();

    let index = get_index().lock().unwrap();
    let mut results: Vec<SearchResult> = index
        .iter()
        .filter_map(|entry| {
            let score = matcher.fuzzy_match(&entry.name.to_lowercase(), &q)?;
            Some(SearchResult {
                name: entry.name.clone(),
                path: entry.path.clone(),
                kind: entry.kind.clone(),
                score,
                icon_base64: entry.icon_base64.clone(),
            })
        })
        .collect();

    results.sort_by(|a, b| b.score.cmp(&a.score).then(a.name.cmp(&b.name)));
    results.truncate(12);

    if q.contains("config") || q.contains("settings") || q.contains("hotkey") {
        if let Ok(config_dir) = app.path().app_config_dir() {
            results.insert(0, SearchResult {
                name: "Open Settings / Config Folder".to_string(),
                path: config_dir.to_string_lossy().to_string(),
                kind: "setting".to_string(),
                score: i64::MAX - 1,
                icon_base64: None,
            });
        }
    }

    if q.starts_with("kill ") {
        let process_name = q.strip_prefix("kill ").unwrap().trim();
        if !process_name.is_empty() {
            results.insert(0, SearchResult {
                name: format!("Kill Process: {}.exe", process_name),
                path: process_name.to_string(),
                kind: "kill".to_string(),
                score: i64::MAX,
                icon_base64: None,
            });
        }
    }

    if let Ok(res) = eval_str(&query) {
        if query.chars().any(|c| "+-*/()^".contains(c)) {
            results.insert(0, SearchResult {
                name: format!("= {}", res),
                path: res.to_string(),
                kind: "math".to_string(),
                score: i64::MAX,
                icon_base64: None,
            });
        }
    }

    println!("PERF: Search for '{}' took {:?}", query, start.elapsed());
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexer::build_index;
    use crate::launcher::launch_item;

    #[test]
    fn test_search_finds_settings() {
        build_index();
        let results = search_items("task".to_string());
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.name == "Task Manager"));
    }

    #[test]
    fn test_launcher_handles_urls() {
        // We can't easily test the actual spawning without side effects, 
        // but we can ensure the logic doesn't panic for these strings.
        let _ = launch_item("https://google.com".to_string());
        let _ = launch_item("ms-settings:display".to_string());
    }
}
