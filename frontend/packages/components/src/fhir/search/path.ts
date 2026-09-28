/**
 * Parses a whole request path - `Patient/123`, `Patient/123/$everything`,
 * `_history?_count=5` - so completion knows which part the cursor is in.
 *
 * Layered over {@link parseSearch}, which still handles everything after `?`.
 */

/** The three levels a path can address. */
export type PathLevel = "system" | "type" | "instance";

/** What follows the resource or instance, if anything. */
export type PathInteraction =
  | { kind: "resource" }
  | { kind: "history" }
  | { kind: "operation"; operation: string };

export interface ParsedPath {
  level: PathLevel;
  resourceType?: string;
  id?: string;
  versionId?: string;
  interaction: PathInteraction;
  /** The path portion, before any `?`. */
  pathFrom: number;
  pathTo: number;
  /** `/` separated segments, with their offsets. */
  segments: { text: string; from: number; to: number }[];
  /** Where the search begins, after the `?`. */
  searchFrom?: number;
  /** Everything after the `?`. */
  search?: string;
}

/** A resource type: capitalised, may contain digits (`AccessPolicyV2`). */
export const RESOURCE_TYPE_RE = /^[A-Z][A-Za-z0-9]*$/;

/** Splits a path into segments, keeping their offsets. */
function splitSegments(
  path: string,
  offset: number,
): { text: string; from: number; to: number }[] {
  const segments: { text: string; from: number; to: number }[] = [];
  let start = 0;
  for (let i = 0; i <= path.length; i++) {
    if (i === path.length || path[i] === "/") {
      // Keep empty spans: `Patient/` has an empty second segment, and that
      // is where the cursor sits.
      segments.push({
        text: path.slice(start, i),
        from: offset + start,
        to: offset + i,
      });
      start = i + 1;
    }
  }
  return segments;
}

/** Parses a request path. Tolerant: half typed text yields a partial shape. */
export function parsePath(text: string): ParsedPath {
  const question = text.indexOf("?");
  const pathText = question === -1 ? text : text.slice(0, question);
  const search = question === -1 ? undefined : text.slice(question + 1);
  const searchFrom = question === -1 ? undefined : question + 1;

  const segments = splitSegments(pathText, 0);
  const names = segments.map((s) => s.text).filter((s) => s.length > 0);

  const parsed: ParsedPath = {
    level: "system",
    interaction: { kind: "resource" },
    pathFrom: 0,
    pathTo: pathText.length,
    segments,
    searchFrom,
    search,
  };

  /** Takes a trailing `_history` or `$op` off the names. */
  const takeInteraction = (parts: string[]) => {
    const last = parts.at(-1);
    if (last === "_history") {
      parsed.interaction = { kind: "history" };
      return parts.slice(0, -1);
    }
    if (last?.startsWith("$")) {
      parsed.interaction = { kind: "operation", operation: last.slice(1) };
      return parts.slice(0, -1);
    }
    return parts;
  };

  // A versioned read is the only shape where `_history` is not last.
  if (
    names.length === 4 &&
    names[2] === "_history" &&
    RESOURCE_TYPE_RE.test(names[0])
  ) {
    parsed.level = "instance";
    parsed.resourceType = names[0];
    parsed.id = names[1];
    parsed.versionId = names[3];
    return parsed;
  }

  const rest = takeInteraction(names);

  if (rest.length === 0) {
    parsed.level = "system";
    return parsed;
  }
  if (!RESOURCE_TYPE_RE.test(rest[0])) {
    // Not a type yet; treat it as one being typed so types are offered.
    parsed.level = "type";
    parsed.resourceType = undefined;
    return parsed;
  }

  parsed.resourceType = rest[0];
  if (rest.length === 1) {
    parsed.level = "type";
  } else {
    parsed.level = "instance";
    parsed.id = rest[1];
  }
  return parsed;
}

/** Where in a path the cursor sits, which decides what is offered. */
export type PathContextKind =
  /** First segment: a type, or `_history` at system level. */
  | "resourceType"
  /** Second segment: an id, `_history` or an operation. */
  | "instanceOrInteraction"
  /** Third segment: `_history` or an operation on an instance. */
  | "instanceInteraction"
  /** Past the `?`, where the search grammar takes over. */
  | "search";

export interface PathContext {
  kind: PathContextKind;
  /** Text already typed in this segment. */
  token: string;
  /** Offsets the completion should replace. */
  from: number;
  to: number;
  /** The parsed path, for a caller that needs the type in scope. */
  path: ParsedPath;
}

/**
 * Which part of a path the cursor at `pos` is in. Returns `search` past a `?`,
 * where the caller hands off to the search grammar.
 */
export function pathContextAt(text: string, pos: number): PathContext {
  const path = parsePath(text);

  if (path.searchFrom !== undefined && pos >= path.searchFrom) {
    return {
      kind: "search",
      token: "",
      from: path.searchFrom,
      to: text.length,
      path,
    };
  }

  // Search from the end: just after a `/` the cursor belongs to the segment
  // starting there, not the one that ended at the same offset.
  let index = -1;
  for (let i = path.segments.length - 1; i >= 0; i--) {
    const s = path.segments[i];
    if (pos >= s.from && pos <= s.to) {
      index = i;
      break;
    }
  }
  if (index === -1) index = Math.max(0, path.segments.length - 1);

  const segment = path.segments[index] ?? { text: "", from: pos, to: pos };

  /** Segment 0 names the type, 1 an id or type level interaction, 2+ an
   * instance level one. */
  const KINDS_BY_INDEX: PathContextKind[] = [
    "resourceType",
    "instanceOrInteraction",
  ];
  const kind: PathContextKind = KINDS_BY_INDEX[index] ?? "instanceInteraction";

  return {
    kind,
    token: text.slice(segment.from, Math.min(pos, segment.to)),
    from: segment.from,
    to: segment.to,
    path,
  };
}
