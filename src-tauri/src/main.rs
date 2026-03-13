#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod indexer;
mod search;
mod launcher;
mod icons;
mod clipboard;

use tauri::{Manager, WindowEvent};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use std::str::FromStr;

#[derive(serde::Deserialize, serde::Serialize)]
struct AppConfig {
    hotkey: Option<String>,
}

fn get_hotkey(app: &tauri::AppHandle) -> String {
    let default_hotkey = "Ctrl+Space".to_string();
    if let Ok(config_dir) = app.path().app_config_dir() {
        let config_path = config_dir.join("config.json");
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                    if let Some(hotkey) = config.hotkey {
                        return hotkey;
                    }
                }
            }
        } else {
            let _ = std::fs::create_dir_all(&config_dir);
            let default_config = AppConfig { hotkey: Some(default_hotkey.clone()) };
            if let Ok(content) = serde_json::to_string_pretty(&default_config) {
                let _ = std::fs::write(&config_path, content);
            }
        }
    }
    default_hotkey
}

#[tauri::command]
fn get_hotkey_string(app: tauri::AppHandle) -> String {
    get_hotkey(&app)
}

#[tauri::command]
fn hide_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

#[tauri::command]
fn copy_to_clipboard(app: tauri::AppHandle, text: String) {
    use tauri_plugin_clipboard_manager::ClipboardExt;
    let _ = app.clipboard().write_text(text);
}

fn register_autostart() {
    // Only register in release mode or if explicitly desired
    if let Ok(exe) = std::env::current_exe() {
        if let Ok(key) = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
            .open_subkey_with_flags(
                r"Software\Microsoft\Windows\CurrentVersion\Run",
                winreg::enums::KEY_WRITE,
            ) 
        {
            let _ = key.set_value("windows-search-tool", &exe.to_string_lossy().as_ref());
        }
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let handle = app.handle().clone();

            // Register for auto-start
            register_autostart();

            // Init clipboard listener
            clipboard::init_clipboard_listener();

            // Pre-build index on startup (background thread)
            std::thread::spawn(|| {
                indexer::build_index();
            });

            // System Tray
            let show_item = MenuItem::with_id(app, "show", "Show windows-search-tool (Ctrl+Space)", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&tray_menu)
                .show_menu_on_left_click(true)
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "quit" => {
                            app.exit(0);
                        }
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            // Register Ctrl+Space or custom global hotkey
            let hotkey_str = get_hotkey(app.handle());
            let shortcut = match Shortcut::from_str(&hotkey_str) {
                Ok(s) => s,
                Err(_) => Shortcut::new(Some(Modifiers::CONTROL), Code::Space),
            };
            app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    if let Some(window) = handle.get_webview_window("main") {
                        if window.is_visible().unwrap_or(false) {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
            })?;

            Ok(())
        })
        .on_window_event(|window, event| {
            // Hide window on close instead of quitting
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
            // Hide when focus is lost
            if let WindowEvent::Focused(false) = event {
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            search::search_items,
            launcher::launch_item,
            launcher::open_path,
            launcher::kill_process,
            clipboard::get_clipboard_history,
            get_hotkey_string,
            hide_window,
            copy_to_clipboard,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
