/* eslint-disable @typescript-eslint/no-explicit-any */
import { code } from "@haste-health/fhir-types/lib/generated/r4/types";
import {
  AllResourceTypes,
  FHIR_VERSION,
  Resource,
  ResourceType,
} from "@haste-health/fhir-types/versions";
import type { IOperation, OPMetadata } from "@haste-health/operation-execution";

import { MiddlewareAsync } from "./middleware/index.js";
import type {
  AllInteractions,
  FHIRRequest,
  FHIRResponse,
} from "./types/index.js";
import type { ParsedParameter } from "./url.js";

export type FHIRClient<CTX> = FHIRClientAsync<CTX>;

export type InvokeParameter<
  FHIRVersion extends FHIR_VERSION,
  Op,
  direction extends "Output" | "Input"
> = Op extends IOperation<any, any>
  ? OPMetadata<Op>[direction]
  : Resource<FHIRVersion, "Parameters">;

export interface FHIRClientAsync<CTX> {
  request<Version extends FHIR_VERSION, I extends AllInteractions>(
    ctx: CTX,
    request: FHIRRequest<Version, I>
  ): Promise<FHIRResponse<Version, I | "error">>;
  middleware: MiddlewareAsync<CTX>;
  capabilities<FHIRVersion extends FHIR_VERSION>(
    ctx: CTX,
    fhirVersion: FHIRVersion
  ): Promise<Resource<FHIRVersion, "CapabilityStatement">>;
  search_system<FHIRVersion extends FHIR_VERSION>(
    ctx: CTX,
    fhirVersion: FHIRVersion,
    parameters: ParsedParameter<string | number>[] | string
  ): Promise<{
    total?: number;
    resources: Resource<FHIRVersion, ResourceType<FHIRVersion>>[];
  }>;
  search_type<
    FHIRVersion extends FHIR_VERSION,
    T extends ResourceType<FHIRVersion>
  >(
    ctx: CTX,
    fhirVersion: FHIRVersion,
    type: T,
    parameters: ParsedParameter<string | number>[] | string
  ): Promise<{
    total?: number;
    resources: Resource<FHIRVersion, T>[];
  }>;
  create<
    FHIRVersion extends FHIR_VERSION,
    Value extends Resource<FHIRVersion, AllResourceTypes>
  >(
    ctx: CTX,
    fhirVersion: FHIRVersion,
    resource: Value
  ): Promise<Value>;

  update<FHIRVersion extends FHIR_VERSION, T extends ResourceType<FHIRVersion>>(
    ctx: CTX,
    fhirVersion: FHIRVersion,
    resourceType: T,
    id: NonNullable<Resource<FHIRVersion, ResourceType<FHIRVersion>>["id"]>,
    resource: Resource<FHIRVersion, T>
  ): Promise<Resource<FHIRVersion, T>>;
  conditionalUpdate<
    FHIRVersion extends FHIR_VERSION,
    T extends ResourceType<FHIRVersion>
  >(
    ctx: CTX,
    fhirVersion: FHIRVersion,
    resourceType: T,
    parameters: ParsedParameter<string | number>[] | string,
    resource: Resource<FHIRVersion, T>
  ): Promise<Resource<FHIRVersion, T>>;
  patch<FHIRVersion extends FHIR_VERSION, T extends ResourceType<FHIRVersion>>(
    ctx: CTX,
    fhirVersion: FHIRVersion,
    resourceType: T,
    id: NonNullable<Resource<FHIRVersion, ResourceType<FHIRVersion>>["id"]>,
    patches: any
  ): Promise<Resource<FHIRVersion, T>>;
  read<FHIRVersion extends FHIR_VERSION, T extends ResourceType<FHIRVersion>>(
    ctx: CTX,
    fhirVersion: FHIRVersion,
    resourceType: T,
    id: NonNullable<Resource<FHIRVersion, ResourceType<FHIRVersion>>["id"]>
  ): Promise<Resource<FHIRVersion, T> | undefined>;
  vread<FHIRVersion extends FHIR_VERSION, T extends ResourceType<FHIRVersion>>(
    ctx: CTX,
    fhirVersion: FHIRVersion,
    resourceType: T,
    id: NonNullable<Resource<FHIRVersion, ResourceType<FHIRVersion>>["id"]>,
    versionId: NonNullable<
      Resource<FHIRVersion, ResourceType<FHIRVersion>>["id"]
    >
  ): Promise<Resource<FHIRVersion, T> | undefined>;
  delete_instance<
    FHIRVersion extends FHIR_VERSION,
    T extends ResourceType<FHIRVersion>
  >(
    ctx: CTX,
    fhirVersion: FHIRVersion,
    resourceType: T,
    id: NonNullable<Resource<FHIRVersion, ResourceType<FHIRVersion>>["id"]>
  ): Promise<void>;
  delete_type<
    FHIRVersion extends FHIR_VERSION,
    T extends ResourceType<FHIRVersion>
  >(
    ctx: CTX,
    fhirVersion: FHIRVersion,
    resourceType: T,
    parameters?: ParsedParameter<string | number>[] | string
  ): Promise<void>;
  delete_system<FHIRVersion extends FHIR_VERSION>(
    ctx: CTX,
    fhirVersion: FHIRVersion,
    parameters?: ParsedParameter<string | number>[] | string
  ): Promise<void>;

