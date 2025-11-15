import { expect, test } from "@jest/globals";
import { fileURLToPath } from "url";

import { loadArtifacts } from "@haste-health/artifacts";
import {
  Observation,
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

import { ValidationCTX } from "../types";
import { validateProfile } from "./index.js";

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
        "@haste-health/hl7.fhir.us.core",
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

const bloodProfile: StructureDefinition = memDatabase[
  "StructureDefinition"
].find(
  (p) =>
    (p as StructureDefinition).url ===
    "http://hl7.org/fhir/us/core/StructureDefinition/us-core-blood-pressure"
) as StructureDefinition;
test("Test BP profile", async () => {
  await validateProfile(CTX, bloodProfile, bloodPressureObservation);
  expect(
    validateProfile(CTX, bloodProfile, bloodPressureObservation)
  ).resolves.toEqual([]);

  expect(
    validateProfile(CTX, bloodProfile, {
      ...bloodPressureObservation,
      component: [],
    })
  ).resolves.toEqual([
    {
      code: "structure",
      diagnostics:
        "Slice 'systolic' does not have the minimum number of values.",
      expression: [""],
      severity: "error",
    },
    {
      code: "structure",
      diagnostics:
        "Slice 'diastolic' does not have the minimum number of values.",
      expression: [""],
      severity: "error",
    },
  ]);

  expect(
    validateProfile(CTX, bloodProfile, {
      ...bloodPressureObservation,
      component: [
        {
          code: {
            coding: [
              {
                system: "http://loinc.org",
                code: "bad",
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
    })
  ).resolves.toEqual([
    {
      severity: "error",
      code: "structure",
      diagnostics:
        "Slice 'systolic' does not have the minimum number of values.",
      expression: [""],
    },
  ]);
});

test("Invalid type constraint systolic.value", async () => {
  await validateProfile(CTX, bloodProfile, {
    ...bloodPressureObservation,
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
        valueInteger: 5,
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
  });
  expect(
    validateProfile(CTX, bloodProfile, {
      ...bloodPressureObservation,
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
          valueInteger: 5,
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
    })
  ).resolves.toEqual([
    {
      severity: "error",
      code: "structure",
      diagnostics:
        "Additional fields found at path '/component/0': 'valueInteger'",
      expression: ["/component/0"],
    },
  ]);
});
