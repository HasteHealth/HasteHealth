import { expect, test } from "@jest/globals";
import { fileURLToPath } from "url";

import { loadArtifacts } from "@oxidized-health/artifacts";
import { descend, get, pointer } from "@oxidized-health/fhir-pointer";
import {
  Observation,
  StructureDefinition,
  canonical,
  id,
  uri,
} from "@oxidized-health/fhir-types/r4/types";
import {
  AllResourceTypes,
  FHIR_VERSION,
  R4,
  Resource,
  ResourceType,
} from "@oxidized-health/fhir-types/versions";

import { ElementLoc, ValidationCTX } from "../../types.js";
import { getSliceIndices, validateSliceDescriptor } from "./index.js";

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
        "@oxidized-health/hl7.fhir.us.core",
        "@oxidized-health/hl7.fhir.r4.core",
        "@oxidized-health/hl7.fhir.r4.test-data",
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

const bloodProfile: StructureDefinition = memDatabase[
  "StructureDefinition"
].find(
  (p) =>
    (p as StructureDefinition).url ===
    "http://hl7.org/fhir/us/core/StructureDefinition/us-core-blood-pressure"
) as StructureDefinition;

const usCorePatientProfile: StructureDefinition = memDatabase[
  "StructureDefinition"
].find(
  (p) =>
    (p as StructureDefinition).url ===
    "http://hl7.org/fhir/us/core/StructureDefinition/us-core-patient"
) as StructureDefinition;

test("getSliceIndices", () => {
  const elementsLoc = descend(
    descend(
      pointer(R4, "StructureDefinition", bloodProfile.id as id),
      "snapshot"
    ),
    "element"
  );
  const elementLoc = descend(elementsLoc, 0);
  const elements = get(elementsLoc, bloodProfile as StructureDefinition) ?? [];

  const sliceIndexes = getSliceIndices(
    bloodProfile,
    elementLoc as unknown as ElementLoc
  );

  expect(sliceIndexes).toEqual([
    {
      discriminator: 13,
      slices: [14],
    },
    {
      discriminator: 53,
      slices: [62, 78],
    },
  ]);

  expect(elements[sliceIndexes[1].discriminator]?.path).toEqual(
    "Observation.component"
  );

  expect(elements[sliceIndexes[1].slices[0]]?.path).toEqual(
    "Observation.component"
  );
  expect(elements[sliceIndexes[1].slices[0]]?.id).toEqual(
    "Observation.component:systolic"
  );

  expect(elements[sliceIndexes[1].slices[1]]?.path).toEqual(
    "Observation.component"
  );
  expect(elements[sliceIndexes[1].slices[1]]?.id).toEqual(
    "Observation.component:diastolic"
  );
});

const bloodPressureObservation: Observation = {
  id: "blood-pressure-observation",
  resourceType: "Observation",
  status: "final",
  category: [
    {
      coding: [
        {
          system: "http://terminology.hl7.org/CodeSystem/observation-category",
          code: "vital-signs",
          display: "Vital Signs",
        },
      ],
      text: "Vital Signs",
    },
  ],
  code: {
    coding: [
      {
        system: "http://loinc.org",
        code: "85354-9",
        display: "Blood pressure panel with all children optional",
      },
    ],
    text: "Blood pressure systolic and diastolic",
  },
  subject: {
    reference: "Patient/example",
    display: "Amy Shaw",
  },
  encounter: {
    display: "GP Visit",
  },
  effectiveDateTime: "1999-07-02",
  component: [
    {
      code: {
        coding: [
          {
            system: "http://loinc.org",
            code: "8480-6",
            display: "Systolic blood pressure",
          },
        ],
        text: "Systolic blood pressure",
      },
      valueQuantity: {
        value: 109,
        unit: "mmHg",
        system: "http://unitsofmeasure.org",
        code: "mm[Hg]",
      },
    },
    {
      code: {
        coding: [
          {
            system: "http://loinc.org",
            code: "8462-4",
            display: "Diastolic blood pressure",
          },
        ],
        text: "Diastolic blood pressure",
      },
      valueQuantity: {
        value: 44,
        unit: "mmHg",
        system: "http://unitsofmeasure.org",
        code: "mm[Hg]",
      },
    },
  ],
} as Observation;

