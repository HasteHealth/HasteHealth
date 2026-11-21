/**
 * Parses `Type?name(:modifier)?(.chain)*=(prefix)?value(,value)*` search text,
 * e.g. `Patient?name:exact=Smith&birthdate=ge2010-01-01`.
 *
 * Tolerant by design: it runs on every keystroke, so half typed text yields
 * partial tokens rather than errors.
 */

/** A `name=value` pair, with each piece's offsets in the source. */
export interface ParsedParameterToken {
  /** Whole parameter; `from` inclusive, `to` exclusive. */
  from: number;
  to: number;
  /** Name without modifier or chain, e.g. `name` in `name:exact`. */
  name: string;
  nameFrom: number;
  nameTo: number;
  /**
   * Names after the base one, e.g. `["organization", "name"]` for
   * `subject:Patient.organization.name`. Empty when unchained.
   */
  chain: string[];
  /** Modifier after `:`, without the colon. */
  modifier?: string;
  /** Present whenever a `:` was typed. */
  modifierFrom?: number;
  modifierTo?: number;
  /** Everything after `=`, if an `=` has been typed. */
  value?: string;
  valueFrom?: number;
  valueTo?: number;
}

/** A search string split into resource type and parameters. */
export interface ParsedSearch {
  /** Type before the `?`; absent for a system level search. */
  resourceType?: string;
  resourceTypeFrom: number;
  resourceTypeTo: number;
  /** True once a `?` has been typed. */
  hasQuestionMark: boolean;
  parameters: ParsedParameterToken[];
}

/** Splits `text` on `separators`, keeping each span's offsets. */
function splitWithOffsets(
  text: string,
  offset: number,
  separators: string[],
): { text: string; from: number; to: number }[] {
  const spans: { text: string; from: number; to: number }[] = [];
  let start = 0;
  for (let i = 0; i <= text.length; i++) {
    if (i === text.length || separators.includes(text[i])) {
      spans.push({
        text: text.slice(start, i),
        from: offset + start,
        to: offset + i,
      });
      start = i + 1;
    }
  }
  return spans;
}

/** Parses one `name:modifier.chain=value` span into a token. */
function parseParameter(
  text: string,
  from: number,
  to: number,
): ParsedParameterToken {
  const equals = text.indexOf("=");
  const left = equals === -1 ? text : text.slice(0, equals);
  const token: ParsedParameterToken = {
    from,
    to,
    name: "",
    nameFrom: from,
    nameTo: from,
    chain: [],
  };

  // `:` binds tighter than `.`: `subject:Patient.name` is a type modifier on
  // `subject` plus the chain `name`.
  const colon = left.indexOf(":");
  const nameAndChain = colon === -1 ? left : left.slice(0, colon);
  const [name, ...chain] = nameAndChain.split(".");
  token.name = name;
  token.nameFrom = from;
  token.nameTo = from + name.length;
  token.chain = chain;

  if (colon !== -1) {
    // The modifier ends at the next `.` or the `=`.
    const afterColon = left.slice(colon + 1);
    const dot = afterColon.indexOf(".");
    const modifier = dot === -1 ? afterColon : afterColon.slice(0, dot);
    token.modifier = modifier;
    token.modifierFrom = from + colon + 1;
    token.modifierTo = from + colon + 1 + modifier.length;
    if (dot !== -1) {
      token.chain = afterColon.slice(dot + 1).split(".");
    }
  }

  if (equals !== -1) {
    token.value = text.slice(equals + 1);
    token.valueFrom = from + equals + 1;
    token.valueTo = to;
  }

  return token;
}

/**
 * Parses a search string. Never throws.
 *
 * `parametersOnly` reads text with no `?` as parameters rather than as a
 * resource type, for an input locked to one type.
 */
export function parseSearch(
  text: string,
  parametersOnly = false,
): ParsedSearch {
  const question = text.indexOf("?");
  const hasQuestionMark = question !== -1;
  const head = hasQuestionMark ? text.slice(0, question) : text;

  // Without a `?` the text is either a type being typed or a system level
  // parameter like `_id=1`; a character no type can contain decides it.
  const looksLikeParameters =
    !hasQuestionMark &&
    (parametersOnly || (/[=&:._]/.test(head) && !/^[A-Z]/.test(head)));

  const resourceTypeText = looksLikeParameters ? "" : head;

  let parametersText = "";
  let parametersOffset = text.length;
  if (hasQuestionMark) {
    parametersText = text.slice(question + 1);
    parametersOffset = question + 1;
  } else if (looksLikeParameters) {
    parametersText = head;
    parametersOffset = 0;
  }

  // Otherwise the text is a bare resource type, with no parameters yet.
  const hasParameterText = hasQuestionMark || looksLikeParameters;

  return {
    resourceType: resourceTypeText === "" ? undefined : resourceTypeText,
    resourceTypeFrom: 0,
    resourceTypeTo: resourceTypeText.length,
    hasQuestionMark,
    parameters: hasParameterText
      ? splitWithOffsets(parametersText, parametersOffset, ["&"]).map((span) =>
          parseParameter(span.text, span.from, span.to),
        )
      : [],
  };
}

