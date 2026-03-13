use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use crate::indexer::{get_index};
use meval::eval_str;
use tauri::Manager;
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible, GetWindowThreadProcessId};
use windows::Win32::Foundation::{HWND, LPARAM, BOOL};

struct WindowInfo {
    title: String,
    pid: u32,
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    if IsWindowVisible(hwnd).as_bool() {
        let length = GetWindowTextLengthW(hwnd);
        if length > 0 {
            let mut buffer = vec![0u16; (length + 1) as usize];
            GetWindowTextW(hwnd, &mut buffer);
            if let Ok(title) = String::from_utf16(&buffer[..length as usize]) {
                let title = title.trim_end_matches('\0').to_string();
                if !title.is_empty() && title != "Program Manager" && title != "Settings" {
                    let mut pid = 0;
                    GetWindowThreadProcessId(hwnd, Some(&mut pid));
                    let windows = &mut *(lparam.0 as *mut Vec<WindowInfo>);
                    windows.push(WindowInfo { title, pid });
                }
            }
        }
    }
    BOOL(1)
}

fn get_open_windows() -> Vec<WindowInfo> {
    let mut windows: Vec<WindowInfo> = Vec::new();
    unsafe {
        let _ = EnumWindows(Some(enum_windows_proc), LPARAM(&mut windows as *mut _ as isize));
    }
    windows
}

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
        let search_term = q.strip_prefix("kill ").unwrap().trim();
        if !search_term.is_empty() {
            let open_windows = get_open_windows();
            let mut kill_results = vec![];
            for win in open_windows {
                if let Some(score) = matcher.fuzzy_match(&win.title.to_lowercase(), search_term) {
                    kill_results.push(SearchResult {
                        name: format!("Kill Application: {}", win.title),
                        path: win.pid.to_string(),
                        kind: "kill".to_string(),
                        score: i64::MAX - 100 + score,
                        icon_base64: None,
                    });
                }
            }
            kill_results.sort_by(|a, b| b.score.cmp(&a.score));
            // Keep top 5 window matches to prevent clutter
            kill_results.truncate(5);
            for result in kill_results.into_iter().rev() {
                results.insert(0, result);
            }
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
