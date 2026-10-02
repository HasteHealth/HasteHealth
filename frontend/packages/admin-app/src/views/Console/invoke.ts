/**
 * Invoking an operation, at whichever level it declares.
 *
 * An `OperationDefinition` says where it can be called with `system`, `type`
 * and `instance`. Today the console invokes from the operation's own page, so
 * a type or instance call needs to be told what to act on; later the same
 * functions back an "Invoke" action offered on a resource type's listing and
 * on an instance, which is why the level is data here rather than a branch at
 * the call site.
 */
import { OperationDefinition, id } from "@haste-health/fhir-types/r4/types";
import { R4, ResourceType } from "@haste-health/fhir-types/versions";
import { Operation } from "@haste-health/operation-execution";

import { createAdminAppClient } from "../../db/client";

type Client = ReturnType<typeof createAdminAppClient>;

/** Where an operation is invoked from. */
export type InvokeLevel = "system" | "type" | "instance";

/** What an invocation acts on, which depends on the level. */
export interface InvokeTarget {
  level: InvokeLevel;
  /** Required for `type` and `instance`. */
  resourceType?: string;
  /** Required for `instance`. */
  id?: string;
}

/** The levels `operation` declares it can be invoked at, in FHIR's order. */
export function declaredLevels(
  operation: OperationDefinition | undefined,
): InvokeLevel[] {
  if (!operation) return [];
  const levels: InvokeLevel[] = [];
  if (operation.system) levels.push("system");
  if (operation.type) levels.push("type");
  if (operation.instance) levels.push("instance");
  return levels;
}

/** The resource types `operation` applies to, for a type or instance call. */
export function declaredResourceTypes(
  operation: OperationDefinition | undefined,
): string[] {
  return (operation?.resource ?? []) as string[];
}

/** How the request would read, e.g. `POST /Patient/123/$everything`. */
export function describeInvocation(
  operation: OperationDefinition | undefined,
  target: InvokeTarget,
): string {
  const code = operation?.code ?? "operation";
  switch (target.level) {
    case "system":
      return `POST /$${code}`;
    case "type":
      return `POST /${target.resourceType ?? "{type}"}/$${code}`;
    case "instance":
      return `POST /${target.resourceType ?? "{type}"}/${
        target.id ?? "{id}"
      }/$${code}`;
  }
}

/** Why `target` cannot be invoked yet, or undefined when it can. */
export function invocationBlocker(
  operation: OperationDefinition | undefined,
  target: InvokeTarget,
): string | undefined {
  if (!operation) return "No operation to invoke.";
  if (!declaredLevels(operation).includes(target.level)) {
    return `This operation does not declare ${target.level} level invocation.`;
  }
  if (target.level !== "system" && !target.resourceType) {
    return "Choose a resource type to invoke against.";
  }
  if (target.level === "instance" && !target.id) {
    return "Enter the id of the resource to invoke against.";
  }
  return undefined;
}

/**
 * Invokes `operation` against `target`.
 *
 * The client builds the request body from the operation's declared `in`
 * parameters, so anything in `input` that the operation does not declare is
 * dropped rather than sent.
 */
export async function invoke(
  client: Client,
  operation: OperationDefinition,
  target: InvokeTarget,
  input: unknown,
): Promise<unknown> {
  const op = new Operation(operation);

  switch (target.level) {
    case "system":
      return await client.invoke_system(op, {}, R4, input as never);
    case "type":
      return await client.invoke_type(
        op,
        {},
        R4,
        target.resourceType as ResourceType<R4>,
        input as never,
      );
    case "instance":
      return await client.invoke_instance(
        op,
        {},
        R4,
        target.resourceType as ResourceType<R4>,
        target.id as id,
        input as never,
      );
  }
}
