/* eslint-disable @typescript-eslint/no-explicit-any */
import { Logger } from "pino";

import createHTTPClient, {
  isResponseError,
} from "@oxidized-health/client/http";
import {
  AllInteractions,
  FHIRRequest,
  FHIRResponse,
} from "@oxidized-health/client/lib/types";
import { parseQuery } from "@oxidized-health/client/url";
import {
  Loc,
  NullGuard,
  descend,
  pointer as fhirPointer,
  get,
  root,
} from "@oxidized-health/fhir-pointer";
import { code, id, markdown } from "@oxidized-health/fhir-types/r4/types";
import {
  AllResourceTypes,
  DataType,
  FHIR_VERSION,
  Resource,
  ResourceType,
} from "@oxidized-health/fhir-types/versions";
import * as fhirpath from "@oxidized-health/fhirpath";
import {
  OperationError,
  outcomeError,
  outcomeFatal,
} from "@oxidized-health/operation-outcomes";

type ResourceFixture<Version extends FHIR_VERSION> = {
  type: "resource";
  data: Resource<Version, AllResourceTypes>;
};

type ResponseFixture = {
  type: "response";
  data: FHIRResponse<FHIR_VERSION, AllInteractions | "error">;
};

type RequestFixture = {
  type: "request";
  data: FHIRRequest<FHIR_VERSION, AllInteractions>;
};

type Fixture<Version extends FHIR_VERSION> =
  | ResourceFixture<Version>
  | ResponseFixture
  | RequestFixture;

type TestScriptState<Version extends FHIR_VERSION> = {
  testScript: Resource<Version, "TestScript">;
  logger: Logger<string>;
  result: "pass" | "fail" | "pending";
  version: Version;
  fixtures: Record<string, Fixture<Version> | undefined>;
  latestResponse?: ResponseFixture;
  latestRequest?: RequestFixture;
  client: ReturnType<typeof createHTTPClient>;
  timeout?: number;
};

type TestScriptAction<Version extends FHIR_VERSION> = NonNullable<
  Resource<Version, "TestScript">["setup"]
>["action"][number];

type TestReportAction<Version extends FHIR_VERSION> = NonNullable<
  Resource<Version, "TestReport">["setup"]
>["action"][number];

function getFixtureResource<Version extends FHIR_VERSION>(
  state: TestScriptState<Version>,
  id: id
): Resource<Version, AllResourceTypes> {
  const v = state.fixtures[id];

  if (!v) {
    throw new OperationError(
      outcomeFatal("invalid", `fixture with id '${id}' not found`)
    );
  }

  switch (v.type) {
    case "response":
    case "request": {
      if ("body" in v.data) {
        return v.data.body as Resource<Version, AllResourceTypes>;
      }
      throw new OperationError(
        outcomeFatal(
          "invalid",
          `Response or Request body not found on fixture '${id}'`
        )
      );
    }
    case "resource": {
      if (
        v.data.resourceType === "Binary" &&
        v.data.contentType === "application/fhir+json" &&
        v.data.data
      ) {
        return JSON.parse(v.data.data);
      }
      return v.data;
    }
  }
}
const PATCH_CONTENT_TYPE = "application/json-patch+json";

function getPatches<Version extends FHIR_VERSION>(
  state: TestScriptState<Version>,
  id: id
) {
  const fixture = state.fixtures[id];
  if (fixture?.type !== "resource" || fixture.data.resourceType !== "Binary") {
    throw new OperationError(
      outcomeFatal("invalid", `Binary with id '${id}' not found`)
    );
  }

  if (fixture.data.contentType !== PATCH_CONTENT_TYPE) {
    throw new OperationError(
      outcomeFatal(
        "invalid",
        `Binary with id '${id}' must be of content type '${PATCH_CONTENT_TYPE}' `
      )
    );
  }
  if (!fixture.data.data) {
    throw new OperationError(
      outcomeFatal("invalid", `Binary with id '${id}' must have data.`)
    );
  }
  let patches;
  try {
    patches = JSON.parse(fixture.data.data);
  } catch (e) {
    throw new OperationError(outcomeError("invalid", `Invalid JSON Patch`));
  }

  if (!Array.isArray(patches)) {
    throw new OperationError(outcomeError("invalid", `Invalid JSON Patch`));
  }

  return patches;
}

