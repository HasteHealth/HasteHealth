import { code, id } from "@haste-health/fhir-types/r4/types";
import {
  AllResourceTypes,
  FHIR_VERSION,
  Resource,
  ResourceType,
} from "@haste-health/fhir-types/versions";

import type { Parameters } from "../url.js";
import {
  Interaction,
  Request,
  RequestLevel,
  RequestType,
  ResponseType,
} from "./utilities.js";

export * from "./utilities.js";

export interface InstanceInteraction<Version extends FHIR_VERSION>
  extends Request<Version, "instance"> {
  resource: ResourceType<Version>;
  id: id;
}

export interface TypeInteraction<Version extends FHIR_VERSION>
  extends Request<Version, "type"> {
  resource: ResourceType<Version>;
}

export interface SystemInteraction<Version extends FHIR_VERSION>
  extends Request<Version, "system"> {}

export interface ReadRequest<Version extends FHIR_VERSION>
  extends InstanceInteraction<Version> {
  type: RequestType["read"];
}

export interface VersionReadRequest<Version extends FHIR_VERSION>
  extends InstanceInteraction<Version> {
  type: RequestType["vread"];
  versionId: string;
}

export interface InstanceUpdateRequest<Version extends FHIR_VERSION>
  extends InstanceInteraction<Version> {
  type: RequestType["update"];
  body: Resource<Version, ResourceType<Version>>;
}

// TODO - implement patch type
export interface PatchRequest<Version extends FHIR_VERSION>
  extends InstanceInteraction<Version> {
  type: RequestType["patch"];
  body: unknown;
}

export interface InstanceDeleteRequest<Version extends FHIR_VERSION>
  extends InstanceInteraction<Version> {
  type: RequestType["delete"];
}

export interface TypeDeleteRequest<Version extends FHIR_VERSION>
  extends TypeInteraction<Version> {
  parameters: Parameters<Version>;
  type: RequestType["delete"];
}

export interface SystemDeleteRequest<Version extends FHIR_VERSION>
  extends SystemInteraction<Version> {
  parameters: Parameters<Version>;
  type: RequestType["delete"];
}

export interface HistoryInstanceRequest<Version extends FHIR_VERSION>
  extends InstanceInteraction<Version> {
  type: RequestType["history"];
  parameters?: Parameters<Version>;
}

export interface CreateRequest<Version extends FHIR_VERSION>
  extends TypeInteraction<Version> {
  type: RequestType["create"];
  body: Resource<Version, ResourceType<Version>>;
}

export interface ConditionalUpdateRequest<Version extends FHIR_VERSION>
  extends TypeInteraction<Version> {
  type: RequestType["update"];
  parameters: Parameters<Version>;
  body: Resource<Version, ResourceType<Version>>;
}

export interface TypeSearchRequest<Version extends FHIR_VERSION>
  extends TypeInteraction<Version> {
  parameters: Parameters<Version>;
  type: RequestType["search"];
}

export interface TypeHistoryRequest<Version extends FHIR_VERSION>
  extends TypeInteraction<Version> {
  type: RequestType["history"];
  parameters?: Parameters<Version>;
}

export interface CapabilitiesRequest<Version extends FHIR_VERSION>
  extends SystemInteraction<Version> {
  type: RequestType["capabilities"];
}

export interface BatchRequest<Version extends FHIR_VERSION>
  extends SystemInteraction<Version> {
  type: RequestType["batch"];
  body: Resource<Version, "Bundle">;
}

export interface TransactionRequest<Version extends FHIR_VERSION>
  extends SystemInteraction<Version> {
  type: RequestType["transaction"];
  body: Resource<Version, "Bundle">;
}

export interface SystemHistoryRequest<Version extends FHIR_VERSION>
  extends SystemInteraction<Version> {
  type: RequestType["history"];
  parameters?: Parameters<Version>;
}

export interface SystemSearchRequest<Version extends FHIR_VERSION>
  extends SystemInteraction<Version> {
  parameters: Parameters<Version>;
  type: RequestType["search"];
}

export interface InvokeInstanceRequest<Version extends FHIR_VERSION>
  extends InstanceInteraction<Version> {
  type: RequestType["invoke"];
  operation: code;
  body: Resource<Version, "Parameters">;
}

