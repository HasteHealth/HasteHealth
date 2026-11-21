import { resourceTypes } from "@haste-health/fhir-types/r4/sets";
import {
  OperationDefinition,
  OperationDefinitionParameter,
  Parameters,
  ParametersParameter,
} from "@haste-health/fhir-types/r4/types";

export type ParameterUse = "in" | "out";

/** The levels an operation can be invoked at, as `[base]`, `[base]/Type` and `[base]/Type/id`. */
export type InvocationLevel = "system" | "type" | "instance";

/** Where an invocation is sent. */
export type InvocationTarget =
  | { level: "system" }
  | { level: "type"; resourceType: string }
  | { level: "instance"; resourceType: string; id: string };

/**
 * The parameters an operation declares for `use`. A `max` of `0` marks a
 * parameter as not used, so it is left out.
 */
export function parameterDefinitions(
  operationDefinition: OperationDefinition | undefined,
  use: ParameterUse,
): OperationDefinitionParameter[] {
  return (
    operationDefinition?.parameter?.filter(
      (parameter) => parameter.use === use && parameter.max !== "0",
    ) ?? []
  );
}

/** The levels an operation allows, in the order they are offered. */
export function invocationLevels(
  operationDefinition: OperationDefinition | undefined,
): InvocationLevel[] {
  const levels: InvocationLevel[] = [];
  if (operationDefinition?.system) levels.push("system");
  if (operationDefinition?.type) levels.push("type");
  if (operationDefinition?.instance) levels.push("instance");
  return levels;
}

/** The path an invocation is posted to, relative to the FHIR base. */
export function invocationPath(
  target: InvocationTarget,
  operationCode: string,
): string {
  switch (target.level) {
    case "system":
      return `$${operationCode}`;
    case "type":
      return `${target.resourceType || "[type]"}/$${operationCode}`;
    case "instance":
      return `${target.resourceType || "[type]"}/${target.id || "[id]"}/$${operationCode}`;
  }
}

/** Whether a target names everything its level needs. */
export function isTargetComplete(target: InvocationTarget): boolean {
  switch (target.level) {
    case "system":
      return true;
    case "type":
      return target.resourceType !== "";
    case "instance":
      return target.resourceType !== "" && target.id.trim() !== "";
  }
}

/** Whether a parameter is carried in `resource` rather than `value[x]`. */
export function isResourceParameter(
  definition: OperationDefinitionParameter,
): boolean {
  const type = definition.type;
  return (
    type === "Any" ||
    type === "Resource" ||
    type === "DomainResource" ||
    (type !== undefined && resourceTypes.has(type))
  );
}

/** Whether a parameter is a group of `part`s rather than a single value. */
export function isPartParameter(
  definition: OperationDefinitionParameter,
): boolean {
  return definition.type === undefined && (definition.part?.length ?? 0) > 0;
}

/** Whether a parameter may appear more than once. */
export function isRepeating(definition: OperationDefinitionParameter): boolean {
  return definition.max !== "1";
}

/** Whether a parameter must be supplied. */
export function isRequired(definition: OperationDefinitionParameter): boolean {
  return definition.min >= 1;
}

/** The `value[x]` field a value of `type` is carried in: `valueString` for `string`. */
export function valueField(type: string): keyof ParametersParameter {
  return `value${type.charAt(0).toUpperCase()}${type.slice(1)}` as keyof ParametersParameter;
}

/** The `value[x]` field an entry carries a value in, if any. */
export function entryValueField(
  entry: ParametersParameter,
): keyof ParametersParameter | undefined {
  return (Object.keys(entry) as (keyof ParametersParameter)[]).find(
    (key) =>
      key.startsWith("value") &&
      !isEmptyValue(entry[key as keyof typeof entry]),
  );
}

/** Whether a value carries nothing: unset, an empty string, or an empty object or array. */
export function isEmptyValue(value: unknown): boolean {
  if (value === undefined || value === null || value === "") return true;
  if (Array.isArray(value)) return value.every(isEmptyValue);
  if (typeof value === "object") {
    return Object.values(value as Record<string, unknown>).every(isEmptyValue);
  }
  return false;
}

