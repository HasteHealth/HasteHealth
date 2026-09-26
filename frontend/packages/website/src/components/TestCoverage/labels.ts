import type { Interaction, ParamType } from "./data";

export const TYPE_ORDER: readonly ParamType[] = [
  "string",
  "token",
  "reference",
  "date",
  "number",
  "quantity",
  "uri",
];

export const TYPE_NAMES: Record<ParamType, string> = {
  string: "String",
  token: "Token",
  reference: "Reference",
  date: "Date",
  number: "Number",
  quantity: "Quantity",
  uri: "URI",
};

export const TYPE_DESCRIPTIONS: Record<ParamType, string> = {
  string: "Names, addresses and other free text",
  token: "Codes and identifiers, with or without a system",
  reference: "Links to other resources",
  date: "Dates, date-times and periods",
  number: "Plain decimal values",
  quantity: "Values with units, such as lab results",
  uri: "Canonical URLs and other URIs",
};

export const INTERACTION_NAMES: Record<Interaction, string> = {
  create: "Create",
  read: "Read",
  update: "Update",
  patch: "Patch",
  delete: "Delete",
};

export const INTERACTION_EXAMPLES: Record<Interaction, string> = {
  create: "POST [type]",
  read: "GET [type]/[id]",
  update: "PUT [type]/[id]",
  patch: "PATCH [type]/[id]",
  delete: "DELETE [type]/[id]",
};

type Feature = { label: string; example: string };

const PREFIXES: Record<string, Feature> = {
  eq: { label: "eq (equals)", example: "=eq2024-01-01" },
  ne: { label: "ne (not equal)", example: "=ne2024-01-01" },
  gt: { label: "gt (greater than)", example: "=gt2024-01-01" },
  lt: { label: "lt (less than)", example: "=lt2024-01-01" },
  ge: { label: "ge (greater or equal)", example: "=ge2024-01-01" },
  le: { label: "le (less or equal)", example: "=le2024-01-01" },
  sa: { label: "sa (starts after)", example: "=sa2024-01-01" },
  eb: { label: "eb (ends before)", example: "=eb2024-01-01" },
  ap: { label: "ap (approximately)", example: "=ap2024-01-01" },
};

const MISSING: Feature = { label: ":missing", example: ":missing=true" };

const FEATURES: Record<ParamType, Record<string, Feature>> = {
  string: {
    prefix: { label: "Starts with (default)", example: "=smi" },
    exact: { label: ":exact", example: ":exact=Smith" },
    contains: { label: ":contains", example: ":contains=mit" },
    missing: MISSING,
  },
  token: {
    default: { label: "code", example: "=12345" },
    "system-code": { label: "system|code", example: "=http://loinc.org|8867-4" },
    "no-system": { label: "|code (no system)", example: "=|12345" },
    "any-code-in-system": { label: "system| (any code)", example: "=http://loinc.org|" },
    "any-token": { label: "| (any value)", example: "=|" },
    not: { label: ":not", example: ":not=http://loinc.org|8867-4" },
    missing: MISSING,
  },
  reference: {
    "id-only": { label: "id", example: "=123" },
    "type-slash-id": { label: "Type/id", example: "=Patient/123" },
    missing: MISSING,
  },
  date: { ...PREFIXES, missing: MISSING },
  number: {
    ...Object.fromEntries(
      Object.entries(PREFIXES).map(([key, feature]) => [
        key,
        { ...feature, example: feature.example.replace("2024-01-01", "0.5") },
      ]),
    ),
    missing: MISSING,
  },
  quantity: {
    "value-only": { label: "value (any unit)", example: "=5.4" },
    "value-system-code": {
      label: "value|system|code",
      example: "=5.4|http://unitsofmeasure.org|mg",
    },
    "value-code-only": { label: "value||code", example: "=5.4||mg" },
    ...Object.fromEntries(
      Object.entries(PREFIXES).map(([key, feature]) => [
        key,
        { ...feature, example: feature.example.replace("2024-01-01", "5.4") },
      ]),
    ),
    missing: MISSING,
  },
  uri: {
    default: { label: "Exact URL", example: "=http://example.org/fhir/ValueSet/x" },
    missing: MISSING,
  },
};

export function featureInfo(type: ParamType | undefined, feature: string): Feature {
  return (type && FEATURES[type][feature]) ?? { label: feature, example: "" };
}

/** Features in a consistent reading order: basic forms, prefixes, modifiers. */
export function featureOrder(type: ParamType | undefined, feature: string): number {
  if (!type) return 999;
  const keys = Object.keys(FEATURES[type]);
  const index = keys.indexOf(feature);
  return index === -1 ? 998 : index;
}
