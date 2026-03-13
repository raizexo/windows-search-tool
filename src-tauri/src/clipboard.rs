use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};
use std::thread;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, RegisterClassW,
    CW_USEDEFAULT, MSG, WNDCLASSW, WS_OVERLAPPEDWINDOW,
    GetMessageW, DispatchMessageW, TranslateMessage,
    CS_HREDRAW, CS_VREDRAW, WM_CLIPBOARDUPDATE,
};
use windows::Win32::System::DataExchange::AddClipboardFormatListener;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::core::PCWSTR;
use tauri_plugin_clipboard_manager::ClipboardExt;

pub static CLIPBOARD_HISTORY: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();

fn get_history() -> &'static Mutex<VecDeque<String>> {
    CLIPBOARD_HISTORY.get_or_init(|| Mutex::new(VecDeque::with_capacity(10)))
}

unsafe extern "system" fn window_proc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    if msg == WM_CLIPBOARDUPDATE {
        // The handle is passed via the window's user data or we can just use a global
        // But since we are using tauri-plugin-clipboard-manager, we need an AppHandle.
        // For simplicity in a background thread, we'll use arboard or windows crate directly
        // to avoid passing AppHandle through the C-style window proc.
        
        thread::spawn(|| {
            // Slight delay to allow the app that wrote to clipboard to finish
            thread::sleep(std::time::Duration::from_millis(100));
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                if let Ok(text) = clipboard.get_text() {
                    let text = text.trim().to_string();
                    if !text.is_empty() {
                        let mut history = get_history().lock().unwrap();
                        if history.front() != Some(&text) {
                            history.retain(|x| x != &text);
                            history.push_front(text);
                            if history.len() > 10 {
                                history.pop_back();
                            }
                        }
                    }
                }
            }
        });
        return windows::Win32::Foundation::LRESULT(0);
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

pub fn init_clipboard_listener() {
    thread::spawn(|| unsafe {
        let h_instance = GetModuleHandleW(None).unwrap();
        let class_name = windows::core::w!("WinSearchClipboardListener");

        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: h_instance.into(),
            hIcon: Default::default(),
            hCursor: Default::default(),
            hbrBackground: Default::default(),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: class_name,
        };

        if RegisterClassW(&wc) == 0 {
            // Already registered or failed
        }

        let hwnd_result = CreateWindowExW(
            windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE::default(),
            class_name,
            windows::core::w!("ClipboardListener"),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            h_instance,
            None,
        );

        if let Ok(hwnd) = hwnd_result {
            let _ = AddClipboardFormatListener(hwnd);
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).into() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    });
}

#[tauri::command]
pub fn get_clipboard_history() -> Vec<crate::search::SearchResult> {
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        if let Ok(text) = clipboard.get_text() {
            let text = text.trim().to_string();
            if !text.is_empty() {
                let mut history = get_history().lock().unwrap();
                if history.front() != Some(&text) {
                    history.retain(|x| x != &text);
                    history.push_front(text);
                    if history.len() > 10 {
                        history.pop_back();
                    }
                }
            }
        }
    }

    let history = get_history().lock().unwrap();
    history.iter().map(|text| {
        let display_name = if text.len() > 60 {
            format!("{}...", &text[..57])
        } else {
            text.clone()
        };
        crate::search::SearchResult {
            name: display_name.replace('\n', " ").replace('\r', ""),
            path: text.clone(),
            kind: "clipboard".to_string(),
            score: 0,
            icon_base64: None,
        }
    }).collect()
}
