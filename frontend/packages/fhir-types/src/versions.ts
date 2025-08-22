import * as r4 from "./generated/r4/types.js";
import * as r4b from "./generated/r4b/types.js";

export type R4 = "4.0";
export type R4B = "4.3";

export const R4: R4 = "4.0";
export const R4B: R4B = "4.3";

export const FHIR_VERSIONS_SUPPORTED: [R4, R4B] = [R4, R4B];
export type FHIR_VERSION = (typeof FHIR_VERSIONS_SUPPORTED)[number];

type ResourceTypeMap = {
  [R4]: r4.ResourceType;
  [R4B]: r4b.ResourceType;
};

export type ResourceType<
  Version extends (typeof FHIR_VERSIONS_SUPPORTED)[number],
> = ResourceTypeMap[Version];

export type AllResourceTypes = ResourceType<FHIR_VERSION>;

export type Resource<
  Version extends (typeof FHIR_VERSIONS_SUPPORTED)[number],
  Type extends AllResourceTypes,
> = Version extends R4
  ? Type extends r4.ResourceType
    ? r4.AResource<Type>
    : never
  : Version extends R4B
    ? Type extends r4b.ResourceType
      ? r4b.AResource<Type>
      : never
    : never;

// [DATA TYPES]
// ------------------------------------------------------------------------------------------------

export type DataType<Version extends (typeof FHIR_VERSIONS_SUPPORTED)[number]> =
  Version extends R4 ? r4.DataType : Version extends R4B ? r4b.DataType : never;

export type AllDataTypes = r4.DataType | r4b.DataType;

export type Data<
  Version extends (typeof FHIR_VERSIONS_SUPPORTED)[number],
  Type extends AllDataTypes,
> = Version extends R4
  ? Type extends r4.DataType
    ? r4.AData<Type>
    : never
  : Version extends R4B
    ? Type extends r4b.DataType
      ? r4b.AData<Type>
      : never
    : never;

// ------------------------------------------------------------------------------------------------
