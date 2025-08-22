# OxidizedHealth Client

HTTPClient and classes + middleware to define your own clients to conform to the FHIR specification.

## Client Types

Clients are built using middleware. To define a client you create a middleware handler and pass it in as an argument to either the SynchronousClient, AsynchronousClient constructor.

## Middleware

### Arguments

Middleware is built using an array of functions each function is passed the following arguments:

| Name       | Description                                            | type        |
| ---------- | ------------------------------------------------------ | ----------- |
| request    | A FHIRRequest object information varies based on type. | FHIRRequest |
| args       | Object contains state and ctx                          | state,ctx   |
| args.state | Internal state of the middleware.                      | unknown     |
| args.ctx   | Context of the call (will vary based on client).       | unknown     |

### Response

Middleware should return the following object:

```typescript
{
  ctx: CTX;
  state: State;
  response: FHIRResponse;
}
```

Depending on whether client is asynchronous or syncrhonous will either be the object directly or a promise resolving to this object.

### Client

Constructor that allows you to create asynchronous FHIR clients.

```typescript
import {
  createMiddlewareAsync,
  MiddlewareAsync,
} from "@oxidized-health/client/lib/middleware/index.js";
import { AsynchronousClient } from "@oxidized-health/client/lib/index.js";

return new AsynchronousClient<StateType, CTX>(
  initialState,
  createMiddlewareAsync<State, CTX>(middlewarefunctions)
);
```

## HTTPClient

We provide an HTTP client by default that allows you to call FHIR servers via API calls.
Supports all the method calls within the AsynchronousClient interface:

```typescript
export interface FHIRClientAsync<CTX> {
  request(ctx: CTX, request: FHIRRequest): Promise<FHIRResponse>;
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
    resources: Resource<FHIRVersion, AllResourceTypes>[];
  }>;
  search_type<FHIRVersion extends FHIR_VERSION, T extends AllResourceTypes>(
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
  update<FHIRVersion extends FHIR_VERSION, T extends AllResourceTypes>(
    ctx: CTX,
    fhirVersion: FHIRVersion,
    resourceType: T,
    id: NonNullable<Resource<FHIRVersion, AllResourceTypes>["id"]>,
    resource: Resource<FHIRVersion, T>
  ): Promise<Resource<FHIRVersion, T>>;
  patch<FHIRVersion extends FHIR_VERSION, T extends AllResourceTypes>(
    ctx: CTX,
    fhirVersion: FHIRVersion,
    resourceType: T,
    id: NonNullable<Resource<FHIRVersion, AllResourceTypes>["id"]>,
    patches: any
  ): Promise<Resource<FHIRVersion, T>>;
  read<FHIRVersion extends FHIR_VERSION, T extends AllResourceTypes>(
    ctx: CTX,
    fhirVersion: FHIRVersion,
    resourceType: T,
    id: NonNullable<Resource<FHIRVersion, AllResourceTypes>["id"]>
  ): Promise<Resource<FHIRVersion, T> | undefined>;
  vread<FHIRVersion extends FHIR_VERSION, T extends AllResourceTypes>(
    ctx: CTX,
    fhirVersion: FHIRVersion,
    resourceType: T,
    id: NonNullable<Resource<FHIRVersion, AllResourceTypes>["id"]>,
    versionId: NonNullable<Resource<FHIRVersion, AllResourceTypes>["id"]>
  ): Promise<Resource<FHIRVersion, T> | undefined>;
  delete<FHIRVersion extends FHIR_VERSION, T extends ResourceType<FHIRVersion>>(
    ctx: CTX,
    fhirVersion: FHIRVersion,
    resourceType: T,
    id: NonNullable<Resource<FHIRVersion, AllResourceTypes>["id"]>
  ): Promise<void>;

  historySystem<FHIRVersion extends FHIR_VERSION>(
    ctx: CTX,
    fhirVersion: FHIRVersion,
    parameters?: ParsedParameter<string | number>[] | string
  ): Promise<NonNullable<Resource<FHIRVersion, "Bundle">["entry"]>>;
  historyType<FHIRVersion extends FHIR_VERSION, T extends AllResourceTypes>(
    ctx: CTX,
    fhirVersion: FHIRVersion,
    resourceType: T,
    parameters?: ParsedParameter<string | number>[] | string
  ): Promise<NonNullable<Resource<FHIRVersion, "Bundle">["entry"]>>;
  historyInstance<FHIRVersion extends FHIR_VERSION, T extends AllResourceTypes>(
    ctx: CTX,
    fhirVersion: FHIRVersion,
    resourceType: T,
    id: NonNullable<Resource<FHIRVersion, AllResourceTypes>["id"]>,
    parameters?: ParsedParameter<string | number>[] | string
  ): Promise<NonNullable<Resource<FHIRVersion, "Bundle">["entry"]>>;
  invoke_system<
    FHIRVersion extends FHIR_VERSION,
    Op extends IOperation<any, any>
  >(
    op: Op,
    ctx: CTX,
    fhirVersion: FHIRVersion,
    input: OPMetadata<Op>["Input"]
  ): Promise<OPMetadata<Op>["Output"]>;
  invoke_type<
    FHIRVersion extends FHIR_VERSION,
    Op extends IOperation<any, any>,
    T extends AllResourceTypes
  >(
    op: Op,
    ctx: CTX,
    fhirVersion: FHIRVersion,
    resourceType: T,
    input: OPMetadata<Op>["Input"]
  ): Promise<OPMetadata<Op>["Output"]>;
  invoke_instance<
    FHIRVersion extends FHIR_VERSION,
    Op extends IOperation<any, any>,
    T extends AllResourceTypes
  >(
    op: Op,
    ctx: CTX,
    fhirVersion: FHIRVersion,
    resourceType: T,
    id: NonNullable<Resource<FHIRVersion, AllResourceTypes>["id"]>,
    input: OPMetadata<Op>["Input"]
  ): Promise<OPMetadata<Op>["Output"]>;
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
```

### Usage

```typescript
import { expect, test } from "@jest/globals";

import HTTPClient from "@oxidized-health/client/http";
import { OperationDefinition } from "@oxidized-health/fhir-types/r4/types";
import { R4 } from "@oxidized-health/fhir-types/version";

const client = HTTPClient({
  url: "FHIR_API_ROOT_URL",
  getAccessToken: async function () {
    return "API TOKEN";
  },
});

const operationDefinition: OperationDefinition = {
  resourceType: "OperationDefinition",
  name: "test",
  status: "draft",
  kind: "operation",
  code: "my-operation",
  system: false,
  instance: false,
  type: false,
  parameter: [],
};
const response = await client.create({}, R4, operationDefinition);
expect(response).toMatchObject({
  resourceType: "OperationDefinition",
  name: "test",
  status: "draft",
  kind: "operation",
  code: "my-operation",
  system: false,
  instance: false,
  type: false,
  parameter: [],
});

await client.delete({}, R4, "OperationDefinition", response.id as string);
```