async function getVariable<Version extends FHIR_VERSION>(
  state: TestScriptState<Version>,
  variables: NonNullable<Resource<Version, "TestScript">["variable"]>,
  variableName: string
): Promise<unknown> {
  const variable = variables.find((v) => v.name === variableName);
  if (!variable)
    throw new OperationError(
      outcomeFatal("invalid", `variable with id '${variableName}' not found`)
    );

  switch (true) {
    case variable.expression !== undefined: {
      return (
        await fhirpath.evaluate(
          variable.expression,
          variable.sourceId
            ? getFixtureResource(state, variable.sourceId)
            : undefined
        )
      )[0];
    }
    default: {
      throw new OperationError(
        outcomeFatal(
          "not-supported",
          `expression variables are only supported for now.`
        )
      );
    }
  }
}

/**
 * Return the default source (by default is Response in unless specified a sourceId or direction request)
 * @param state Current test state
 * @param assertion
 * @returns
 */
function getSource<Version extends FHIR_VERSION>(
  state: TestScriptState<Version>,
  assertion: NonNullable<TestScriptAction<Version>["assert"]>
): Resource<Version, AllResourceTypes> | undefined {
  switch (true) {
    case assertion.sourceId !== undefined: {
      return getFixtureResource(state, assertion.sourceId);
    }
    case assertion.direction === "request": {
      if (!state.latestRequest?.data || !("body" in state.latestRequest.data))
        return undefined;
      return state.latestRequest?.data.body as Resource<
        Version,
        AllResourceTypes
      >;
    }
    // Default can be assumed as response.
    case assertion.direction === "response":
    default: {
      if (!state.latestResponse?.data || !("body" in state.latestResponse.data))
        return undefined;
      return state.latestResponse?.data.body as Resource<
        Version,
        AllResourceTypes
      >;
    }
  }
}

const EXPRESSION_REGEX = /\${([^}]*)}/;

async function evaluateVariables<Version extends FHIR_VERSION>(
  state: TestScriptState<Version>,
  pointer: Loc<Resource<Version, "TestScript">, any, any>,
  value: string
): Promise<string> {
  let output = value;
  const variables =
    get(descend(root(pointer), "variable"), state.testScript) ?? [];

  while (EXPRESSION_REGEX.test(output)) {
    const result = EXPRESSION_REGEX.exec(output);
    const variableName = result?.[1];
    const match = result?.[0];

    if (!match || !variableName) {
      throw new Error("Invalid expression");
    }

    const variableValue = (
      await getVariable(state, variables, variableName)
    )?.toString();
    if (!variableValue)
      throw new OperationError(
        outcomeError("invalid", `Variable with id '${variableName}' not found.`)
      );

    output = output.replace(match, variableValue);
  }

  return output;
}