export interface InvokeTypeRequest<Version extends FHIR_VERSION>
  extends TypeInteraction<Version> {
  type: RequestType["invoke"];
  operation: code;
  body: Resource<Version, "Parameters">;
}

export interface InvokeSystemRequest<Version extends FHIR_VERSION>
  extends SystemInteraction<Version> {
  type: RequestType["invoke"];
  operation: code;
  body: Resource<Version, "Parameters">;
}

export interface ReadResponse<Version extends FHIR_VERSION>
  extends InstanceInteraction<Version> {
  type: ResponseType["read"];
  body: Resource<Version, ResourceType<Version>>;
}

export interface VersionReadResponse<Version extends FHIR_VERSION>
  extends InstanceInteraction<Version> {
  type: ResponseType["vread"];
  versionId: string;
  body: Resource<Version, ResourceType<Version>>;
}

export interface UpdateResponse<Version extends FHIR_VERSION>
  extends InstanceInteraction<Version> {
  type: ResponseType["update"];
  created?: boolean;
  body: Resource<Version, ResourceType<Version>>;
}

// TODO - implement patch type
export interface PatchResponse<Version extends FHIR_VERSION>
  extends InstanceInteraction<Version> {
  type: ResponseType["patch"];
  body: Resource<Version, ResourceType<Version>>;
}

export interface InstanceDeleteResponse<Version extends FHIR_VERSION>
  extends InstanceInteraction<Version> {
  type: ResponseType["delete"];
  deletion?: Resource<Version, AllResourceTypes>;
}

export interface TypeDeleteResponse<Version extends FHIR_VERSION>
  extends TypeInteraction<Version> {
  parameters: Parameters<Version>;
  type: ResponseType["delete"];
  // For conditional deletes include the resources that were deleted.
  deletion?: Resource<Version, AllResourceTypes>[];
}

export interface SystemDeleteResponse<Version extends FHIR_VERSION>
  extends SystemInteraction<Version> {
  parameters: Parameters<Version>;
  type: ResponseType["delete"];
  // For conditional deletes include the resources that were deleted.
  deletion?: Resource<Version, AllResourceTypes>[];
}

export interface InstanceHistoryResponse<Version extends FHIR_VERSION>
  extends InstanceInteraction<Version> {
  type: ResponseType["history"];
  body: Resource<Version, "Bundle">;
}

export interface CreateResponse<Version extends FHIR_VERSION>
  extends TypeInteraction<Version> {
  type: ResponseType["create"];
  body: Resource<Version, ResourceType<Version>>;
}

export interface TypeSearchResponse<Version extends FHIR_VERSION>
  extends TypeInteraction<Version> {
  parameters: Parameters<Version>;
  type: ResponseType["search"];
  body: Resource<Version, "Bundle">;
}

export interface TypeHistoryResponse<Version extends FHIR_VERSION>
  extends TypeInteraction<Version> {
  type: ResponseType["history"];
  body: Resource<Version, "Bundle">;
}

export interface CapabilitiesResponse<Version extends FHIR_VERSION>
  extends SystemInteraction<Version> {
  type: ResponseType["capabilities"];
  body: Resource<Version, "CapabilityStatement">;
}

export interface BatchResponse<Version extends FHIR_VERSION>
  extends SystemInteraction<Version> {
  type: ResponseType["batch"];
  body: Resource<Version, "Bundle">;
}

export interface TransactionResponse<Version extends FHIR_VERSION>
  extends SystemInteraction<Version> {
  type: ResponseType["transaction"];
  body: Resource<Version, "Bundle">;
}

export interface SystemHistoryResponse<Version extends FHIR_VERSION>
  extends SystemInteraction<Version> {
  type: ResponseType["history"];
  body: Resource<Version, "Bundle">;
}

export interface SystemSearchResponse<Version extends FHIR_VERSION>
  extends SystemInteraction<Version> {
  parameters: Parameters<Version>;
  type: ResponseType["search"];
  body: Resource<Version, "Bundle">;
}

