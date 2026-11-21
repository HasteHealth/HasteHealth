/**
 * The metadata {@link FHIRSearchInput} completes against: resource types, their
 * search parameters, and the modifiers and prefixes those accept.
 *
 * Read from the server's CapabilityStatement and `SearchParameter`s, so the
 * suggestions track the server rather than a bundled copy of the spec.
 */
import createHTTPClient from "@haste-health/client/http";
import { FHIR_VERSION, Resource } from "@haste-health/fhir-types/versions";

type HTTPClient = ReturnType<typeof createHTTPClient>;

export type SearchParameterResource = Resource<FHIR_VERSION, "SearchParameter">;

/** A completion candidate and what to show beside it. */
export interface Suggestion {
  label: string;
  /** Type or category, shown right of the label. */
  detail?: string;
  /** Description for the info panel. */
  info?: string;
  /** Sort weight; higher sorts first. */
  boost?: number;
}

/**
 * Modifiers each parameter type accepts, matching what `fhir-search` validates.
 * Composite and special parameters are not indexed and take none.
 */
const MODIFIERS_BY_TYPE: Record<string, Suggestion[]> = {
  string: [
    {
      label: "exact",
      info: "Match the whole value, case and accent sensitive.",
    },
    { label: "contains", info: "Match anywhere in the value." },
    { label: "missing", info: "Match resources where the value is absent." },
  ],
  token: [
    { label: "not", info: "Match resources that do not have this value." },
    { label: "missing", info: "Match resources where the value is absent." },
  ],
  date: [
    { label: "missing", info: "Match resources where the value is absent." },
  ],
  number: [
    { label: "missing", info: "Match resources where the value is absent." },
  ],
  quantity: [
    { label: "missing", info: "Match resources where the value is absent." },
  ],
  uri: [
    { label: "missing", info: "Match resources where the value is absent." },
  ],
  reference: [
    { label: "missing", info: "Match resources where the value is absent." },
  ],
};

/** Prefixes the ordered types accept, per both search backends. */
const ORDERED_PREFIXES: Suggestion[] = [
  { label: "eq", info: "Equal (the default when no prefix is given)." },
  { label: "ne", info: "Not equal." },
  { label: "gt", info: "Greater than." },
  { label: "lt", info: "Less than." },
  { label: "ge", info: "Greater than or equal." },
  { label: "le", info: "Less than or equal." },
  { label: "sa", info: "Starts after." },
  { label: "eb", info: "Ends before." },
  { label: "ap", info: "Approximately." },
];

const PREFIX_TYPES = new Set(["date", "number", "quantity"]);

/** Modifiers valid for `type`. */
export function modifiersForType(type: string | undefined): Suggestion[] {
  return (type ? MODIFIERS_BY_TYPE[type] : undefined) ?? [];
}

/** Prefixes valid for `type`; only ordered types accept any. */
export function prefixesForType(type: string | undefined): Suggestion[] {
  return type && PREFIX_TYPES.has(type) ? ORDERED_PREFIXES : [];
}

/**
 * Result parameters, valid on every resource. A `SearchParameter` search for a
 * specific base does not return them, so they are listed here and merged in.
 */
export const RESULT_PARAMETERS: Suggestion[] = [
  { label: "_count", detail: "number", info: "Number of resources per page." },
  {
    label: "_sort",
    detail: "string",
    info: "Comma separated sort order; prefix a name with `-` to reverse it.",
  },
  {
    label: "_total",
    detail: "token",
    info: "How to compute the total: `none`, `estimate` or `accurate`.",
  },
  {
    label: "_include",
    detail: "string",
    info: "Include resources referenced by the results.",
  },
  {
    label: "_revinclude",
    detail: "string",
    info: "Include resources that reference the results.",
  },
  {
    label: "_summary",
    detail: "token",
    info: "Return a subset of elements: `true`, `text`, `data`, `count` or `false`.",
  },
  {
    label: "_elements",
    detail: "string",
    info: "Comma separated list of elements to return.",
  },
  {
    label: "_contained",
    detail: "token",
    info: "Whether to search contained resources.",
  },
];

