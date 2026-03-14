import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { applyThemeVariables } from "./utils/theme";
import "./App.css";
import "./Settings.css";

interface FullConfig {
  hotkey: string;
  theme: string;
  startup: boolean;
}

const THEMES = [
  "amoled", "aura", "ayu", "carbonfox", "catppuccin-frappe", "catppuccin-macchiato", 
  "catppuccin", "cobalt2", "cursor", "dracula", "everforest", "flexoki", "github", 
  "gruvbox", "kanagawa", "lucent-orng", "material", "matrix", "mercury", "monokai", 
  "nightowl", "nord", "oc-2", "one-dark", "onedarkpro", "orng", "osaka-jade", 
  "palenight", "rosepine", "shadesofpurple", "solarized", "synthwave84", 
  "tokyonight", "vercel", "vesper", "zenburn"
];

export default function Settings() {
  const [config, setConfig] = useState<FullConfig | null>(null);
  const [tempTheme, setTempTheme] = useState<string>("system");
  const [isRecording, setIsRecording] = useState(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  useEffect(() => {
    invoke<FullConfig>("get_full_config").then(cfg => {
      setConfig(cfg);
      setTempTheme(cfg.theme);
    }).catch(console.error);
  }, []);

  useEffect(() => {
    applyThemeVariables(tempTheme);
  }, [tempTheme]);

  const handleSave = async () => {
    if (!config) return;
    try {
      await invoke("save_full_config", { config: { ...config, theme: tempTheme } });
      await invoke("close_settings_window");
    } catch (e) {
      setErrorMsg(String(e));
    }
  };

  const handleCancel = async () => {
    await invoke("close_settings_window");
  };

  const handleDrag = async () => {
    await invoke("start_settings_window_drag");
  };

  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    if (!isRecording) return;
    e.preventDefault();
    e.stopPropagation();

    // Ignore naked modifiers
    if (["Control", "Alt", "Shift", "Meta"].includes(e.key)) return;

    let keys = [];
    if (e.ctrlKey) keys.push("Ctrl");
    if (e.altKey) keys.push("Alt");
    if (e.shiftKey) keys.push("Shift");
    if (e.metaKey) keys.push("Super");

    if (keys.length === 0) {
      setErrorMsg("Hotkey must include at least one modifier (Ctrl, Alt, or Windows/Super).");
      return;
    }

    let keyName = e.key.toUpperCase();
    if (e.code.startsWith("Key")) keyName = e.code.replace("Key", "");
    else if (e.code.startsWith("Digit")) keyName = e.code.replace("Digit", "");
    else if (e.key === " ") keyName = "Space";
    
    keys.push(keyName);
    
    const newHotkey = keys.join("+");
    setConfig(prev => prev ? { ...prev, hotkey: newHotkey } : null);
    setIsRecording(false);
    setErrorMsg(null);
  }, [isRecording]);

  useEffect(() => {
    if (isRecording) {
      window.addEventListener("keydown", handleKeyDown);
      return () => window.removeEventListener("keydown", handleKeyDown);
    }
  }, [isRecording, handleKeyDown]);

  if (!config) return <div className="settings-container loading">Loading...</div>;

  return (
    <div className="settings-container">
      <div className="settings-header" onMouseDown={handleDrag}>
        <h2>Settings</h2>
      </div>
      
      <div className="settings-content">
        {errorMsg && <div className="settings-error">{errorMsg}</div>}
        
        <div className="settings-group">
          <label>Theme</label>
          <select 
            className="theme-select"
            value={tempTheme}
            onChange={(e) => setTempTheme(e.target.value)}
          >
            <option value="system">System Default</option>
            <option value="light">Default Light</option>
            <option value="dark">Default Dark</option>
            {THEMES.map(t => (
              <option key={t} value={t}>{t}</option>
            ))}
          </select>
        </div>

        <div className="settings-group">
          <label>Global Hotkey</label>
          <div className="hotkey-input-wrapper">
            <input 
              type="text" 
              readOnly 
              value={isRecording ? "Recording... (Press shortcut)" : config.hotkey}
              className={`hotkey-input ${isRecording ? "recording" : ""}`}
              onClick={() => { setIsRecording(true); setErrorMsg(null); }}
            />
            <button className="hotkey-btn" onClick={() => setIsRecording(true)}>
              Record
            </button>
          </div>
          <small className="settings-hint">Requires Ctrl, Alt, or Windows (Super) key.</small>
        </div>

        <div className="settings-group checkbox-group">
          <label className="checkbox-label">
            <input 
              type="checkbox" 
              checked={config.startup}
              onChange={(e) => setConfig({ ...config, startup: e.target.checked })}
            />
            Start Windows Search Tool automatically on login
          </label>
        </div>

      </div>

      <div className="settings-footer">
        <button className="btn-cancel" onClick={handleCancel}>Cancel</button>
        <button className="btn-save" onClick={handleSave}>Save</button>
      </div>
    </div>
  );
}