import { useEffect, useState } from "react";

let nextRowId = 0;
function newRowId(): string {
  nextRowId += 1;
  return `row-${nextRowId}`;
}

/**
 * Keys for the rows of an editable list whose items carry no identity.
 *
 * Keying on the index remounts a row's input when an earlier row is removed,
 * and keying on the value remounts it on every keystroke, so each row keeps an
 * id for as long as it exists.
 *
 * @param length How many rows the list has.
 */
export function useRowKeys(length: number): {
  /** The key of each row, in order. */
  keys: string[];
  /** Call when a row is appended, so it has its id from its first render. */
  onAdd: () => void;
  /** Call when the row at `index` is removed, so its id goes with it. */
  onRemove: (index: number) => void;
} {
  const [ids, setIds] = useState<string[]>(() =>
    Array.from({ length }, newRowId),
  );
  // Rows the parent adds some other way get ids on the next render; ids of
  // rows it removed are dropped with them.
  useEffect(() => {
    if (ids.length !== length) {
      setIds((current) =>
        Array.from({ length }, (_, i) => current[i] ?? newRowId()),
      );
    }
  }, [ids.length, length]);

  return {
    keys: Array.from({ length }, (_, i) => ids[i] ?? `unkeyed-${i}`),
    onAdd: () => setIds((current) => [...current, newRowId()]),
    onRemove: (index) =>
      setIds((current) => current.filter((_, i) => i !== index)),
  };
}

/**
 * Keys for a read-only list whose items carry no identity: each item's own
 * content, numbered so that repeats of the same content stay distinct.
 */
export function contentKeys(items: readonly unknown[]): string[] {
  const seen = new Map<string, number>();
  return items.map((item) => {
    const content = typeof item === "string" ? item : `${JSON.stringify(item)}`;
    const repeat = seen.get(content) ?? 0;
    seen.set(content, repeat + 1);
    return `${content}#${repeat}`;
  });
}
