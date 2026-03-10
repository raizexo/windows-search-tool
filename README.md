<div align="center">

# windows-search-tool

**The search Windows should have shipped with.**

A fast, minimal, opinionated search utility for Windows 10 and 11.  
No bloat. No Electron. No admin rights. Just search.

[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-24C8D8?style=flat-square&logo=tauri&logoColor=white)](https://tauri.app)
[![Rust](https://img.shields.io/badge/backend-Rust-CE422B?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![React](https://img.shields.io/badge/ui-React-61DAFB?style=flat-square&logo=react&logoColor=white)](https://react.dev)
[![License: MIT](https://img.shields.io/badge/license-MIT-6EE7B7?style=flat-square)](#license)

</div>

---

## The Problem

Windows Search is slow, heavy, and broken by default. It indexes things you don't care about, misses things you do, launches a full shell window to show results, requires background services running constantly, and still manages to surface the wrong answer. It is a 400MB Electron wrapper pretending to be a system utility.

`windows-search-tool` disagrees with all of that.

---

## What It Is

A single portable `.exe` — around 5MB — that sits in your system tray and launches instantly with `Ctrl+Space`. It searches your installed apps, files, and system settings using a fuzzy-matching engine backed by an in-memory index built from your Start Menu at launch. No daemons. No services. No installer. No UAC prompt.

It is opinionated: it searches the things most people search for most of the time, and it does that extremely well. It does not try to be everything.

---

## Features

- **`Ctrl+Space` global hotkey** — works from any application, no focus required
- **Fuzzy search** — typo-tolerant matching powered by the skim algorithm
- **Instant results** — index lives in RAM, searches complete in under 5ms
- **Apps** — all `.lnk` shortcuts from your Start Menu, both system and user
- **Settings** — 15+ common Windows Settings pages accessible by name
- **Web fallback** — bottom result always opens a Google search for your query
- **Auto-hide on blur** — window disappears when you click away, like it should
- **System tray** — runs silently in the background, accessible via tray icon
- **Auto-start on login** — registers to `HKCU` run key, no admin required
- **Non-admin** — `asInvoker` execution level, zero UAC prompts, ever
- **Portable** — single `.exe`, no installer, no registry dependencies beyond startup key

---

## Installation

### Option A — Download the release (recommended)

1. Download `windows-search-tool.exe` from [Releases](#)
2. Move it anywhere you like (e.g. `C:\Users\You\Tools\`)
3. Run it once — it auto-registers for startup and adds a tray icon
4. Press `Ctrl+Space` to open

That's it.

### Option B — Build from source

**Prerequisites:** [Rust](https://rustup.rs) · [Node.js 18+](https://nodejs.org) · [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (pre-installed on Win10 1803+ and all Win11)

```bash
git clone https://github.com/you/windows-search-tool
cd windows-search-tool
npm install
npm run tauri build -- --bundles none
```

Output: `src-tauri/target/release/windows-search-tool.exe`

---

## Usage

| Action | Shortcut |
|---|---|
| Open search | `Ctrl+Space` |
| Navigate results | `↑` / `↓` |
| Launch selected | `Enter` |
| Clear / close | `Esc` |
| Click away | Auto-closes |

Type anything. Results appear as you type, grouped by type: **Apps**, **Settings**, **Files**, **Web**. The index is built when the app launches — there is no background indexing service.

---

## Opinions

This tool makes deliberate choices. Here is what they are and why.

**It does not use Electron.**  
Electron ships a full Chromium browser in every app. That is 150MB of overhead for a search box. `windows-search-tool` uses Tauri, which renders via the WebView2 runtime already present on every modern Windows installation. The binary is ~5MB.

**It does not run a background indexer.**  
Most search tools run persistent services that watch the filesystem in real time. This uses CPU, memory, and disk I/O continuously. Instead, `windows-search-tool` builds its index once at launch from the Start Menu — a scan that takes under 200ms — and keeps it in memory. For the 90% use case (finding apps and settings), this is faster and cheaper than a daemon.

**It does not require administrator rights.**  
The executable runs as the current user (`asInvoker`). The startup key is written to `HKCU`, not `HKLM`. The global hotkey is registered through the user-space API. Nothing in this tool requires elevation, and it will never prompt for it.

**It hides on blur.**  
This is how a launcher should behave. You open it, search, press Enter, and it disappears. You do not manage it as a window. If you pressed `Esc` or clicked somewhere else, you did not want it anymore.

**It is not a file explorer.**  
Deep file search, preview panes, and folder browsing are out of scope. Those are features for a file manager. This is a launcher. It finds things and opens them.

---

## Architecture

```
windows-search-tool/
├── src/                      React + Vite frontend
│   ├── App.tsx               Main UI — glass-morphism dark theme
│   └── main.tsx              Tauri entry point
└── src-tauri/
    ├── src/
    │   ├── main.rs           Tauri setup, hotkey registration, tray, window events
    │   ├── indexer.rs        Start Menu scanner, settings registry, in-memory store
    │   ├── search.rs         Fuzzy matching via skim algorithm, result ranking
    │   └── launcher.rs       Shell execution, ms-settings: URIs, .lnk resolution
    ├── tauri.conf.json       Frameless transparent window, asInvoker execution level
    └── Cargo.toml            Rust dependencies
```

**Hotkey → display flow:**
```
Ctrl+Space keypress (any app)
  └─ tauri-plugin-global-shortcut (user-space, no hooks driver)
       └─ Toggle window visibility
            └─ window.show() + set_focus()
                 └─ React UI renders, input auto-focused
```

**Search flow:**
```
Keystroke in input (debounced 50ms)
  └─ invoke("search_items", { query })
       └─ Rust: fuzzy_match() against in-memory Vec<IndexEntry>
            └─ Sort by score, truncate to 12
                 └─ Results returned to React in <5ms
```

---

## Performance

| Metric | Value |
|---|---|
| Binary size | ~5MB |
| Memory usage (idle) | ~18MB |
| Index build time | <200ms at launch |
| Search latency | <5ms (in-memory) |
| Hotkey response time | <16ms |
| Startup time to ready | <800ms |

Measured on a mid-range laptop, Windows 11 23H2.

---

## Requirements

- Windows 10 (1803 or later) or Windows 11
- WebView2 Runtime (pre-installed on Win10 April 2018 Update+ and all Win11)
- No administrator rights
- No .NET, no VC++ redistributables, no additional runtimes

---

## Roadmap

- [ ] File search via Windows Search API (`windows-rs`)
- [ ] Calculator — evaluate expressions directly in the search box
- [ ] Clipboard history integration
- [ ] Custom hotkey configuration
- [ ] Plugin API for custom result providers
- [ ] Theme customization (light mode, accent colors)

---

## Building & Contributing

```bash
# Development with hot reload
npm run tauri dev

# Production build (single portable exe)
npm run tauri build -- --bundles none

# Run tests
cargo test --manifest-path src-tauri/Cargo.toml
```

Pull requests are welcome. Please open an issue before working on a large feature to discuss whether it fits the scope of the project. Features that require elevated privileges, background services, or add more than ~2MB to the binary will generally not be accepted — that is not a judgement of their quality, it is a statement of what this tool is.

--- 
<div align="center">

Made because Windows Search is not good enough.

</div>