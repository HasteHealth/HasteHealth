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
    <div className="rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface)] p-4 shadow-sm">
      <div className="text-xs font-mono uppercase tracking-wider text-[var(--ci-muted)]">{title}</div>
      <div className="mt-2 text-2xl font-mono font-semibold text-[var(--ci-ink)]">{value}</div>
      <div className="mt-1 text-xs font-mono text-[var(--ci-muted)]">{detail}</div>
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
    <section className="rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface)] p-5 shadow-sm">
      <div className="mb-4 flex items-start justify-between gap-3">
        <div>
          <h2 className="text-sm font-mono font-semibold uppercase tracking-wider text-[var(--ci-ink)]">{title}</h2>
          <p className="mt-1 text-xs text-[var(--ci-muted)]">{description}</p>
        </div>
        {action}
      </div>
      {children}
    </section>
  );
}

function StatusBadge({ resolvedAt }: Readonly<{ resolvedAt?: string }>) {
  return resolvedAt ? (
    <span className="inline-flex items-center rounded-none border border-emerald-500/30 bg-emerald-500/10 px-2 py-0.5 font-mono text-[10px] uppercase text-emerald-500">
      Resolved
    </span>
  ) : (
    <span className="inline-flex items-center rounded-none border border-red-500/30 bg-red-500/10 px-2 py-0.5 font-mono text-[10px] uppercase text-red-500">
      Unresolved
    </span>
  );
}

/** The failures table, or what stands in for it while loading or empty. */
function FailedResources({
  loading,
  errors,
}: Readonly<{
  loading: boolean;
  errors: ErrorRecord[];
}>) {
  if (loading) {
    return (
      <div className="flex items-center gap-2 py-4 font-mono text-xs text-[var(--ci-muted)]">
        <Loading />
        <span>Loading indexing errors...</span>
      </div>
    );
  }

  if (errors.length === 0) {
    return (
      <div className="flex flex-col items-center gap-2 py-10 font-mono text-xs text-[var(--ci-muted)]">
        <ExclamationTriangleIcon className="h-8 w-8 text-[var(--ci-muted)] opacity-40" />
        <span>No indexing errors recorded.</span>
      </div>
    );
  }

  return (
    <Table
      columns={[
        {
          id: "id",
          content: "ID",
          selectorType: "fhirpath",
          selector: "$this.id",
        },
        {
          id: "version_id",
          content: "Version ID",
          selectorType: "fhirpath",
          selector: "$this.version_id",
        },
        {
          id: "resource_type",
          content: "Resource Type",
          selectorType: "fhirpath",
          selector: "$this.resource_type",
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
      data={errors}
    />
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
      <header className="rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface)] p-5 shadow-sm">
        <div className="space-y-1">
          <h1 className="text-xl font-mono font-semibold text-[var(--ci-ink)]">
            Indexing Errors
          </h1>
          <p className="text-xs text-[var(--ci-muted)]">
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
              className="rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface)] px-3 py-1.5 font-mono text-xs uppercase tracking-wider text-[var(--ci-ink)] hover:border-[var(--ci-signal)] hover:text-[var(--ci-signal)] transition-colors"
              onClick={loadErrors}
            >
              Refresh
            </button>
            <button
              className="rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface)] px-3 py-1.5 font-mono text-xs uppercase tracking-wider text-[var(--ci-ink)] hover:border-[var(--ci-signal)] hover:text-[var(--ci-signal)] transition-colors disabled:pointer-events-none disabled:opacity-50"
              onClick={resolveAll}
              disabled={resolving || unresolvedCount === 0}
            >
              Mark all resolved
            </button>
          </div>
        }
      >
        <FailedResources loading={loading} errors={errors ?? []} />
      </SectionCard>
    </div>
  );
}
