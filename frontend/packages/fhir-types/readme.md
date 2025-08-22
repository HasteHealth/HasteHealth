# FHIR Typescript Types

Typescript types generated off of FHIR StructureDefinitions.

## Usage

```typescript
import { Observation, Patient, id } from "@oxidized-health/fhir-types/r4/types";

const patient: Patient = {
  name: [
    {
      use: "official",
      given: ["Eve"],
      family: "Williams",
    },
  ],
  active: true,
  gender: "female",
  address: [
    {
      use: "home",
      line: ["2222 Home Street"],
    },
  ],
  telecom: [
    {
      use: "work",
      value: "555-555-2003",
      system: "phone",
    },
  ],
  birthDate: "1973-05-31",
  identifier: [
    {
      type: {
        coding: [
          {
            code: "SS",
            system: "http://terminology.hl7.org/CodeSystem/v2-0203",
          },
        ],
      },
      value: "444222222",
      system: "http://hl7.org/fhir/sid/us-ssn",
    },
  ],
  resourceType: "Patient",
  managingOrganization: {
    reference: "Organization/hl7",
  },
};
```

## Sets

Sets can be used to determine whether a type is a resource, primitive or complex type.
These are utilities that can be useful

| name           | description                                                      |
| -------------- | ---------------------------------------------------------------- |
| resourceTypes  | String set of FHIR resource types                                |
| complexTypes   | String set of FHIR complex types (HumanName, Identifier etc...). |
| primitiveTypes | String set of FHIR primitive types (string, id, decimal etc...). |

### Usage

```typescript
import {
  primitiveTypes,
  resourceTypes,
} from "@oxidized-health/fhir-types/r4/sets";

function isPrimitiveType(type: string) {
  returnprimitiveTypes.has(type);
}
function isResourceType(type: string) {
  returnresourceTypes.has(type);
}
```

## Utility Types

Various utility types generally used for Typescript generic functions.

| Name         | Description                                                                                                                            |
| ------------ | -------------------------------------------------------------------------------------------------------------------------------------- |
| ResourceType | One of the resourcetypes, ie "Patient", "Observation".                                                                                 |
| AResource    | Generic type with an argument of ResourceType, when passed in, returns an instance of ResourceType IE AResource\<"Patient"\> = Patient |
| Resource     | An instance of one of the resource types                                                                                               |

### Example

Pulled from @oxidized-health/client this example passes in an argument of ResourceType
which is a string of only allowed ResourceTypes and using AResource returns an object in conformance with the type passed in.

```typescript
read<T extends ResourceType>(
ctx: CTX,
resourceType: T,
id: id
): Promise<AResource<T> | undefined>;
```