async function operationToFHIRRequest<Version extends FHIR_VERSION>(
  state: TestScriptState<Version>,
  pointer: Loc<
    Resource<Version, "TestScript">,
    | NonNullable<
        NonNullable<Resource<Version, "TestScript">["setup"]>["action"]
      >[number]["operation"]
    | undefined,
    any
  >
): Promise<FHIRRequest<Version, AllInteractions>> {
  // See https://hl7.org/fhir/r4/valueset-testscript-operation-codes.html

  const operation = get(pointer, state.testScript);
  // Extending this with "invoke" for invoking generic operations.
  switch (operation?.type?.code) {
    case "invoke": {
      // Assume the op code is provided by url.
      const op = operation.url;
      if (!op) {
        throw new OperationError(
          outcomeFatal(
            "invalid",
            `url is required for invoke operation which is used as the operation code.`
          )
        );
      }
      if (!operation.sourceId) {
        throw new OperationError(
          outcomeError("invalid", "sourceId is required for invoke operation")
        );
      }
      const parameters = getFixtureResource(state, operation.sourceId);
      if (parameters.resourceType !== "Parameters") {
        throw new OperationError(
          outcomeError(
            "invalid",
            `sourceId must point to Parameters resource for ${operation.sourceId}`
          )
        );
      }
      switch (true) {
        case operation.targetId !== undefined: {
          const target = getFixtureResource(state, operation.targetId);
          if (!target.id)
            throw new OperationError(
              outcomeFatal(
                "invalid",
                `targetId must have an id on resource for '${operation.targetId}'`
              )
            );
          return {
            fhirVersion: state.version,
            level: "instance",
            type: "invoke-request",
            resource: operation.resource ?? target.resourceType,
            id: target.id,
            operation: op as code,
            body: parameters,
          } as FHIRRequest<Version, "invoke">;
        }
        case operation.resource !== undefined: {
          return {
            fhirVersion: state.version,
            level: "type",
            type: "invoke-request",
            resource: operation.resource,
            operation: op as code,
            body: parameters,
          } as FHIRRequest<Version, "invoke">;
        }
        default: {
          return {
            fhirVersion: state.version,
            level: "system",
            type: "invoke-request",
            operation: op,
            body: parameters,
          } as FHIRRequest<Version, "invoke">;
        }
      }
    }
    case "read": {
      if (!operation.targetId)
        throw new OperationError(
          outcomeFatal("invalid", "targetId is required for read operation")
        );

      const target = getFixtureResource(state, operation.targetId);

      return {
        fhirVersion: state.version,
        level: "instance",
        type: "read-request",
        resource:
          operation.resource ??
          (target?.resourceType as unknown as ResourceType<Version>),
        id: target?.id as id,
      } as FHIRRequest<Version, "read">;
    }
    case "create": {
      if (!operation.sourceId)
        throw new OperationError(
          outcomeFatal("invalid", "sourceId is required for read operation")
        );
      const target = getFixtureResource(state, operation.sourceId);
      return {
        fhirVersion: state.version,
        level: "type",
        type: "create-request",
        resource: operation.resource ?? target.resourceType,
        body: target,
      } as FHIRRequest<Version, "create">;
    }
    case "patch": {
      if (!operation.targetId)
        throw new OperationError(
          outcomeFatal("invalid", "targetId is required for patch operation")
        );
      if (!operation.sourceId)
        throw new OperationError(
          outcomeFatal("invalid", "sourceId is required for patch operation")
        );
      const target = getFixtureResource(state, operation.targetId);
      return {
        fhirVersion: state.version,
        level: "instance",
        type: "patch-request",
        resource:
          operation.resource ??
          (target?.resourceType as unknown as ResourceType<Version>),
        id: target?.id as id,
        body: getPatches(state, operation.sourceId),
      } as FHIRRequest<Version, "patch">;
    }
    case "vread": {
      if (!operation.targetId)
        throw new OperationError(
          outcomeFatal("invalid", "targetId is required for vread operation")
        );
      const target = getFixtureResource(state, operation.targetId);
      return {
        fhirVersion: state.version,
        level: "instance",
        type: "vread-request",
        resource: operation.resource ?? target?.resourceType,
        id: target?.id as id,
        versionId: getFixtureResource(state, operation.targetId)?.meta
          ?.versionId as id,
      } as FHIRRequest<Version, "vread">;
    }

    case "update": {
      if (!operation.sourceId)
        throw new OperationError(
          outcomeFatal("invalid", "sourceId is required for update operation")
        );
      switch (true) {
        case operation.params !== undefined: {
          let source = getFixtureResource(state, operation.sourceId);
          if (operation.targetId) {
            const target = getFixtureResource(state, operation.targetId);
            source = {
              ...source,
              resourceType: (operation.resource ??
                target?.resourceType) as unknown as ResourceType<Version>,
              id: target?.id as id,
            };
          }
          return {
            fhirVersion: state.version,
            level: "type",
            type: "update-request",
            resource: operation.resource,
            parameters: parseQuery(
              await evaluateVariables(state, pointer, operation.params)
            ),
            body: source,
          } as FHIRRequest<Version, "update">;
        }
        case operation.targetId !== undefined: {
          const target = getFixtureResource(state, operation.targetId);
          return {
            fhirVersion: state.version,
            level: "instance",
            type: "update-request",
            resource:
              operation.resource ??
              (target?.resourceType as unknown as ResourceType<Version>),
            id: target?.id as id,
            body: getFixtureResource(state, operation.sourceId),
          } as FHIRRequest<Version, "update">;
        }
        default: {
          throw new OperationError(
            outcomeFatal(
              "business-rule",
              "Must have either a targetId or params for update."
            )
          );
        }
      }
    }
    case "delete": {
      switch (true) {
        case operation.targetId !== undefined: {
          const target = getFixtureResource(state, operation.targetId);
          return {
            fhirVersion: state.version,
            level: "instance",
            type: "delete-request",
            resource: operation.resource ?? target?.resourceType,
            id: target?.id as id,
          } as FHIRRequest<Version, "delete">;
        }
        case operation.resource !== undefined: {
          return {
            fhirVersion: state.version,
            level: "type",
            type: "delete-request",
            resource: operation.resource,
            parameters: parseQuery(
              await evaluateVariables(state, pointer, operation.params ?? "")
            ),
          } as FHIRRequest<Version, "delete">;
        }
        case operation.params !== undefined: {
          return {
            fhirVersion: state.version,
            level: "system",
            type: "delete-request",
            parameters: parseQuery(
              await evaluateVariables(state, pointer, operation.params ?? "")
            ),
          } as FHIRRequest<Version, "delete">;
        }
        default: {
          throw new OperationError(
            outcomeFatal("invalid", "Delete request is invalid.")
          );
        }
      }
    }

    case "search": {
      if (operation.resource) {
        return {
          fhirVersion: state.version,
          level: "type",
          type: "search-request",
          resource: operation.resource,
          parameters: parseQuery(
            await evaluateVariables(state, pointer, operation.params ?? "")
          ),
        } as FHIRRequest<Version, "search">;
      } else {
        return {
          fhirVersion: state.version,
          level: "system",
          type: "search-request",
          parameters: parseQuery(
            await evaluateVariables(state, pointer, operation.params ?? "")
          ),
        } as FHIRRequest<Version, "search">;
      }
    }
    case "batch": {
      if (!operation.sourceId)
        throw new OperationError(
          outcomeFatal("invalid", "sourceId is required for read operation")
        );

      const batch = getFixtureResource(state, operation.sourceId);

      if (batch.resourceType !== "Bundle" || batch.type !== "batch") {
        throw new OperationError(
          outcomeFatal(
            "invalid",
            `Batch request must be a bundle and a of type batch. Found instead '${
              batch.resourceType
            }' of type '${"type" in batch ? batch.type : "invalid"}'`
          )
        );
      }

      return {
        fhirVersion: state.version,
        level: "system",
        type: "batch-request",
        body: batch,
      } as FHIRRequest<Version, "batch">;
    }
    case "transaction": {
      if (!operation.sourceId)
        throw new OperationError(
          outcomeFatal("invalid", "sourceId is required for read operation")
        );

      const transaction = getFixtureResource(state, operation.sourceId);

      if (
        transaction.resourceType !== "Bundle" ||
        transaction.type !== "transaction"
      ) {
        throw new OperationError(
          outcomeFatal(
            "invalid",
            "Transaction request must be a bundle and a of type transaction."
          )
        );
      }

      return {
        fhirVersion: state.version,
        level: "system",
        type: "transaction-request",
        body: transaction,
      } as FHIRRequest<Version, "transaction">;
    }
    case "history": {
      switch (true) {
        case operation.targetId !== undefined: {
          const target = getFixtureResource(state, operation.targetId);
          return {
            fhirVersion: state.version,
            level: "instance",
            type: "history-request",
            resource:
              operation.resource ??
              (target?.resourceType as unknown as ResourceType<Version>),
            id: target?.id as id,
            parameters: parseQuery(
              await evaluateVariables(state, pointer, operation.params ?? "")
            ),
          } as FHIRRequest<Version, "history">;
        }
        case operation.resource !== undefined: {
          return {
            fhirVersion: state.version,
            level: "type",
            type: "history-request",
            resource: operation.resource,
            parameters: parseQuery(
              await evaluateVariables(state, pointer, operation.params ?? "")
            ),
          } as FHIRRequest<Version, "history">;
        }
        default: {
          return {
            fhirVersion: state.version,
            level: "system",
            type: "history-request",
            parameters: parseQuery(
              await evaluateVariables(state, pointer, operation.params ?? "")
            ),
          } as FHIRRequest<Version, "history">;
        }
      }
    }
    default: {
      throw new OperationError(
        outcomeFatal(
          "not-supported",
          `operation of type '${operation?.type?.code}' is not supported`
        )
      );
    }
  }
}

