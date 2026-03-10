<div align="center">

<img src="windows-search-tool.png" width="128" height="128" alt="windows-search-tool icon" />

# windows-search-tool

**The search Windows should have shipped with.**

A fast, minimal, opinionated search utility for Windows 11.  
No bloat. No Electron. No admin rights. Just search.

[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-24C8D8?style=flat-square&logo=tauri&logoColor=white)](https://tauri.app)
[![Rust](https://img.shields.io/badge/backend-Rust-CE422B?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![React](https://img.shields.io/badge/ui-React-61DAFB?style=flat-square&logo=react&logoColor=white)](https://react.dev)
[![License: MIT](https://img.shields.io/badge/license-MIT-6EE7B7?style=flat-square)](LICENSE)
[![Developer: Pranav Raizada](https://img.shields.io/badge/developer-Pranav%20Raizada-blue?style=flat-square)](https://github.com/PranavRaizada)

</div>

---

![windows-search-tool demo](demo/demo.gif)

---

## The Problem

Windows Search is slow, heavy, and broken by default. It indexes things you don't care about, misses things you do, launches a full shell window to show results, requires background services running constantly, and still manages to surface the wrong answer. It is a 400MB Electron wrapper pretending to be a system utility.

`windows-search-tool` disagrees with all of that.

---

## What It Is

A single portable `.exe` - > 10MB - that sits in your system tray and launches instantly with `Ctrl+Space`. It searches your installed apps, files, and system settings using a fuzzy-matching engine backed by an in-memory index built from your Start Menu at launch. No daemons. No services. No installer. No UAC prompt.

It searches the things most people search for most of the time, and it does that extremely well. It does not try to be everything.

---

## Features

- **`Ctrl+Space` global hotkey** — works from any application, no focus required.
- **Fluent Design (Mica)** — A high-end Windows 11-inspired interface with deep 32px backdrop blur and high-contrast typography.
- **Native Icon Extraction** — Rust-powered extraction of real icons from `.lnk` and `.exe` files for instant recognition.
- **Progressive Disclosure** — UI starts as a minimal search bar and expands gracefully as you type.
- **Fuzzy Search** — Typo-tolerant matching powered by the `skim` algorithm.
- **Open Folder Action** — Functional action buttons to jump directly to an item's containing directory.
- **Theme Switching** — Auto-detects system Light/Dark mode with a manual toggle in the search bar.
- **Instant results** — Index lives in RAM, searches complete in under 5ms.
- **Web fallback** — Bottom result always opens a Google search for your query in your default browser.
- **System tray** — Runs silently in the background, accessible via tray icon.
- **Auto-start on login** — Registers to `HKCU` run key, no admin required.
- **Non-admin** — `asInvoker` execution level, zero UAC prompts, ever.

---

## Installation

### Option A — Download the release (recommended)

1. Download `windows-search-tool.exe` from [Releases](https://github.com/raizexo/windows-search-tool/releases)
2. Run it once — it auto-registers for startup and adds a tray icon
3. Press `Ctrl+Space` to open

That's it.

### Option B — Build from source

**Prerequisites:** [Rust](https://rustup.rs) · [Node.js 18+](https://nodejs.org) · [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (pre-installed on Win10 1803+ and all Win11)

```bash
git clone https://github.com/PranavRaizada/windows-search-tool
cd windows-search-tool
npm install
npm run tauri build
```

Output: `src-tauri/target/release/windows-search-tool.exe`

---

## Usage

| Action | Shortcut |
|---|---|
| Open search | `Ctrl+Space` |
| Navigate results | `↑` / `↓` |
| Launch selected | `Enter` |
| Open Folder | Click folder icon |
| Switch Theme | Click sun/moon icon |
| Clear / close | `Esc` |
| Click away | Auto-closes |

Type anything. Results appear as you type, grouped by type: **APPS**, **SETTINGS**, **FILES**, **WEB**.

---


## Architecture

```
windows-search-tool/
├── src/                      React + Vite frontend (Fluent / Mica UI)
│   ├── App.tsx               Main UI & Theme Logic
│   └── App.css               Windows 11 Obsidian Material Styles
└── src-tauri/
    ├── src/
    │   ├── main.rs           Tauri setup, hotkey, tray, and auto-start
    │   ├── indexer.rs        Start Menu scanner & settings registry
    │   ├── icons.rs          Native HICON to Base64 PNG extraction (WinAPI)
    │   ├── search.rs         Fuzzy matching via skim algorithm
    │   └── launcher.rs       Shell execution & URL handling
    ├── tauri.conf.json       Branding, window config, and capabilities
    └── Cargo.toml            Rust dependencies (WinAPI, image, fuzzy-matcher)
```

---

## Performance

| Metric | Value |
|---|---|
| Binary size | 6.46 MB |
| Memory usage (idle) | ~25 MB |
| Index build time | <1.6s (includes icons) |
| Search latency | <1ms (in-memory) |

Measured on a mid-range machine, Windows 11.

---

## Requirements

- Windows 11
- WebView2 Runtime (pre-installed Win11)
- No administrator rights
- No .NET, no VC++ redistributables, no additional runtimes

---

## Building & Contributing

```bash
# Development with hot reload
npm run tauri dev

# Production build (single portable exe)
npm run tauri build

# Run backend tests
cargo test --manifest-path src-tauri/Cargo.toml
```

--- 
<div align="center">

**Created by Pranav Raizada**  
Made because Windows Search is not good enough.

</div>
