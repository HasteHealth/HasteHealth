import { ElementDefinition } from "@oxidized-health/fhir-types/lib/generated/r4/types";
import {
  OperationError,
  outcomeFatal,
} from "@oxidized-health/operation-outcomes";

/**
 * Removes the type from the path.
 * @param path The ElementDefinition path to remove the type from.
 */
export function removeTypeOnPath(path: string): string {
  const firstDot = path.indexOf(".");
  // If first element this would be the entire path as no subfield.
  return path.substring(firstDot !== -1 ? path.indexOf(".") + 1 : path.length);
}

export function joinPaths(...paths: string[]) {
  return paths.filter((p) => p !== "").join(".");
}

export type Discriminator = NonNullable<
  NonNullable<ElementDefinition["slicing"]>["discriminator"]
>[number];

/**
 *
 * @param discriminator the discrimator to convert to an ElementDefinition path.
 * @returns the path of the discriminator as an ElementDefinition path.
 */
export function convertPathToElementPointer(
  discriminatorElement: ElementDefinition,
  discriminator: Discriminator
): string {
  if (
    discriminator.path.includes("ofType(") ||
    discriminator.path.includes("resolve()") ||
    discriminator.path.includes("extension(")
  ) {
    throw new OperationError(
      outcomeFatal(
        "not-supported",
        `Discriminator path '${discriminator.path}' is not supported`
      )
    );
  }

  const parentPath = removeTypeOnPath(discriminatorElement.path);
  const path = discriminator.path.replace("$this", "");

  if (path === "") {
    return parentPath;
  }
  return `${parentPath}.${path}`;
}
