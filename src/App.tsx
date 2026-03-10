import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { 
  Search24Regular, 
  Apps20Regular, 
  Settings20Regular, 
  Document20Regular, 
  Globe20Regular, 
  FolderOpen20Regular,
  WeatherMoon20Regular,
  WeatherSunny20Regular
} from "@fluentui/react-icons";
import "./App.css";

interface SearchResult {
  name: string;
  path: string;
  kind: "app" | "file" | "setting" | "web";
  score: number;
  icon_base64?: string;
}

type Theme = "light" | "dark" | "system";

const KIND_ICONS = {
  app:     <Apps20Regular />,
  file:    <Document20Regular />,
  setting: <Settings20Regular />,
  web:     <Globe20Regular />,
};

export default function App() {
  const [query, setQuery]     = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [selected, setSelected] = useState(0);
  const [loading, setLoading] = useState(false);
  const [theme, setTheme]     = useState<Theme>("system");
  const [effectiveTheme, setEffectiveTheme] = useState<"light" | "dark">("light");

  const inputRef = useRef<HTMLInputElement>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => { inputRef.current?.focus(); }, []);

  useEffect(() => {
    const applyTheme = () => {
      if (theme === "system") {
        const isDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
        setEffectiveTheme(isDark ? "dark" : "light");
      } else {
        setEffectiveTheme(theme);
      }
    };
    applyTheme();
    if (theme === "system") {
      const media = window.matchMedia("(prefers-color-scheme: dark)");
      const listener = () => applyTheme();
      media.addEventListener("change", listener);
      return () => media.removeEventListener("change", listener);
    }
  }, [theme]);

  useEffect(() => {
    document.documentElement.setAttribute("data-theme", effectiveTheme);
  }, [effectiveTheme]);

  const toggleTheme = () => {
    setTheme(prev => prev === "system" ? "light" : prev === "light" ? "dark" : "system");
  };

  useEffect(() => {
    if (debounceRef.current) clearTimeout(debounceRef.current);
    if (!query.trim()) {
      setResults([]);
      setLoading(false);
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

  const handleLaunch = async (idx: number) => {
    const items = getCombinedResults();
    if (items[idx]) {
      await invoke("launch_item", { path: items[idx].path });
    }
  };

  const handleOpenFolder = async (e: React.MouseEvent, path: string) => {
    e.stopPropagation();
    const parentDir = path.substring(0, Math.max(path.lastIndexOf('\\'), path.lastIndexOf('/')));
    if (parentDir) {
      await invoke("open_path", { path: parentDir });
    }
  };

  const handleKey = useCallback((e: KeyboardEvent) => {
    const total = getCombinedResults().length;
    if (total === 0) return;

    if (e.key === "ArrowDown") { e.preventDefault(); setSelected(s => (s + 1) % total); }
    if (e.key === "ArrowUp")   { e.preventDefault(); setSelected(s => (s - 1 + total) % total); }
    if (e.key === "Enter")     { handleLaunch(selected); }
    if (e.key === "Escape")    { setQuery(""); setResults([]); }
  }, [results, selected, query]);

  useEffect(() => {
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [handleKey]);

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

  const allResults = getCombinedResults();
  const isExpanded = query.trim().length > 0;

  const grouped: { kind: string, items: { item: SearchResult, idx: number }[] }[] = [];
  allResults.forEach((item, idx) => {
    let group = grouped.find(g => g.kind === item.kind);
    if (!group) {
      group = { kind: item.kind, items: [] };
      grouped.push(group);
    }
    group.items.push({ item, idx });
  });

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
        <div className="theme-toggle" onClick={toggleTheme} title="Switch Theme">
          {effectiveTheme === "dark" ? <WeatherMoon20Regular /> : <WeatherSunny20Regular />}
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
                onMouseEnter={() => setSelected(idx)}
              >
                <div className="result-icon">
                  {item.icon_base64 ? (
                    <img src={`data:image/png;base64,${item.icon_base64}`} alt="" />
                  ) : (
                    KIND_ICONS[item.kind]
                  )}
                </div>
                <div className="result-info">
                  <div className="result-name">{item.name}</div>
                </div>
                {item.kind !== 'web' && item.kind !== 'setting' && (
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
        <div style={{ fontWeight: 700, letterSpacing: '0.05em' }}>WINDOWS SEARCH TOOL</div>
      </div>
    </div>
  );
}
