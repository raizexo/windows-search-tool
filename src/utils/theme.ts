export const applyThemeVariables = async (themeName: string) => {
  const root = document.documentElement;

  // Clear any existing inline CSS variables
  for (let i = root.style.length - 1; i >= 0; i--) {
    const propName = root.style[i];
    if (propName.startsWith('--')) {
      root.style.removeProperty(propName);
    }
  }

  if (themeName === "system") {
    const isDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    root.setAttribute("data-theme", isDark ? "dark" : "light");
    return;
  }

  if (themeName === "light" || themeName === "dark") {
    root.setAttribute("data-theme", themeName);
    return;
  }

  // Load custom theme
  try {
    const themeModule = await import(`../themes/${themeName}.json`);
    const themeVars = themeModule.default;
    
    // Determine if the theme is light or dark based on background color
    const bgMica = themeVars["--bg-mica"];
    let isLight = false;
    if (bgMica) {
      const match = bgMica.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)/);
      if (match) {
        const r = parseInt(match[1]) / 255;
        const g = parseInt(match[2]) / 255;
        const b = parseInt(match[3]) / 255;
        const luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        isLight = luminance > 0.5;
      }
    }

    root.setAttribute("data-theme", isLight ? "light" : "dark");

    for (const [key, value] of Object.entries(themeVars)) {
      if (typeof value === "string" && key.startsWith("--")) {
        root.style.setProperty(key, value);
      }
    }
  } catch (error) {
    console.error(`Failed to load theme: ${themeName}`, error);
    root.setAttribute("data-theme", "dark");
  }
};
