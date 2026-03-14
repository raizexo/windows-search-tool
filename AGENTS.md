# Windows Search Tool - Agent Guide

## Project Overview

Windows Search Tool is a fast, minimal, opinionated search utility for Windows 11 built with **Tauri 2 + Rust + React**. It's designed as a single portable `.exe` (~6-10MB) that requires no administrator privileges.

**Key Features:**
- Global hotkey activation (customizable, default: `Ctrl+Space`)
- Fuzzy search for apps, settings, files
- Glass-morphism UI with 35+ themes
- Clipboard history (last 10 items)
- Calculator with instant evaluation
- Kill command to terminate processes
- Web search fallback
- System tray integration
- Auto-start on login (HKCU registry)

---

## Tech Stack

| Component | Technology |
|-----------|------------|
| Frontend | React 19 + TypeScript + Vite |
| Backend | Rust (Tauri 2) |
| UI Framework | Custom CSS (Fluent Design inspired) |
| Icons | @fluentui/react-icons |
| Search | fuzzy-matcher (Skim algorithm) |
| Window Management | Tauri frameless windows |

---

## File Structure

```
windows-search-tool/
├── src/                          # React Frontend
│   ├── App.tsx                   # Main search UI
│   ├── App.css                   # Main window styles
│   ├── Settings.tsx              # Settings window UI
│   ├── Settings.css              # Settings window styles
│   ├── main.tsx                  # Entry point (routes to App or Settings)
│   ├── utils/
│   │   └── theme.ts              # Theme loading and CSS variable application
│   └── themes/                   # 35+ JSON theme files
│       ├── dracula.json
│       ├── nord.json
│       └── ...
├── src-tauri/
│   ├── src/                      # Rust Backend
│   │   ├── main.rs               # Tauri setup, hotkeys, tray, config
│   │   ├── indexer.rs            # Start Menu scanner, settings shortcuts
│   │   ├── search.rs             # Fuzzy matching, math eval, kill command
│   │   ├── launcher.rs           # App launching, path opening
│   │   ├── icons.rs              # Native Windows icon extraction (HICON → PNG)
│   │   └── clipboard.rs          # Clipboard history listener
│   ├── Cargo.toml                # Rust dependencies
│   └── tauri.conf.json           # Tauri configuration
├── package.json                  # Node dependencies
├── vite.config.ts                # Vite configuration
└── tsconfig.json                 # TypeScript configuration
```

---

## Architecture Patterns

### Frontend-Backend Communication

All communication happens via Tauri commands using `invoke()`:

```typescript
// Frontend calling backend
const results = await invoke<SearchResult[]>("search_items", { query });
await invoke("launch_item", { path: item.path });
await invoke("hide_window");
```

```rust
// Backend exposing command
#[tauri::command]
pub fn search_items(app: tauri::AppHandle, query: String) -> Vec<SearchResult> {
    // ... implementation
}
```

### State Management

- **No Redux/Context**: Uses React `useState` and `useRef` for local state
- **Global State**: Rust backend owns the search index (static Mutex)
- **Config**: Persisted to `app_config_dir()/config.json`
- **Theme**: Applied via CSS variables to `:root`

### Window Management

Two windows managed by Tauri:
1. **Main window** (`label: "main"`): Search UI, frameless, always-on-top, hides on focus loss
2. **Settings window** (`label: "settings"`): Separate frameless window, draggable header

---

## Key Commands Reference

### Frontend Commands (Available in React)

| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `search_items` | `{ query: string }` | `SearchResult[]` | Fuzzy search index |
| `get_clipboard_history` | - | `SearchResult[]` | Get last 10 clipboard items |
| `launch_item` | `{ path: string }` | `void` | Open app/file/URL |
| `open_path` | `{ path: string }` | `void` | Open folder in Explorer |
| `kill_process` | `{ name: string }` | `void` | Kill process by PID |
| `hide_window` | - | `void` | Hide main window |
| `copy_to_clipboard` | `{ text: string }` | `void` | Copy text to clipboard |
| `get_full_config` | - | `FullConfig` | Get hotkey, theme, startup |
| `save_full_config` | `{ config: FullConfig }` | `void` | Save configuration |
| `open_settings_window` | - | `void` | Open settings window |
| `close_settings_window` | - | `void` | Close settings window |
| `start_settings_window_drag` | - | `void` | Start dragging settings window |

