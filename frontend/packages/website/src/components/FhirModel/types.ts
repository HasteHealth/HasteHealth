import type { Backend, Interaction, Result } from "../TestCoverage/data";

export type Kind = "resource" | "complex-type" | "primitive-type";
export type Source = "hl7" | "haste-health" | "sql-on-fhir";
export type BackendResults = Record<Backend, Result>;

export type Link = { name: string; href?: string };

export type ElementType = {
  code: string;
  href?: string;
  targets?: Link[];
};

export type Element = {
  path: string;
  min: number;
  max: string;
  types: ElementType[];
  short: string;
  contentReference?: string;
  modifier?: boolean;
  summary?: boolean;
  /** Defined by a base type (Resource, DomainResource, Element…). */
  inherited?: boolean;
  binding?: { strength: string; valueSet: string };
};

export type SearchParameter = {
  code: string;
  type: string;
  description: string;
  expression?: string;
  coverage?: BackendResults;
};

export type ModelData = {
  name: string;
  kind: Kind;
  url: string;
  source: Source;
  definition: string;
  category?: string;
  status?: string;
  maturity?: number;
  hl7?: string;
  raw: string;
  elements: Element[];
  searchParameters?: SearchParameter[];
  commonParameters?: Omit<SearchParameter, "expression" | "coverage">[];
  coverage?: {
    overall: BackendResults;
    interactions: Partial<Record<Interaction, BackendResults>>;
  };
  /** Type- and instance-level operations the server defines on this type. */
  operations?: Operation[];
};

export type OperationLevel = "system" | "type" | "instance";

export type OperationParameter = {
  name: string;
  use: "in" | "out";
  min: number;
  max: string;
  type?: string;
  href?: string;
  targets?: Link[];
  searchType?: string;
  documentation?: string;
  binding?: { strength: string; valueSet: string };
  parts?: OperationParameter[];
};

export type Operation = {
  code: string;
  id: string;
  url?: string;
  title?: string;
  levels: OperationLevel[];
  /** The resource types it is defined on; none means any. */
  resources: Link[];
  description: string;
  affectsState: boolean;
  hl7?: string;
  raw: string;
  parameters: OperationParameter[];
};

export type SystemData = {
  fhirVersion: string;
  formats: string[];
  cors: boolean;
  /** The system-level interaction codes the CapabilityStatement declares. */
  interactions: string[];
  operations: Operation[];
  /** Resource types in the CapabilityStatement. */
  resourceTypes: number;
  /** The resource types that have operations, with their codes. */
  resources: {
    name: string;
    href: string;
    operations: { code: string; levels: OperationLevel[] }[];
  }[];
  raw: string;
};

export type IndexEntry = {
  name: string;
  kind: Kind;
  source: Source;
  href: string;
  short: string;
  category?: string;
  maturity?: number;
  status?: string;
  searchParameters?: number;
  operations?: number;
  overall?: BackendResults;
};
