import React, { createContext, useCallback, useContext, useEffect, useMemo, useState } from "react";

export type Theme = "light" | "dark";
export type ThemePreference = Theme | "system";

export type ThemeContextValue = {
  theme: Theme;
  preference: ThemePreference;
  setThemePreference: (preference: ThemePreference) => void;
  toggleTheme: () => void;
};

const ThemeContext = createContext<ThemeContextValue | undefined>(undefined);

const STORAGE_KEY = "hastehealth-theme-preference";
const LEGACY_STORAGE_KEY = "hastehealth-theme";

const getSystemTheme = (): Theme => {
  if (typeof window === "undefined") return "light";
  return window.matchMedia && window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
};

/**
 * Applies the resolved theme attribute without letting per-element colour
 * transitions animate from the previous theme.
 */
const applyThemeAttribute = (theme: Theme) => {
  const root = document.documentElement;
  root.classList.add("theme-switching");
  root.setAttribute("data-theme", theme);
  const release = () => root.classList.remove("theme-switching");
  if (typeof window !== "undefined" && typeof window.requestAnimationFrame === "function") {
    window.requestAnimationFrame(() => window.requestAnimationFrame(release));
  } else {
    release();
  }
};

const getInitialPreference = (): ThemePreference => {
  if (typeof window === "undefined") return "light";

  const stored = window.localStorage.getItem(STORAGE_KEY) as ThemePreference | null;
  if (stored === "light" || stored === "dark" || stored === "system") {
    return stored;
  }

  const legacy = window.localStorage.getItem(LEGACY_STORAGE_KEY) as Theme | null;
  if (legacy === "light" || legacy === "dark") {
    return legacy;
  }

  return "system";
};

export const ThemeProvider = ({ children }: { children: React.ReactNode }) => {
  const initialPreference = useMemo(getInitialPreference, []);
  const [preference, setPreference] = useState<ThemePreference>(initialPreference);
  const [theme, setTheme] = useState<Theme>(() =>
    initialPreference === "system" ? getSystemTheme() : initialPreference
  );

  useEffect(() => {
    const resolvedTheme = preference === "system" ? getSystemTheme() : preference;
    setTheme(resolvedTheme);
    applyThemeAttribute(resolvedTheme);
    window.localStorage.setItem(STORAGE_KEY, preference);
  }, [preference]);

  useEffect(() => {
    if (typeof window === "undefined" || !window.matchMedia) return;
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = (event: MediaQueryListEvent) => {
      if (preference === "system") {
        const resolvedTheme = event.matches ? "dark" : "light";
        setTheme(resolvedTheme);
        applyThemeAttribute(resolvedTheme);
      }
    };
    media.addEventListener("change", handler);
    return () => media.removeEventListener("change", handler);
  }, [preference]);

  const setThemePreference = useCallback((next: ThemePreference) => setPreference(next), []);
  const toggleTheme = useCallback(
    () =>
      setPreference((prev) => {
        const resolved = prev === "system" ? getSystemTheme() : prev;
        return resolved === "light" ? "dark" : "light";
      }),
    []
  );

  const contextValue = useMemo(
    () => ({ theme, preference, setThemePreference, toggleTheme }),
    [theme, preference, setThemePreference, toggleTheme]
  );

  return (
    <ThemeContext.Provider value={contextValue}>
      {children}
    </ThemeContext.Provider>
  );
};

export const useTheme = (): ThemeContextValue => {
  const ctx = useContext(ThemeContext);
  if (!ctx) {
    throw new Error("useTheme must be used within ThemeProvider");
  }
  return ctx;
};
