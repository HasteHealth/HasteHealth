import { OperationOutcome } from "@oxidized-health/fhir-types/r4/types";

export function getErrorMessage(error: any): string {
  if ("response" in error) {
    const message = (error.response.body as OperationOutcome).issue
      .map((issue) => issue.diagnostics)
      .join("\n");

    return message;
  }
  return "Unknown Error";
}
