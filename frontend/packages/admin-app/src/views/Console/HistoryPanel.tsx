import { useAtomValue } from "jotai";
import React, { useCallback, useEffect, useState } from "react";
import { generatePath, useNavigate } from "react-router";

import {
  Button,
  Outcome,
  OutcomePanel,
  Table,
  toOutcome,
} from "@haste-health/components";
import { BundleEntry, id } from "@haste-health/fhir-types/r4/types";
import { R4, ResourceType } from "@haste-health/fhir-types/versions";

import { getClient } from "../../db/client";
import { Target } from "../../query/model";

const PAGE_SIZE = 25;

export interface HistoryPanelProps {
  /** Which level to list, taken straight from the command's target. */
  target: Target;
}

/**
 * History at whichever level the target names.
 *
 * System, type and instance history differ only in which client call they
 * make - the columns and paging are identical - so they are one view rather
 * than the three near-copies this replaces.
 */
export function HistoryPanel({ target }: Readonly<HistoryPanelProps>) {
  const client = useAtomValue(getClient);
  const navigate = useNavigate();
  const [entries, setEntries] = useState<BundleEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [hasMore, setHasMore] = useState(false);
  const [outcome, setOutcome] = useState<Outcome>();
  const [elapsedMs, setElapsedMs] = useState<number>();

  const load = useCallback(
    (offset: number) => {
      const parameters = `_offset=${offset}&_count=${PAGE_SIZE}`;
      if (offset === 0) setLoading(true);
      else setLoadingMore(true);
      setOutcome(undefined);
      const startedAt = performance.now();

      const request = (() => {
        switch (target.level) {
          case "system":
            return client.history_system({}, R4, parameters);
          case "type":
            return client.history_type(
              {},
              R4,
              target.resourceType as ResourceType<R4>,
              parameters,
            );
          case "instance":
            return client.history_instance(
              {},
              R4,
              target.resourceType as ResourceType<R4>,
              target.id as id,
              parameters,
            );
        }
      })();

      request
        .then((response) => {
          setEntries((current) =>
            offset === 0 ? response : [...current, ...response],
          );
          // A full page implies there is probably another.
          setHasMore(response.length === PAGE_SIZE);
          setElapsedMs(Math.round(performance.now() - startedAt));
        })
        .catch((error) => {
          setOutcome(toOutcome(error));
          if (offset === 0) setEntries([]);
        })
        .finally(() => {
          setLoading(false);
          setLoadingMore(false);
        });
    },
    [client, target],
  );

  useEffect(() => load(0), [load]);

  if (outcome) {
    return <OutcomePanel outcome={outcome} />;
  }

  return (
    <div className="flex flex-col">
      {elapsedMs !== undefined && (
        <div className="mb-2 flex items-center gap-2 text-xs text-slate-500">
          <span>{entries.length} entries</span>
          <span
            className="rounded bg-slate-100 px-1.5 py-0.5 font-mono text-[11px] text-slate-600"
            title="Round trip time as the browser measured it"
          >
            {elapsedMs} ms
          </span>
        </div>
      )}
      <Table
        isLoading={loading}
        data={entries}
        onRowClick={(row: unknown) => {
          const entry = row as BundleEntry;
          const resource = entry.resource;
          if (!resource?.id || !resource.meta?.versionId) return;
          navigate(
            generatePath("/r/:resourceType/:id/_history/:versionId", {
              resourceType: resource.resourceType,
              id: resource.id,
              versionId: resource.meta.versionId,
            }),
          );
        }}
        columns={[
          {
            id: "interaction",
            content: "Method",
            selector: "$this.request.method",
            selectorType: "fhirpath",
          },
          // The type is fixed at type and instance level, so it only earns a
          // column in a system wide listing.
          ...(target.level === "system"
            ? [
                {
                  id: "resource",
                  content: "Type",
                  selector: "$this.resource.type().type",
                  selectorType: "fhirpath" as const,
                },
              ]
            : []),
          ...(target.level === "instance"
            ? []
            : [
                {
                  id: "id",
                  content: "ID",
                  selector: "$this.resource.id",
                  selectorType: "fhirpath" as const,
                },
              ]),
          {
            id: "version",
            content: "Version",
            selector: "$this.resource.meta.versionId",
            selectorType: "fhirpath",
          },
          {
            id: "author",
            content: "Author",
            selector:
              "$this.resource.meta.extension.where(url='https://haste.health/author').value.reference",
            selectorType: "fhirpath",
          },
          {
            id: "updated-at",
            content: "Updated",
            selector: "$this.resource.meta.lastUpdated",
            selectorType: "fhirpath",
          },
        ]}
      />

      {hasMore && (
        <div className="mt-3 flex justify-center">
          <Button
            buttonType="secondary"
            buttonSize="small"
            disabled={loadingMore}
            onClick={() => load(entries.length)}
          >
            {loadingMore ? "Loading…" : "Load more"}
          </Button>
        </div>
      )}
    </div>
  );
}
