import { expect, test } from "@jest/globals";
import { fileURLToPath } from "url";

import { loadArtifacts } from "@oxidized-health/artifacts";
import {
  OperationDefinition,
  Parameters,
  canonical,
  uri,
} from "@oxidized-health/fhir-types/lib/generated/r4/types";
import {
  AllResourceTypes,
  FHIR_VERSION,
  R4,
  Resource,
  ResourceType,
} from "@oxidized-health/fhir-types/lib/versions";
import {
  OperationError,
  outcome,
  outcomeFatal,
} from "@oxidized-health/operation-outcomes";

import {
  IOperation,
  Invocation,
  OpCTX,
  Operation,
  parseParameters,
  toParametersResource,
} from "./index";

const operationDefinitions = loadArtifacts({
  fhirVersion: R4,
  loadDevelopmentPackages: true,
  resourceType: "OperationDefinition",
  currentDirectory: fileURLToPath(import.meta.url),
});

const structureDefinitions = loadArtifacts({
  fhirVersion: R4,
  loadDevelopmentPackages: true,
  resourceType: "StructureDefinition",
  currentDirectory: fileURLToPath(import.meta.url),
});

const valueSetExpandOp = operationDefinitions.find(
  (op) => op.id === "ValueSet-expand"
);

if (!valueSetExpandOp)
  throw new Error("could not resolve valueset expand operation definition");

test("parseParameters", () => {
  expect(
    parseParameters(valueSetExpandOp, "in", {
      resourceType: "Parameters",
      parameter: [
        { name: "url", valueUri: "https://my-valueset.com" },
        {
          name: "valueSetVersion",
          valueString: "12",
        },
        {
          name: "filter",
          valueString: "test",
        },
      ],
    } as Parameters)
  ).toEqual({
    filter: "test",
    url: "https://my-valueset.com",
    valueSetVersion: "12",
  });
});

test("No Extra Parameters Allowed", () => {
  expect(() => {
    parseParameters(valueSetExpandOp, "in", {
      resourceType: "Parameters",
      parameter: [
        { name: "extra parameter", valueString: "test" },
        { name: "url", valueUri: "https://my-valueset.com" },
        {
          name: "valueSetVersion",
          valueString: "12",
        },
        {
          name: "filter",
          valueString: "test",
        },
      ],
    } as Parameters);
  }).toThrow();
});

const operationTest: OperationDefinition = {
  resourceType: "OperationDefinition",
  id: "test",
  status: "active",
  kind: "operation",
  name: "test",
  code: "test",
  system: true,
  type: false,
  instance: false,

  parameter: [
    { name: "name", type: "HumanName", use: "in", min: 0, max: "1" },
    { name: "patient", type: "Patient", use: "in", min: 0, max: "1" },
    {
      name: "testOut",
      type: "string",
      use: "out",
      min: 1,
      max: "1",
    },
    {
      name: "test",
      type: "string",
      use: "in",
      min: 1,
      max: "1",
    },
    {
      name: "test2",
      type: "integer",
      use: "in",
      min: 0,
      max: "*",
    },
    {
      name: "nested",
      use: "in",
      min: 0,
      max: "*",
      part: [
        {
          name: "nested1",
          type: "string",
          use: "in",
          min: 1,
          max: "1",
        },
        {
          name: "nested2",
          use: "in",
          min: 1,
          max: "1",
          part: [
            {
              name: "nested3",
              type: "string",
              use: "in",
              min: 1,
              max: "*",
            },
          ],
        },
      ],
    },
  ],
} as OperationDefinition;

test("Test Operation 1", () => {
  expect(
    parseParameters(operationTest, "in", {
      resourceType: "Parameters",
      parameter: [
        { name: "test", valueString: "value1" },
        { name: "test2", valueInteger: 5 },
        { name: "nested", part: [{ name: "nested1", valueString: "value2" }] },
        {
          name: "nested",
          part: [
            { name: "nested1", valueString: "value3" },
            {
              name: "nested2",
              part: [{ name: "nested3", valueString: "value4" }],
            },
          ],
        },
      ],
    } as Parameters)
  ).toEqual({
    test: "value1",
    test2: [5],
    nested: [
      {
        nested1: "value2",
      },
      {
        nested1: "value3",
        nested2: {
          nested3: ["value4"],
        },
      },
    ],
  });
});

