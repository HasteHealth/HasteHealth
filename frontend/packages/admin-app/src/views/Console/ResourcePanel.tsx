import {
  ArrowTopRightOnSquareIcon,
  ArrowUturnLeftIcon,
  ClockIcon,
} from "@heroicons/react/24/outline";
import { useAtomValue } from "jotai";
import React, { useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router";

import {
  Button,
  JSONResourceEditor,
  Loading,
  Outcome,
  OutcomePanel,
  Tabs,
  Toaster,
  toOutcome,
} from "@haste-health/components";
import { Resource, id } from "@haste-health/fhir-types/r4/types";
import { R4, ResourceType } from "@haste-health/fhir-types/versions";

import { getClient } from "../../db/client";
import { useRequest } from "../../hooks/useRequest";
import { ElementInfo, getStructures } from "../../db/structures";
import { Target } from "../../query/model";
import { fhirResourceDocsUrl, getErrorMessage } from "../../utilities";
import { HistoryPanel } from "./HistoryPanel";
import { Badge, PanelHeader } from "./PanelHeader";
import { resourceViews } from "./registry";
import { useViewTabs } from "./useViewTabs";
import "./views";

export interface ResourcePanelProps {
  target: Extract<Target, { level: "instance" }>;
}

/** Flattens a resource into the leaf paths present on it. */
function leafPaths(value: unknown, prefix = ""): string[] {
  if (value === null || typeof value !== "object")
    return prefix ? [prefix] : [];
  if (Array.isArray(value)) {
    return value.flatMap((item) => leafPaths(item, prefix));
  }
  return Object.entries(value as Record<string, unknown>)
    .filter(([key]) => !key.startsWith("_") && key !== "resourceType")
    .flatMap(([key, child]) =>
      leafPaths(child, prefix ? `${prefix}.${key}` : key),
    );
}

/**
 * Finds what the definition says about `path`.
 *
 * A resource's snapshot defines its own elements but not the innards of the
 * datatypes they use: `Patient.meta` is there, `Patient.meta.lastUpdated` is
 * not, because that belongs to `Meta`. Rather than fetch every datatype, the
 * nearest defined ancestor is used and reported as such, which is enough to
 * say what part of the resource a field belongs to.
 */
function describe(
  elements: Map<string, ElementInfo>,
  path: string,
): { info?: ElementInfo; inherited?: string } {
  const exact = elements.get(path);
  if (exact) return { info: exact };

  const parts = path.split(".");
  for (let i = parts.length - 1; i > 0; i--) {
    const ancestor = parts.slice(0, i).join(".");
    const info = elements.get(ancestor);
    if (info) return { info, inherited: ancestor };
  }
  return {};
}

/**
 * What the resource says, element by element, with the definition from the
 * StructureDefinition beside it.
 *
 * Raw JSON is the right default for a developer, but it assumes you know what
 * every field means. This is the answer to "what is this field", drawn from
 * the same StructureDefinition the server validates against, so someone who
 * does not know FHIR can read a resource without leaving the console.
 */
/**
 * The type cell's text. An inherited row describes an ancestor, so showing
 * that ancestor's type against a leaf would be misleading.
 */
function typeLabel(
  info: ElementInfo | undefined,
  inherited: string | undefined,
): string {
  if (inherited) return "—";
  const types = info?.types.join(" | ") ?? "—";
  return `${types}${info?.isArray ? "[]" : ""}`;
}

function Elements({
  resource,
  resourceType,
}: Readonly<{ resource: Resource; resourceType: string }>) {
  const structures = useAtomValue(getStructures);
  const [elements, setElements] = useState<Map<string, ElementInfo>>();

  useEffect(() => {
    let cancelled = false;
    structures?.elements(resourceType).then((found) => {
      if (!cancelled) setElements(found);
    });
    return () => {
      cancelled = true;
    };
  }, [structures, resourceType]);

  // Only the paths this resource actually uses, deduplicated and in a stable
  // order; a full element list would be mostly empty rows.
  const present = useMemo(() => {
    const paths = new Set<string>();
    for (const path of leafPaths(resource)) {
      // Array indices are not part of an element's path.
      paths.add(path.replace(/\.\d+/g, ""));
    }
    return Array.from(paths).sort((a, b) => a.localeCompare(b));
  }, [resource]);

  if (!elements) {
    return (
      <div className="flex justify-center py-8">
        <Loading />
      </div>
    );
  }

  return (
    <div className="overflow-auto">
      <table className="w-full text-sm">
        <thead className="sticky top-0 bg-slate-50 text-left text-xs uppercase tracking-wide text-slate-500">
          <tr>
            <th className="px-3 py-2 font-medium">Element</th>
            <th className="px-3 py-2 font-medium">Type</th>
            <th className="px-3 py-2 font-medium">Meaning</th>
          </tr>
        </thead>
        <tbody>
          {present.map((path) => {
            const { info, inherited } = describe(elements, path);
            return (
              <tr key={path} className="border-t border-slate-100 align-top">
                <td className="whitespace-nowrap px-3 py-1.5 font-mono text-xs text-slate-800">
                  {path}
                </td>
                <td className="whitespace-nowrap px-3 py-1.5 text-xs text-slate-500">
                  {/* An inherited row describes its ancestor, so showing that
                      ancestor's type against a leaf would be misleading. */}
                  {typeLabel(info, inherited)}
                </td>
                <td className="px-3 py-1.5 text-xs text-slate-600">
                  {info?.short ? (
                    <>
                      {info.short}{" "}
                      {inherited && (
                        <span className="text-slate-400">
                          (part of{" "}
                          <span className="font-mono">{inherited}</span>
                          {info.types.length > 0 && `, a ${info.types[0]}`})
                        </span>
                      )}
                    </>
                  ) : (
                    <span className="text-slate-400">
                      An extension or a profile element.
                    </span>
                  )}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

/**
 * One resource: its JSON, what its elements mean, and its history.
 *
 * This is what a `read` command lands on, and what a row in the results table
 * opens.
 */
export function ResourcePanel({ target }: Readonly<ResourcePanelProps>) {
  const client = useAtomValue(getClient);
  const navigate = useNavigate();
  const [draft, setDraft] = useState<Resource>();
  const [saving, setSaving] = useState(false);

  const { resourceType, id: resourceId, versionId } = target;
  const readOnly = versionId !== undefined;

  // A specific version is read with `vread`; otherwise the current one.
  const {
    data: fetched,
    loading,
    outcome,
  } = useRequest<Resource>(
    () =>
      (versionId
        ? client.vread(
            {},
            R4,
            resourceType as ResourceType<R4>,
            resourceId as id,
            versionId as id,
          )
        : client.read(
            {},
            R4,
            resourceType as ResourceType<R4>,
            resourceId as id,
          )) as Promise<Resource>,
    [client, resourceType, resourceId, versionId],
  );

  // What was saved last wins over what was fetched, so the editor keeps
  // showing the result of a save rather than reverting to the read.
  const [saved, setSaved] = useState<Resource>();
  const resource = saved ?? fetched;

  // A newly fetched resource replaces whatever the editor was holding.
  useEffect(() => {
    setSaved(undefined);
    setDraft(fetched);
  }, [fetched]);

  /** An outcome from a failed save, shown above the editor. */
  const [saveOutcome, setSaveOutcome] = useState<Outcome>();

  // Whether the draft has diverged from what the server last gave us. The
  // specialized tabs use it to warn that a run uses the saved version.
  const dirty = useMemo(
    () =>
      draft !== undefined &&
      resource !== undefined &&
      JSON.stringify(draft) !== JSON.stringify(resource),
    [draft, resource],
  );

  const save = () => {
    if (!draft) return;
    setSaving(true);
    setSaveOutcome(undefined);
    Toaster.promise(
      client
        .update(
          {},
          R4,
          resourceType as ResourceType<R4>,
          resourceId as id,
          draft,
        )
        .then((result) => {
          setSaved(result as Resource);
          setDraft(result as Resource);
          return result;
        })
        .catch((error) => {
          // A toast cannot carry an OperationOutcome's issues, and a rejected
          // save is exactly when they matter, so it is kept on the page too.
          setSaveOutcome(toOutcome(error));
          throw error;
        })
        .finally(() => setSaving(false)),
      {
        loading: "Saving",
        success: "Saved",
        error: (error) => getErrorMessage(error),
      },
    );
  };

  const generic = [
    {
      id: "json",
      title: "JSON",
      content: (
        <JSONResourceEditor
          resource={resource as Resource}
          onChange={readOnly ? undefined : setDraft}
        />
      ),
    },
    {
      id: "elements",
      title: "Elements",
      content: (
        <Elements resource={resource as Resource} resourceType={resourceType} />
      ),
    },
    ...(versionId
      ? []
      : [
          {
            id: "history",
            title: "History",
            content: (
              <HistoryPanel
                target={{
                  level: "instance" as const,
                  resourceType,
                  id: resourceId,
                }}
              />
            ),
          },
        ]),
  ];
  const { tabs, selectedTab, onTab } = useViewTabs((paneState, showPane) =>
    resourceViews(
      {
        // Custom views read and write the draft, so an edit here shows in the
        // JSON tab and vice versa.
        resource: draft ?? resource ?? ({} as Resource),
        resourceType,
        resourceId,
        dirty,
        readOnly,
        saved: true,
        onChange: setDraft,
        paneState,
        onPaneStateChange: showPane,
      },
      generic,
    ),
  );

  if (loading) {
    return (
      <div className="flex flex-1 items-center justify-center py-12">
        <Loading />
      </div>
    );
  }
  if (outcome) return <OutcomePanel outcome={outcome} />;
  if (!resource) return null;

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <PanelHeader
        title={`${resourceType}/${resourceId}`}
        badges={
          <>
            {resource.meta?.versionId && (
              <Badge>Version {resource.meta.versionId}</Badge>
            )}
            {versionId && <Badge tone="amber">read only</Badge>}
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
            {versionId ? (
              <Button
                buttonSize="small"
                buttonType="secondary"
                onClick={() => navigate(`/r/${resourceType}/${resourceId}`)}
              >
                <span className="flex items-center">
                  <ArrowUturnLeftIcon className="mr-1 h-4 w-4" />
                  Current
                </span>
              </Button>
            ) : (
              <Button
                buttonSize="small"
                buttonType="secondary"
                onClick={() =>
                  navigate(`/r/${resourceType}/${resourceId}/_history`)
                }
              >
                <span className="flex items-center">
                  <ClockIcon className="mr-1 h-4 w-4" />
                  History
                </span>
              </Button>
            )}
            {!readOnly && (
              <Button buttonSize="small" disabled={saving} onClick={save}>
                {saving ? "Saving…" : "Save"}
              </Button>
            )}
          </>
        }
        description={
          resource.meta?.lastUpdated
            ? `Updated ${resource.meta.lastUpdated}`
            : ""
        }
      />

      {saveOutcome && (
        <div className="mb-2">
          <OutcomePanel
            outcome={saveOutcome}
            request={{
              method: "PUT",
              path: `/${resourceType}/${resourceId}`,
            }}
          />
        </div>
      )}

      <Tabs selectedTab={selectedTab} onTab={onTab} tabs={tabs} />
    </div>
  );
}
