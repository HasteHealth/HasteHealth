import { descend, typedPointer } from "@haste-health/fhir-pointer";
import { resourceTypes } from "@haste-health/fhir-types/r4/sets";
import {
  OperationDefinition,
  OperationDefinitionParameter,
  OperationOutcomeIssue,
  Parameters,
  Resource,
  code,
  uri,
} from "@haste-health/fhir-types/r4/types";
import { R4, ResourceType } from "@haste-health/fhir-types/versions";
import validate from "@haste-health/fhir-validation";
import type { ValidationCTX } from "@haste-health/fhir-validation/types";
import {
  OperationError,
  issueError,
  outcomeError,
} from "@haste-health/operation-outcomes";

type ParameterDefinitions = NonNullable<OperationDefinition["parameter"]>;

function capitalize(string: string) {
  return string.charAt(0).toUpperCase() + string.slice(1);
}

export function mapToParameter(
  definition: NonNullable<ParameterDefinitions[number]>,
  use: "out" | "in",
  valueMap: any
): NonNullable<Parameters["parameter"]> {
  const isArray = definition.max !== "1";
  let value = valueMap[definition.name];
  if (value === undefined || value === null) return [];

  if (!isArray) value = [value];
  if (isArray && !Array.isArray(value)) value = [value];

  const params: NonNullable<Parameters["parameter"]> = value.map(
    (value: any): NonNullable<Parameters["parameter"]>[number] => {
      if (definition.type) {
        if (definition.type === "Type")
          throw new Error("Cannot process 'Type'");
        if (definition.type === "Any" || resourceTypes.has(definition.type)) {
          return { name: definition.name, resource: value };
        }
        const fieldName = `value${capitalize(definition.type || "")}`;
        return {
          name: definition.name,
          [fieldName]: value,
        };
      } else {
        if (!definition.part)
          throw new OperationError(
            outcomeError(
              "invalid",
              `No type or part found on parameter definition ${definition.name}`
            )
          );
      }

      return {
        name: definition.name,
        part: definition.part
          .map((partDefinition) => {
            return mapToParameter(partDefinition, use, value);
          })
          .flat(),
      };
    }
  );

  return params;
}

export function toParametersResource(
  operationDefinition: OperationDefinition,
  use: "out" | "in",
  value: Record<string, unknown>
): Parameters {
  const definitions =
    operationDefinition.parameter?.filter((param) => param.use === use) || [];
  const parameters: Parameters = {
    resourceType: "Parameters",
    parameter: definitions
      .map((definition) => mapToParameter(definition, use, value))
      .flat(),
  };

  return parameters;
}

function validateNoExtraFields(
  definitions: ParameterDefinitions,
  use: "out" | "in",
  parameters: NonNullable<Parameters["parameter"]>
) {
  const definitionNames = new Set(
    definitions
      .filter((d) => d.use === use)
      .map((definition) => definition.name)
  );
  const paramNames = new Set(parameters.map((param) => param.name));
  paramNames.forEach((paramName) => {
    if (!definitionNames.has(paramName as code)) {
      throw new OperationError(
        outcomeError("invalid", `Parameter '${paramName}' not supported`)
      );
    }
  });
}

function parseParameter(
  definition: ParameterDefinitions[number],
  use: "out" | "in",
  parameters: NonNullable<Parameters["parameter"]>
) {
  const isRequired = definition.min > 1;
  const isArray = definition.max !== "1";

  const parsedParameters = parameters
    .map((param) => {
      if (definition.type || definition.searchType) {
        if (
          definition.type === "Any" ||
          resourceTypes.has(definition.type || "")
        ) {
          return param.resource;
        } else {
          if (!definition.type) {
            throw new OperationError(
              outcomeError(
                "invalid",
                `No type found on parameter definition ${definition.name}`
              )
            );
          }
          // if (definition.searchType)
          //   throw new OperationError(
          //     outcomeError("not-supported", `SearchType not supported`)
          //   );
          if (definition.type === "Type")
            throw new Error("Cannot process 'Type'");
          // @ts-ignore
          return param[`value${capitalize(definition.type || "")}`];
        }
        // Means this is a primitive
      } else {
        if (!definition.part)
          throw new OperationError(
            outcomeError(
              "invalid",
              `No type or part found on parameter definition ${definition.name}`
            )
          );
        validateNoExtraFields(definition.part, use, param.part || []);
        return (definition.part || []).reduce(
          (acc: Record<string, unknown>, paramDefinition) => {
            const parsedParam = parseParameter(
              paramDefinition,
              use,
              (param.part || []).filter(
                (param) => param.name === paramDefinition.name
              )
            );
            if (parsedParam !== undefined)
              acc[paramDefinition.name] = parsedParam;
            return acc;
          },
          {}
        );
      }
    })
    .filter((v) => v !== undefined);

  if (isRequired && parsedParameters.length === 0)
    throw new OperationError(
      outcomeError("required", `Missing required parameter ${definition.name}`)
    );
  if (
    definition.max !== "*" &&
    parsedParameters.length > parseInt(definition.max)
  ) {
    throw new OperationError(
      outcomeError("too-costly", `Too many parameters ${definition.name}`)
    );
  }

  if (!isArray) {
    return parsedParameters[0];
  }
  return parsedParameters.length > 0 ? parsedParameters : undefined;
}