test("roundTrip", () => {
  const operation: IOperation<{ test: string }, { testOut: string }> =
    new Operation<{ test: string }, { testOut: string }>(operationTest);

  const parameters: Parameters = {
    resourceType: "Parameters",
    parameter: [
      { name: "test", valueString: "value1" },
      { name: "test2", valueInteger: 5 },
      { name: "nested", part: [{ name: "nested1", valueString: "value2" }] },
      {
        name: "nested",
        part: [
          { name: "nested1", valueString: "value3" },
          {
            name: "nested2",
            part: [{ name: "nested3", valueString: "value4" }],
          },
        ],
      },
    ],
  } as Parameters;
  expect(
    operation.parseToParameters("in", operation.parseToObject("in", parameters))
  ).toEqual(parameters);
});

test("execution", async () => {
  const operation: IOperation<{ test: string }, { testOut: string }> =
    new Operation<{ test: string }, { testOut: string }>(operationTest);

  const ctx: OpCTX = {
    fhirVersion: R4,
    async resolveCanonical<
      FHIRVersion extends FHIR_VERSION,
      Type extends ResourceType<FHIRVersion>
    >(
      fhirVersion: FHIRVersion,
      type: Type,
      url: canonical
    ): Promise<Resource<FHIRVersion, Type> | undefined> {
      const sd = structureDefinitions.find((sd) => sd.url === url);
      if (!sd) throw new Error(`Could not resolve url ${url}`);
      return sd as Resource<FHIRVersion, Type> | undefined;
    },
    level: "instance",
  };

  const invoke: Invocation = async (op, ctx, input) => {
    const inputIssues = await op.validate(ctx, "in", input);
    if (inputIssues.length > 0) throw new OperationError(outcome(inputIssues));

    // @ts-ignore
    const output = { testOut: input.test };
    const outputIssues = await op.validate(ctx, "out", output);
    if (outputIssues.length > 0)
      throw new OperationError(outcome(outputIssues));
    return output;
  };

  expect(invoke(operation, ctx, { test: "asdf" })).resolves.toEqual({
    testOut: "asdf",
  });

  expect(
    invoke(
      operation,
      ctx,
      // @ts-ignore
      { test: "asdf", bad: "value" }
    )
  ).rejects.toThrow();

  const invokeBadOutput: Invocation = async (op, ctx, input) => {
    const inputIssues = await op.validate(ctx, "in", input);
    if (inputIssues.length > 0) throw new OperationError(outcome(inputIssues));
    // @ts-ignore
    const output = { testOut: input.test, z: 5 };
    const outputIssues = await op.validate(ctx, "out", output);
    if (outputIssues.length > 0)
      throw new OperationError(outcome(outputIssues));

    return output;
  };

  expect(invokeBadOutput(operation, ctx, { test: "asdf" })).rejects.toThrow();
});

test("paramValidation", async () => {
  const operation: IOperation<
    { test: string; [keyword: string]: unknown },
    { testOut: string }
  > = new Operation<
    { test: string; [keyword: string]: unknown },
    { testOut: string }
  >(operationTest);

  const ctx: OpCTX = {
    fhirVersion: R4,
    async resolveCanonical<
      FHIRVersion extends FHIR_VERSION,
      Type extends ResourceType<FHIRVersion>
    >(
      fhirVersion: FHIRVersion,
      type: Type,
      url: canonical
    ): Promise<Resource<FHIRVersion, Type> | undefined> {
      const sd = structureDefinitions.find((sd) => sd.url === url);
      if (!sd) throw new Error(`Could not resolve url ${url}`);
      return sd as Resource<FHIRVersion, Type> | undefined;
    },
    level: "instance",
  };

  const invoke: Invocation = async (op, ctx, input) => {
    const inputIssues = await op.validate(ctx, "in", input);
    if (inputIssues.length > 0) throw new OperationError(outcome(inputIssues));
    // @ts-ignore
    const output = { testOut: input.test };
    const outputIssues = await op.validate(ctx, "out", output);
    if (outputIssues.length > 0)
      throw new OperationError(outcome(outputIssues));
    return output;
  };

  expect(
    invoke(operation, ctx, { test: "asdf", name: { given: "Bob" } }).catch(
      (e) => {
        throw e.operationOutcome;
      }
    )
  ).rejects.toEqual({
    issue: [
      {
        code: "structure",
        diagnostics: "Element is expected to be an array.",
        expression: ["/0/given"],
        severity: "error",
      },
    ],
    resourceType: "OperationOutcome",
  });

  expect(
    invoke(operation, ctx, { test: "test", name: { given: ["Bob"] } })
  ).resolves.toEqual({ testOut: "test" });

  expect(
    invoke(operation, ctx, {
      test: "test",
      name: { given: ["Bob"] },
      patient: { resourceType: "Patien", name: [{ given: ["Hello"] }] },
    })
  ).rejects.toThrow();

  expect(
    invoke(operation, ctx, {
      test: "test",
      name: { given: ["Bob"] },
      patient: { resourceType: "Patient", name: [{ given: ["Hello"] }] },
    })
  ).resolves.toEqual({ testOut: "test" });

  expect(
    invoke(operation, ctx, {
      test: "test",
      name: { given: ["Bob"] },
      patient: { resourceType: "Patient", name: [{ given: [4] }] },
    }).catch((e) => {
      throw e.operationOutcome;
    })
  ).rejects.toEqual({
    issue: [
      {
        code: "structure",
        diagnostics:
          "Expected primitive type 'string' at path '/0/name/0/given/0'",
        expression: ["/0/name/0/given/0"],
        severity: "error",
      },
    ],
    resourceType: "OperationOutcome",
  });
  try {
    await invoke(operation, ctx, {
      test: "test",
      name: { given: ["Bob"] },
      patient: {
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
      },
    });
  } catch (e) {
    console.log(JSON.stringify(e));
  }

  expect(
    invoke(operation, ctx, {
      test: "test",
      name: { given: ["Bob"] },
      patient: {
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
      },
    })
  ).resolves.toEqual({ testOut: "test" });

  expect(
    invoke(operation, ctx, {
      test: "test",
      name: { given: ["Bob"] },
      patient: {
        resourceType: "Patient",
        name: [
          {
            _given: [
              {
                id: "123",
                extension: [{ url: "testing", valueString: 4 }],
              },
            ],
          },
        ],
      },
    })
  ).rejects.toThrow();
});

