/**
 * The Roles & permissions studio: build an AccessPolicyV2 from a resource
 * CRUD matrix and assign it to a membership, client or operation.
 *
 * The matrix is the friendly view; the generated policy is the source of
 * truth. The verification panel calls the server's $evaluate-policy operation
 * on every matrix cell, so "what the checkboxes say" is compared with what the
 * engine actually decides.
 */
import {
  CheckIcon,
  ClipboardDocumentIcon,
  MagnifyingGlassIcon,
  ShieldCheckIcon,
  XMarkIcon,
} from "@heroicons/react/24/outline";
import classNames from "classnames";
import { useAtomValue } from "jotai";
import React, { useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router";

import {
  FHIRReferenceEditable,
  Loading,
  Select,
  Toaster,
} from "@haste-health/components";
import {
  AccessPolicyV2,
  AccessPolicyV2Assignment,
  Bundle,
  BundleEntry,
  Reference,
  Resource,
  ResourceType,
  code,
  id,
  uri,
} from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";
import { HasteHealthEvaluatePolicy } from "@haste-health/generated-ops/lib/r4/ops";

import { getCapabilities } from "../../db/capabilities";
import { getClient } from "../../db/client";
import { getErrorMessage, resourceInstancePath } from "../../utilities";
import {
  applyPreset,
  buildAssignment,
  buildPolicy,
  configuredTypes,
  emptyPermission,
  hasAnyPermission,
  PRESET_LABELS,
  type CrudKey,
  type CrudPreset,
  type PermissionMap,
} from "./policyDraft";
import {
  buildChecks,
  runPolicyChecks,
  type CheckInteraction,
  type CheckResult,
} from "./policyCheck";

const CRUD: { key: CrudKey; label: string }[] = [
  { key: "create", label: "Create" },
  { key: "read", label: "Read" },
  { key: "update", label: "Update" },
  { key: "delete", label: "Delete" },
];

const CHECK_COLUMNS: { key: CheckInteraction; label: string }[] = [
  { key: "create", label: "Create" },
  { key: "read", label: "Read" },
  { key: "search", label: "Search" },
  { key: "update", label: "Update" },
  { key: "delete", label: "Delete" },
];

const ASSIGNABLE_TYPES: ResourceType[] = [
  "Membership",
  "ClientApplication",
  "OperationDefinition",
];

const INPUT_CLASS =
  "w-full rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] px-2.5 py-1.5 font-mono text-xs text-[var(--ci-ink)] placeholder:text-[var(--ci-muted)] focus:border-[var(--ci-signal)] focus:outline-none focus:ring-1 focus:ring-[var(--ci-signal)]";

const SMALL_BUTTON =
  "rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] px-1.5 py-0.5 font-mono text-[10px] uppercase tracking-wider text-[var(--ci-ink-muted)] transition-colors hover:border-[var(--ci-signal)] hover:text-[var(--ci-ink)]";

const PRIMARY_BUTTON =
  "inline-flex items-center justify-center gap-1.5 rounded-none border border-[var(--ci-signal)] bg-[var(--ci-signal)] px-3.5 py-1.5 font-mono text-xs font-semibold uppercase tracking-wider text-[var(--ci-on-signal)] transition-colors hover:bg-[var(--ci-signal-hover)] disabled:cursor-not-allowed disabled:opacity-50";

const SECONDARY_BUTTON =
  "inline-flex items-center justify-center gap-1.5 rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] px-3.5 py-1.5 font-mono text-xs font-semibold uppercase tracking-wider text-[var(--ci-ink)] transition-colors hover:bg-[var(--ci-surface-sunken)] disabled:cursor-not-allowed disabled:opacity-50";

function errorMessage(error: unknown): string {
  try {
    if (error instanceof Error && !("response" in error)) return error.message;
    return getErrorMessage(error);
  } catch {
    return error instanceof Error ? error.message : String(error);
  }
}

function Section({
  title,
  description,
  action,
  children,
  className,
}: Readonly<{
  title: string;
  description?: string;
  action?: React.ReactNode;
  children: React.ReactNode;
  className?: string;
}>) {
  return (
    <section
      className={classNames(
        "rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface)] shadow-sm",
        className,
      )}
    >
      <header className="flex flex-wrap items-start justify-between gap-2 border-b border-[var(--ci-line)] px-4 py-2.5">
        <div>
          <h2 className="font-mono text-xs font-semibold uppercase tracking-wider text-[var(--ci-ink)]">
            {title}
          </h2>
          {description && (
            <p className="mt-0.5 text-[11px] text-[var(--ci-muted)]">
              {description}
            </p>
          )}
        </div>
        {action}
      </header>
      <div className="p-4">{children}</div>
    </section>
  );
}

function PermissionCheck({
  resourceType,
  crudKey,
  label,
  checked,
  onChange,
}: Readonly<{
  resourceType: string;
  crudKey: CrudKey;
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}>) {
  return (
    <input
      type="checkbox"
      className="h-4 w-4 cursor-pointer accent-[var(--ci-signal)]"
      checked={checked}
      onChange={(event) => onChange(event.target.checked)}
      aria-label={`${label} ${resourceType}`}
      data-crud={crudKey}
    />
  );
}

function ResultCell({ result }: Readonly<{ result: CheckResult | undefined }>) {
  if (!result) {
    return <span className="text-[var(--ci-ink-subtle)]">—</span>;
  }
  if (result.status === "skipped") {
    return (
      <span
        className="text-[var(--ci-ink-subtle)]"
        title={`Not sampled: ${result.message}`}
      >
        —
      </span>
    );
  }
  const Icon = result.actual ? CheckIcon : XMarkIcon;
  return (
    <span
      className={
        result.status === "pass" ? "text-emerald-500" : "text-red-500"
      }
      title={`${result.actual ? "Allowed" : "Denied"}: ${result.message}`}
    >
      <Icon className="mx-auto h-4 w-4" />
    </span>
  );
}

export default function PolicyStudio() {
  const capabilities = useAtomValue(getCapabilities);
  const client = useAtomValue(getClient);
  const navigate = useNavigate();

  const resourceTypes = useMemo(
    () =>
      (capabilities?.rest?.[0]?.resource ?? [])
        .map((entry) => entry.type as string)
        .filter(Boolean)
        .sort((a, b) => a.localeCompare(b)),
    [capabilities],
  );

  const [preset, setPreset] = useState<CrudPreset>("custom");
  const [permissions, setPermissions] = useState<PermissionMap>(() =>
    applyPreset("custom", resourceTypes),
  );
  const [policyName, setPolicyName] = useState("");
  const [description, setDescription] = useState("");
  const [target, setTarget] = useState<Reference | undefined>();
  const [filter, setFilter] = useState("");
  const [configuredOnly, setConfiguredOnly] = useState(false);
  const [preview, setPreview] = useState<"policy" | "assignment">("policy");
  const [saving, setSaving] = useState(false);
  const [savedPolicyId, setSavedPolicyId] = useState<string | undefined>();
  const [savedPolicyName, setSavedPolicyName] = useState<string | undefined>();
  const [checking, setChecking] = useState(false);
  const [checkResults, setCheckResults] = useState<CheckResult[]>([]);
  const [checkProgress, setCheckProgress] = useState({ done: 0, total: 0 });
  const [checkError, setCheckError] = useState<string | undefined>();

  // The capability statement resolves asynchronously, so add rows for types
  // that were not present when the matrix was initialised.
  useEffect(() => {
    setPermissions((current) => {
      let changed = false;
      const next: PermissionMap = { ...current };
      for (const resourceType of resourceTypes) {
        if (!next[resourceType]) {
          next[resourceType] = emptyPermission();
          changed = true;
        }
      }
      return changed ? next : current;
    });
  }, [resourceTypes]);

  const markDirty = () => {
    setSavedPolicyId(undefined);
    setSavedPolicyName(undefined);
    setCheckResults([]);
    setCheckProgress({ done: 0, total: 0 });
    setCheckError(undefined);
  };

  const configured = useMemo(() => configuredTypes(permissions), [permissions]);

  const visibleTypes = useMemo(() => {
    const needle = filter.trim().toLowerCase();
    return resourceTypes.filter((resourceType) => {
      if (configuredOnly && !hasAnyPermission(permissions[resourceType])) {
        return false;
      }
      return needle === "" || resourceType.toLowerCase().includes(needle);
    });
  }, [resourceTypes, permissions, filter, configuredOnly]);

  const previewPolicy = useMemo(
    () =>
      buildPolicy({
        name: policyName.trim() || "Untitled access policy",
        description,
        permissions,
      }),
    [policyName, description, permissions],
  );

  const previewAssignment = useMemo<AccessPolicyV2Assignment>(
    () => ({
      resourceType: "AccessPolicyV2Assignment",
      accessPolicy: { reference: "AccessPolicyV2/<server-assigned-id>" },
      link: { reference: target?.reference ?? "<select an assignment target>" },
    }),
    [target],
  );

  const previewJson = useMemo(
    () =>
      JSON.stringify(
        preview === "policy" ? previewPolicy : previewAssignment,
        null,
        2,
      ),
    [preview, previewPolicy, previewAssignment],
  );

  const canSave =
    policyName.trim().length > 0 &&
    Boolean(target?.reference) &&
    configured.length > 0;

  const changePreset = (value: CrudPreset) => {
    setPreset(value);
    if (value === "custom") return;
    setPermissions(applyPreset(value, resourceTypes));
    markDirty();
  };

  const setPermission = (
    resourceType: string,
    key: CrudKey,
    value: boolean,
  ) => {
    setPermissions((current) => ({
      ...current,
      [resourceType]: {
        ...(current[resourceType] ?? emptyPermission()),
        [key]: value,
      },
    }));
    setPreset("custom");
    markDirty();
  };

  const setRow = (resourceType: string, mode: "full" | "read" | "none") => {
    const permission =
      mode === "full"
        ? { create: true, read: true, update: true, delete: true }
        : mode === "read"
          ? { create: false, read: true, update: false, delete: false }
          : emptyPermission();
    setPermissions((current) => ({ ...current, [resourceType]: permission }));
    setPreset("custom");
    markDirty();
  };

  const setAll = (value: boolean) => {
    const next: PermissionMap = {};
    for (const resourceType of resourceTypes) {
      next[resourceType] = value
        ? { create: true, read: true, update: true, delete: true }
        : emptyPermission();
    }
    setPermissions(next);
    setPreset("custom");
    markDirty();
  };

  const copyPreview = () => {
    navigator.clipboard
      .writeText(previewJson)
      .then(() => Toaster.success("Copied to clipboard"))
      .catch(() => Toaster.error("Failed to copy"));
  };

  const save = async () => {
    if (!canSave || !target?.reference) return;

    setSaving(true);
    try {
      // A transaction so the policy and its assignment land together. The
      // assignment points at the policy's fullUrl, which the server replaces
      // with the id it assigns.
      const policyReference = `urn:uuid:${crypto.randomUUID()}`;
      const policy = buildPolicy({
        name: policyName,
        description,
        permissions,
      });
      const assignment = buildAssignment(policyReference, target);

      const response = await client.transaction(
        {},
        R4,
        {
          resourceType: "Bundle",
          type: "transaction",
          entry: [
            {
              fullUrl: policyReference as uri,
              request: {
                method: "POST" as code,
                url: "AccessPolicyV2" as uri,
              },
              resource: policy as Resource,
            },
            {
              request: {
                method: "POST" as code,
                url: "AccessPolicyV2Assignment" as uri,
              },
              resource: assignment as Resource,
            },
          ],
        } as Bundle,
      );

      const created = response.entry
        ?.map((entry) => entry.resource)
        .find(
          (resource) => resource?.resourceType === "AccessPolicyV2",
        ) as AccessPolicyV2 | undefined;

      if (!created?.id) {
        throw new Error("The server did not return the created AccessPolicyV2.");
      }

      setSavedPolicyId(created.id);
      setSavedPolicyName(created.name);
      setCheckResults([]);
      setCheckProgress({ done: 0, total: 0 });
      setCheckError(undefined);
      Toaster.success(`Created ${created.name}.`);
    } catch (error) {
      Toaster.error(errorMessage(error));
    } finally {
      setSaving(false);
    }
  };

  const evaluate = async (entry: BundleEntry): Promise<string> => {
    if (!savedPolicyId) throw new Error("Save the policy first.");

    try {
      const outcome = await client.invoke_instance(
        HasteHealthEvaluatePolicy.Op,
        {},
        R4,
        "AccessPolicyV2",
        savedPolicyId as id,
        {
          request: {
            resourceType: "Bundle",
            type: "batch" as code,
            entry: [entry],
          } as Bundle,
        },
      );

      return (
        (outcome.issue ?? [])
          .map((issue) => issue.diagnostics)
          .filter((diagnostic): diagnostic is string => Boolean(diagnostic))
          .join(" ") || "The operation returned no diagnostic."
      );
    } catch (error) {
      // Surface the operation's diagnostic so a fixture problem is reported
      // as "not sampled" rather than as a policy decision.
      throw new Error(errorMessage(error));
    }
  };

  const verify = async () => {
    if (!savedPolicyId) return;

    setChecking(true);
    setCheckError(undefined);
    setCheckResults([]);
    setCheckProgress({ done: 0, total: 0 });

    try {
      const checks = buildChecks({
        permissions,
        allResourceTypes: resourceTypes,
        probeId: "role-check",
      });

      setCheckProgress({ done: 0, total: checks.length });
      const results: CheckResult[] = [];
      for (let index = 0; index < checks.length; index += 1) {
        const check = checks[index]!;
        const [result] = await runPolicyChecks({
          checks: [check],
          evaluate,
        });
        if (result) results.push(result);
        setCheckResults([...results]);
        setCheckProgress({ done: index + 1, total: checks.length });
      }
    } catch (error) {
      setCheckError(errorMessage(error));
    } finally {
      setChecking(false);
    }
  };

  const resultsByType = useMemo(() => {
    const rows = new Map<
      string,
      Partial<Record<CheckInteraction, CheckResult>>
    >();
    for (const result of checkResults) {
      const row = rows.get(result.resourceType) ?? {};
      row[result.interaction] = result;
      rows.set(result.resourceType, row);
    }
    return Array.from(rows.entries()).sort(([a], [b]) => a.localeCompare(b));
  }, [checkResults]);

  const failResults = checkResults.filter((result) => result.status === "fail");
  const skippedCount = checkResults.filter(
    (result) => result.status === "skipped",
  ).length;

  return (
    <div className="flex h-full min-h-0 w-full flex-col gap-5 overflow-y-auto pb-4 pr-1">
      <header className="flex flex-col gap-3 xl:flex-row xl:items-start xl:justify-between">
        <div>
          <div className="flex items-center gap-2">
            <span className="border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] px-1.5 py-0.5 font-mono text-[10px] uppercase tracking-wider text-[var(--ci-signal-ink)]">
              FHIR R4
            </span>
            <h1 className="font-mono text-xl font-semibold text-[var(--ci-ink)]">
              Roles &amp; permissions
            </h1>
          </div>
          <p className="mt-1 max-w-2xl text-xs text-[var(--ci-muted)]">
            Build an <span className="font-mono">AccessPolicyV2</span> from a
            resource CRUD matrix and assign it to a membership, client
            application or operation in one step.
          </p>
        </div>

        <div className="flex flex-wrap items-end gap-2">
          <div>
            <label
              className="mb-1 block font-mono text-[10px] uppercase tracking-wider text-[var(--ci-muted)]"
              htmlFor="role-preset"
            >
              Role preset
            </label>
            <div className="w-72">
              <Select
                id="role-preset"
                ariaLabel="Role preset"
                value={preset}
                onChange={(option) =>
                  changePreset(option.value as CrudPreset)
                }
                options={(
                  Object.entries(PRESET_LABELS) as [CrudPreset, string][]
                ).map(([value, label]) => ({ value, label }))}
              />
            </div>
          </div>
          <button
            type="button"
            className={PRIMARY_BUTTON}
            disabled={!canSave || saving}
            onClick={save}
            title={
              canSave
                ? undefined
                : "Enter a name, choose a target, and grant at least one permission."
            }
          >
            {saving ? (
              <Loading className="h-4 w-4" />
            ) : (
              <ShieldCheckIcon className="h-4 w-4" />
            )}
            {saving ? "Saving…" : "Save policy"}
          </button>
          {savedPolicyId && (
            <button
              type="button"
              className={SECONDARY_BUTTON}
              onClick={() =>
                navigate(resourceInstancePath("AccessPolicyV2", savedPolicyId))
              }
            >
              Open policy
            </button>
          )}
        </div>
      </header>

      <div className="grid gap-4 xl:grid-cols-[minmax(0,1.55fr)_minmax(0,1fr)]">
        <div className="contents">
          <Section
            className="order-1 min-w-0 xl:col-start-1 xl:row-start-1"
            title="1. Policy & assignment"
            description="Name the policy and choose what it applies to."
            action={
              <span className="font-mono text-[10px] text-[var(--ci-muted)]">
                {configured.length} type
                {configured.length === 1 ? "" : "s"} configured
              </span>
            }
          >
            <div className="grid gap-3 sm:grid-cols-2">
              <div>
                <label
                  className="mb-1 block font-mono text-[10px] uppercase tracking-wider text-[var(--ci-muted)]"
                  htmlFor="policy-name"
                >
                  Policy name
                </label>
                <input
                  id="policy-name"
                  className={INPUT_CLASS}
                  placeholder="e.g. Clinician standard access"
                  value={policyName}
                  onChange={(event) => {
                    setPolicyName(event.target.value);
                    markDirty();
                  }}
                />
              </div>
              <div>
                <label
                  className="mb-1 block font-mono text-[10px] uppercase tracking-wider text-[var(--ci-muted)]"
                  htmlFor="policy-description"
                >
                  Description <span className="normal-case">(optional)</span>
                </label>
                <input
                  id="policy-description"
                  className={INPUT_CLASS}
                  placeholder="What this role is for"
                  value={description}
                  onChange={(event) => {
                    setDescription(event.target.value);
                    markDirty();
                  }}
                />
              </div>
            </div>

            <div className="mt-3">
              <label className="mb-1 block font-mono text-[10px] uppercase tracking-wider text-[var(--ci-muted)]">
                Assign to
              </label>
              <div className="border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] px-2.5 py-2">
                <FHIRReferenceEditable
                  label="Membership, ClientApplication or OperationDefinition"
                  resourceTypesAllowed={ASSIGNABLE_TYPES}
                  fhirVersion={R4}
                  client={client}
                  value={target}
                  onChange={(reference) => {
                    setTarget(reference);
                    markDirty();
                  }}
                />
              </div>
              <p className="mt-1 font-mono text-[10px] text-[var(--ci-muted)]">
                {target?.reference
                  ? `Target: ${target.reference}`
                  : "Choose the identity that receives this policy."}
              </p>
            </div>

            <p className="mt-3 border-t border-[var(--ci-line)] pt-2 text-[11px] text-[var(--ci-muted)]">
              Assignments take effect the next time the user or client gets a
              token. Admin and owner roles are not restricted by policies.
            </p>
          </Section>

          <section className="order-3 min-w-0 rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface)] shadow-sm xl:col-start-1 xl:row-start-2">
            <div className="flex flex-wrap items-center gap-2 border-b border-[var(--ci-line)] px-3 py-2">
              <div>
                <h2 className="font-mono text-xs font-semibold uppercase tracking-wider text-[var(--ci-ink)]">
                  2. Resource permissions
                </h2>
                <p className="mt-0.5 text-[11px] text-[var(--ci-muted)]">
                  {configured.length} of {resourceTypes.length} types configured
                </p>
              </div>

              <div className="ml-auto flex flex-wrap items-center gap-2">
                <div className="relative">
                  <MagnifyingGlassIcon className="pointer-events-none absolute left-2 top-1.5 h-3.5 w-3.5 text-[var(--ci-ink-subtle)]" />
                  <input
                    className={classNames(INPUT_CLASS, "w-44 pl-7")}
                    placeholder="Filter resources…"
                    value={filter}
                    onChange={(event) => setFilter(event.target.value)}
                  />
                </div>
                <label className="flex items-center gap-1 font-mono text-[10px] text-[var(--ci-muted)]">
                  <input
                    type="checkbox"
                    className="h-3.5 w-3.5 cursor-pointer accent-[var(--ci-signal)]"
                    checked={configuredOnly}
                    onChange={(event) => setConfiguredOnly(event.target.checked)}
                  />
                  Configured only
                </label>
                <button
                  type="button"
                  className={SMALL_BUTTON}
                  onClick={() => setAll(true)}
                >
                  All
                </button>
                <button
                  type="button"
                  className={SMALL_BUTTON}
                  onClick={() => setAll(false)}
                >
                  Clear
                </button>
              </div>
            </div>

            <div className="max-h-[560px] overflow-auto">
              <table className="w-full border-collapse text-left">
                <thead className="sticky top-0 z-10 bg-[var(--ci-surface-raised)]">
                  <tr className="border-b border-[var(--ci-line)]">
                    <th className="px-3 py-2 font-mono text-[10px] font-medium uppercase tracking-wider text-[var(--ci-muted)]">
                      Resource type
                    </th>
                    {CRUD.map(({ key, label }) => (
                      <th
                        key={key}
                        className="w-16 px-2 py-2 text-center font-mono text-[10px] font-medium uppercase tracking-wider text-[var(--ci-muted)]"
                      >
                        {label}
                      </th>
                    ))}
                    <th className="w-32 px-3 py-2 text-right font-mono text-[10px] font-medium uppercase tracking-wider text-[var(--ci-muted)]">
                      Quick
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {visibleTypes.map((resourceType) => {
                    const permission =
                      permissions[resourceType] ?? emptyPermission();
                    return (
                      <tr
                        key={resourceType}
                        className="border-b border-[var(--ci-line)] transition-colors hover:bg-[var(--ci-surface-raised)]"
                      >
                        <td className="px-3 py-1.5 font-mono text-xs text-[var(--ci-ink)]">
                          {resourceType}
                        </td>
                        {CRUD.map(({ key, label }) => (
                          <td
                            key={key}
                            className="px-2 py-1.5 text-center"
                          >
                            <PermissionCheck
                              resourceType={resourceType}
                              crudKey={key}
                              label={label}
                              checked={permission[key]}
                              onChange={(checked) =>
                                setPermission(resourceType, key, checked)
                              }
                            />
                          </td>
                        ))}
                        <td className="px-3 py-1.5 text-right">
                          <div className="inline-flex gap-1">
                            <button
                              type="button"
                              className={SMALL_BUTTON}
                              onClick={() => setRow(resourceType, "full")}
                            >
                              Full
                            </button>
                            <button
                              type="button"
                              className={SMALL_BUTTON}
                              onClick={() => setRow(resourceType, "read")}
                            >
                              Read
                            </button>
                            <button
                              type="button"
                              className={SMALL_BUTTON}
                              onClick={() => setRow(resourceType, "none")}
                            >
                              None
                            </button>
                          </div>
                        </td>
                      </tr>
                    );
                  })}
                  {visibleTypes.length === 0 && (
                    <tr>
                      <td
                        colSpan={CRUD.length + 2}
                        className="px-3 py-6 text-center font-mono text-xs text-[var(--ci-muted)]"
                      >
                        {resourceTypes.length === 0
                          ? "The server returned no resource types."
                          : "No resource types match this filter."}
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>
          </section>

          <p className="order-4 min-w-0 rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] px-3 py-2 text-[11px] text-[var(--ci-muted)] xl:col-start-1 xl:row-start-3">
            Each configured type becomes one rule. Its target checks{" "}
            <span className="font-mono">%request.resource_type</span> and{" "}
            <span className="font-mono">%request.type</span>; the condition is{" "}
            <span className="font-mono">true</span>, so the target alone decides
            whether the rule applies. Unselected types are left undetermined and
            therefore denied.
          </p>
        </div>

        <div className="order-2 flex min-w-0 flex-col gap-4 xl:order-none xl:col-start-2 xl:row-span-3 xl:row-start-1">
          <Section
            title="Live preview"
            description="Both resources are written in one transaction."
            action={
              <button
                type="button"
                className={SECONDARY_BUTTON}
                onClick={copyPreview}
              >
                <ClipboardDocumentIcon className="h-4 w-4" />
                Copy JSON
              </button>
            }
          >
            <div className="mb-2 flex items-center gap-1 border-b border-[var(--ci-line)]">
              {(
                [
                  ["policy", "AccessPolicyV2"],
                  ["assignment", "Assignment"],
                ] as const
              ).map(([value, label]) => (
                <button
                  key={value}
                  type="button"
                  className={classNames(
                    "-mb-px border-b-2 px-2 py-1.5 font-mono text-[10px] uppercase tracking-wider transition-colors",
                    preview === value
                      ? "border-[var(--ci-signal)] text-[var(--ci-signal-ink)]"
                      : "border-transparent text-[var(--ci-muted)] hover:text-[var(--ci-ink)]",
                  )}
                  onClick={() => setPreview(value)}
                >
                  {label}
                </button>
              ))}
            </div>
            <pre className="max-h-[360px] overflow-auto whitespace-pre-wrap break-all border border-[var(--ci-line)] bg-[var(--ci-surface-sunken)] p-3 font-mono text-[11px] leading-relaxed text-[var(--ci-ink)]">
              {previewJson}
            </pre>
          </Section>

          <Section
            title="Verify access"
            description="Runs the saved policy through $evaluate-policy for every matrix cell."
          >
            {!savedPolicyId ? (
              <p className="text-xs text-[var(--ci-muted)]">
                Save the policy to verify it. Editing the matrix clears the
                saved result.
              </p>
            ) : (
              <div className="space-y-3">
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <p className="min-w-0 break-all font-mono text-[11px] text-[var(--ci-muted)]">
                    Saved AccessPolicyV2/{savedPolicyId}
                    {savedPolicyName ? ` · ${savedPolicyName}` : ""}
                  </p>
                  <button
                    type="button"
                    className={PRIMARY_BUTTON}
                    disabled={checking}
                    onClick={verify}
                  >
                    {checking ? (
                      <Loading className="h-4 w-4" />
                    ) : (
                      <ShieldCheckIcon className="h-4 w-4" />
                    )}
                    {checking
                      ? `Checking ${checkProgress.done}/${checkProgress.total}`
                      : "Run checks"}
                  </button>
                </div>

                <p className="text-[10px] font-mono text-[var(--ci-muted)]">
                  Create and update checks send a small probe body; the
                  operation decides the request without storing it.
                </p>

                {checkError && (
                  <div className="border border-red-500/30 bg-red-500/10 px-2 py-1.5 text-xs text-red-500">
                    {checkError}
                  </div>
                )}

                {checkResults.length > 0 && (
                  <>
                    <p className="text-xs text-[var(--ci-muted)]">
                      {checkResults.length - failResults.length - skippedCount}{" "}
                      matched · {failResults.length} failed
                      {skippedCount > 0 ? ` · ${skippedCount} not sampled` : ""}
                    </p>

                    <div className="overflow-auto border border-[var(--ci-line)]">
                      <table className="w-full border-collapse text-left">
                        <thead className="bg-[var(--ci-surface-raised)]">
                          <tr className="border-b border-[var(--ci-line)]">
                            <th className="px-2 py-1.5 font-mono text-[9px] font-medium uppercase tracking-wider text-[var(--ci-muted)]">
                              Type
                            </th>
                            {CHECK_COLUMNS.map(({ key, label }) => (
                              <th
                                key={key}
                                className="px-1 py-1.5 text-center font-mono text-[9px] font-medium uppercase tracking-wider text-[var(--ci-muted)]"
                              >
                                {label}
                              </th>
                            ))}
                          </tr>
                        </thead>
                        <tbody>
                          {resultsByType.map(([resourceType, row]) => (
                            <tr
                              key={resourceType}
                              className="border-b border-[var(--ci-line)] last:border-b-0"
                            >
                              <td className="px-2 py-1.5 font-mono text-[10px] text-[var(--ci-ink)]">
                                {resourceType}
                              </td>
                              {CHECK_COLUMNS.map(({ key }) => (
                                <td
                                  key={key}
                                  className="px-1 py-1.5 text-center"
                                >
                                  <ResultCell result={row[key]} />
                                </td>
                              ))}
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>

                    {failResults.length > 0 && (
                      <ul className="space-y-1">
                        {failResults.map((result) => (
                          <li
                            key={`${result.resourceType}-${result.interaction}`}
                            className="text-[11px] text-red-500"
                          >
                            <span className="font-mono">
                              {result.resourceType} {result.interaction}
                            </span>
                            : {result.message}
                          </li>
                        ))}
                      </ul>
                    )}

                    <p className="text-[10px] font-mono text-[var(--ci-muted)]">
                      Green means the server agreed with the matrix; a denial on
                      an unselected cell is a pass.
                    </p>
                  </>
                )}

                {!checking && checkResults.length === 0 && !checkError && (
                  <p className="text-[11px] text-[var(--ci-muted)]">
                    Run the checks to confirm the saved policy permits and
                    denies exactly what the matrix says.
                  </p>
                )}
              </div>
            )}
          </Section>
        </div>
      </div>
    </div>
  );
}
