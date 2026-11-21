import { useAtomValue } from "jotai";
import React, { useEffect, useMemo, useState } from "react";

import { FHIRGenerativeForm, Loading, Setter } from "@haste-health/components";
import {
  Resource,
  StructureDefinition,
} from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";

import { getClient } from "../../db/client";
import { getStructures } from "../../db/structures";

/**
 * A resource as a form built from its StructureDefinition: the one view in
 * the console that can fill in a field the resource does not already carry.
 */
export function GeneratedForm({
  resource,
  resourceType,
  onChange,
}: Readonly<{
  resource: Resource;
  resourceType: string;
  /** Omitted where the resource cannot be written, such as a past version. */
  onChange?: React.Dispatch<React.SetStateAction<Resource | undefined>>;
}>) {
  const client = useAtomValue(getClient);
  const structures = useAtomValue(getStructures);
  const [sd, setSd] = useState<StructureDefinition>();

  useEffect(() => {
    let cancelled = false;
    structures?.get(resourceType).then((found) => {
      if (!cancelled) setSd(found);
    });
    return () => {
      cancelled = true;
    };
  }, [structures, resourceType]);

  // Stable, because the form memoises on it, and applied through React's
  // updater so the edit lands on the current value.
  const setValue = useMemo(
    () => (getResource: Setter) =>
      onChange?.((current) => getResource((current ?? {}) as Resource)),
    [onChange],
  );

  if (!sd) {
    return (
      <div className="flex flex-1 items-center justify-center py-8">
        <Loading />
      </div>
    );
  }

  // The form has no read-only mode, and omitting its setter would only make
  // edits vanish silently, so a disabled fieldset does the work.
  return (
    <fieldset className="min-w-0" disabled={!onChange}>
      <FHIRGenerativeForm
        client={client}
        fhirVersion={R4}
        structureDefinition={sd}
        value={resource}
        setValue={setValue}
      />
    </fieldset>
  );
}