/** The value an entry carries for its definition. */
export function entryValue(
  definition: OperationDefinitionParameter,
  entry: ParametersParameter | undefined,
): unknown {
  if (!entry) return undefined;
  if (isResourceParameter(definition)) return entry.resource;
  if (definition.type) return entry[valueField(definition.type)];
  return entry.part;
}

/**
 * `entry` carrying `value` in the field its definition calls for. Any other
 * value, resource or part is dropped, since a parameter carries exactly one.
 */
export function withEntryValue(
  definition: OperationDefinitionParameter,
  entry: ParametersParameter | undefined,
  value: unknown,
): ParametersParameter {
  const kept: ParametersParameter = { name: definition.name };
  if (entry?.id) kept.id = entry.id;
  if (entry?.extension) kept.extension = entry.extension;

  if (isResourceParameter(definition)) {
    return { ...kept, resource: value as ParametersParameter["resource"] };
  }
  if (definition.type) {
    return { ...kept, [valueField(definition.type)]: value };
  }
  return { ...kept, part: value as ParametersParameter["part"] };
}

/** Whether an entry carries nothing: no value, no resource and no part with content. */
export function isEmptyEntry(entry: ParametersParameter): boolean {
  if (entry.resource !== undefined) return false;
  if (entryValueField(entry) !== undefined) return false;
  return (entry.part ?? []).every(isEmptyEntry);
}

function pruneList(
  list: ParametersParameter[] | undefined,
): ParametersParameter[] | undefined {
  const pruned = (list ?? [])
    .map((entry) =>
      entry.part ? { ...entry, part: pruneList(entry.part) } : entry,
    )
    .filter((entry) => !isEmptyEntry(entry));
  return pruned.length > 0 ? pruned : undefined;
}

/**
 * `parameters` without the entries that carry nothing, which a form leaves
 * behind for values not filled in yet. This is what is sent.
 */
export function pruneParameters(parameters: Parameters): Parameters {
  const parameter = pruneList(parameters.parameter);
  const { parameter: _dropped, ...rest } = parameters;
  return parameter ? { ...rest, parameter } : rest;
}

/**
 * `list` with the entries for `name` replaced by `entries`. Entries are kept in
 * the order the definitions declare them, and entries no definition declares
 * stay at the end, so nothing typed into the JSON view is lost.
 */
export function replaceEntries(
  list: ParametersParameter[],
  definitions: OperationDefinitionParameter[],
  name: string,
  entries: ParametersParameter[],
): ParametersParameter[] {
  const declared = new Set(definitions.map((definition) => definition.name));
  const ordered = definitions.flatMap((definition) =>
    definition.name === name
      ? entries
      : list.filter((entry) => entry.name === definition.name),
  );
  const undeclared = list.filter(
    (entry) =>
      !declared.has(entry.name as OperationDefinitionParameter["name"]),
  );
  return [...ordered, ...undeclared];
}

/** The names of entries in `list` that no definition declares. */
export function undeclaredNames(
  list: ParametersParameter[] | undefined,
  definitions: OperationDefinitionParameter[],
): string[] {
  const declared = new Set<string>(
    definitions.map((definition) => definition.name),
  );
  return [
    ...new Set(
      (list ?? [])
        .map((entry) => entry.name)
        .filter((name) => !declared.has(name)),
    ),
  ];
}

/** The required parameters in `list` that have no value yet. */
export function missingRequired(
  list: ParametersParameter[] | undefined,
  definitions: OperationDefinitionParameter[],
): string[] {
  return definitions
    .filter(isRequired)
    .filter(
      (definition) =>
        !(list ?? []).some(
          (entry) => entry.name === definition.name && !isEmptyEntry(entry),
        ),
    )
    .map((definition) => definition.name);
}

/** `0..1`, `1..*`. */
export function cardinality(definition: OperationDefinitionParameter): string {
  return `${definition.min}..${definition.max}`;
}

/** What a parameter holds, for display: its type, `Resource` or `parts`. */
export function typeLabel(definition: OperationDefinitionParameter): string {
  if (definition.type) {
    return definition.searchType
      ? `${definition.type} (${definition.searchType} search)`
      : definition.type;
  }
  return "parts";
}

/** Whether a response is a Parameters resource rather than a bare resource. */
export function isParameters(value: unknown): value is Parameters {
  return (
    typeof value === "object" &&
    value !== null &&
    (value as { resourceType?: unknown }).resourceType === "Parameters"
  );
}