### Rust Module Functions

| Module | Key Functions |
|--------|---------------|
| `indexer` | `build_index()`, `get_index()` |
| `search` | `search_items()`, `get_open_windows()` |
| `launcher` | `launch_item()`, `open_path()`, `kill_process()` |
| `icons` | `extract_icon_as_base64()` |
| `clipboard` | `init_clipboard_listener()`, `get_clipboard_history()` |

---

## Data Types

### SearchResult

```typescript
interface SearchResult {
  name: string;           // Display name
  path: string;           // Launch path/URL/PID
  kind: "app" | "file" | "setting" | "web" | "math" | "clipboard" | "kill";
  score: number;          // Fuzzy match score
  icon_base64?: string;   // PNG icon (apps only)
}
```

### FullConfig

```typescript
interface FullConfig {
  hotkey: string;    // e.g., "Ctrl+Space"
  theme: string;     // e.g., "dracula", "system"
  startup: boolean;  // Auto-start on login
}
```

---

## Styling Conventions

### CSS Variables (Theme System)

Themes are applied via CSS custom properties on `:root`:

```css
:root {
  --bg-mica: rgba(18, 18, 20, 0.92);
  --text-primary: rgba(255, 255, 255, 0.90);
  --text-secondary: rgba(255, 255, 255, 0.40);
  --accent: #3B9EFF;
  --border: rgba(255, 255, 255, 0.10);
  --hover: rgba(255, 255, 255, 0.08);
  /* ... see src/themes/*.json for all variables */
}
```

### Theme Loading

1. Built-in: `system`, `light`, `dark` (use data-theme attribute)
2. Custom: Load from `src/themes/{name}.json`
3. Auto-detection: Calculates luminance to determine if theme is light/dark

### Glass-morphism Pattern

```css
.container {
  background: var(--bg-mica);
  backdrop-filter: blur(32px);
  -webkit-backdrop-filter: blur(32px);
  border: 1px solid var(--border);
  border-radius: 12px;
}
```

---

## Development Commands

```bash
# Development with hot reload
npm run tauri dev

# Production build (single portable exe)
npm run tauri build

# Build without bundling
npm run tauri build -- --bundles none

# Run Rust tests
cargo test --manifest-path src-tauri/Cargo.toml

# Install dependencies
npm install
```

---

## Code Conventions

### TypeScript/React

- **Strict TypeScript**: All types defined, no `any`
- **Functional Components**: All components are function components
- **Hooks**: `useState`, `useEffect`, `useRef`, `useCallback` only
- **Event Handling**: Keyboard events handled at window level
- **Icons**: Use `@fluentui/react-icons` (e.g., `Search24Regular`)

### Rust

- **Error Handling**: Use `Result<T, String>` for commands
- **Unsafe Blocks**: Isolated to Windows API calls, well-documented
- **Static State**: Use `OnceLock<Mutex<T>>` for global state
- **Windows API**: Use `windows-rs` crate with explicit feature flags

### File Naming

- React: PascalCase (`Settings.tsx`)
- Rust: snake_case (`indexer.rs`)
- CSS: Same name as component (`Settings.css`)
- Themes: kebab-case (`catppuccin-frappe.json`)

---

## Search Features Logic

### Fuzzy Search

Uses `SkimMatcherV2` from `fuzzy-matcher` crate:
- Matches against app names and settings
- Scores sorted descending
- Top 12 results returned

### Special Queries

| Prefix | Behavior |
|--------|----------|
| `kill <name>` | Search open windows, terminate by PID |
| Math expression | Evaluate with `meval`, show result |
| Empty query | Show clipboard history |
| Any text | + "Search Google for..." at bottom |

---

## Important Implementation Details

### Icon Extraction

- Extracts from `.lnk` and `.exe` files
- Converts Windows HICON to PNG via WinAPI
- Base64 encoded for transfer to frontend
- 48x48 resolution for crisp display

