use std::process::Command;

#[tauri::command]
pub fn launch_item(path: String) -> Result<(), String> {
    // Web URLs, ms-settings: URIs, .msc, and common control panel apps need explorer/cmd execution
    if path.starts_with("http://") || path.starts_with("https://") || 
       path.starts_with("ms-settings:") || path.ends_with(".msc") || 
       path == "taskmgr" || path == "control" || path == "regedit" {
        
        Command::new("cmd")
            .args(["/c", "start", "", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    } else if std::path::Path::new(&path).is_dir() {
        Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    } else if path.ends_with(".lnk") {
        // .lnk shortcuts need shell execution
        Command::new("cmd")
            .args(["/c", "start", "", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    } else {
        Command::new(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn open_path(path: String) -> Result<(), String> {
    Command::new("explorer")
        .arg(&path)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn kill_process(name: String) -> Result<(), String> {
    Command::new("taskkill")
        .args(["/F", "/PID", &name])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}
