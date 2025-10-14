import { OperationOutcome } from "@oxidized-health/fhir-types/r4/types";
import { ProjectId, TenantId } from "@oxidized-health/jwt/types";

export function deriveTenantId(): TenantId {
  const host = window.location.host;
  const tenantID = host.split(".")[0]?.split("_")[0];

  return tenantID as TenantId;
}

export function deriveProjectId(): ProjectId {
  const host = window.location.host;
  const projectId = host.split(".")[0]?.split("_")[1];

  return projectId as ProjectId;
}

export function getErrorMessage(error: any): string {
  if ("response" in error) {
    const message = (error.response.body as OperationOutcome).issue
      .map((issue) => issue.diagnostics)
      .join("\n");

    return message;
  }
  return "Unknown Error";
}
