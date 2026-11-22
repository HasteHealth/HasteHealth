import React, {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useLayoutEffect,
  useMemo,
  useState,
  useSyncExternalStore,
} from "react";

export type Theme = "light" | "dark";
export type ThemePreference = Theme | "system";

export type ThemeContextValue = {
  /** The theme on screen: the preference, with "system" resolved. */
  theme: Theme;
  preference: ThemePreference;
  setThemePreference: (preference: ThemePreference) => void;
  toggleTheme: () => void;
};

const ThemeContext = createContext<ThemeContextValue | undefined>(undefined);

const STORAGE_KEY = "hastehealth-theme-preference";
// The app is light until a user chooses otherwise.
const DEFAULT_PREFERENCE: ThemePreference = "light";
const SYSTEM_DARK_QUERY = "(prefers-color-scheme: dark)";

const isPreference = (value: string | null): value is ThemePreference =>
  value === "light" || value === "dark" || value === "system";

// localStorage throws when the browser blocks site data. The preference then
// lasts only as long as the page.
const readPreference = (): ThemePreference => {
  try {
    const stored = window.localStorage.getItem(STORAGE_KEY);
    return isPreference(stored) ? stored : DEFAULT_PREFERENCE;
  } catch {
    return DEFAULT_PREFERENCE;
  }
};

const savePreference = (preference: ThemePreference) => {
  try {
    window.localStorage.setItem(STORAGE_KEY, preference);
  } catch {
    // Not saved; the provider still holds it in state.
  }
};

const subscribeToSystemTheme = (onChange: () => void) => {
  const media = window.matchMedia(SYSTEM_DARK_QUERY);
  media.addEventListener("change", onChange);
  return () => media.removeEventListener("change", onChange);
};

const getSystemTheme = (): Theme =>
  window.matchMedia(SYSTEM_DARK_QUERY).matches ? "dark" : "light";

/**
 * Sets the theme on the root element, where the theme stylesheet reads it.
 * `theme-switching` turns transitions off until the new colours have painted,
 * so the page changes at once instead of element by element.
 */
const applyTheme = (theme: Theme) => {
  const root = document.documentElement;
  if (root.dataset.theme === theme) {
    return;
  }

  root.classList.add("theme-switching");
  root.dataset.theme = theme;
  window.requestAnimationFrame(() =>
    window.requestAnimationFrame(() =>
      root.classList.remove("theme-switching"),
    ),
  );
};

export const ThemeProvider = ({
  children,
}: Readonly<{ children: React.ReactNode }>) => {
  const [preference, setPreference] = useState(readPreference);
  const systemTheme = useSyncExternalStore(
    subscribeToSystemTheme,
    getSystemTheme,
  );
  const theme = preference === "system" ? systemTheme : preference;

  // A layout effect, so the attribute changes in the same frame as the
  // components that render from `theme`.
  useLayoutEffect(() => applyTheme(theme), [theme]);

  // Follow a preference changed in another tab.
  useEffect(() => {
    const onStorage = (event: StorageEvent) => {
      // A null key means the whole store was cleared.
      if (event.key === STORAGE_KEY || event.key === null) {
        setPreference(readPreference());
      }
    };
    window.addEventListener("storage", onStorage);
    return () => window.removeEventListener("storage", onStorage);
  }, []);

  const setThemePreference = useCallback((next: ThemePreference) => {
    savePreference(next);
    setPreference(next);
  }, []);

  // From "system" this pins the opposite of what the system shows.
  const toggleTheme = useCallback(
    () => setThemePreference(theme === "dark" ? "light" : "dark"),
    [theme, setThemePreference],
  );

  const contextValue = useMemo(
    () => ({ theme, preference, setThemePreference, toggleTheme }),
    [theme, preference, setThemePreference, toggleTheme],
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
