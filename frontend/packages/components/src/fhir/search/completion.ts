/**
 * CodeMirror completion sources behind {@link FHIRSearchInput}: resource types
 * at the root, search parameters after the `?`, modifiers after a `:`, and
 * prefixes or codes after the `=`, as {@link contextAt} directs.
 */
import type {
  Completion,
  CompletionContext,
  CompletionResult,
} from "@codemirror/autocomplete";

import { SearchContext, contextAt } from "./grammar";
import { pathContextAt } from "./path";
import {
  MISSING_VALUES,
  RESULT_PARAMETERS,
  SearchMetadata,
  SearchParameterResource,
  Suggestion,
  modifiersForType,
  prefixesForType,
  referenceTargets,
  valuesForParameterName,
} from "./metadata";

/** Suggestions as CodeMirror completions of one kind. */
function toCompletions(
  suggestions: Suggestion[],
  type: string,
  apply?: (label: string) => string,
): Completion[] {
  return suggestions.map((suggestion) => ({
    label: suggestion.label,
    type,
    detail: suggestion.detail,
    info: suggestion.info,
    boost: suggestion.boost,
    apply: apply ? apply(suggestion.label) : undefined,
  }));
}

/** A search parameter as a suggestion, showing its type. */
function parameterSuggestion(parameter: SearchParameterResource): Suggestion {
  return {
    label: parameter.code as string,
    detail: parameter.type as string,
    info: parameter.description as string | undefined,
  };
}

/**
 * Resource types a chain such as `subject.organization` can land on.
 *
 * Only reference parameters chain, so a link that is not one ends the walk
 * empty. Several targets are all kept unless a `:Type` modifier picks one.
 */
async function resolveChain(
  metadata: SearchMetadata,
  resourceType: string | undefined,
  chain: string[],
  /** Type modifier on the base parameter, as in `subject:Patient.name`. */
  typeModifier?: string,
): Promise<string[]> {
  let current = resourceType ? [resourceType] : [];
  for (const [index, link] of chain.entries()) {
    const parameters = await Promise.all(
      current.map((type) => metadata.getParameter(type, link)),
    );
    // Every type this link reaches from the types in hand.
    const targets = [
      ...new Set(
        parameters.flatMap((parameter) => referenceTargets(parameter)),
      ),
    ];
    // A `:Type` modifier narrows only the link it was written on.
    current =
      index === 0 && typeModifier && targets.includes(typeModifier)
        ? [typeModifier]
        : targets;
    if (current.length === 0) return [];
  }
  return current;
}

/** Parameter names, including chained ones. */
async function completeParameterName(
  metadata: SearchMetadata,
  context: SearchContext,
): Promise<Completion[]> {
  // In a chain the names come from the resource the chain reached.
  if (context.chainPrefix.length > 0) {
    const targets = await resolveChain(
      metadata,
      context.resourceType,
      context.chainPrefix,
      context.parameter?.modifier,
    );
    if (targets.length === 0) return [];

    const perTarget = await Promise.all(
      targets.map(async (target) =>
        (await metadata.getParameters(target)).map((parameter) => ({
          target,
          parameter,
        })),
      ),
    );

    // Offer a shared parameter once, naming the types it came from.
    const byCode = new Map<
      string,
      { targets: string[]; suggestion: Suggestion }
    >();
    for (const { target, parameter } of perTarget.flat()) {
      const code = parameter.code as string;
      const existing = byCode.get(code);
      if (existing) {
        if (!existing.targets.includes(target)) existing.targets.push(target);
      } else {
        byCode.set(code, {
          targets: [target],
          suggestion: parameterSuggestion(parameter),
        });
      }
    }

    return toCompletions(
      [...byCode.values()].map(({ targets: from, suggestion }) => ({
        ...suggestion,
        detail:
          targets.length > 1
            ? `${suggestion.detail} · ${from.join(", ")}`
            : suggestion.detail,
      })),
      "property",
    );
  }

  const parameters = await metadata.getParameters(context.resourceType);
  return [
    ...parameters.map((parameter) => {
      const suggestion = parameterSuggestion(parameter);
      // A reference is more often given a value than chained, so insert the
      // name as is and only advertise the chain in the detail.
      const chainable = parameter.type === "reference";
      return {
        label: suggestion.label,
        type: "property",
        detail: chainable ? `${suggestion.detail} ·chain` : suggestion.detail,
        info: chainable
          ? `${suggestion.info ?? ""}\n\nChain with \`.\` to search the referenced resource.`.trim()
          : suggestion.info,
      };
    }),
    // Valid everywhere, but sorted below the real parameters.
    ...toCompletions(
      RESULT_PARAMETERS.map((p) => ({ ...p, boost: -1 })),
      "keyword",
    ),
  ];
}

