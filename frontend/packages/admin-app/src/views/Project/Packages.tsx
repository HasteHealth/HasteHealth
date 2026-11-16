import { ArrowPathIcon, PuzzlePieceIcon } from "@heroicons/react/24/outline";
import { useAtomValue } from "jotai";
import React, { useCallback, useEffect, useState } from "react";
import { useNavigate } from "react-router";

import {
  Button,
  Input,
  Loading,
  Table,
  Toaster,
} from "@haste-health/components";
import {
  ImplementationGuide,
  base64Binary,
} from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";
import {
  HasteHealthImplementationguideInstall as Install,
  HasteHealthImplementationguideUninstall as Uninstall,
} from "@haste-health/generated-ops/lib/r4/ops";

import { getClient } from "../../db/client";
import { getErrorMessage, readAsBase64 } from "../../utilities";

type PackageResult = NonNullable<Install.Output["package"]>[number];

/** The server's message for a failed request, whatever shape it came in. */
function errorMessage(error: unknown): string {
  try {
    return getErrorMessage(error);
  } catch {
    return error instanceof Error ? error.message : "Unknown error";
  }
}

function SectionCard({
  title,
  description,
  children,
  action,
}: Readonly<{
  title: string;
  description: React.ReactNode;
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

function Checkbox({
  label,
  checked,
  onChange,
}: Readonly<{
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}>) {
  return (
    <label className="flex items-center gap-2 text-xs text-[var(--ci-ink)]">
      <input
        type="checkbox"
        className="h-4 w-4 rounded-none border border-[var(--ci-line-strong)] accent-[var(--ci-signal)]"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
      />
      {label}
    </label>
  );
}

const STATUS_STYLES: Record<string, string> = {
  installed: "border-emerald-600/40 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300",
  updated: "border-sky-600/40 bg-sky-500/10 text-sky-700 dark:text-sky-300",
  skipped: "border-[var(--ci-line)] bg-[var(--ci-surface-sunken)] text-[var(--ci-muted)]",
};

function StatusBadge({ status }: Readonly<{ status: string }>) {
  return (
    <span
      className={`inline-flex items-center rounded-none border px-2 py-0.5 font-mono text-[10px] uppercase tracking-wider ${
        STATUS_STYLES[status] ?? STATUS_STYLES.skipped
      }`}
    >
      {status}
    </span>
  );
}

/** What a result row says after the package's name. */
function resultSummary(result: PackageResult): string | undefined {
  if (result.status === "skipped") return result.reason;
  const replaced = result.previousVersion
    ? `, replaced ${result.previousVersion}`
    : "";
  return `${result.resources ?? 0} resources${replaced}`;
}

/** What the last install did: one row per package, then any warnings. */
function InstallResult({ output }: Readonly<{ output: Install.Output }>) {
  const warnings = (output.outcome?.issue ?? [])
    .map((issue) => issue.diagnostics)
    .filter((diagnostics): diagnostics is string => !!diagnostics)
    // Distinct, so each warning is its own key.
    .filter((warning, index, all) => all.indexOf(warning) === index);

  return (
    <div className="mt-5 flex flex-col gap-3 border-t border-[var(--ci-line)] pt-4">
      <h3 className="text-xs font-mono font-semibold uppercase tracking-wider text-[var(--ci-ink)]">Result</h3>
      <ul className="flex flex-col gap-1.5 text-xs">
        {(output.package ?? []).map((result: PackageResult) => (
          <li
            key={`${result.name}@${result.version}`}
            className="flex flex-wrap items-center gap-2"
          >
            <StatusBadge status={result.status} />
            <span className="font-medium text-[var(--ci-ink)]">
              {result.name}@{result.version}
            </span>
            <span className="text-[var(--ci-ink-muted)]">{resultSummary(result)}</span>
          </li>
        ))}
      </ul>
      {warnings.length > 0 && (
        <details className="rounded-none border border-amber-500/40 bg-amber-500/10 px-3 py-2 text-xs text-amber-900 dark:text-amber-200">
          <summary className="cursor-pointer font-medium">
            {warnings.length} warning{warnings.length === 1 ? "" : "s"}
          </summary>
          <ul className="mt-2 flex list-disc flex-col gap-1 pl-4">
            {warnings.map((warning) => (
              <li key={warning}>{warning}</li>
            ))}
          </ul>
        </details>
      )}
    </div>
  );
}

/**
 * A button inside a table row. Its clicks and keys stay with it rather than
 * reaching the row, which opens its resource on a click, Enter or Space.
 */
function RowButton({
  className,
  onClick,
  children,
}: Readonly<{
  className: string;
  onClick: () => void;
  children: React.ReactNode;
}>) {
  return (
    <button
      type="button"
      className={className}
      onClick={(e) => {
        e.stopPropagation();
        onClick();
      }}
      onKeyDown={(e) => e.stopPropagation()}
    >
      {children}
    </button>
  );
}

function UninstallControl({
  guide,
  onUninstall,
}: Readonly<{
  guide: ImplementationGuide;
  onUninstall: (guide: ImplementationGuide) => void;
}>) {
  const [confirming, setConfirming] = useState(false);

  return (
    <div className="flex items-center justify-end gap-1">
      {confirming ? (
        <>
          <span className="mr-1 text-xs text-[var(--ci-ink-muted)]">
            Delete what it installed?
          </span>
          <RowButton
            className="rounded-none px-2 py-1 text-xs font-medium text-[var(--ci-ink-muted)] hover:bg-[var(--ci-surface-sunken)] hover:text-[var(--ci-ink)]"
            onClick={() => setConfirming(false)}
          >
            Cancel
          </RowButton>
          <RowButton
            className="rounded-none bg-red-600 px-2 py-1 text-xs font-medium text-white hover:bg-red-700"
            onClick={() => {
              setConfirming(false);
              onUninstall(guide);
            }}
          >
            Uninstall
          </RowButton>
        </>
      ) : (
        <RowButton
          className="rounded-none border border-[var(--ci-line-strong)] px-2 py-1 text-xs font-medium text-[var(--ci-ink)] hover:border-red-400 hover:bg-red-500/10 hover:text-red-600"
          onClick={() => setConfirming(true)}
        >
          Uninstall
        </RowButton>
      )}
    </div>
  );
}

function InstalledPackages({
  loading,
  guides,
  onUninstall,
}: Readonly<{
  loading: boolean;
  guides: ImplementationGuide[];
  onUninstall: (guide: ImplementationGuide) => void;
}>) {
  const navigate = useNavigate();

  if (loading) {
    return (
      <div className="flex items-center gap-2 py-4 text-xs text-[var(--ci-ink-muted)]">
        <Loading />
        <span>Loading packages...</span>
      </div>
    );
  }

  if (guides.length === 0) {
    return (
      <div className="flex flex-col items-center gap-2 py-10 text-xs text-[var(--ci-ink-muted)]">
        <PuzzlePieceIcon className="h-8 w-8 text-[var(--ci-ink-subtle)]" />
        <span>No packages installed.</span>
      </div>
    );
  }

  return (
    <Table
      columns={[
        {
          id: "package",
          content: "Package",
          selectorType: "fhirpath",
          selector: "$this.packageId",
        },
        {
          id: "version",
          content: "Version",
          selectorType: "fhirpath",
          selector: "$this.version",
        },
        {
          id: "title",
          content: "Title",
          selectorType: "fhirpath",
          selector: "$this.title",
        },
        {
          id: "resources",
          content: "Resources",
          selectorType: "fhirpath",
          selector: "$this.definition.resource.count()",
        },
        {
          id: "actions",
          content: "",
          selectorType: "fhirpath",
          selector: "$this",
          renderer: (data) => {
            const guide = data[0] as ImplementationGuide | undefined;
            return guide ? (
              <UninstallControl guide={guide} onUninstall={onUninstall} />
            ) : null;
          },
        },
      ]}
      data={guides}
      onRowClick={(row) => {
        const guide = row as ImplementationGuide;
        navigate(`/r/ImplementationGuide/${guide.id}`);
      }}
    />
  );
}

export default function Packages() {
  const client = useAtomValue(getClient);

  const [names, setNames] = useState("");
  const [file, setFile] = useState<File>();
  const [dependencies, setDependencies] = useState(true);
  const [examples, setExamples] = useState(false);
  const [installing, setInstalling] = useState(false);
  const [result, setResult] = useState<Install.Output>();

  const [guides, setGuides] = useState<ImplementationGuide[]>([]);
  const [loading, setLoading] = useState(true);

  const loadGuides = useCallback(() => {
    setLoading(true);
    client
      .search_type({}, R4, "ImplementationGuide", [
        { name: "_count", value: [1000] },
      ])
      .then((res) =>
        setGuides(
          [...res.resources].sort((a, b) =>
            a.packageId.localeCompare(b.packageId),
          ),
        ),
      )
      .catch(() => Toaster.error("Failed to load the installed packages."))
      .finally(() => setLoading(false));
  }, [client]);

  useEffect(() => {
    loadGuides();
  }, [loadGuides]);

  const packages = names.split(/[\s,]+/).filter((name) => name.length > 0);

  const onInstall = () => {
    setInstalling(true);
    setResult(undefined);
    const input: Install.Input = { dependencies, examples };
    if (packages.length > 0) input.package = packages;
    const promise = (file ? readAsBase64(file) : Promise.resolve(undefined))
      .then((content) => {
        if (content) input.npmContent = [content as base64Binary];
        return client.invoke_type(
          Install.Op,
          {},
          R4,
          "ImplementationGuide",
          input,
        );
      })
      .then((output) => {
        setResult(output);
        loadGuides();
        return output;
      })
      .finally(() => setInstalling(false));

    Toaster.promise(promise, {
      loading: "Installing packages",
      success: "Packages installed",
      error: errorMessage,
    });
  };

  const onUninstall = useCallback(
    (guide: ImplementationGuide) => {
      const promise = client
        .invoke_type(Uninstall.Op, {}, R4, "ImplementationGuide", {
          name: guide.packageId,
          version: guide.version,
        })
        .then((outcome) => {
          setGuides((current) => current.filter((g) => g.id !== guide.id));
          return (
            outcome.issue[0]?.diagnostics ?? `Uninstalled ${guide.packageId}`
          );
        });

      Toaster.promise(promise, {
        loading: `Uninstalling ${guide.packageId}`,
        success: String,
        error: errorMessage,
      });
    },
    [client],
  );

  return (
    <div className="flex w-full flex-col gap-6 overflow-y-auto text-[var(--ci-ink)]">
      <header className="rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface)] p-5 shadow-sm">
        <h1 className="text-xl font-mono font-semibold text-[var(--ci-ink)]">Packages</h1>
        <p className="mt-1 max-w-3xl text-xs text-[var(--ci-muted)]">
          Install FHIR packages (implementation guides such as US Core) into
          this project. Their profiles are enforced when resources are written,
          their SearchParameters can be searched, and their ValueSets and
          CodeSystems are used for validation and terminology.
        </p>
      </header>

      <SectionCard
        title="Install"
        description={
          <>
            Name packages from{" "}
            <a
              className="text-[var(--ci-signal-ink)] hover:underline"
              href="https://packages.fhir.org"
              target="_blank"
              rel="noreferrer"
            >
              the FHIR package registry
            </a>{" "}
            as <span className="font-medium">name@version</span>, or upload a
            package <span className="font-medium">.tgz</span>. Installing an
            installed package again replaces it with the version given.
          </>
        }
      >
        <div className="flex max-w-3xl flex-col gap-4">
          <Input
            label="Packages"
            placeholder="hl7.fhir.us.core@6.1.0"
            value={names}
            onChange={(e) => setNames(e.target.value)}
          />
          <Input
            label="Or upload a package (.tgz)"
            type="file"
            accept=".tgz,.tar.gz,application/gzip"
            onChange={(e) => setFile(e.target.files?.[0])}
          />
          <div className="flex flex-col gap-2">
            <Checkbox
              label="Install the packages they depend on"
              checked={dependencies}
              onChange={setDependencies}
            />
            <Checkbox
              label="Install their example resources"
              checked={examples}
              onChange={setExamples}
            />
          </div>
          <div className="flex justify-end">
            <Button
              buttonSize="medium"
              disabled={installing || (packages.length === 0 && !file)}
              onClick={onInstall}
            >
              {installing ? "Installing..." : "Install"}
            </Button>
          </div>
        </div>
        {result && <InstallResult output={result} />}
      </SectionCard>

      <SectionCard
        title="Installed packages"
        description="Each package is recorded as an ImplementationGuide. Uninstalling deletes the resources it installed; packages it depends on stay installed."
        action={
          <button
            title="Refresh"
            className="rounded-none border border-[var(--ci-line)] p-1.5 text-[var(--ci-ink-muted)] hover:bg-[var(--ci-surface-sunken)] hover:text-[var(--ci-ink)] transition-colors"
            onClick={loadGuides}
          >
            <ArrowPathIcon className="h-4 w-4" />
          </button>
        }
      >
        <InstalledPackages
          loading={loading}
          guides={guides}
          onUninstall={onUninstall}
        />
      </SectionCard>
    </div>
  );
}