export interface InvokeInstanceResponse<Version extends FHIR_VERSION>
  extends InstanceInteraction<Version> {
  type: ResponseType["invoke"];
  operation: code;
  body: Resource<Version, "Parameters">;
}

export interface InvokeTypeResponse<Version extends FHIR_VERSION>
  extends TypeInteraction<Version> {
  type: ResponseType["invoke"];
  operation: code;
  body: Resource<Version, "Parameters">;
}

export interface InvokeSystemResponse<Version extends FHIR_VERSION>
  extends SystemInteraction<Version> {
  type: ResponseType["invoke"];
  operation: code;
  body: Resource<Version, "Parameters">;
}

export type InvokeResponse<Version extends FHIR_VERSION> =
  | InvokeInstanceResponse<Version>
  | InvokeTypeResponse<Version>
  | InvokeSystemResponse<Version>;

export type InvokeRequest<Version extends FHIR_VERSION> =
  | InvokeInstanceRequest<Version>
  | InvokeTypeRequest<Version>
  | InvokeSystemRequest<Version>;

export type DeleteRequest<Version extends FHIR_VERSION> =
  | InstanceDeleteRequest<Version>
  | TypeDeleteRequest<Version>
  | SystemDeleteRequest<Version>;
export type DeleteResponse<Version extends FHIR_VERSION> =
  | InstanceDeleteResponse<Version>
  | TypeDeleteResponse<Version>
  | SystemDeleteResponse<Version>;

export type HistoryRequest<Version extends FHIR_VERSION> =
  | HistoryInstanceRequest<Version>
  | TypeHistoryRequest<Version>
  | SystemHistoryRequest<Version>;
export type HistoryResponse<Version extends FHIR_VERSION> =
  | InstanceHistoryResponse<Version>
  | TypeHistoryResponse<Version>
  | SystemHistoryResponse<Version>;

export type SearchRequest<Version extends FHIR_VERSION> =
  | TypeSearchRequest<Version>
  | SystemSearchRequest<Version>;
export type SearchResponse<Version extends FHIR_VERSION> =
  | TypeSearchResponse<Version>
  | SystemSearchResponse<Version>;

export type UpdateRequest<Version extends FHIR_VERSION> =
  | InstanceUpdateRequest<Version>
  | ConditionalUpdateRequest<Version>;

interface ErrorResponse<
  Version extends FHIR_VERSION,
  Level extends keyof RequestLevel
> extends Request<Version, Level> {
  type: ResponseType["error"];
  body: Resource<Version, "OperationOutcome">;
}

export type FHIRErrorResponse<Version extends FHIR_VERSION> =
  | ErrorResponse<Version, "system">
  | ErrorResponse<Version, "type">
  | ErrorResponse<Version, "instance">;

type InteractionToRequest<Version extends FHIR_VERSION> = {
  invoke: InvokeRequest<Version>;
  read: ReadRequest<Version>;
  vread: VersionReadRequest<Version>;
  update: UpdateRequest<Version>;
  patch: PatchRequest<Version>;
  delete: DeleteRequest<Version>;
  history: HistoryRequest<Version>;
  create: CreateRequest<Version>;
  search: SearchRequest<Version>;
  capabilities: CapabilitiesRequest<Version>;
  batch: BatchRequest<Version>;
  transaction: TransactionRequest<Version>;
};

export type FHIRRequest<
  Version extends FHIR_VERSION,
  I extends Interaction[keyof Interaction]
> = InteractionToRequest<Version>[I];

type InteractionToResponse<Version extends FHIR_VERSION> = {
  error: FHIRErrorResponse<Version>;
  invoke: InvokeResponse<Version>;
  read: ReadResponse<Version>;
  vread: VersionReadResponse<Version>;
  update: UpdateResponse<Version>;
  patch: PatchResponse<Version>;
  history: HistoryResponse<Version>;
  delete: DeleteResponse<Version>;
  create: CreateResponse<Version>;
  capabilities: CapabilitiesResponse<Version>;
  batch: BatchResponse<Version>;
  transaction: TransactionResponse<Version>;
  search: SearchResponse<Version>;
};

export type FHIRResponse<
  Version extends FHIR_VERSION,
  I extends Interaction[keyof Interaction] | "error"
> = InteractionToResponse<Version>[I];
