/**
 * What the command bar runs.
 *
 * The console has one input and everything reachable through it is a
 * `Command`: a verb plus a target. Splitting it that way is what lets
 * `GET Patient?name=Smith` and `DELETE Patient?name=Smith` be the same target
 * with different consequences, and it is what makes conditional writes - which
 * are exactly "a write whose target is a search" - fall out rather than need
 * their own shape.
 *
 * The workspace picks its view from the target level and the response, so
 * parsing happens here, once.
 */

/** The verbs the console can send. */
export type Verb = "GET" | "POST" | "PUT" | "PATCH" | "DELETE";

export const VERBS: Verb[] = ["GET", "POST", "PUT", "PATCH", "DELETE"];

/** Verbs that carry a request body. */
export const VERBS_WITH_BODY: Verb[] = ["POST", "PUT", "PATCH"];

/**
 * What a command points at. The three levels mirror FHIR's own: the whole
 * server, a resource type, or one instance. A `search` on a type or instance
 * level target is what makes a write conditional.
 */
export type Target =
  /** The server as a whole: `_history`, `$export`, a transaction bundle. */
  | { level: "system"; search?: string }
  /** A resource type: `Patient`, `Patient?name=Smith`. */
  | { level: "type"; resourceType: string; search?: string }
  /** One instance: `Patient/123`, optionally one version of it. */
  | {
      level: "instance";
      resourceType: string;
      id: string;
      versionId?: string;
    };

/** A history listing, an operation, or the plain interaction for the verb. */
export type Interaction =
  | { kind: "resource" }
  | { kind: "history" }
  /** `$name`, invoked at whatever level the target names. */
  | { kind: "operation"; operation: string };

export interface Command {
  verb: Verb;
  target: Target;
  interaction: Interaction;
  /** Request body for the verbs that take one, as JSON text. */
  body?: string;
}

/** A command the user ran, as kept in the recency list. */
export interface CommandHistoryEntry {
  command: Command;
  /** Epoch milliseconds the command was last run. */
  at: number;
}

/** The path part of a command, without the verb. */
export function targetPath(command: Command): string {
  const { target, interaction } = command;

  const base = (() => {
    switch (target.level) {
      case "system":
        return "";
      case "type":
        return target.resourceType;
      case "instance":
        return target.versionId
          ? `${target.resourceType}/${target.id}/_history/${target.versionId}`
          : `${target.resourceType}/${target.id}`;
    }
  })();

  const suffix = (() => {
    switch (interaction.kind) {
      case "resource":
        return "";
      case "history":
        return "_history";
      case "operation":
        return `$${interaction.operation}`;
    }
  })();

  const path = [base, suffix].filter((p) => p.length > 0).join("/");
  // A search on a type or system target is the conditional part of a
  // conditional write, and the query part of a plain search.
  const search = target.level === "instance" ? undefined : target.search;
  const query = search ? `?${search}` : "";
  return `${path}${query}`;
}

/**
 * The command as the user would type it: the verb, then the path. `GET` is
 * left implicit because reading is the common case and typing it every time
 * would be noise.
 */
export function describeCommand(command: Command): string {
  const path = targetPath(command);
  return command.verb === "GET" ? path : `${command.verb} ${path}`;
}

/** True when two commands are the same request, ignoring when they were run. */
export function sameCommand(a: Command, b: Command): boolean {
  return describeCommand(a) === describeCommand(b);
}

/** The HTTP request a command turns into, shown beneath the command bar. */
export function describeRequest(command: Command): {
  method: string;
  path: string;
} {
  return { method: command.verb, path: `/${targetPath(command)}` };
}

/**
 * Whether a command changes data, which decides whether it runs on Enter or
 * has to be confirmed first.
 */
export function isMutation(command: Command): boolean {
  return command.verb !== "GET";
}

/**
 * Whether a command acts on whatever a search matches rather than on one
 * named resource - a conditional update, patch or delete. `POST` is excluded:
 * creating at type level is how every create is written, and is not
 * conditional on anything.
 *
 * A conditional delete with no search at all is a delete of the entire type,
 * which is the most destructive thing the console can send.
 */
export function isConditional(command: Command): boolean {
  return (
    isMutation(command) &&
    command.verb !== "POST" &&
    command.target.level !== "instance" &&
    command.interaction.kind === "resource"
  );
}

/** What a DELETE would remove. */
function deleteWarning(target: Target): string {
  switch (target.level) {
    case "system":
      return "Deletes across the whole server.";
    case "type":
      return target.search
        ? `Deletes every ${target.resourceType} matching this search.`
        : `Deletes every ${target.resourceType} on the server.`;
    case "instance":
      return "Deletes this resource.";
  }
}

/** What a POST would create. */
function createWarning(target: Target): string {
  if (target.level === "system") return "Submits a bundle to the server.";
  const what = target.level === "type" ? target.resourceType : "resource";
  return `Creates a new ${what}.`;
}

/** What a conditional PUT or PATCH would act on. */
function conditionalWriteWarning(verb: Verb, target: Target): string {
  const action = verb === "PATCH" ? "Patches" : "Updates";
  const upsert = verb === "PUT" ? ", or creates one if none match" : "";
  const type = target.level === "type" ? target.resourceType : "resource";
  return `${action} every ${type} matching this search${upsert}.`;
}

