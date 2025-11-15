import { resourceTypes } from "@haste-health/fhir-types/r4/sets";
import { FHIR_VERSION, Resource } from "@haste-health/fhir-types/versions";

function generateParameterType(
  parameters: NonNullable<
    Resource<FHIR_VERSION, "OperationDefinition">["parameter"]
  >
): string {
  if (parameters.length === 0) return `Record<string, never>`;
  const fields = parameters
    .map((p) => {
      const isArray = p.max !== "1";
      const required = p.min > 0;

      const fieldName = `"${p.name}"${required ? "" : "?"}`;
      const type = p.type === "Any" ? "Resource" : p.type;

      const singularValue = p.type
        ? `fhirTypes.${type}`
        : generateParameterType(p.part || []);
      const value = isArray ? `Array<${singularValue}>` : singularValue;

      return `${fieldName}: ${value}`;
    })
    .join(",\n");
  return `{${fields}}`;
}

function capitalize(s: string): string {
  return s.charAt(0).toUpperCase() + s.slice(1);
}

function getName(op: Resource<FHIR_VERSION, "OperationDefinition">): string {
  if (!op.id) throw new Error("Must have id for generating operation");
  return capitalize(
    op.id
      .split("-")
      .map((s) => capitalize(s))
      .join("")
  );
}

function generateOutput(
  parameters: NonNullable<
    Resource<FHIR_VERSION, "OperationDefinition">["parameter"]
  >
): string {
  if (
    parameters.length === 1 &&
    parameters[0].name === "return" &&
    (parameters[0].type === "Any" ||
      resourceTypes.has(parameters[0].type || ""))
  ) {
    return `fhirTypes.${
      parameters[0].type === "Any" ? "Resource" : parameters[0].type
    }`;
  }
  return generateParameterType(parameters);
}

export function generateOp(
  op: Resource<FHIR_VERSION, "OperationDefinition">
): string {
  const namespace = getName(op);
  const interfaceName = "IOp";
  const operationName = "Op";
  const inputName = `Input`;
  const outputName = `Output`;
  const inputParameters = (op.parameter || []).filter((op) => op.use === "in");
  const outputParameters = (op.parameter || []).filter(
    (op) => op.use === "out"
  );

  const inputType = `export type ${inputName} = ${generateParameterType(
    inputParameters
  )}`;

  const outputType = `export type ${outputName} = ${generateOutput(
    outputParameters
  )}`;

  const operationType = `export type ${interfaceName} = IOperation<${inputName}, ${outputName}>`;
  const operationInstance = `export const ${operationName}: ${interfaceName} = new Operation<${inputName}, ${outputName}>(${JSON.stringify(
    op
  )} as fhirTypes.OperationDefinition)`;

  return `export namespace ${namespace} {
  ${[inputType, outputType, operationType, operationInstance].join("\n")}}`;
}

export default async function operationGeneration<Version extends FHIR_VERSION>(
  fhirVersion: Version,
  operations: Readonly<Array<Resource<Version, "OperationDefinition">>>
): Promise<string> {
  if (fhirVersion !== "4.0") throw new Error("Only support 4.0");
  const code = [
    `import type * as fhirTypes from "@haste-health/fhir-types/r4/types";`,
    `import { Operation, IOperation } from "@haste-health/operation-execution";`,
    ...operations.map((op) => generateOp(op)),
  ].join("\n");

  return code;
}