test("Test mapToParameters", () => {
  const op: OperationDefinition = {
    resourceType: "OperationDefinition",
    id: "test",
    status: "active",
    kind: "operation",
    name: "test",
    code: "test",
    system: true,
    type: false,
    instance: false,

    parameter: [
      {
        max: "1",
        min: 1,
        use: "out",
        name: "test",
        type: "string",
      },
      {
        max: "*",
        min: 1,
        use: "in",
        name: "payload",
        type: "Resource",
      },
    ],
  } as OperationDefinition;
  expect(toParametersResource(op, "in", { payload: "test" })).toEqual({
    resourceType: "Parameters",
    parameter: [
      {
        name: "payload",
        resource: "test",
      },
    ],
  });
});

test("Test invalid resource validation", async () => {
  const op: OperationDefinition = {
    resourceType: "OperationDefinition",
    id: "test",
    status: "active",
    kind: "operation",
    name: "test",
    code: "test",
    system: true,
    type: false,
    instance: false,

    parameter: [
      {
        max: "1",
        min: 1,
        use: "out",
        name: "test",
        type: "string",
      },
      {
        max: "*",
        min: 1,
        use: "in",
        name: "payload",
        type: "Resource",
      },
    ],
  } as OperationDefinition;
  const operation: IOperation<
    { payload: Resource<R4, AllResourceTypes>[] },
    { test: string }
  > = new Operation<
    { payload: Resource<R4, AllResourceTypes>[] },
    { test: string }
  >(op);

  const ctx: OpCTX = {
    fhirVersion: R4,
    async resolveCanonical<
      FHIRVersion extends FHIR_VERSION,
      Type extends ResourceType<FHIRVersion>
    >(
      fhirVersion: FHIRVersion,
      type: Type,
      url: canonical
    ): Promise<Resource<FHIRVersion, Type> | undefined> {
      const sd = structureDefinitions.find((sd) => sd.url === url);
      if (!sd) throw new Error(`Could not resolve url ${url}`);
      return sd as Resource<FHIRVersion, Type> | undefined;
    },
    level: "instance",
  };

  const invoke = async (
    op: IOperation<
      { payload: Resource<R4, AllResourceTypes>[] },
      { test: string }
    >,
    ctx: OpCTX,
    input: unknown
  ) => {
    await op.validate(ctx, "in", input);
    const output = { test: "whatever" };
    await op.validate(ctx, "out", output);
    return output;
  };

  expect(
    invoke(operation, ctx, { payload: "asdf" } as unknown)
  ).rejects.toThrow(
    new OperationError(
      outcomeFatal(
        "structure",
        "Unable to resolve canonical for type 'undefined'"
      )
    )
  );
});
