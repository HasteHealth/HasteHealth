/**
 * The workspace state the console keeps between visits.
 *
 * These are jotai atoms backed by `localStorage`, rather than a load/save API
 * each component has to remember to call: reading an atom gives the current
 * value and writing one persists it, so a component cannot forget the second
 * half. Everything is scoped by tenant and project so two workspaces in the
 * same browser do not share state.
 */
import { atom } from "jotai";
import { atomWithStorage, createJSONStorage } from "jotai/utils";

import { deriveProjectId, deriveTenantId } from "../utilities";
import { Command, CommandHistoryEntry, sameCommand } from "./model";

/** How many past commands to keep per workspace. */
const HISTORY_LIMIT = 50;

/** Namespaces a key to the current tenant and project. */
function scopedKey(name: string): string {
  return `haste.${name}.${deriveTenantId() ?? "-"}.${deriveProjectId() ?? "-"}`;
}

/**
 * `localStorage`, or an in-memory stand-in when it is unavailable.
 *
 * Private browsing and blocked site data both make it throw, and a stored
 * preference is never worth failing a render over.
 */
function backing(): Storage {
  try {
    // Touching it is what throws, so this has to happen eagerly.
    window.localStorage.getItem("haste.probe");
    return window.localStorage;
  } catch {
    const memory = new Map<string, string>();
    return {
      getItem: (key: string) => memory.get(key) ?? null,
      setItem: (key: string, value: string) => {
        memory.set(key, value);
      },
      removeItem: (key: string) => {
        memory.delete(key);
      },
      clear: () => memory.clear(),
      key: (index: number) => Array.from(memory.keys())[index] ?? null,
      get length() {
        return memory.size;
      },
    } satisfies Storage;
  }
}

/**
 * A stored atom, scoped to this workspace.
 *
 * Wrapped because the storage has to be created per value type, and every
 * caller wants the same key scoping and read-on-init behaviour.
 */
function workspaceAtom<T>(name: string, initial: T) {
  return atomWithStorage<T>(
    scopedKey(name),
    initial,
    createJSONStorage<T>(backing),
    {
      getOnInit: true,
    },
  );
}

/** Past commands, most recently run first. */
export const historyAtom = workspaceAtom<CommandHistoryEntry[]>(
  "commandHistory",
  [],
);

/**
 * Records that a command was run.
 *
 * Re-running one moves it to the top rather than adding a duplicate, so the
 * list stays a set of distinct commands.
 */
export const recordCommandAtom = atom(null, (get, set, command: Command) => {
  const existing = get(historyAtom).filter(
    (entry) => !sameCommand(entry.command, command),
  );
  set(
    historyAtom,
    [{ command, at: Date.now() }, ...existing].slice(0, HISTORY_LIMIT),
  );
});

/** Drops one command from the history. */
export const removeCommandAtom = atom(null, (get, set, command: Command) => {
  set(
    historyAtom,
    get(historyAtom).filter((entry) => !sameCommand(entry.command, command)),
  );
});

/** Empties the history for this workspace. */
export const clearHistoryAtom = atom(null, (_get, set) => {
  set(historyAtom, []);
});

/**
 * Columns chosen per resource type. A type with no entry falls back to the
 * caller's default set.
 */
export const columnsAtom = workspaceAtom<Record<string, string[]>>(
  "columns",
  {},
);

/** Sets the columns for one resource type, leaving the others alone. */
export const setColumnsAtom = atom(
  null,
  (get, set, resourceType: string, codes: string[]) => {
    set(columnsAtom, { ...get(columnsAtom), [resourceType]: codes });
  },
);

/**
 * Resource types pinned to the top of the sidebar. A server exposes well over
 * a hundred and any one workspace cares about a few, so which few is worth
 * remembering.
 */
export const pinnedTypesAtom = workspaceAtom<string[]>("pinnedTypes", []);

/** Adds or removes a type from the pinned list. */
export const togglePinnedTypeAtom = atom(
  null,
  (get, set, resourceType: string) => {
    const pinned = get(pinnedTypesAtom);
    set(
      pinnedTypesAtom,
      pinned.includes(resourceType)
        ? pinned.filter((type) => type !== resourceType)
        : [...pinned, resourceType],
    );
  },
);
