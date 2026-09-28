/**
 * Groups resource types for the sidebar.
 *
 * A server exposes around 155 types, which is an unreadable list. The
 * grouping is FHIR's own `Module.Section` taxonomy - the one the reference
 * documentation at /docs/reference/fhir/model/resources is organised by -
 * rather than one invented here, so a reader who knows the spec already knows
 * where to look.
 *
 * The table lives in `resource-categories.json`, generated from the same model
 * data that documentation is built from. It is deliberately not read from the
 * server: the category lives in a `structuredefinition-category` extension
 * that the patched artifacts drop, so the definitions a server serves do not
 * carry it.
 *
 * A type missing from the table - the Haste Health ones, and SQL on FHIR's
 * ViewDefinition - falls back to a group below rather than being hidden.
 */

import SPEC_CATEGORIES from "./resource-categories.json";


/** Types the spec gives no category, grouped by where they come from. */
const EXTRA_GROUPS: Record<string, string[]> = {
  "Haste Health.Access control": [
    "AccessPolicyV2",
    "AccessPolicyV2Assignment",
    "ClientApplication",
    "IdentityProvider",
    "Membership",
    "User",
  ],
  "Haste Health.Workspace": ["Project"],
  "Haste Health.Exchange": ["HL7V2"],
  // ViewDefinition is SQL on FHIR's, not ours, even though it is carried here
  // with some changes of our own.
  "SQL on FHIR.Projections": ["ViewDefinition"],
};

/**
 * Every type to its category, the spec's and ours together.
 *
 * Built once rather than searching `EXTRA_GROUPS` per lookup, since the
 * sidebar asks for all ~155 types on each render of the list.
 */
const CATEGORY_BY_TYPE: ReadonlyMap<string, string> = new Map([
  ...Object.entries(SPEC_CATEGORIES as Record<string, string>),
  ...Object.entries(EXTRA_GROUPS).flatMap(([group, types]) =>
    types.map((type): [string, string] => [type, group]),
  ),
]);

/** One module, holding the sections beneath it. */
export interface CategoryGroup {
  /** The part before the dot, e.g. `Clinical`. */
  module: string;
  sections: { label: string; types: string[] }[];
}

/**
 * The order modules are shown in.
 *
 * Roughly how often a developer on a clinical system reaches for them, not
 * the spec's own order, which leads with Foundation - mostly machinery.
 */
const MODULE_ORDER = [
  "Base",
  "Clinical",
  "Financial",
  "Specialized",
  "Foundation",
  "SQL on FHIR",
  "Haste Health",
  "Other",
];

function moduleRank(module: string): number {
  const index = MODULE_ORDER.indexOf(module);
  return index === -1 ? MODULE_ORDER.length : index;
}

/** The `Module.Section` a type belongs to. */
export function categoryOf(resourceType: string): string {
  return CATEGORY_BY_TYPE.get(resourceType) ?? "Other.Uncategorised";
}

/** Splits `Module.Section`, treating a bare string as its own module. */
function splitCategory(category: string): { module: string; label: string } {
  const dot = category.indexOf(".");
  return dot === -1
    ? { module: category, label: category }
    : { module: category.slice(0, dot), label: category.slice(dot + 1) };
}

/** Collects `items` into groups under the key each one maps to. */
function groupBy<T, K>(items: readonly T[], key: (item: T) => K): Map<K, T[]> {
  // A Map built in one pass: the functional alternatives all rebuild an
  // object per element, which for 155 types on every render is worth avoiding.
  return items.reduce((groups, item) => {
    const k = key(item);
    const existing = groups.get(k);
    if (existing) existing.push(item);
    else groups.set(k, [item]);
    return groups;
  }, new Map<K, T[]>());
}

/** Groups `available` into modules and the sections beneath them. */
export function categorise(available: readonly string[]): CategoryGroup[] {
  const sorted = [...available].sort((a, b) => a.localeCompare(b));
  const byCategory = groupBy(sorted, categoryOf);

  const sections = Array.from(byCategory).map(([category, types]) => ({
    ...splitCategory(category),
    types,
  }));

  return Array.from(groupBy(sections, (section) => section.module))
    .map(([module, moduleSections]) => ({
      module,
      sections: moduleSections
        .map(({ label, types }) => ({ label, types }))
        .sort((a, b) => a.label.localeCompare(b.label)),
    }))
    .sort(
      (a, b) =>
        moduleRank(a.module) - moduleRank(b.module) ||
        a.module.localeCompare(b.module),
    );
}
