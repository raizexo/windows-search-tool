#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod indexer;
mod search;
mod launcher;
mod icons;
mod clipboard;

use tauri::{Manager, WindowEvent, Emitter};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use std::str::FromStr;
use std::process::Command;

#[derive(serde::Deserialize, serde::Serialize, Default, Clone)]
struct AppConfig {
    hotkey: Option<String>,
    theme: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct FullConfig {
    pub hotkey: String,
    pub theme: String,
    pub startup: bool,
}

fn get_config_from_disk(app: &tauri::AppHandle) -> AppConfig {
    if let Ok(config_dir) = app.path().app_config_dir() {
        let config_path = config_dir.join("config.json");
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                    return config;
                }
            }
        }
    }
    AppConfig::default()
}

fn check_autostart() -> bool {
    if let Ok(key) = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
        .open_subkey_with_flags(
            r"Software\Microsoft\Windows\CurrentVersion\Run",
            winreg::enums::KEY_READ,
        ) 
    {
        let val: Result<String, _> = key.get_value("windows-search-tool");
        return val.is_ok();
    }
    false
}

fn create_vbs_launcher(exe_path: &std::path::Path, vbs_path: &std::path::Path) -> Result<(), String> {
    let exe_path_str = exe_path.to_string_lossy().replace("\\", "\\\\");
    let vbs_content = format!(
        r#"Set WshShell = CreateObject("WScript.Shell")
WshShell.Run "{}", 0, False
Set WshShell = Nothing"#,
        exe_path_str
    );
    std::fs::write(vbs_path, vbs_content)
        .map_err(|e| format!("Failed to create VBS launcher: {}", e))
}

fn get_system_uptime_seconds() -> u64 {
    use windows::Win32::System::SystemInformation::GetTickCount64;
    unsafe {
        GetTickCount64() / 1000
    }
}

fn is_webview2_available() -> bool {
    use winreg::enums::*;
    
    let check_keys = [
        r"SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}",
        r"SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}",
    ];
    
    for key_path in &check_keys {
        if let Ok(key) = winreg::RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey_with_flags(key_path, KEY_READ) {
            if let Ok(_) = key.get_value::<String, _>("pv") {
                return true;
            }
        }
        if let Ok(key) = winreg::RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(key_path, KEY_READ) {
            if let Ok(_) = key.get_value::<String, _>("pv") {
                return true;
            }
        }
    }
    
    // Also check for Evergreen Standalone
    if let Ok(key) = winreg::RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey_with_flags(
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Microsoft Edge WebView2 Runtime",
        KEY_READ
    ) {
        if let Ok(_) = key.get_value::<String, _>("DisplayVersion") {
            return true;
        }
    }
    
    false
}

fn set_autostart(enable: bool) {
    if let Ok(exe) = std::env::current_exe() {
        let config_dir = exe.parent().unwrap_or(std::path::Path::new("."));
        let vbs_path = config_dir.join("windows-search-tool-launcher.vbs");
        
        if let Ok(key) = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
            .open_subkey_with_flags(
                r"Software\Microsoft\Windows\CurrentVersion\Run",
                winreg::enums::KEY_WRITE | winreg::enums::KEY_READ,
            ) 
        {
            if enable {
                // Create VBS wrapper to hide console window
                if let Err(e) = create_vbs_launcher(&exe, &vbs_path) {
                    eprintln!("Failed to create VBS launcher: {}", e);
                    // Fallback to direct exe path
                    let _ = key.set_value("windows-search-tool", &exe.to_string_lossy().as_ref());
                } else {
                    let _ = key.set_value("windows-search-tool", &vbs_path.to_string_lossy().as_ref());
                }
            } else {
                let _ = key.delete_value("windows-search-tool");
                // Clean up VBS file if it exists
                let _ = std::fs::remove_file(&vbs_path);
            }
        }
    }
}

#[tauri::command]
fn get_full_config(app: tauri::AppHandle) -> FullConfig {
    let cfg = get_config_from_disk(&app);
    FullConfig {
        hotkey: cfg.hotkey.unwrap_or_else(|| "Ctrl+Space".to_string()),
        theme: cfg.theme.unwrap_or_else(|| "system".to_string()),
        startup: check_autostart(),
    }
}