export function parseParameters(
  operationDefinition: OperationDefinition,
  use: "out" | "in",
  parameters: Parameters
) {
  const paramDefinitions =
    operationDefinition.parameter?.filter((param) => param.use === use) || [];
  validateNoExtraFields(paramDefinitions, use, parameters.parameter || []);
  const parametersParsed: Record<string, unknown> = paramDefinitions.reduce(
    (parametersParsed: Record<string, unknown>, parameterDefinition) => {
      const curParameters =
        parameters.parameter?.filter(
          (param) => param.name === parameterDefinition.name
        ) || [];
      const parsedParam = parseParameter(
        parameterDefinition,
        use,
        curParameters
      );
      if (parsedParam !== undefined) {
        parametersParsed[parameterDefinition.name] = parsedParam;
      }
      return parametersParsed;
    },
    {}
  );

  return parametersParsed;
}

type InputOutput<I, O> = { in: I; out: O };

export function isParameters(input: unknown): input is Parameters {
  return (
    Object.prototype.hasOwnProperty.call(input, "resourceType") &&
    (input as Record<string, unknown>).resourceType === "Parameters"
  );
}

function isRecord(input: unknown): input is Record<string, unknown> {
  return typeof input === "object" && input !== null;
}

function validateRequired(
  definitions: NonNullable<OperationDefinition["parameter"]>,
  value: Record<string, unknown>
): OperationOutcomeIssue[] {
  return definitions
    .filter((d) => d.min > 0)
    .reduce(
      (
        issues: OperationOutcomeIssue[],
        definition: OperationDefinitionParameter
      ) => {
        if (!(definition.name in value))
          return [
            ...issues,
            issueError(
              "required",
              `Missing required parameter '${definition.name}'`
            ),
          ];

        return issues;
      },
      []
    );
}

function validateCardinalities(
  definition: NonNullable<OperationDefinition["parameter"]>[number],
  value: unknown[]
) {
  if (definition.min > value.length)
    throw new OperationError(
      outcomeError(
        "required",
        `Must have '${definition.min}' minimum for field ${definition.min}`
      )
    );
  if (definition.max !== "*" && value.length > parseInt(definition.max)) {
    throw new OperationError(
      outcomeError("too-costly", `Too many parameters ${definition.name}`)
    );
  }
}

async function validateParameter<Use extends "in" | "out">(
  ctx: ValidationCTX,
  paramDefinition: NonNullable<OperationDefinition["parameter"]>[number],
  use: Use,
  value: any
): Promise<OperationOutcomeIssue[]> {
  let arr: Array<any> = (value = Array.isArray(value) ? value : [value]);
  validateCardinalities(paramDefinition, value);

  let issues: OperationOutcomeIssue[] = [];

  for (const index in arr) {
    if (paramDefinition.type) {
      const type =
        paramDefinition.type === "Any" ||
        paramDefinition.type === "Resource" ||
        paramDefinition.type === "DomainResource"
          ? arr[index].resourceType
          : paramDefinition.type;

      issues = [
        ...issues,
        ...(await validate(
          ctx,
          type,
          arr,
          descend(typedPointer<typeof arr, (typeof arr)[number]>(), index)
        )),
      ];
    } else {
      if (!paramDefinition.part) {
        issues = [
          ...issues,
          issueError(
            "invalid",
            `Invalid definition '${paramDefinition.name}' must have either part or type`
          ),
        ];
      } else {
        issues = [
          ...issues,
          ...validateRequired(paramDefinition.part, arr[index]),
        ];

        for (const part of paramDefinition.part) {
          if (!isRecord(arr[index])) {
            issues = [
              ...issues,
              issueError(
                "invalid",
                `Parameter ${part.name} must be an object found: '${arr[index]}'`
              ),
            ];
          }
          issues = [
            ...issues,
            ...(await validateParameter(ctx, part, use, arr[index][part.name])),
          ];
        }
      }
    }
  }
  return issues;
}

async function validateParameters<T extends IOperation<unknown, unknown>>(
  ctx: ValidationCTX,
  op: T,
  use: "in" | "out",
  value: unknown
): Promise<OperationOutcomeIssue[]> {
  let issues: OperationOutcomeIssue[] = [];
  const definitions = (op.operationDefinition.parameter || []).filter(
    (p) => p.use === (use as code)
  );
  const parameterName = use === "in" ? "input" : "output";

  if (!isRecord(value))
    return [
      ...issues,
      issueError(
        "invalid",
        `Invalid ${parameterName}: Must be an object but instead is '${
          value === null ? "null" : typeof value
        }'`
      ),
    ];

  issues = [...issues, ...validateRequired(definitions, value)];

  for (const key of Object.keys(value)) {
    const paramDefinition = definitions.find((d) => d.name === key);
    if (paramDefinition === undefined) {
      issues = [
        ...issues,
        issueError(
          "invalid",
          `Invalid ${parameterName}: '${key}' not found in definition`
        ),
      ];
    } else {
      issues = [
        ...issues,
        ...(await validateParameter(ctx, paramDefinition, use, value[key])),
      ];
    }
  }

  return issues;
}

