import React, { createContext, useCallback, useContext, useEffect, useMemo, useState } from "react";

import { DEFAULT_ACCENT, accentAttribute } from "./accents";

export type Theme = "light" | "dark";
export type ThemePreference = Theme | "system";

export type ThemeContextValue = {
  theme: Theme;
  preference: ThemePreference;
  setThemePreference: (preference: ThemePreference) => void;
  toggleTheme: () => void;
  accent: string;
  setAccent: (accent: string) => void;
};

const ThemeContext = createContext<ThemeContextValue | undefined>(undefined);

const STORAGE_KEY = "hastehealth-theme-preference";
const LEGACY_STORAGE_KEY = "hastehealth-theme";
const ACCENT_STORAGE_KEY = "hastehealth-theme-accent";

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

/** Applies the accent without animating every colour from the previous theme. */
const applyAccentAttribute = (accent: string) => {
  const root = document.documentElement;
  root.classList.add("theme-switching");
  const attribute = accentAttribute(accent);
  if (attribute) {
    root.setAttribute("data-accent", attribute);
  } else {
    root.removeAttribute("data-accent");
  }
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

  return "light";
};

const getInitialAccent = (): string => {
  if (typeof window === "undefined") return DEFAULT_ACCENT;

  const stored = window.localStorage.getItem(ACCENT_STORAGE_KEY)?.trim();
  return stored || DEFAULT_ACCENT;
};

export const ThemeProvider = ({ children }: { children: React.ReactNode }) => {
  const initialPreference = useMemo(getInitialPreference, []);
  const [preference, setPreference] = useState<ThemePreference>(initialPreference);
  const [theme, setTheme] = useState<Theme>(() =>
    initialPreference === "system" ? getSystemTheme() : initialPreference
  );
  const [accent, setAccentState] = useState<string>(getInitialAccent);

  useEffect(() => {
    const resolvedTheme = preference === "system" ? getSystemTheme() : preference;
    setTheme(resolvedTheme);
    applyThemeAttribute(resolvedTheme);
    window.localStorage.setItem(STORAGE_KEY, preference);
  }, [preference]);

  useEffect(() => {
    applyAccentAttribute(accent);
    window.localStorage.setItem(ACCENT_STORAGE_KEY, accent);
  }, [accent]);

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
  const setAccent = useCallback(
    (next: string) => setAccentState(next.trim() || DEFAULT_ACCENT),
    []
  );
  const toggleTheme = useCallback(
    () =>
      setPreference((prev) => {
        const resolved = prev === "system" ? getSystemTheme() : prev;
        return resolved === "light" ? "dark" : "light";
      }),
    []
  );

  const contextValue = useMemo(
    () => ({ theme, preference, setThemePreference, toggleTheme, accent, setAccent }),
    [theme, preference, setThemePreference, toggleTheme, accent, setAccent]
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
