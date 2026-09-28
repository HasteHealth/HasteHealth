/**
 * The console's keyboard shortcuts.
 *
 * A developer tool should be usable without the mouse, so every destination
 * in the sidebar and every region of the workspace has a key. Chords use the
 * platform's own modifier - Cmd on a Mac, Ctrl elsewhere - so the bindings
 * read the way each platform expects.
 */
import { useEffect, useMemo } from "react";

/** True when the user is on a Mac, which decides the modifier and its label. */
export function isMac(): boolean {
  if (typeof navigator === "undefined") return false;
  return /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent);
}

/** The modifier's symbol, for showing a binding in the UI. */
export function modLabel(): string {
  return isMac() ? "⌘" : "Ctrl";
}

export interface Shortcut {
  /** Lower case key, as `KeyboardEvent.key` reports it. */
  key: string;
  /** Requires the platform modifier (Cmd on a Mac, Ctrl elsewhere). */
  mod?: boolean;
  shift?: boolean;
  /** What the shortcut does. */
  run: () => void;
  /** Shown in the shortcut help. */
  description: string;
  /**
   * Runs even while a text field has focus. Off by default, so that typing
   * a letter into the command bar never triggers a navigation.
   */
  whileTyping?: boolean;
}

/** True when the event's target is somewhere the user is typing. */
function isTypingTarget(target: EventTarget | null): boolean {
  const element = target as HTMLElement | null;
  if (!element) return false;
  if (element.isContentEditable) return true;
  const tag = element.tagName;
  return tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT";
}

/**
 * Keys that still act while a field has focus.
 *
 * This has to stay tiny. The command bar is autofocused and holds focus for
 * most of a session, so anything listed here is a character the user can no
 * longer type - which ruled out `?`, the separator between a resource type
 * and its search parameters and so one of the most typed characters there.
 */
const ALWAYS_LIVE = new Set(["escape"]);

/**
 * Binds `shortcuts` for as long as the component is mounted.
 *
 * Bindings without the modifier are suppressed while typing, so single key
 * shortcuts can stay short without stealing characters from an input.
 */
export function useShortcuts(shortcuts: Shortcut[]): void {
  // The list is rebuilt on every render by most callers, so it is compared by
  // its content rather than its identity.
  const key = shortcuts
    .map((s) => `${s.mod ? "m" : ""}${s.shift ? "s" : ""}${s.key}`)
    .join(",");

  const bound = useMemo(() => shortcuts, [key]); // eslint-disable-line react-hooks/exhaustive-deps

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const mod = isMac() ? event.metaKey : event.ctrlKey;
      const typing = isTypingTarget(event.target);

      for (const shortcut of bound) {
        if (event.key.toLowerCase() !== shortcut.key) continue;
        if (Boolean(shortcut.mod) !== mod) continue;
        if (Boolean(shortcut.shift) !== event.shiftKey) continue;
        // A bare key would otherwise be typed into whatever has focus. The
        // command bar keeps focus for most of a session, so anything meant
        // to work from there has to say so or carry the modifier.
        const live =
          shortcut.mod ||
          shortcut.whileTyping ||
          ALWAYS_LIVE.has(shortcut.key);
        if (typing && !live) continue;

        event.preventDefault();
        shortcut.run();
        return;
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [bound]);
}
