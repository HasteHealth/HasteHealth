/**
 * Built-in accent themes for the Appearance settings.
 *
 * The `id` is written to `data-accent` on <html> (the default theme has no
 * attribute) and the matching CSS variables live in `src/index.css` under
 * `html[data-accent="<id>"]`. To add another theme:
 *
 *   1. add an entry to `ACCENT_THEMES` below, and
 *   2. add an `html[data-accent="<id>"]` block with the same brand channels as
 *      the default/ember blocks in `src/index.css`.
 *
 * The components themselves do not change: they read the `--ci-*` tokens, so a
 * new accent only needs those CSS variables. A deployment can also keep its own
 * accent entirely in CSS and skip this registry.
 */
export type AccentTheme = {
  id: string;
  label: string;
  description: string;
  /** Representative light/dark colours for the Appearance picker. */
  swatches: { light: string; dark: string };
};

export const DEFAULT_ACCENT = "teal";

export const ACCENT_THEMES: AccentTheme[] = [
  {
    id: "teal",
    label: "Teal",
    description: "The default Haste Health palette.",
    swatches: { light: "#0d9488", dark: "#2dd4bf" },
  },
  {
    id: "ember",
    label: "Ember",
    description: "The warm paper palette with an orange-red accent.",
    swatches: { light: "#e5451a", dark: "#ff5e26" },
  },
];

/** The `data-accent` value to apply, or `null` for the default theme. */
export function accentAttribute(accent: string): string | null {
  return accent === DEFAULT_ACCENT ? null : accent;
}