/**
 * Completions after a `:`: modifiers, plus a reference's target types, since
 * `subject:Patient` narrows the type.
 */
async function completeModifier(
  metadata: SearchMetadata,
  context: SearchContext,
): Promise<Completion[]> {
  const parameter = await metadata.getParameter(
    context.resourceType,
    context.parameter?.name ?? "",
  );
  const modifiers = toCompletions(
    modifiersForType(parameter?.type as string | undefined),
    "keyword",
  );
  const targets = toCompletions(
    referenceTargets(parameter).map((target) => ({
      label: target,
      detail: "type",
      info: `Only match references to ${target}.`,
    })),
    "class",
  );
  return [...modifiers, ...targets];
}

/** Completions, and optionally a narrower range to replace. */
interface ValueCompletions {
  options: Completion[];
  /** Overrides the context's `to` to replace part of the value. */
  to?: number;
}

/**
 * Completions after the `=`: fixed codes where a parameter has them, and
 * prefixes for the ordered types while the value is still empty or partial.
 */
async function completeValue(
  metadata: SearchMetadata,
  context: SearchContext,
): Promise<ValueCompletions> {
  const name = context.parameter?.name ?? "";

  // `:missing` takes a boolean whatever the parameter's type.
  if (context.parameter?.modifier === "missing") {
    return { options: toCompletions(MISSING_VALUES, "constant") };
  }

  const fixed = valuesForParameterName(name);
  if (fixed.length > 0) return { options: toCompletions(fixed, "constant") };

  // `_sort` takes parameter names, optionally `-` prefixed.
  if (name === "_sort") {
    const parameters = await metadata.getParameters(context.resourceType);
    const descending = context.token.startsWith("-");
    return {
      options: toCompletions(
        parameters.map((parameter) => ({
          ...parameterSuggestion(parameter),
          label: `${descending ? "-" : ""}${parameter.code as string}`,
        })),
        "property",
      ),
    };
  }

  const parameter = await metadata.getParameter(context.resourceType, name);
  const type = parameter?.type as string | undefined;

  // A reference value is a type or `Type/id`.
  if (type === "reference") {
    return {
      options: toCompletions(
        referenceTargets(parameter).map((target) => ({
          label: `${target}/`,
          detail: "reference",
          info: `A reference to a ${target}.`,
        })),
        "class",
      ),
    };
  }

  // Prefixes only start a value.
  const prefixes = prefixesForType(type);
  const isPrefixPosition =
    context.token.length <= 2 &&
    prefixes.some((prefix) => prefix.label.startsWith(context.token));
  if (prefixes.length > 0 && isPrefixPosition) {
    // Replace only the prefix typed: accepting `ge` on `ge|2010` must not
    // drop the date.
    return {
      options: toCompletions(prefixes, "keyword"),
      to: context.from + context.token.length,
    };
  }

  return { options: [] };
}

/** Options for {@link fhirSearchCompletions}. */
export interface FHIRSearchCompletionOptions {
  metadata: SearchMetadata;
  /** Type the input is locked to; the text is then parameters only. */
  resourceType?: string;
}

/**
 * Builds the search completion source. Exported for a caller with its own
 * CodeMirror setup.
 */
export function fhirSearchCompletions({
  metadata,
  resourceType,
}: FHIRSearchCompletionOptions) {
  return async function complete(
    cmContext: CompletionContext,
  ): Promise<CompletionResult | null> {
    const text = cmContext.state.doc.toString();
    const context = contextAt(text, cmContext.pos, resourceType);

    // Don't open a menu unprompted on an empty position.
    if (!cmContext.explicit && context.token.length === 0 && text.length > 0) {
      // Except right after a separator, where what comes next is exactly
      // what we suggest.
      const previous = text[cmContext.pos - 1];
      if (
        !previous ||
        ![":", "=", ".", "?", "&", ",", "-"].includes(previous)
      ) {
        return null;
      }
    }

    let options: Completion[] = [];
    // The context's range, unless a prefix narrows it.
    let to = context.to;
    switch (context.kind) {
      case "resourceType": {
        // Nothing to complete when the type is locked.
        if (resourceType) return null;
        const types = await metadata.getResourceTypes();
        options = [
          ...toCompletions(
            types.map((type) => ({ label: type, detail: "resource" })),
            "class",
            // Accepting a type opens its parameter list.
            (label) => `${label}?`,
          ),
          // A system level parameter is valid at the root too.
          ...toCompletions(
            RESULT_PARAMETERS.map((p) => ({ ...p, boost: -1 })),
            "keyword",
          ),
        ];
        break;
      }
      case "parameterName":
        options = await completeParameterName(metadata, context);
        break;
      case "modifier":
        options = await completeModifier(metadata, context);
        break;
      case "value": {
        const completions = await completeValue(metadata, context);
        options = completions.options;
        to = completions.to ?? context.to;
        break;
      }
    }

    if (options.length === 0) return null;

    return {
      from: context.from,
      to,
      options,
      // The options are fixed per position, so CodeMirror can filter them.
      validFor: /^[-\w.]*$/,
    };
  };
}

