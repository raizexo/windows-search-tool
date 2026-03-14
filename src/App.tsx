import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { applyThemeVariables } from "./utils/theme";
import { 
  Search24Regular, 
  Apps20Regular, 
  Settings20Regular, 
  Document20Regular, 
  Globe20Regular, 
  FolderOpen20Regular,
  Calculator20Regular,
  Clipboard20Regular,
  Dismiss20Regular
} from "@fluentui/react-icons";
import "./App.css";

interface SearchResult {
  name: string;
  path: string;
  kind: "app" | "file" | "setting" | "web" | "math" | "clipboard" | "kill";
  score: number;
  icon_base64?: string;
}

interface FullConfig {
  hotkey: string;
  theme: string;
  startup: boolean;
}

const KIND_ICONS: Record<string, React.ReactNode> = {
  app:     <Apps20Regular />,
  file:    <Document20Regular />,
  setting: <Settings20Regular />,
  web:     <Globe20Regular />,
  math:    <Calculator20Regular />,
  clipboard: <Clipboard20Regular />,
  kill:    <Dismiss20Regular />,
};

export default function App() {
  const [query, setQuery]     = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [selected, setSelected] = useState(0);
  const [loading, setLoading] = useState(false);
  const [theme, setTheme]     = useState<string>("system");
  const [hotkey, setHotkey]   = useState("Ctrl+Space");

  const [copiedIndex, setCopiedIndex] = useState<number | null>(null);

  const inputRef = useRef<HTMLInputElement>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const isKeyboardNav = useRef(false);

  useEffect(() => { 
    inputRef.current?.focus(); 

    // Initial load
    invoke<FullConfig>("get_full_config").then(cfg => {
      setHotkey(cfg.hotkey);
      setTheme(cfg.theme);
    }).catch(console.error);

    // Listen for config changes from Settings window
    const unlisten = listen<FullConfig>("config-changed", (event) => {
      setHotkey(event.payload.hotkey);
      setTheme(event.payload.theme);
    });

    return () => {
      unlisten.then(f => f());
    };
  }, []);

  useEffect(() => {
    applyThemeVariables(theme);
    if (theme === "system") {
      const media = window.matchMedia("(prefers-color-scheme: dark)");
      const listener = () => applyThemeVariables(theme);
      media.addEventListener("change", listener);
      return () => media.removeEventListener("change", listener);
    }
  }, [theme]);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      const container = document.querySelector('.container');
      if (container && !container.contains(e.target as Node)) {
        invoke("hide_window").catch(console.error);
      }
    };
    window.addEventListener("mousedown", handleClickOutside);
    return () => window.removeEventListener("mousedown", handleClickOutside);
  }, []);

  const openSettings = async () => {
    await invoke("open_settings_window");
    await invoke("hide_window"); // Hide search bar while settings is open
  };

  useEffect(() => {
    if (debounceRef.current) clearTimeout(debounceRef.current);
    if (!query.trim()) {
      setLoading(true);
      invoke<SearchResult[]>("get_clipboard_history")
        .then(res => {
          setResults(res);
          setSelected(0);
        })
        .catch(console.error)
        .finally(() => setLoading(false));
      return;
    }

    setLoading(true);
    debounceRef.current = setTimeout(async () => {
      try {
        const res = await invoke<SearchResult[]>("search_items", { query });
        setResults(res);
        setSelected(0);
      } catch (e) {
        console.error(e);
      } finally {
        setLoading(false);
      }
    }, 50);
  }, [query]);

  const getCombinedResults = (): SearchResult[] => {
    const combined = [...results];
    if (query.trim()) {
      combined.push({
        name: `Search Google for "${query}"`,
        path: `https://google.com/search?q=${encodeURIComponent(query)}`,
        kind: "web",
        score: 0
      });
    }
    return combined;
  };

  const getVisualResults = (): SearchResult[] => {
    const combined = getCombinedResults();
    const tempGrouped: { kind: string, items: SearchResult[] }[] = [];
    combined.forEach(item => {
      let g = tempGrouped.find(g => g.kind === item.kind);
      if (!g) { g = { kind: item.kind, items: [] }; tempGrouped.push(g); }
      g.items.push(item);
    });
    return tempGrouped.flatMap(g => g.items);
  };

  const allResults = getCombinedResults();
  const isExpanded = query.trim().length > 0 || results.length > 0;

  const grouped: { kind: string, items: { item: SearchResult, idx: number }[] }[] = [];
  let currentVisualIdx = 0;
  allResults.forEach((item) => {
    let group = grouped.find(g => g.kind === item.kind);
    if (!group) {
      group = { kind: item.kind, items: [] };
      grouped.push(group);
    }
    group.items.push({ item, idx: 0 }); // Placeholder
  });

  grouped.forEach(group => {
    group.items.forEach(groupItem => {
      groupItem.idx = currentVisualIdx++;
    });
  });

  const handleLaunch = async (idx: number) => {
    const visualItems = getVisualResults();
    const item = visualItems[idx];
    if (item) {
      if (item.kind === "math") {
        await invoke("copy_to_clipboard", { text: item.path });
        setCopiedIndex(idx);
        setTimeout(() => {
          setCopiedIndex(null);
          invoke("hide_window");
        }, 800);
      } else if (item.kind === "clipboard") {
        setQuery(item.path);
      } else if (item.kind === "kill") {
        await invoke("kill_process", { name: item.path });
        await invoke("hide_window");
      } else {
        await invoke("launch_item", { path: item.path });
        await invoke("hide_window");
      }
    }
  };

  const handleOpenFolder = async (e: React.MouseEvent, path: string) => {
    e.stopPropagation();
    const parentDir = path.substring(0, Math.max(path.lastIndexOf('\\'), path.lastIndexOf('/')));
    if (parentDir) {
      await invoke("open_path", { path: parentDir });
    }
  };

  const handleKey = useCallback(async (e: KeyboardEvent) => {
    if (e.key === "Escape") { 
      if (query === "") {
        await invoke("hide_window");
      } else {
        setQuery(""); 
      }
      return;
    }

    const visualItems = getVisualResults();
    const total = visualItems.length;
    if (total === 0) return;

    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      isKeyboardNav.current = true;
      if ((window as any).keyboardNavTimeout) clearTimeout((window as any).keyboardNavTimeout);
      (window as any).keyboardNavTimeout = setTimeout(() => { isKeyboardNav.current = false; }, 150);
    }

    if (e.key === "ArrowDown") { e.preventDefault(); setSelected(s => (s + 1) % total); }
    if (e.key === "ArrowUp")   { e.preventDefault(); setSelected(s => (s - 1 + total) % total); }
    if (e.key === "Enter")     { handleLaunch(selected); }
  }, [results, selected, query]);

  useEffect(() => {
    window.addEventListener("keydown", handleKey as any);
    return () => window.removeEventListener("keydown", handleKey as any);
  }, [handleKey]);

  useEffect(() => {
    const selectedEl = document.querySelector('.result-item.selected');
    if (selectedEl) {
      selectedEl.scrollIntoView({ block: 'nearest' });
    }
  }, [selected]);

  return (
    <div className={`container ${isExpanded ? 'expanded' : ''}`}>
      <div className="search-header">
        <div className={`search-icon ${loading ? 'loading-shimmer' : ''}`}>
          <Search24Regular />
        </div>
        <input
          ref={inputRef}
          className="search-input"
          placeholder="Search apps, settings, and web..."
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          spellCheck={false}
          autoComplete="off"
        />
        <div className="theme-toggle" onClick={openSettings} title="Open Settings">
          <Settings20Regular />
        </div>
      </div>

      <div className="results-list">
        {grouped.map(group => (
          <div key={group.kind}>
            <div className="result-group-title">{group.kind.toUpperCase()}</div>
            {group.items.map(({ item, idx }) => (
              <div
                key={`${item.kind}-${item.path}`}
                className={`result-item ${selected === idx ? 'selected' : ''}`}
                onClick={() => handleLaunch(idx)}
                onMouseMove={() => {
                  if (!isKeyboardNav.current && selected !== idx) {
                    setSelected(idx);
                  }
                }}
              >
                <div className="result-icon">
                  {item.icon_base64 ? (
                    <img src={`data:image/png;base64,${item.icon_base64}`} alt="" />
                  ) : (
                    KIND_ICONS[item.kind]
                  )}
                </div>
                <div className="result-info">
                  <div className="result-name">
                    {item.kind === 'clipboard' ? `Search: ${item.name}` : item.name}
                  </div>
                </div>
                {copiedIndex === idx && <span className="copied-label">Copied!</span>}
                {item.kind !== 'web' && item.kind !== 'setting' && item.kind !== 'clipboard' && item.kind !== 'math' && item.kind !== 'kill' && (
                  <div 
                    className="action-btn" 
                    title="Open in folder"
                    onClick={(e) => handleOpenFolder(e, item.path)}
                  >
                    <FolderOpen20Regular />
                  </div>
                )}
              </div>
            ))}
          </div>
        ))}
      </div>

      <div className="footer">
        <div className="footer-keys">
          <div className="footer-item">
            <span className="key-hint">↑↓</span>
            <span className="hint-label">Move</span>
          </div>
          <div className="footer-item">
            <span className="key-hint">Enter</span>
            <span className="hint-label">Open</span>
          </div>
          <div className="footer-item">
            <span className="key-hint">Esc</span>
            <span className="hint-label">Clear</span>
          </div>
        </div>
        <div style={{ fontWeight: 700, letterSpacing: '0.05em' }}>WINDOWS SEARCH TOOL • {hotkey}</div>
      </div>
    </div>
  );
}