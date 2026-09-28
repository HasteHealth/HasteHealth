/**
 * StructureDefinitions, cached per resource type.
 *
 * The console shows resources as raw JSON, which is the right default for a
 * developer but assumes you already know what `Patient.deceasedBoolean` is.
 * The StructureDefinition is what turns that assumption into something the UI
 * can answer, so an element can be explained on demand rather than having to
 * be looked up in the spec.
 */
import { atom } from "jotai";

import {
  ElementDefinition,
  StructureDefinition,
  id,
} from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";

import { createAdminAppClient } from "./client";

type Client = ReturnType<typeof createAdminAppClient>;

/** What the inspector needs to describe one element. */
export interface ElementInfo {
  /** Dotted path, e.g. `Patient.name.given`. */
  path: string;
  /** Types the element may take, e.g. `["HumanName"]`. */
  types: string[];
  short?: string;
  definition?: string;
  min?: number;
  max?: string;
  /** True when the element may repeat. */
  isArray: boolean;
}

function toElementInfo(element: ElementDefinition): ElementInfo {
  return {
    path: element.path as string,
    types: (element.type ?? []).map((t) => t.code as string),
    short: element.short as string | undefined,
    definition: element.definition as string | undefined,
    min: element.min as number | undefined,
    max: element.max as string | undefined,
    isArray: element.max === "*" || Number(element.max) > 1,
  };
}

/**
 * Loads and caches StructureDefinitions. One instance is shared through
 * {@link getStructures} so that two panels asking about the same type do not
 * both fetch it.
 */
export class StructureCache {
  private readonly client: Client;
  private readonly pending = new Map<
    string,
    Promise<StructureDefinition | undefined>
  >();

  constructor(client: Client) {
    this.client = client;
  }

  /** The StructureDefinition for `resourceType`, or `undefined`. */
  async get(resourceType: string): Promise<StructureDefinition | undefined> {
    let promise = this.pending.get(resourceType);
    if (!promise) {
      promise = this.client
        .read({}, R4, "StructureDefinition", resourceType as id)
        // A type with no StructureDefinition is not an error worth surfacing;
        // the inspector simply has nothing extra to say about it.
        .catch(() => undefined);
      this.pending.set(resourceType, promise);
    }
    return promise;
  }

  /**
   * Element definitions for `resourceType`, keyed by their dotted path with
   * the leading type stripped, so `name.given` rather than
   * `Patient.name.given`. Choice elements are stored under the `[x]` name as
   * written in the definition and under each concrete type.
   */
  async elements(resourceType: string): Promise<Map<string, ElementInfo>> {
    const structure = await this.get(resourceType);
    const elements = new Map<string, ElementInfo>();
    const definitions =
      structure?.snapshot?.element ?? structure?.differential?.element ?? [];

    for (const element of definitions) {
      const info = toElementInfo(element);
      const relative = info.path.startsWith(`${resourceType}.`)
        ? info.path.slice(resourceType.length + 1)
        : undefined;
      if (!relative) continue;

      elements.set(relative, info);

      // `deceased[x]` also answers to `deceasedBoolean` and `deceasedDateTime`,
      // which is how the field actually appears in the JSON.
      if (relative.endsWith("[x]")) {
        const stem = relative.slice(0, -"[x]".length);
        for (const type of info.types) {
          const concrete = `${stem}${type[0].toUpperCase()}${type.slice(1)}`;
          elements.set(concrete, info);
        }
      }
    }

    return elements;
  }
}

/** The shared cache, created once the client is available. */
export const getStructures = atom<StructureCache | undefined>(undefined);
