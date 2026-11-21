/**
 * What a resource type is, above its results.
 *
 * The console addresses everything as a FHIR path, which is precise but says
 * nothing about the type you are looking at. This is where the type explains
 * itself: what it holds, what state its definition is in, and the three
 * things people do with it.
 */
import {
  ArrowTopRightOnSquareIcon,
  ClockIcon,
  PlusIcon,
} from "@heroicons/react/24/outline";
import { useAtomValue } from "jotai";
import React, { useEffect, useState } from "react";

import { Button } from "@haste-health/components";
import { StructureDefinition } from "@haste-health/fhir-types/r4/types";

import { getStructures } from "../../db/structures";

/** Shown until a type's own definition says otherwise. */
const FHIR_VERSION_LABEL = "4.0.1";
import { Badge, PanelHeader } from "./PanelHeader";
import { fhirResourceDocsUrl } from "../../utilities";

export interface ResourceTypeHeaderProps {
  resourceType: string;
  /** Opens the type's history. */
  onHistory: () => void;
  /** Stages a create for this type. */
  onNew: () => void;
}

/**
 * The first sentence of `text`.
 *
 * A StructureDefinition's description can run for paragraphs, and the header
 * is not where someone reads it; the Docs link is.
 */
function firstSentence(text: string): string {
  const stop = text.indexOf(". ");
  return stop === -1 ? text : text.slice(0, stop + 1);
}

export function ResourceTypeHeader({
  resourceType,
  onHistory,
  onNew,
}: Readonly<ResourceTypeHeaderProps>) {
  const structures = useAtomValue(getStructures);
  const [structure, setStructure] = useState<StructureDefinition>();

  useEffect(() => {
    let cancelled = false;
    setStructure(undefined);
    // Already cached per type, so switching back to a type costs nothing.
    structures?.get(resourceType).then((found) => {
      if (!cancelled) setStructure(found);
    });
    return () => {
      cancelled = true;
    };
  }, [structures, resourceType]);

  const description = structure?.description
    ? firstSentence(structure.description as string)
    : undefined;

  return (
    <PanelHeader
      title={resourceType}
      badges={
        <>
          <Badge>{(structure?.status as string) ?? "active"}</Badge>
          <Badge>
            Version {(structure?.version as string) ?? FHIR_VERSION_LABEL}
          </Badge>
        </>
      }
      actions={
        <>
          <Button
            buttonSize="small"
            buttonType="secondary"
            onClick={() =>
              window.open(
                fhirResourceDocsUrl(resourceType),
                "_blank",
                "noopener,noreferrer",
              )
            }
          >
            <span className="flex items-center">
              <ArrowTopRightOnSquareIcon className="mr-1 h-4 w-4" />
              Docs
            </span>
          </Button>
          <Button buttonSize="small" buttonType="secondary" onClick={onHistory}>
            <span className="flex items-center">
              <ClockIcon className="mr-1 h-4 w-4" />
              History
            </span>
          </Button>
          <Button buttonSize="small" onClick={onNew}>
            <span className="flex items-center">
              <PlusIcon className="mr-1 h-4 w-4" />
              New
            </span>
          </Button>
        </>
      }
      description={description}
    />
  );
}
