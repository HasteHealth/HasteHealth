import { expect, test } from "@jest/globals";
import { fileURLToPath } from "url";

import { loadArtifacts } from "@oxidized-health/artifacts";
import { descend, get, pointer } from "@oxidized-health/fhir-pointer";
import {
  Patient,
  StructureDefinition,
  canonical,
  id,
} from "@oxidized-health/fhir-types/r4/types";
import {
  AllResourceTypes,
  FHIR_VERSION,
  R4,
  Resource,
  ResourceType,
} from "@oxidized-health/fhir-types/versions";

import { ElementLoc, ValidationCTX } from "../../types.js";
import { validateProfile } from "../index.js";
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

const usCorePatientProfile: StructureDefinition = memDatabase[
  "StructureDefinition"
].find(
  (p) =>
    (p as StructureDefinition).url ===
    "http://hl7.org/fhir/us/core/StructureDefinition/us-core-patient"
) as StructureDefinition;

const usCorePatient: Patient = {
  resourceType: "Patient",
  id: "example",
  extension: [
    {
      extension: [
        {
          url: "ombCategory",
          valueCoding: {
            system: "urn:oid:2.16.840.1.113883.6.238",
            code: "2106-3",
            display: "White",
          },
        },
        {
          url: "ombCategory",
          valueCoding: {
            system: "urn:oid:2.16.840.1.113883.6.238",
            code: "1002-5",
            display: "American Indian or Alaska Native",
          },
        },
        {
          url: "ombCategory",
          valueCoding: {
            system: "urn:oid:2.16.840.1.113883.6.238",
            code: "2028-9",
            display: "Asian",
          },
        },
        {
          url: "detailed",
          valueCoding: {
            system: "urn:oid:2.16.840.1.113883.6.238",
            code: "1586-7",
            display: "Shoshone",
          },
        },
        {
          url: "detailed",
          valueCoding: {
            system: "urn:oid:2.16.840.1.113883.6.238",
            code: "2036-2",
            display: "Filipino",
          },
        },
        {
          url: "text",
          valueString: "Mixed",
        },
      ],
      url: "http://hl7.org/fhir/us/core/StructureDefinition/us-core-race",
    },
    {
      extension: [
        {
          url: "ombCategory",
          valueCoding: {
            system: "urn:oid:2.16.840.1.113883.6.238",
            code: "2135-2",
            display: "Hispanic or Latino",
          },
        },
        {
          url: "detailed",
          valueCoding: {
            system: "urn:oid:2.16.840.1.113883.6.238",
            code: "2184-0",
            display: "Dominican",
          },
        },
        {
          url: "detailed",
          valueCoding: {
            system: "urn:oid:2.16.840.1.113883.6.238",
            code: "2148-5",
            display: "Mexican",
          },
        },
        {
          url: "text",
          valueString: "Hispanic or Latino",
        },
      ],
      url: "http://hl7.org/fhir/us/core/StructureDefinition/us-core-ethnicity",
    },
    {
      url: "http://hl7.org/fhir/us/core/StructureDefinition/us-core-birthsex",
      valueCode: "F",
    },
  ],
  identifier: [
    {
      use: "usual",
      type: {
        coding: [
          {
            system: "http://terminology.hl7.org/CodeSystem/v2-0203",
            code: "MR",
            display: "Medical Record Number",
          },
        ],
        text: "Medical Record Number",
      },
      system: "http://hospital.smarthealthit.org",
      value: "1032702",
    },
  ],
  active: true,
  name: [
    {
      family: "Shaw",
      given: ["Amy", "V."],
      period: {
        start: "2016-12-06",
        end: "2020-07-22",
      },
    },
    {
      family: "Baxter",
      given: ["Amy", "V."],
      suffix: ["PharmD"],
      period: {
        start: "2020-07-22",
      },
    },
  ],
  telecom: [
    {
      system: "phone",
      value: "555-555-5555",
      use: "home",
    },
    {
      system: "email",
      value: "amy.shaw@example.com",
    },
  ],
  gender: "female",
  birthDate: "1987-02-20",
  address: [
    {
      line: ["49 Meadow St"],
      city: "Mounds",
      state: "OK",
      postalCode: "74047",
      country: "US",
      period: {
        start: "2016-12-06",
        end: "2020-07-22",
      },
    },
    {
      line: ["183 Mountain View St"],
      city: "Mounds",
      state: "OK",
      postalCode: "74048",
      country: "US",
      period: {
        start: "2020-07-22",
      },
    },
  ],
} as Patient;

test("us-core patient slicing", async () => {
  const elementsLoc = descend(
    descend(
      pointer(R4, "StructureDefinition", usCorePatientProfile.id as id),
      "snapshot"
    ),
    "element"
  );
  const elementLoc = descend(elementsLoc, 0);
  const elements =
    get(elementsLoc, usCorePatientProfile as StructureDefinition) ?? [];

  const sliceIndexes = getSliceIndices(
    usCorePatientProfile,
    elementLoc as unknown as ElementLoc
  );

  expect(sliceIndexes).toEqual([
    {
      discriminator: 7,
      slices: [8, 9, 10],
    },
  ]);
});

test("Validate US-CORE", async () => {
  expect(
    validateProfile(CTX, usCorePatientProfile, usCorePatient)
  ).resolves.toEqual([]);
});