### Clipboard History

- Windows message queue listener in background thread
- Uses `arboard` crate for cross-platform clipboard
- Stores max 10 unique items (deduplicated)
- 100ms delay to avoid race conditions

### Auto-start

- Writes to `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`
- No admin required (HKCU = HKEY_CURRENT_USER)
- Toggleable in settings

### Window Behavior

- Main window: Hides on `Escape`, focus loss, or outside click
- Settings window: Draggable header, always-on-top
- System tray: Show/Quit menu, persists when main hidden

---

## Adding New Features

### Adding a New Theme

1. Create `src/themes/mytheme.json` with all CSS variables
2. Add theme name to `THEMES` array in `Settings.tsx`
3. Test with both light and dark system settings

### Adding a New Search Provider

1. Add to `indexer.rs` (for static) or `search.rs` (for dynamic)
2. Use appropriate `kind` string
3. Add icon mapping in `App.tsx` `KIND_ICONS`
4. Handle in `handleLaunch` if special behavior needed

### Adding a New Command

1. Implement in appropriate Rust module with `#[tauri::command]`
2. Register in `main.rs` `invoke_handler!` macro
3. Call from frontend via `invoke("command_name", args)`
4. Add TypeScript types for args and return value

---

## Testing Strategy

### Unit Tests (Rust)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_index_adds_settings() {
        build_index();
        let index = get_index().lock().unwrap();
        let has_taskmgr = index.iter().any(|e| e.name == "Task Manager");
        assert!(has_taskmgr);
    }
}
```

### Manual Testing Checklist

- [ ] Hotkey toggles window visibility
- [ ] Search returns results in <100ms
- [ ] App icons display correctly
- [ ] Settings save and persist
- [ ] Theme switching works
- [ ] Clipboard history populates
- [ ] Calculator evaluates expressions
- [ ] Kill command lists and terminates processes
- [ ] Auto-start registry entry created
- [ ] System tray menu functional

---

## Dependencies to Know

### Production (npm)

- `@tauri-apps/api`: Frontend Tauri bindings
- `@tauri-apps/plugin-*`: Various Tauri plugins
- `@fluentui/react-icons`: Icon library
- `react`/`react-dom`: UI framework

### Production (Cargo)

- `tauri`: Core framework
- `tauri-plugin-global-shortcut`: Hotkey handling
- `tauri-plugin-shell`: Process spawning
- `fuzzy-matcher`: Fuzzy search algorithm
- `walkdir`: Directory traversal
- `image` + `base64`: Icon processing
- `windows`: WinAPI bindings
- `meval`: Math expression evaluation
- `arboard`: Clipboard access
- `winreg`: Windows registry manipulation

---

## Performance Considerations

- **Index Size**: ~1000-3000 items typical
- **Search Time**: <5ms (in-memory, fuzzy match)
- **Memory Usage**: ~25MB idle
- **Binary Size**: ~6-10MB (optimized release)
- **Startup Time**: <1.6s (includes icon extraction)

### Optimization Flags (Cargo.toml)

```toml
[profile.release]
opt-level = "z"      # Optimize for size
lto = true           # Link-time optimization
codegen-units = 1    # Single codegen unit
strip = true         # Strip symbols
```

---

## Security Notes

- Runs as `asInvoker` (no admin elevation)
- Only accesses `HKCU` registry (user-specific)
- Shell execution via `cmd /c start` for safety
- No network requests except user-initiated web search
- CSP disabled for local resources only

---

## Common Issues & Solutions

### Icons not displaying
- Check `icons.rs` extract logic
- Ensure SHGetFileInfoW returns valid HICON

### Hotkey not working
- Verify not conflicting with system hotkey
- Check `Shortcut::from_str` parsing
- Try unregistering old hotkey before registering new

### Window not hiding on focus loss
- Check `on_window_event` handler in `main.rs`
- Ensure `WindowEvent::Focused(false)` is captured

### Clipboard not updating
- Verify `init_clipboard_listener()` called in setup
- Check AddClipboardFormatListener succeeded

---

## License

MIT License - Created by Pranav Raizada
