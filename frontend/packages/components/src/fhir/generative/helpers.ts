import {
  ElementDefinition,
  StructureDefinition,
} from "@haste-health/fhir-types/r4/types";

function _resolveContentReferenceIndex(
  sd: StructureDefinition,
  element: ElementDefinition
): number {
  const contentReference = element.contentReference?.split("#")[1];
  const referenceElementIndex = sd.snapshot?.element.findIndex(
    (element) => element.id === contentReference
  );
  if (!referenceElementIndex)
    throw new Error(
      "unable to resolve contentreference: '" + element.contentReference + "'"
    );

  return referenceElementIndex;
}

export function getElementDefinition(
  sd: StructureDefinition,
  elementIndex: number
): { elementIndex: number; element: ElementDefinition } {
  const element = sd.snapshot?.element?.[elementIndex];

  if (!element)
    throw new Error(`SD did not have element at index ${elementIndex}`);

  if (element.contentReference) {
    const referenceElementIndex = _resolveContentReferenceIndex(sd, element);
    const resolvedElement = sd.snapshot?.element?.[referenceElementIndex];
    if (!resolvedElement)
      throw new Error(
        `Unable to resolve content reference ${element.contentReference}`
      );
    return { elementIndex: referenceElementIndex, element: resolvedElement };
  }
  return { elementIndex, element };
}