const numberRegex = /^-?\d+(\.\d+)?$/;

async function deriveComparision<Version extends FHIR_VERSION>(
  state: TestScriptState<Version>,
  assertion: NonNullable<TestScriptAction<Version>["assert"]>
): Promise<unknown> {
  if (assertion.compareToSourceId) {
    const data = getFixtureResource(state, assertion.compareToSourceId as id);

    switch (true) {
      case assertion.compareToSourceExpression !== undefined: {
        return (
          await fhirpath.evaluate(assertion.compareToSourceExpression, data)
        )[0];
      }
    }
  } else if (assertion.value) {
    switch (true) {
      case numberRegex.test(assertion.value): {
        return parseFloat(assertion.value);
      }
      case assertion.value === "true": {
        return true;
      }
      case assertion.value === "false": {
        return false;
      }
      default: {
        return assertion.value;
      }
    }
  }

  throw new Error("Invalid Value could not derive comparison.");
}

/**
 * [https://hl7.org/fhir/r4/valueset-assert-operator-codes.html]
 * equals	equals	Default value. Equals comparison.
 * notEquals	notEquals	Not equals comparison.
 * in	in	Compare value within a known set of values.
 * notIn	notIn	Compare value not within a known set of values.
 * greaterThan	greaterThan	Compare value to be greater than a known value.
 * lessThan	lessThan	Compare value to be less than a known value.
 * empty	empty	Compare value is empty.
 * notEmpty	notEmpty	Compare value is not empty.
 * contains	contains	Compare value string contains a known value.
 * notContains	notContains	Compare value string does not contain a known value.
 * eval	evaluate	Evaluate the FHIRPath expression as a boolean condition.
 */
