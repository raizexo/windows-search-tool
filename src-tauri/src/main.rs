#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod indexer;
mod search;
mod launcher;
mod icons;

use tauri::{Manager, WindowEvent};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;

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
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let handle = app.handle().clone();

            // Register for auto-start
            register_autostart();

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

            // Register Ctrl+Space global hotkey
            let shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::Space);
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
