import { expect, test } from "@jest/globals";
import { fileURLToPath } from "url";

import { loadArtifacts } from "@haste-health/artifacts";
import {
  StructureDefinition,
  canonical,
  uri,
} from "@haste-health/fhir-types/r4/types";
import {
  AllResourceTypes,
  FHIR_VERSION,
  R4,
  Resource,
  ResourceType,
} from "@haste-health/fhir-types/versions";

import validate from "./index.js";
import { ValidationCTX } from "./types";

function createMemoryDatabase(
  resourceTypes: ResourceType<R4>[]
): Record<ResourceType<R4>, Resource<FHIR_VERSION, AllResourceTypes>[]> {
  const data: Record<
    ResourceType<R4>,
    Resource<FHIR_VERSION, AllResourceTypes>[]
  > = {} as Record<
    ResourceType<R4>,
    Resource<FHIR_VERSION, AllResourceTypes>[]
  >;
  for (const resourceType of resourceTypes) {
    const resources = loadArtifacts({
      fhirVersion: R4,
      loadDevelopmentPackages: true,
      resourceType: resourceType,
      silence: true,
      currentDirectory: fileURLToPath(import.meta.url),
      onlyPackages: [
        "hl7.fhir.us.core",
        "@haste-health/hl7.fhir.r4.core",
        "@haste-health/hl7.fhir.r4.test-data",
      ],
    });
    data[resourceType] = resources;
  }

  return data;
}

const memDatabase = createMemoryDatabase([
  "StructureDefinition",
] as ResourceType<R4>[]);

const CTX: ValidationCTX = {
  fhirVersion: R4,
  resolveCanonical: async <
    Version extends FHIR_VERSION,
    Type extends ResourceType<Version>
  >(
    version: Version,
    t: Type,
    url: canonical
  ): Promise<Resource<Version, Type>> => {
    // @ts-ignore
    const sd: Resource<Version, Type> = memDatabase[t].find(
      (sd: unknown) => (sd as StructureDefinition).url === url
    ) as Resource<Version, Type>;
    if (!sd) throw new Error(`Couldn't find sd with url '${url}'`);
    return sd as Resource<Version, Type>;
  },
};

test("TEST with extensions", async () => {
  const patient = {
    resourceType: "Patient",
    name: [
      {
        _given: [
          {
            id: "123",
            extension: [{ url: "testing", valueString: "Hello" }],
          },
        ],
      },
    ],
  };
  expect(validate(CTX, "Patient" as uri, patient)).resolves.toEqual([]);
});