export interface OpCTX extends ValidationCTX {
  level: "system" | "type" | "instance";
}

export interface IOperation<I, O> {
  code: code;
  get operationDefinition(): OperationDefinition;
  parseToObject<Use extends "in" | "out">(
    use: Use,
    input: unknown
  ): InputOutput<I, O>[Use];
  parseToParameters<Use extends "in" | "out">(
    use: Use,
    input: InputOutput<I, O>[Use]
  ): Parameters | I | O;
  validate<Use extends "in" | "out">(
    ctx: OpCTX,
    use: Use,
    value: unknown
  ): Promise<OperationOutcomeIssue[]>;
}

function isStrictlyReturn(op: OperationDefinition): boolean {
  const outputParameters = op.parameter?.filter((p) => p.use === "out") || [];
  if (outputParameters.length === 1) {
    return (
      outputParameters[0].name === "return" &&
      outputParameters[0].max === "1" &&
      outputParameters[0].min === 1 &&
      (outputParameters[0].type === "Any" ||
        resourceTypes.has(outputParameters[0].type || ""))
    );
  }
  return false;
}
/**
 * If, when invoking the operation, there is exactly one input parameter of type Resource (irrespective of whether other possible parameters are defined), that the operation can also be executed by a POST with that resource as the body of the request (and no parameters on the url).
 * @param op  OperationDefinition
 */
function singleResource(
  op: OperationDefinition,
  type: ResourceType<R4>
): code | undefined {
  const foundName = op.parameter?.filter(
    (p) =>
      p.type === type || p.type === "Resource" || p.type === "DomainResource"
  );
  if (foundName?.length !== 1) return undefined;
  return foundName[0].name;
}

export type Executor<CTX, I, O> = (ctx: CTX, input: I) => Promise<O>;

export type OPMetadata<O> = O extends IOperation<infer Input, infer Output>
  ? { Input: Input; Output: Output }
  : never;

export class Operation<I, O> implements IOperation<I, O> {
  private _operationDefinition: OperationDefinition;
  code: code;
  constructor(operationDefinition: OperationDefinition) {
    this.code = operationDefinition.code;
    this._operationDefinition = operationDefinition;
  }
  get operationDefinition(): OperationDefinition {
    return this._operationDefinition;
  }
  parseToObject<Use extends "in" | "out">(
    use: Use,
    passedInInput: Parameters | Resource
  ): InputOutput<I, O>[Use] {
    let input = passedInInput;
    if (
      use === "out" &&
      !isParameters(input) &&
      isStrictlyReturn(this.operationDefinition)
    ) {
      return input as InputOutput<I, O>[Use];
    }
    // Allow for single resource input.
    if (
      use === "in" &&
      input.resourceType !== "Parameters" &&
      resourceTypes.has(input.resourceType)
    ) {
      const fieldName = singleResource(
        this.operationDefinition,
        input.resourceType
      );
      if (fieldName) {
        return {
          [fieldName]: input,
        } as InputOutput<I, O>[Use];
      }
      throw new OperationError(
        outcomeError("invalid", "Invalid input, input must be a Parameters")
      );
    } else if (!isParameters(input)) {
      throw new OperationError(
        outcomeError("invalid", "Invalid input, input must be a Parameters")
      );
    }

    const output = parseParameters(this._operationDefinition, use, input);
    return output as InputOutput<I, O>[Use];
  }

  parseToParameters(use: "in" | "out", input: I | O): Parameters | I | O {
    if (
      use === "out" &&
      !isParameters(input) &&
      isStrictlyReturn(this.operationDefinition)
    ) {
      return input;
    }
    return toParametersResource(
      this._operationDefinition,
      use,
      input as Record<string, unknown>
    );
  }
  async validate<Use extends "in" | "out">(
    ctx: OpCTX,
    use: Use,
    value: unknown
  ): Promise<OperationOutcomeIssue[]> {
    if (isStrictlyReturn(this.operationDefinition) && use === "out") {
      const type =
        this.operationDefinition.parameter?.find((p) => p.use === "out")
          ?.type || "Resource";

      const fhirtype = type === "Any" ? "Resource" : type;

      const issues = await validate(ctx, fhirtype as uri, value);

      return issues;
    }
    return await validateParameters(ctx, this, use, value);
  }
}

export type Invocation = <
  T extends IOperation<unknown, unknown>,
  CTX extends OpCTX
>(
  op: T,
  ctx: CTX,
  input: OPMetadata<T>["Input"]
) => Promise<OPMetadata<T>["Output"]>;