/** Values for the result parameters that take a fixed set. */
const RESULT_PARAMETER_VALUES: Record<string, Suggestion[]> = {
  _total: [
    { label: "none", info: "Do not compute a total." },
    { label: "estimate", info: "Return an estimated total." },
    { label: "accurate", info: "Return an exact total." },
  ],
  _summary: [
    { label: "true", info: "Return only summary elements." },
    { label: "text", info: "Return only the text, id and meta." },
    { label: "data", info: "Return everything except the text." },
    { label: "count", info: "Return only a count of matches." },
    { label: "false", info: "Return all elements." },
  ],
  _contained: [
    { label: "false", info: "Do not search contained resources." },
    { label: "true", info: "Search contained resources." },
    { label: "both", info: "Search both." },
  ],
};

/** Fixed values for `name`; empty when they are open ended. */
export function valuesForParameterName(name: string): Suggestion[] {
  return RESULT_PARAMETER_VALUES[name] ?? [];
}

/** `:missing` takes a boolean whatever the parameter's type. */
export const MISSING_VALUES: Suggestion[] = [
  { label: "true", info: "The value is absent." },
  { label: "false", info: "The value is present." },
];

/**
 * Caches one server's metadata so typing does not refetch. Parameters are
 * fetched per resource type on first use.
 */
export class SearchMetadata {
  private readonly client: HTTPClient;
  private readonly fhirVersion: FHIR_VERSION;
  private resourceTypes?: Promise<string[]>;
  private readonly parameters = new Map<
    string,
    Promise<SearchParameterResource[]>
  >();

  constructor(client: HTTPClient, fhirVersion: FHIR_VERSION) {
    this.client = client;
    this.fhirVersion = fhirVersion;
  }

  /** Types the server declares in its CapabilityStatement. */
  async getResourceTypes(): Promise<string[]> {
    this.resourceTypes ??= this.client
      .capabilities({}, this.fhirVersion)
      .then((capabilities) =>
        (capabilities.rest ?? [])
          .flatMap((rest) => rest.resource ?? [])
          .map((resource) => resource.type as string)
          .filter((type): type is string => Boolean(type))
          .sort((a, b) => a.localeCompare(b)),
      )
      .catch(() => []);
    return this.resourceTypes;
  }

  /**
   * Search parameters for `resourceType`, including those shared by every
   * resource. With no type, just the shared ones, as a system search uses.
   */
  async getParameters(
    resourceType?: string,
  ): Promise<SearchParameterResource[]> {
    const key = resourceType ?? "";
    let pending = this.parameters.get(key);
    if (!pending) {
      const base = resourceType ? ["Resource", resourceType] : ["Resource"];
      pending = this.client
        .search_type({}, this.fhirVersion, "SearchParameter", [
          { name: "base", value: base },
          { name: "_count", value: ["500"] },
          { name: "_sort", value: ["code"] },
        ])
        .then((bundle) => bundle.resources as SearchParameterResource[])
        .catch(() => []);
      this.parameters.set(key, pending);
    }
    return pending;
  }

  /** The parameter `code` on `resourceType`, if the server has one. */
  async getParameter(
    resourceType: string | undefined,
    code: string,
  ): Promise<SearchParameterResource | undefined> {
    const parameters = await this.getParameters(resourceType);
    return parameters.find((parameter) => parameter.code === code);
  }
}

/**
 * A reference parameter's `target` types. Resolves the next link of a chain
 * like `subject.name`, and suggests the type in `subject:Patient`.
 */
export function referenceTargets(
  parameter: SearchParameterResource | undefined,
): string[] {
  if (parameter?.type !== "reference") return [];
  return (parameter.target ?? []) as string[];
}