function evaluateOperator(
  v1: unknown,
  v2: unknown,
  operator: code = "equals" as code
): boolean {
  switch (operator) {
    case "equals":
      return JSON.stringify(v1) === JSON.stringify(v2);
    case "notEquals":
      return JSON.stringify(v1) !== JSON.stringify(v2);
    case "in":
      return Array.isArray(v2) ? v2.includes(v1) : false;
    case "notIn":
      return Array.isArray(v2) ? !v2.includes(v1) : false;
    case "greaterThan":
      return parseInt(v1?.toString() ?? "") > parseInt(v2?.toString() ?? "");
    case "lessThan":
      return parseInt(v1?.toString() ?? "") < parseInt(v2?.toString() ?? "");
    case "empty":
      return Array.isArray(v1)
        ? v1.length === 0
        : v1 === undefined || v1 === null;
    case "notEmpty":
      return Array.isArray(v1)
        ? v1.length !== 0
        : v1 !== undefined && v1 !== null;
    case "contains":
      return typeof v1 === "string" && typeof v2 === "string"
        ? v1.includes(v2)
        : false;
    case "notContains":
      return typeof v1 === "string" && typeof v2 === "string"
        ? !v1.includes(v2)
        : false;
    case "eval":
      return v1 === v2;
    default:
      throw new OperationError(
        outcomeError("invalid", `Invalid operator '${operator}'`)
      );
  }
}

function assertionFailureMessage(
  found: unknown,
  expected: unknown,
  operator: code = "equals" as code
): string {
  return `Failed Assertion FOUND['${JSON.stringify(
    found
  )}'] !${operator} EXPECTED['${expected ? JSON.stringify(expected) : ""}']`;
}

