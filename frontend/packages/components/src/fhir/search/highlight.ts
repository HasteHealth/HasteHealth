/**
 * Syntax highlighting for the search input.
 *
 * The grammar is small enough that decorating the ranges {@link parseSearch}
 * already reports beats defining a Lezer grammar.
 */
import { RangeSetBuilder } from "@codemirror/state";
import {
  Decoration,
  DecorationSet,
  EditorView,
  ViewPlugin,
  ViewUpdate,
} from "@codemirror/view";

import { ParsedParameterToken, parseSearch } from "./grammar";
import { RESOURCE_TYPE_RE, parsePath } from "./path";

const resourceTypeMark = Decoration.mark({ class: "cm-fhir-resource-type" });
const parameterMark = Decoration.mark({ class: "cm-fhir-parameter" });
const modifierMark = Decoration.mark({ class: "cm-fhir-modifier" });
const valueMark = Decoration.mark({ class: "cm-fhir-value" });
const punctuationMark = Decoration.mark({ class: "cm-fhir-punctuation" });
const idMark = Decoration.mark({ class: "cm-fhir-id" });
const interactionMark = Decoration.mark({ class: "cm-fhir-interaction" });

/** Which mark a path segment gets: the type, an interaction, or an id. */
function pathSegmentMark(isType: boolean, isInteraction: boolean): Decoration {
  if (isType) return resourceTypeMark;
  return isInteraction ? interactionMark : idMark;
}

/**
 * Decorations for a request path. The search half goes to
 * {@link decorateSearch} with its offsets shifted.
 */
function decoratePath(
  text: string,
): { from: number; to: number; mark: Decoration }[] {
  const marks: { from: number; to: number; mark: Decoration }[] = [];
  const parsed = parsePath(text);

  for (const [index, segment] of parsed.segments.entries()) {
    if (segment.to <= segment.from) continue;
    const isType = index === 0 && RESOURCE_TYPE_RE.test(segment.text);
    const isInteraction =
      segment.text === "_history" || segment.text.startsWith("$");
    marks.push({
      from: segment.from,
      to: segment.to,
      mark: pathSegmentMark(isType, isInteraction),
    });
    // The following `/`.
    if (segment.to < parsed.pathTo) {
      marks.push({
        from: segment.to,
        to: segment.to + 1,
        mark: punctuationMark,
      });
    }
  }

  if (parsed.searchFrom !== undefined) {
    marks.push({
      from: parsed.searchFrom - 1,
      to: parsed.searchFrom,
      mark: punctuationMark,
    });
    for (const mark of decorateSearch(parsed.search ?? "", true)) {
      marks.push({
        from: mark.from + parsed.searchFrom,
        to: mark.to + parsed.searchFrom,
        mark: mark.mark,
      });
    }
  }

  return marks;
}

/** Decorations for search text, offset within that text. */
function decorateSearch(
  text: string,
  parametersOnly: boolean,
): { from: number; to: number; mark: Decoration }[] {
  const marks: { from: number; to: number; mark: Decoration }[] = [];
  const parsed = parseSearch(text, parametersOnly);

  if (parsed.resourceType && parsed.resourceTypeTo > parsed.resourceTypeFrom) {
    marks.push({
      from: parsed.resourceTypeFrom,
      to: parsed.resourceTypeTo,
      mark: resourceTypeMark,
    });
  }
  if (parsed.hasQuestionMark) {
    const question = text.indexOf("?");
    marks.push({ from: question, to: question + 1, mark: punctuationMark });
  }

  for (const parameter of parsed.parameters) {
    marks.push(...decorateParameter(parameter, text));
  }

  return marks;
}

/**
 * Decorations for one `name:modifier=value` token: each piece, and the `:`,
 * `=` and trailing `&` around them.
 */
function decorateParameter(
  parameter: ParsedParameterToken,
  text: string,
): { from: number; to: number; mark: Decoration }[] {
  const marks: { from: number; to: number; mark: Decoration }[] = [];

  if (parameter.nameTo > parameter.nameFrom) {
    marks.push({
      from: parameter.nameFrom,
      to: parameter.nameTo,
      mark: parameterMark,
    });
  }

  // The separator is painted as soon as it is typed; the part after it only
  // once it has content.
  const { modifierFrom, modifierTo } = parameter;
  if (modifierFrom !== undefined && modifierTo !== undefined) {
    marks.push({
      from: modifierFrom - 1,
      to: modifierFrom,
      mark: punctuationMark,
    });
    if (modifierTo > modifierFrom) {
      marks.push({ from: modifierFrom, to: modifierTo, mark: modifierMark });
    }
  }

  const { valueFrom, valueTo } = parameter;
  if (valueFrom !== undefined && valueTo !== undefined) {
    marks.push({ from: valueFrom - 1, to: valueFrom, mark: punctuationMark });
    if (valueTo > valueFrom) {
      marks.push({ from: valueFrom, to: valueTo, mark: valueMark });
    }
  }

  if (text[parameter.to] === "&") {
    marks.push({
      from: parameter.to,
      to: parameter.to + 1,
      mark: punctuationMark,
    });
  }

  return marks;
}

/** Decorations for the whole document. */
function decorate(
  view: EditorView,
  parametersOnly: boolean,
  pathMode: boolean,
): DecorationSet {
  const text = view.state.doc.toString();
  const builder = new RangeSetBuilder<Decoration>();
  const marks = pathMode
    ? decoratePath(text)
    : decorateSearch(text, parametersOnly);

  // RangeSetBuilder requires ordered ranges.
  const ordered = [...marks].sort((a, b) => a.from - b.from || a.to - b.to);
  for (const mark of ordered) {
    if (mark.to > mark.from) builder.add(mark.from, mark.to, mark.mark);
  }
  return builder.finish();
}

/**
 * Highlights the query as it is typed. Pass `parametersOnly` when the input is
 * locked to a type, so a bare parameter name is not painted as one.
 */
export function fhirSearchHighlighting(
  parametersOnly = false,
  pathMode = false,
) {
  return ViewPlugin.fromClass(
    class {
      decorations: DecorationSet;

      constructor(view: EditorView) {
        this.decorations = decorate(view, parametersOnly, pathMode);
      }

      update(update: ViewUpdate) {
        if (update.docChanged || update.viewportChanged) {
          this.decorations = decorate(update.view, parametersOnly, pathMode);
        }
      }
    },
    { decorations: (plugin) => plugin.decorations },
  );
}

/** Colours for the decorations above, light and dark. */
export const fhirSearchHighlightTheme = EditorView.baseTheme({
  ".cm-fhir-resource-type": { color: "#0f766e", fontWeight: "600" },
  ".cm-fhir-parameter": { color: "#1d4ed8" },
  ".cm-fhir-modifier": { color: "#7c3aed" },
  ".cm-fhir-value": { color: "#b45309" },
  ".cm-fhir-punctuation": { color: "#6b7280" },
  ".cm-fhir-id": { color: "#475569" },
  ".cm-fhir-interaction": { color: "#be185d" },
  "&dark .cm-fhir-resource-type": { color: "#5eead4" },
  "&dark .cm-fhir-parameter": { color: "#93c5fd" },
  "&dark .cm-fhir-modifier": { color: "#c4b5fd" },
  "&dark .cm-fhir-value": { color: "#fcd34d" },
  "&dark .cm-fhir-punctuation": { color: "#9ca3af" },
  "&dark .cm-fhir-id": { color: "#cbd5e1" },
  "&dark .cm-fhir-interaction": { color: "#f9a8d4" },
});