  history_system<FHIRVersion extends FHIR_VERSION>(
    ctx: CTX,
    fhirVersion: FHIRVersion,
    parameters?: ParsedParameter<string | number>[] | string
  ): Promise<NonNullable<Resource<FHIRVersion, "Bundle">["entry"]>>;
  history_type<
    FHIRVersion extends FHIR_VERSION,
    T extends ResourceType<FHIRVersion>
  >(
    ctx: CTX,
    fhirVersion: FHIRVersion,
    resourceType: T,
    parameters?: ParsedParameter<string | number>[] | string
  ): Promise<NonNullable<Resource<FHIRVersion, "Bundle">["entry"]>>;
  history_instance<
    FHIRVersion extends FHIR_VERSION,
    T extends ResourceType<FHIRVersion>
  >(
    ctx: CTX,
    fhirVersion: FHIRVersion,
    resourceType: T,
    id: NonNullable<Resource<FHIRVersion, ResourceType<FHIRVersion>>["id"]>,
    parameters?: ParsedParameter<string | number>[] | string
  ): Promise<NonNullable<Resource<FHIRVersion, "Bundle">["entry"]>>;
  invoke_system<
    FHIRVersion extends FHIR_VERSION,
    Op extends IOperation<any, any> | code,
    Input extends InvokeParameter<FHIRVersion, Op, "Input">,
    Output extends InvokeParameter<FHIRVersion, Op, "Output">
  >(
    op: Op,
    ctx: CTX,
    fhirVersion: FHIRVersion,
    input: Input
  ): Promise<Output>;
  invoke_type<
    FHIRVersion extends FHIR_VERSION,
    Op extends IOperation<any, any> | code,
    Input extends InvokeParameter<FHIRVersion, Op, "Input">,
    Output extends InvokeParameter<FHIRVersion, Op, "Output">,
    T extends ResourceType<FHIRVersion>
  >(
    op: Op,
    ctx: CTX,
    fhirVersion: FHIRVersion,
    resourceType: T,
    input: Input
  ): Promise<Output>;
  invoke_instance<
    FHIRVersion extends FHIR_VERSION,
    Op extends IOperation<any, any> | code,
    Input extends InvokeParameter<FHIRVersion, Op, "Input">,
    Output extends InvokeParameter<FHIRVersion, Op, "Output">,
    T extends ResourceType<FHIRVersion>
  >(
    op: Op,
    ctx: CTX,
    fhirVersion: FHIRVersion,
    resourceType: T,
    id: NonNullable<Resource<FHIRVersion, ResourceType<FHIRVersion>>["id"]>,
    input: Input
  ): Promise<Output>;
  transaction<FHIRVersion extends FHIR_VERSION>(
    ctx: CTX,
    fhirVersion: FHIRVersion,
    bundle: Resource<FHIRVersion, "Bundle">
  ): Promise<Resource<FHIRVersion, "Bundle">>;
  batch<FHIRVersion extends FHIR_VERSION>(
    ctx: CTX,
    fhirVersion: FHIRVersion,
    bundle: Resource<FHIRVersion, "Bundle">
  ): Promise<Resource<FHIRVersion, "Bundle">>;
}
