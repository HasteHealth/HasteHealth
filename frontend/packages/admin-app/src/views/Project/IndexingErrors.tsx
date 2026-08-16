import { ExclamationTriangleIcon } from "@heroicons/react/24/outline";
import { useAtomValue } from "jotai";
import React, { useCallback, useEffect, useState } from "react";

import { Loading, Table, Toaster } from "@haste-health/components";
import { R4 } from "@haste-health/fhir-types/versions";
import { HasteHealthIndexingErrors } from "@haste-health/generated-ops/lib/r4/ops";

import { getClient } from "../../db/client";

type ErrorRecord = NonNullable<
  HasteHealthIndexingErrors.Output["errors"]
>[number];

function MetricCard({
  title,
  value,
  detail,
}: Readonly<{
  title: string;
  value: string;
  detail: string;
}>) {
  return (
    <div className="rounded-lg border border-slate-200 bg-white p-4 shadow-sm">
      <div className="text-sm font-medium text-slate-500">{title}</div>
      <div className="mt-2 text-2xl font-semibold text-slate-900">{value}</div>
      <div className="mt-1 text-xs text-slate-500">{detail}</div>
    </div>
  );
}

function SectionCard({
  title,
  description,
  children,
  action,
}: Readonly<{
  title: string;
  description: string;
  children: React.ReactNode;
  action?: React.ReactNode;
}>) {
  return (
    <section className="rounded-lg border border-slate-200 bg-white p-5 shadow-sm">
      <div className="mb-4 flex items-start justify-between gap-3">
        <div>
          <h2 className="text-lg font-semibold text-slate-900">{title}</h2>
          <p className="mt-1 text-sm text-slate-500">{description}</p>
        </div>
        {action}
      </div>
      {children}
    </section>
  );
}

function StatusBadge({ resolvedAt }: Readonly<{ resolvedAt?: string }>) {
  return resolvedAt ? (
    <span className="inline-flex items-center rounded-full bg-green-100 px-2 py-0.5 text-xs font-medium text-green-800">
      Resolved
    </span>
  ) : (
    <span className="inline-flex items-center rounded-full bg-red-100 px-2 py-0.5 text-xs font-medium text-red-800">
      Unresolved
    </span>
  );
}

export default function IndexingErrors() {
  const client = useAtomValue(getClient);

  const [errors, setErrors] = useState<
    HasteHealthIndexingErrors.Output["errors"]
  >([]);
  const [loading, setLoading] = useState(true);
  const [resolving, setResolving] = useState(false);

  const loadErrors = useCallback(() => {
    setLoading(true);

    client
      .invoke_system(HasteHealthIndexingErrors.Op, {}, R4, {})
      .then((res) => {
        setErrors(res.errors ?? []);
      })
      .catch(() => {
        Toaster.error("Failed to load indexing errors.");
      })
      .finally(() => {
        setLoading(false);
      });
  }, [client]);

  useEffect(() => {
    loadErrors();
  }, [loadErrors]);

  const resolveAll = useCallback(() => {
    setResolving(true);

    const resolvePromise = client
      .invoke_system(HasteHealthIndexingErrors.Op, {}, R4, { resolve: true })
      .then((res) => {
        setErrors(res.errors ?? []);
      })
      .finally(() => {
        setResolving(false);
      });

    Toaster.promise(resolvePromise, {
      loading: "Marking indexing errors as resolved",
      success: "Indexing errors marked as resolved",
      error: "Failed to resolve indexing errors.",
    });
  }, [client]);

  const unresolvedCount = (errors ?? []).filter(
    (error) => !error.resolved_at,
  ).length;

  return (
    <div className="flex w-full flex-col gap-6 overflow-y-auto">
      <header className="rounded-lg border border-slate-200 bg-white p-5 shadow-sm">
        <div className="space-y-1">
          <h1 className="text-2xl font-semibold text-slate-900">
            Indexing Errors
          </h1>
          <p className="text-sm text-slate-500">
            Resources that failed to index into search. These are skipped rather
            than retried indefinitely.
          </p>
        </div>
      </header>

      <section className="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
        <MetricCard
          title="Total Failures"
          value={(errors?.length ?? 0).toLocaleString()}
          detail="Recorded indexing failures"
        />
        <MetricCard
          title="Unresolved"
          value={unresolvedCount.toLocaleString()}
          detail="Not yet marked resolved"
        />
      </section>

      <SectionCard
        title="Failed Resources"
        description="Each row is a resource version that could not be indexed for search, along with why."
        action={
          <div className="flex items-center gap-2">
            <button
              className="rounded-md border border-slate-300 px-3 py-1.5 text-xs font-medium text-slate-700 hover:bg-slate-50"
              onClick={loadErrors}
            >
              Refresh
            </button>
            <button
              className="rounded-md border border-slate-300 px-3 py-1.5 text-xs font-medium text-slate-700 hover:bg-slate-50 disabled:pointer-events-none disabled:opacity-50"
              onClick={resolveAll}
              disabled={resolving || unresolvedCount === 0}
            >
              Mark all resolved
            </button>
          </div>
        }
      >
        {loading ? (
          <div className="flex items-center gap-2 py-4 text-sm text-slate-500">
            <Loading />
            <span>Loading indexing errors...</span>
          </div>
        ) : (errors?.length ?? 0) === 0 ? (
          <div className="flex flex-col items-center gap-2 py-10 text-sm text-slate-500">
            <ExclamationTriangleIcon className="h-8 w-8 text-slate-300" />
            <span>No indexing errors recorded.</span>
          </div>
        ) : (
          <Table
            columns={[
              {
                id: "id",
                content: "ID",
                selectorType: "fhirpath",
                selector: "$this.id",
              },
              {
                id: "resource_type",
                content: "Resource Type",
                selectorType: "fhirpath",
                selector: "$this.resource_type",
              },
              {
                id: "version_id",
                content: "Version ID",
                selectorType: "fhirpath",
                selector: "$this.version_id",
              },
              {
                id: "fhir_method",
                content: "Method",
                selectorType: "fhirpath",
                selector: "$this.fhir_method",
              },
              {
                id: "attempt_count",
                content: "Attempts",
                selectorType: "fhirpath",
                selector: "$this.attempt_count",
              },
              {
                id: "error_message",
                content: "Error",
                selectorType: "fhirpath",
                selector: "$this.error_message",
              },
              {
                id: "first_failed_at",
                content: "First Failed",
                selectorType: "fhirpath",
                selector: "$this.first_failed_at",
              },
              {
                id: "last_failed_at",
                content: "Last Failed",
                selectorType: "fhirpath",
                selector: "$this.last_failed_at",
              },
              {
                id: "status",
                content: "Status",
                selectorType: "fhirpath",
                selector: "$this",
                renderer: (data) => {
                  const error = data[0] as ErrorRecord | undefined;
                  return <StatusBadge resolvedAt={error?.resolved_at} />;
                },
              },
            ]}
            data={errors ?? []}
          />
        )}
      </SectionCard>
    </div>
  );
}