#[tauri::command]
fn save_full_config(app: tauri::AppHandle, config: FullConfig) -> Result<(), String> {
    let old_cfg = get_full_config(app.clone());
    
    // Update hotkey
    if old_cfg.hotkey != config.hotkey {
        let manager = app.global_shortcut();
        if let Ok(old_shortcut) = Shortcut::from_str(&old_cfg.hotkey) {
            let _ = manager.unregister(old_shortcut);
        }
        let new_shortcut = Shortcut::from_str(&config.hotkey).map_err(|e| e.to_string())?;
        
        let handle = app.clone();
        manager.on_shortcut(new_shortcut, move |_app, _shortcut, event| {
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
        }).map_err(|e| e.to_string())?;
    }

    // Update autostart
    if old_cfg.startup != config.startup {
        set_autostart(config.startup);
    }

    // Save to disk
    if let Ok(config_dir) = app.path().app_config_dir() {
        let _ = std::fs::create_dir_all(&config_dir);
        let config_path = config_dir.join("config.json");
        let new_app_config = AppConfig {
            hotkey: Some(config.hotkey.clone()),
            theme: Some(config.theme.clone()),
        };
        if let Ok(content) = serde_json::to_string_pretty(&new_app_config) {
            let _ = std::fs::write(&config_path, content);
        }
    }

    // Emit event
    let _ = app.emit("config-changed", config);

    Ok(())
}

#[tauri::command]
fn get_hotkey_string(app: tauri::AppHandle) -> String {
    get_full_config(app).hotkey
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

#[tauri::command]
async fn open_settings_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.set_focus();
    } else {
        tauri::WebviewWindowBuilder::new(
            &app,
            "settings",
            tauri::WebviewUrl::App("/?settings=true".into())
        )
        .title("Windows Search Tool Settings")
        .inner_size(500.0, 480.0)
        .center()
        .always_on_top(true)
        .decorations(false)
        .transparent(true)
        .resizable(false)
        .build()
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn close_settings_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.close();
    }
}

#[tauri::command]
fn start_settings_window_drag(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.start_dragging();
    }
}

fn check_and_wait_for_webview2() {
    // Check system uptime - if system just booted (< 60 seconds), wait for WebView2
    let uptime = get_system_uptime_seconds();
    if uptime < 60 {
        println!("System recently booted ({}s). Waiting for WebView2 initialization...", uptime);
        
        // Wait up to 10 seconds for WebView2 to be ready
        for i in 0..20 {
            if is_webview2_available() {
                println!("WebView2 is available after {}s", i * 500);
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        
        // Additional safety delay
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
    
    if !is_webview2_available() {
        eprintln!("WARNING: WebView2 Runtime not detected!");
        // Try to install WebView2
        if let Err(e) = Command::new("cmd")
            .args(["/c", "start", "", "https://developer.microsoft.com/en-us/microsoft-edge/webview2/"])
            .spawn() 
        {
            eprintln!("Failed to open WebView2 download page: {}", e);
        }
    }
}

fn main() {
    // Check WebView2 and add startup delay before initializing Tauri
    check_and_wait_for_webview2();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let handle = app.handle().clone();

            let current_config = get_full_config(handle.clone());

            // Init clipboard listener
            clipboard::init_clipboard_listener();

            // Pre-build index on startup (background thread)
            std::thread::spawn(|| {
                indexer::build_index();
            });

            // System Tray
            let show_item = MenuItem::with_id(app, "show", "Show windows-search-tool", true, None::<&str>)?;
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

            // Register global hotkey
            let shortcut = match Shortcut::from_str(&current_config.hotkey) {
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
                if window.label() == "main" {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
            // Hide when focus is lost (only for main)
            if let WindowEvent::Focused(false) = event {
                if window.label() == "main" {
                    let _ = window.hide();
                }
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
            get_full_config,
            save_full_config,
            open_settings_window,
            close_settings_window,
            start_settings_window_drag,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