/** Operations the server exposes, by the level they are invoked at. */
export interface OperationCatalog {
  system?: string[];
  type?: Record<string, string[]>;
  instance?: Record<string, string[]>;
}

/** Options for {@link fhirPathCompletions}. */
export interface FHIRPathCompletionOptions {
  metadata: SearchMetadata;
  /** Operations to offer after a `$`; `$` offers nothing without it. */
  operations?: OperationCatalog;
}

/**
 * Completions for a whole request path such as `Patient/123/_history?_count=5`:
 * types, `_history` and operations, handing off to the search source past the
 * `?` with whichever type the path named.
 */
export function fhirPathCompletions({
  metadata,
  operations,
}: FHIRPathCompletionOptions) {
  return async function complete(
    cmContext: CompletionContext,
  ): Promise<CompletionResult | null> {
    const text = cmContext.state.doc.toString();
    const context = pathContextAt(text, cmContext.pos);

    // The search source sees only the search text, so shift its offsets back
    // onto the document.
    if (context.kind === "search") {
      const searchFrom = context.path.searchFrom ?? 0;
      const searchText = text.slice(searchFrom);
      const inner = fhirSearchCompletions({
        metadata,
        resourceType: context.path.resourceType,
      });
      const result = await inner({
        explicit: cmContext.explicit,
        pos: cmContext.pos - searchFrom,
        state: { doc: { toString: () => searchText } },
      } as unknown as CompletionContext);
      if (!result) return null;
      return {
        ...result,
        from: result.from + searchFrom,
        to: (result.to ?? searchText.length) + searchFrom,
      };
    }

    const { path, token } = context;
    const options: Completion[] = [];

    switch (context.kind) {
      case "resourceType": {
        const types = await metadata.getResourceTypes();
        options.push(
          ...toCompletions(
            types.map((type) => ({ label: type, detail: "resource" })),
            "class",
          ),
          // System level interactions live in the first segment too.
          {
            label: "_history",
            type: "keyword",
            detail: "system",
            info: "Everything that changed on the server.",
          },
          ...toCompletions(
            (operations?.system ?? []).map((op) => ({
              label: `$${op}`,
              detail: "operation",
            })),
            "function",
          ),
        );
        break;
      }

      case "instanceOrInteraction": {
        // After `Type/` an id cannot be completed, but these can.
        options.push({
          label: "_history",
          type: "keyword",
          detail: "type",
          info: "Everything that changed for this resource type.",
        });
        if (path.resourceType) {
          options.push(
            ...toCompletions(
              (operations?.type?.[path.resourceType] ?? []).map((op) => ({
                label: `$${op}`,
                detail: "operation",
              })),
              "function",
            ),
          );
        }
        break;
      }

      case "instanceInteraction": {
        options.push({
          label: "_history",
          type: "keyword",
          detail: "instance",
          info: "Every version of this resource.",
        });
        if (path.resourceType) {
          options.push(
            ...toCompletions(
              (operations?.instance?.[path.resourceType] ?? []).map((op) => ({
                label: `$${op}`,
                detail: "operation",
              })),
              "function",
            ),
          );
        }
        break;
      }
    }

    if (options.length === 0) return null;
    // Don't open over an id being typed, but do right after a `/`.
    if (
      !cmContext.explicit &&
      token === "" &&
      context.kind !== "resourceType"
    ) {
      if (text[cmContext.pos - 1] !== "/") return null;
    }

    return {
      from: context.from,
      to: context.to,
      options,
      validFor: /^[$\w-]*$/,
    };
  };
}