async function runAssertion<Version extends FHIR_VERSION>(
  state: TestScriptState<Version>,
  pointer: Loc<
    Resource<Version, "TestScript">,
    | NonNullable<
        NonNullable<Resource<Version, "TestScript">["setup"]>["action"]
      >[number]["assert"]
    | undefined,
    any
  >
): Promise<{
  state: TestScriptState<Version>;
  result: NonNullable<TestReportAction<Version>["assert"]>;
}> {
  const assertion = get(pointer, state.testScript);
  if (!assertion)
    throw new OperationError(
      outcomeFatal("invalid", `Assertion not found at ${pointer}`)
    );

  const source = getSource(state, assertion);

  if (assertion.resource) {
    if (
      source === undefined ||
      !("resourceType" in source) ||
      !evaluateOperator(
        source.resourceType,
        assertion.resource,
        assertion.operator
      )
    ) {
      const result = {
        result: "fail",
        message: assertionFailureMessage(
          source?.resourceType ?? "InvalidResourceType",
          assertion.resource,
          assertion.operator
        ),
      } as NonNullable<TestReportAction<Version>["assert"]>;
      state.logger.error({ pointer, assertion, result });

      return {
        state: { ...state, result: "fail" },
        result,
      };
    }
  }
  if (assertion.expression) {
    const compValue = await deriveComparision(state, assertion);
    const fpEvaluation = (
      await fhirpath.evaluate(assertion.expression, source)
    )[0];

    if (!evaluateOperator(fpEvaluation, compValue, assertion.operator)) {
      const result = {
        result: "fail",
        message: assertionFailureMessage(
          fpEvaluation,
          compValue,
          assertion.operator
        ),
      } as NonNullable<TestReportAction<Version>["assert"]>;

      state.logger.error({ pointer, assertion, result });

      return {
        state: { ...state, result: "fail" },
        result,
      };
    }
  }

  const result = {
    result: "pass",
    message: "Assertions passed",
  } as NonNullable<TestReportAction<Version>["assert"]>;
  state.logger.info({ pointer, assertion, result });

  return {
    state,
    result,
  };
}

function associateResponseRequestVariables<Version extends FHIR_VERSION>(
  _fixtures: TestScriptState<Version>["fixtures"],
  operation: NonNullable<TestScriptAction<Version>["operation"]>,
  request: FHIRRequest<Version, AllInteractions>,
  response: FHIRResponse<Version, AllInteractions | "error">
) {
  let fixtures = { ..._fixtures };
  if (operation.responseId) {
    fixtures = {
      ...fixtures,
      [operation.responseId]: { type: "response", data: response },
    };
  }
  if (operation.requestId) {
    fixtures = {
      ...fixtures,
      [operation.requestId]: { type: "request", data: request },
    };
  }

  return fixtures;
}

async function runOperation<Version extends FHIR_VERSION>(
  state: TestScriptState<Version>,
  pointer: Loc<
    Resource<Version, "TestScript">,
    | NonNullable<
        NonNullable<Resource<Version, "TestScript">["setup"]>["action"]
      >[number]["operation"]
    | undefined,
    any
  >
): Promise<{
  state: TestScriptState<Version>;
  result: NonNullable<TestReportAction<Version>["operation"]>;
}> {
  const operation = get(pointer, state.testScript);
  if (!operation)
    throw new OperationError(
      outcomeFatal("invalid", `Operation not found at '${pointer}'`)
    );

  try {
    const request = await operationToFHIRRequest(state, pointer);
    const response = await state.client.request({}, request);

    // Hack because async so will wait a bit.
    await new Promise((resolve) =>
      setTimeout(() => resolve(undefined), state.timeout ?? 0)
    );

    const result = {
      result: "pass" as code,
      message: `[SUCCEEDED]<${operation.type?.code}> label: '${
        operation.label ?? ""
      }'` as markdown,
    };

    state.logger.info({ pointer, operation, result });
    return {
      state: {
        ...state,
        fixtures: associateResponseRequestVariables(
          state.fixtures,
          operation,
          request,
          response
        ),
        latestRequest: { type: "request", data: request },
        latestResponse: { type: "response", data: response },
      },
      result,
    };
  } catch (e) {
    const result = {
      result: "fail" as code,
      message: `[FAILED]<${operation.type?.code}> label: '${
        operation.label ?? ""
      }'` as markdown,
      outcome: e instanceof OperationError ? e.operationOutcome : undefined,
    };
    if (isResponseError(e)) {
      return {
        state: {
          ...state,
          fixtures: associateResponseRequestVariables(
            state.fixtures,
            operation,
            e.request,
            e.response
          ),
          latestRequest: { type: "request", data: e.request },
          latestResponse: { type: "response", data: e.response },
        },
        result,
      };
    }

    throw e;
  }
}