test("Slice Validation", async () => {
  const elementsLoc = descend(
    descend(
      pointer(R4, "StructureDefinition", bloodProfile.id as id),
      "snapshot"
    ),
    "element"
  );
  const elementLoc = descend(elementsLoc, 0);
  const sliceIndexes = getSliceIndices(
    bloodProfile,
    elementLoc as unknown as ElementLoc
  );

  expect(
    validateSliceDescriptor(
      CTX,
      bloodProfile,
      sliceIndexes[1],
      bloodPressureObservation,
      pointer(R4, "Observation", bloodPressureObservation.id as id)
    )
  ).resolves.toEqual([]);

  expect(
    validateSliceDescriptor(
      CTX,
      bloodProfile,
      sliceIndexes[0],
      bloodPressureObservation,
      pointer(R4, "Observation", bloodPressureObservation.id as id)
    )
  ).resolves.toEqual([]);

  expect(
    validateSliceDescriptor(
      CTX,
      bloodProfile,
      sliceIndexes[1],
      {
        id: "asdf",
        resourceType: "Observation",
        component: [
          {
            code: {
              coding: [
                {
                  system: "http://loinc.org",
                  code: "8480-6",
                },
              ],
            },
            valueQuantity: {
              value: 109,
              unit: "mmHg",
              system: "http://unitsofmeasure.org",
              code: "mm[Hg]",
            },
          },
          {
            code: {
              coding: [
                {
                  system: "http://loinc.org",
                  code: "8462-4",
                },
              ],
            },
            valueInteger: 44,
          },
        ],
      },
      pointer(R4, "Observation", "asdf" as id)
    )
  ).resolves.toEqual([
    {
      code: "structure",
      diagnostics:
        "Additional fields found at path '/component/1': 'valueInteger'",
      expression: ["/component/1"],
      severity: "error",
    },
  ]);

  expect(
    validateSliceDescriptor(
      CTX,
      bloodProfile,
      sliceIndexes[1],
      {
        id: "asdf",
        resourceType: "Observation",
        component: [
          {
            code: {
              coding: [
                {
                  system: "http://loinc.org",
                  code: "8480-6",
                },
              ],
            },
            valueQuantity: {
              value: 109,
              unit: "mmHg",
              system: "http://unitsofmeasure.org",
              code: "mm[Hg]",
            },
          },
          {
            code: {
              coding: [
                {
                  system: "http://loinc.org",
                },
              ],
            },
            valueQuantity: {
              value: 44,
              unit: "mmHg",
              system: "http://unitsofmeasure.org",
              code: "mm[Hg]",
            },
          },
        ],
      },
      pointer(R4, "Observation", "asdf" as id)
    )
  ).resolves.toEqual([
    {
      code: "structure",
      diagnostics:
        "Slice 'diastolic' does not have the minimum number of values.",
      expression: [""],
      severity: "error",
    },
  ]);
});

test("Blood Pressure Category", async () => {
  const elementsLoc = descend(
    descend(
      pointer(R4, "StructureDefinition", bloodProfile.id as id),
      "snapshot"
    ),
    "element"
  );

  const elementLoc = descend(elementsLoc, 0);
  const sliceIndexes = getSliceIndices(
    bloodProfile,
    elementLoc as unknown as ElementLoc
  );

  expect(
    validateSliceDescriptor(
      CTX,
      bloodProfile,
      sliceIndexes[0],
      {
        ...bloodProfile,
        category: [
          {
            coding: [
              {
                system:
                  "http://terminology.hl7.org/CodeSystem/observation-category",
                // Incorrect.
                code: "vital-signer",
              },
            ],
          },
        ],
      },
      pointer(R4, "Observation", bloodPressureObservation.id as id)
    )
  ).resolves.toEqual([
    {
      code: "structure",
      diagnostics: "Slice 'VSCat' does not have the minimum number of values.",
      severity: "error",
      expression: [""],
    },
  ]);

  expect(
    validateSliceDescriptor(
      CTX,
      bloodProfile,
      sliceIndexes[0],
      {
        ...bloodProfile,
        category: [{}],
      },
      pointer(R4, "Observation", bloodPressureObservation.id as id)
    )
  ).resolves.toEqual([
    {
      code: "structure",
      diagnostics: "Slice 'VSCat' does not have the minimum number of values.",
      severity: "error",
      expression: [""],
    },
  ]);

  expect(
    validateSliceDescriptor(
      CTX,
      bloodProfile,
      sliceIndexes[0],
      {
        ...bloodProfile,
        category: [
          {
            coding: [
              {
                system:
                  "http://terminology.hl7.org/CodeSystem/observation-category-false",
                code: "vital-signs",
              },
            ],
            text: "Vital Signs",
          },
        ],
      },
      pointer(R4, "Observation", bloodPressureObservation.id as id)
    )
  ).resolves.toEqual([
    {
      code: "structure",
      diagnostics: "Slice 'VSCat' does not have the minimum number of values.",
      severity: "error",
      expression: [""],
    },
  ]);

  expect(
    validateSliceDescriptor(
      CTX,
      bloodProfile,
      sliceIndexes[0],
      {
        ...bloodProfile,
        category: [
          {
            coding: [
              {
                system:
                  "http://terminology.hl7.org/CodeSystem/observation-category",
                code: "vital-signs",
              },
            ],
          },
        ],
      },
      pointer(R4, "Observation", bloodPressureObservation.id as id)
    )
  ).resolves.toEqual([]);
});