/** Where the cursor sits, which decides what is completed. */
export type SearchContextKind =
  /** Before any `?`. */
  | "resourceType"
  /** A parameter name, or a chained name after a `.`. */
  | "parameterName"
  /** After a `:`. */
  | "modifier"
  /** After the `=`, in one comma separated value. */
  | "value";

export interface SearchContext {
  kind: SearchContextKind;
  /** Type in scope, from the text or the `type` prop. */
  resourceType?: string;
  /** The parameter the cursor is in. */
  parameter?: ParsedParameterToken;
  /** Text already typed at this position. */
  token: string;
  /** Offsets the completion should replace. */
  from: number;
  to: number;
  /**
   * For a chained `parameterName`, the names before the cursor; the caller
   * resolves them against the referenced type.
   */
  chainPrefix: string[];
  /** For `value`, which comma separated value the cursor is in. */
  valueIndex?: number;
}

/**
 * What the cursor at `pos` is positioned to complete.
 *
 * `defaultResourceType` stands in when the input is scoped to one type and the
 * text names none.
 */
export function contextAt(
  text: string,
  pos: number,
  defaultResourceType?: string,
): SearchContext {
  // A locked type means the text is parameters only.
  const parsed = parseSearch(text, Boolean(defaultResourceType));
  const resourceType = parsed.resourceType ?? defaultResourceType;

  // The cursor names the type when it is before the `?`, or when there is no
  // `?` and nothing parsed as a parameter.
  const inResourceTypeText =
    pos <= parsed.resourceTypeTo &&
    (parsed.hasQuestionMark || parsed.parameters.length === 0);
  if (inResourceTypeText && !defaultResourceType) {
    return {
      kind: "resourceType",
      resourceType,
      token: text.slice(0, pos),
      from: 0,
      to: parsed.resourceTypeTo,
      chainPrefix: [],
    };
  }

  const parameter =
    parsed.parameters.find((p) => pos >= p.from && pos <= p.to) ??
    parsed.parameters.at(-1);

  if (!parameter) {
    return {
      kind: "parameterName",
      resourceType,
      token: "",
      from: pos,
      to: pos,
      chainPrefix: [],
    };
  }

  // After the `=`.
  if (parameter.valueFrom !== undefined && pos >= parameter.valueFrom) {
    const value = parameter.value ?? "";
    const relative = pos - parameter.valueFrom;
    // Complete only the value the cursor is in.
    const start = value.lastIndexOf(",", Math.max(0, relative - 1)) + 1;
    const comma = value.indexOf(",", relative);
    const end = comma === -1 ? value.length : comma;
    return {
      kind: "value",
      resourceType,
      parameter,
      token: value.slice(start, relative),
      from: parameter.valueFrom + start,
      to: parameter.valueFrom + end,
      chainPrefix: parameter.chain,
      valueIndex: value.slice(0, start).split(",").length - 1,
    };
  }

  // Between the `:` and any `.` or `=`.
  if (
    parameter.modifierFrom !== undefined &&
    parameter.modifierTo !== undefined &&
    pos >= parameter.modifierFrom &&
    pos <= parameter.modifierTo
  ) {
    return {
      kind: "modifier",
      resourceType,
      parameter,
      token: (parameter.modifier ?? "").slice(0, pos - parameter.modifierFrom),
      from: parameter.modifierFrom,
      to: parameter.modifierTo,
      chainPrefix: [],
    };
  }

  // Otherwise a name, possibly a link in a chain.
  const nameText = text.slice(parameter.from, pos);
  const lastDot = nameText.lastIndexOf(".");
  const tokenStart =
    lastDot === -1 ? parameter.nameFrom : parameter.from + lastDot + 1;
  // The name ends at the next `:`, `.` or `=`.
  const rest = text.slice(pos, parameter.to);
  const endMatch = /[:.=]/.exec(rest);
  const tokenEnd = endMatch ? pos + endMatch.index : parameter.to;

  // Dotted names between the base name and the cursor. The first segment is
  // the base name and carries any `:modifier`, so both are dropped.
  const chainPrefix =
    lastDot === -1
      ? []
      : nameText
          .slice(0, lastDot)
          .split(".")
          .slice(1)
          .filter((link) => link.length > 0);

  return {
    kind: "parameterName",
    resourceType,
    parameter,
    token: text.slice(tokenStart, pos),
    from: tokenStart,
    to: tokenEnd,
    // `resolveChain` starts at the base parameter.
    chainPrefix: lastDot === -1 ? [] : [parameter.name, ...chainPrefix],
  };
}