async function runAction<Version extends FHIR_VERSION>(
  state: TestScriptState<Version>,
  pointer: Loc<
    Resource<Version, "TestScript">,
    | NonNullable<
        NonNullable<Resource<Version, "TestScript">["setup"]>["action"]
      >[number]
    | undefined,
    any
  >
): Promise<{
  state: TestScriptState<Version>;
  result: TestReportAction<Version>;
}> {
  const action = get(pointer, state.testScript);
  if (action?.operation && action?.assert) {
    throw new OperationError(
      outcomeFatal(
        "invalid",
        "Must have either operation or assert but not both."
      )
    );
  }
  switch (true) {
    case action?.operation !== undefined: {
      const output = await runOperation(state, descend(pointer, "operation"));
      return {
        state: output.state,
        result: { operation: output.result },
      };
    }
    case action?.assert !== undefined: {
      const output = await runAssertion(state, descend(pointer, "assert"));
      return {
        state: output.state,
        result: { assert: output.result },
      };
    }
    default: {
      throw new OperationError(
        outcomeFatal(
          "code-invalid",
          "Unreachable case in evaluation action hit."
        )
      );
    }
  }
}

async function runTest<Version extends FHIR_VERSION>(
  state: TestScriptState<Version>,
  pointer: Loc<
    Resource<Version, "TestScript">,
    NonNullable<Resource<Version, "TestScript">["test"]>[number] | undefined,
    any
  >
): Promise<{
  state: TestScriptState<Version>;
  result: TestReportAction<Version>[];
}> {
  let curState = state;
  const testReports: TestReportAction<Version>[] = [];
  const test = get(pointer, state.testScript);
  const actions = test?.action ?? [];

  for (let i = 0; i < actions.length; i++) {
    const output = await runAction(
      {
        ...curState,
        logger: state.logger.child({
          test: test?.name,
          description: test?.description,
        }),
      },
      descend(descend(pointer, "action"), i)
    );
    curState = output.state;

    testReports.push(output.result);
  }

  return { state: { ...curState, logger: state.logger }, result: testReports };
}

async function runTests<Version extends FHIR_VERSION>(
  state: TestScriptState<Version>,
  pointer: Loc<
    Resource<Version, "TestScript">,
    NullGuard<Resource<Version, "TestScript">, "test">,
    any
  >
): Promise<{
  state: TestScriptState<Version>;
  result: Resource<Version, "TestReport">["test"];
}> {
  const testResults = [];
  let curState = state;

  const tests = get(pointer, state.testScript) ?? [];

  for (let i = 0; i < tests.length; i++) {
    const output = await runTest(state, descend(pointer, i));
    curState = output.state;
    testResults.push({
      name: get(descend(pointer, i), state.testScript)?.name,
      action: output.result,
    });
  }

  return {
    state:
      curState.result === "pending"
        ? { ...curState, result: "pass" }
        : curState,
    result: testResults,
  };
}

async function runSetup<Version extends FHIR_VERSION>(
  state: TestScriptState<Version>,
  pointer: Loc<
    Resource<Version, "TestScript">,
    NullGuard<Resource<Version, "TestScript">, "setup">,
    any
  >
): Promise<{
  state: TestScriptState<Version>;
  result: Resource<Version, "TestReport">["setup"];
}> {
  let curState = state;
  const setupResults = [];
  const actions = get(pointer, state.testScript)?.action ?? [];
  for (let i = 0; i < actions.length; i++) {
    const output = await runAction(
      curState,
      descend(descend(pointer, "action"), i)
    );
    curState = output.state;
    setupResults.push(output.result);
  }

  return { state: curState, result: { action: setupResults } };
}