/** A plain-language warning for a mutation, or `undefined` when there is none. */
export function mutationWarning(command: Command): string | undefined {
  if (!isMutation(command)) return undefined;
  const { verb, target, interaction } = command;

  // An operation's effect is its own business; we cannot describe it from
  // the name, so say only that it runs.
  if (interaction.kind === "operation") {
    return `Invokes $${interaction.operation}, which may change data.`;
  }

  if (verb === "DELETE") return deleteWarning(target);
  if (verb === "POST") return createWarning(target);
  if (target.level === "type" && target.search) {
    return conditionalWriteWarning(verb, target);
  }
  if (verb === "PUT") return "Replaces this resource.";
  if (verb === "PATCH") return "Applies a patch to this resource.";
  return undefined;
}

/**
 * Parses what was typed into the command bar.
 *
 * The syntax is FHIR's own URL shape with an optional leading verb, so
 * `Patient?name=Smith`, `Patient/123`, `DELETE Patient/123`,
 * `PUT Patient?identifier=mrn|1` and `_history` all mean what someone reading
 * the FHIR spec would expect. Returns `undefined` for text that is not a
 * command yet, which is the normal state while typing.
 */
export function parseCommand(text: string): Command | undefined {
  let rest = text.trim();
  if (rest === "") return undefined;

  // A verb on its own is a command still being typed, not a resource type
  // that happens to be spelled `GET`.
  if (VERBS.includes(rest.toUpperCase() as Verb)) return undefined;

  // An explicit verb, if one was typed.
  let verb: Verb = "GET";
  const verbMatch = /^([A-Za-z]{1,10})\s+(.*)$/.exec(rest);
  if (verbMatch) {
    const candidate = verbMatch[1].toUpperCase() as Verb;
    if (VERBS.includes(candidate)) {
      verb = candidate;
      rest = verbMatch[2].trim();
    }
  }
  if (rest === "") return undefined;

  rest = rest.replace(/^\/+/, "");
  const [pathPart, ...queryParts] = rest.split("?");
  const search = queryParts.join("?") || undefined;
  const segments = pathPart.split("/").filter((s) => s.length > 0);

  /** Pulls a trailing `_history` or `$op` off the segments. */
  const takeInteraction = (
    parts: string[],
  ): { interaction: Interaction; rest: string[] } => {
    const last = parts.at(-1);
    if (last === "_history") {
      return { interaction: { kind: "history" }, rest: parts.slice(0, -1) };
    }
    if (last?.startsWith("$")) {
      return {
        interaction: { kind: "operation", operation: last.slice(1) },
        rest: parts.slice(0, -1),
      };
    }
    return { interaction: { kind: "resource" }, rest: parts };
  };

  // A versioned read is the one shape where `_history` is not trailing.
  const versioned = /^([A-Z][A-Za-z0-9]*)\/([^/]+)\/_history\/([^/]+)$/.exec(
    pathPart,
  );
  if (versioned) {
    return {
      verb,
      target: {
        level: "instance",
        resourceType: versioned[1],
        id: versioned[2],
        versionId: versioned[3],
      },
      interaction: { kind: "resource" },
    };
  }

  const { interaction, rest: path } = takeInteraction(segments);

  // Nothing left: a system level interaction such as `_history` or `$export`,
  // or a bare verb against the server root for a transaction bundle.
  if (path.length === 0) {
    if (interaction.kind === "resource" && verb === "GET") return undefined;
    return { verb, target: { level: "system", search }, interaction };
  }

  const [resourceType, id] = path;
  // Digits are part of several real type names - AccessPolicyV2,
  // AccessPolicyV2Assignment, HL7V2 - so they cannot be excluded here.
  if (!/^[A-Z][A-Za-z0-9]*$/.test(resourceType)) return undefined;

  if (path.length === 1) {
    return {
      verb,
      target: { level: "type", resourceType, search },
      interaction,
    };
  }
  if (path.length === 2) {
    return {
      verb,
      target: { level: "instance", resourceType, id },
      interaction,
    };
  }
  return undefined;
}

/**
 * Builds the path a command lives at.
 *
 * Only reads live at a URL: a mutation is an action, not a place, and putting
 * one in the address bar would mean a refresh re-sends it.
 */
export function commandToPath(command: Command): string {
  return `/r/${targetPath(command)}`;
}

/**
 * Reads a command back out of a location. Returns `undefined` for a path that
 * is not the console, such as settings or an import.
 */
export function commandFromLocation(
  pathname: string,
  search: string,
): Command | undefined {
  if (!pathname.startsWith("/r/")) return undefined;
  const rest = pathname.slice("/r/".length);
  const parameters = search.startsWith("?") ? search.slice(1) : search;
  return parseCommand(parameters ? `${rest}?${parameters}` : rest);
}

/** Convenience for the sidebar and links: a plain search of one type. */
export function searchCommand(resourceType: string, search?: string): Command {
  return {
    verb: "GET",
    target: { level: "type", resourceType, search },
    interaction: { kind: "resource" },
  };
}

/** What a history link points at. */
export type HistoryScope =
  | { level: "system" }
  | { level: "type"; resourceType: string }
  | { level: "instance"; resourceType: string; id: string };

/** `scope` as a command target. */
function targetForScope(scope: HistoryScope): Target {
  switch (scope.level) {
    case "system":
      return { level: "system" };
    case "type":
      return { level: "type", resourceType: scope.resourceType };
    case "instance":
      return {
        level: "instance",
        resourceType: scope.resourceType,
        id: scope.id,
      };
  }
}

/** Convenience for history links at any level. */
export function historyCommand(scope: HistoryScope): Command {
  return {
    verb: "GET",
    target: targetForScope(scope),
    interaction: { kind: "history" },
  };
}
