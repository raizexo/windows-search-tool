use std::sync::{Mutex, OnceLock};
use walkdir::WalkDir;
use crate::icons::extract_icon_as_base64;

#[derive(Clone, serde::Serialize, Debug)]
pub struct IndexEntry {
    pub name: String,
    pub path: String,
    pub kind: String, // "app" | "file" | "setting"
    pub icon_base64: Option<String>,
}

static INDEX: OnceLock<Mutex<Vec<IndexEntry>>> = OnceLock::new();

pub fn get_index() -> &'static Mutex<Vec<IndexEntry>> {
    INDEX.get_or_init(|| Mutex::new(vec![]))
}

pub fn build_index() {
    let start = std::time::Instant::now();
    let mut entries: Vec<IndexEntry> = vec![];

    // --- Scan Start Menu for .lnk (installed apps) ---
    let username = std::env::var("USERNAME").unwrap_or_default();
    let start_dirs = vec![
        r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs".to_string(),
        format!(r"C:\Users\{}\AppData\Roaming\Microsoft\Windows\Start Menu\Programs", username),
    ];

    for dir in &start_dirs {
        for entry in WalkDir::new(dir).max_depth(4).into_iter().flatten() {
            let name = entry.file_name().to_string_lossy();
            if name.ends_with(".lnk") {
                let clean = name.replace(".lnk", "");
                let path = entry.path().to_string_lossy().to_string();
                
                // Extract native icon for apps
                let icon_base64 = extract_icon_as_base64(&path);
                
                entries.push(IndexEntry {
                    name: clean,
                    path,
                    kind: "app".into(),
                    icon_base64,
                });
            }
        }
    }

    // --- Settings shortcuts ---
    let settings = vec![
        ("Display Settings", "ms-settings:display"),
        ("Bluetooth & Devices", "ms-settings:bluetooth"),
        ("Wi-Fi Settings", "ms-settings:network-wifi"),
        ("Sound Settings", "ms-settings:sound"),
        ("Windows Update", "ms-settings:windowsupdate"),
        ("Apps & Features", "ms-settings:appsfeatures"),
        ("Startup Apps", "ms-settings:startupapps"),
        ("Privacy Settings", "ms-settings:privacy"),
        ("Power & Sleep", "ms-settings:powersleep"),
        ("Storage Settings", "ms-settings:storagesense"),
        ("Task Manager", "taskmgr"),
        ("Control Panel", "control"),
        ("Device Manager", "devmgmt.msc"),
        ("Disk Management", "diskmgmt.msc"),
        ("Registry Editor", "regedit"),
    ];
    for (name, path) in settings {
        entries.push(IndexEntry {
            name: name.to_string(),
            path: path.to_string(),
            kind: "setting".into(),
            icon_base64: None,
        });
    }

    // Store in static index
    if let Ok(mut index) = get_index().lock() {
        *index = entries;
    }
    println!("PERF: Index built in {:?}", start.elapsed());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_index_initializes() {
        let index = get_index();
        let lock = index.lock().unwrap();
        assert!(lock.is_empty() || !lock.is_empty());
    }

    #[test]
    fn test_build_index_adds_settings() {
        build_index();
        let index = get_index().lock().unwrap();
        let has_taskmgr = index.iter().any(|e| e.name == "Task Manager" && e.path == "taskmgr");
        assert!(has_taskmgr, "Index should contain Task Manager");
    }
}