async function runTeardown<Version extends FHIR_VERSION>(
  state: TestScriptState<Version>,
  pointer: Loc<
    Resource<Version, "TestScript">,
    NullGuard<Resource<Version, "TestScript">, "teardown">,
    any
  >
): Promise<{
  state: TestScriptState<Version>;
  result: Resource<Version, "TestReport">["teardown"];
}> {
  let curState = state;
  const teardownResults = [];
  const actions = get(pointer, state.testScript)?.action ?? [];
  for (let i = 0; i < actions.length; i++) {
    const output = await runOperation(
      curState,
      descend(descend(descend(pointer, "action"), i), "operation")
    );
    curState = output.state;
    teardownResults.push({ operation: output.result });
  }

  state.logger.info({ message: "teardown results:", teardownResults });

  return { state: curState, result: { action: teardownResults } };
}

async function resolveFixtures<Version extends FHIR_VERSION>(
  state: TestScriptState<Version>,
  pointer: Loc<
    Resource<Version, "TestScript">,
    Resource<Version, "TestScript">,
    any
  >
): Promise<TestScriptState<Version>> {
  for (const fixture of get(descend(pointer, "fixture"), state.testScript) ??
    []) {
    if (!fixture.id) {
      throw new OperationError(
        outcomeFatal("invalid", "fixture must have an id")
      );
    }

    const resource = fixture.resource;
    if (!resource) {
      throw new OperationError(
        outcomeFatal("invalid", "fixture must have a resource")
      );
    }

    let resolvedResource;

    switch (true) {
      case fixture.resource?.reference?.startsWith("#"): {
        const id = (fixture.resource?.reference ?? "").slice(1);
        resolvedResource = get(
          descend(pointer, "contained"),
          state.testScript
        )?.find((r) => r.id === id);
        break;
      }
      default: {
        const [type, id] = fixture.resource?.reference?.split("/") ?? [];

        if (!type || !id) {
          throw new OperationError(
            outcomeFatal(
              "invalid",
              `invalid reference '${fixture.resource?.reference}'`
            )
          );
        }

        resolvedResource = await state.client.read(
          {},
          state.version,
          type as ResourceType<Version>,
          id as id
        );
      }
    }

    if (!resolvedResource) {
      throw new OperationError(
        outcomeFatal(
          "invalid",
          `contained resource with reference '${fixture.resource?.reference}' not found`
        )
      );
    }

    state = {
      ...state,
      fixtures: {
        ...state.fixtures,
        [fixture.id]: {
          type: "resource",
          data: resolvedResource as Resource<Version, AllResourceTypes>,
        },
      },
    };
  }

  return state;
}

/**
 * Execute a TestScript and return results as TestReport.
 * @param client OxidizedHealth client instance
 * @param version FHIR Version
 * @param testScript The testscript to run.
 * @returns TestReport of results from testscript.
 */
export async function run<Version extends FHIR_VERSION>(
  logger: Logger<string>,
  client: ReturnType<typeof createHTTPClient>,
  version: Version,
  testScript: Resource<Version, "TestScript">,
  timeout: number = 0
): Promise<Resource<Version, "TestReport">> {
  const testReport = {
    testScript: { reference: `TestScript/${testScript.id}` },
    result: "pending",
    resourceType: "TestReport",
  } as Resource<Version, "TestReport">;

  const pointer = fhirPointer<Version, DataType<Version>>(
    version,
    "TestScript" as DataType<Version>,
    testScript.id as id
  ) as unknown as Loc<
    Resource<Version, "TestScript">,
    Resource<Version, "TestScript">,
    any
  >;

  let state = await resolveFixtures(
    {
      logger,
      result: "pending",
      version,
      fixtures: {},
      client,
      testScript,
      timeout,
    },
    pointer
  );

  try {
    const output = await runSetup(state, descend(pointer, "setup"));
    state = output.state;
    testReport.setup = output.result;

    const testsOutput = await runTests(state, descend(pointer, "test"));

    testReport.test = testsOutput.result;
    state = testsOutput.state;
    testReport.result = state.result as code;

    return testReport;
  } finally {
    const output = await runTeardown(state, descend(pointer, "teardown"));
    state = output.state;
    testReport.result = state.result as code;
    testReport.teardown = output.result;
  }
}
